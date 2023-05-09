#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Deref,
    sync::{atomic::AtomicU32, Arc, RwLock},
};

use kolekk_types::{
    objects::{
        Bookmark, Fields, Id, Image, Indexed, Meta, SearchableEntry, Tag, Taggable, TypeFacet,
    },
    utility::Path,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tantivy::{
    collector::{ScoreSegmentTweaker, ScoreTweaker, TopDocs},
    directory::{ManagedDirectory, MmapDirectory},
    fastfield::Column,
    query::{AllQuery, BooleanQuery, BoostQuery, FuzzyTermQuery, Occur, PhraseQuery, TermQuery},
    schema::{Facet, FacetOptions, Field, IndexRecordOption, FAST, INDEXED, STORED, TEXT},
    DocAddress, Document, Index, IndexReader, IndexWriter, SegmentReader, Term,
};
use tauri::{AppHandle, Manager, State, WindowEvent};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    config::AppConfig,
};

#[tauri::command]
pub fn new_temp_facet() -> String {
    let id = uuid::Uuid::new_v4();
    let uuid = format!("/temp/{}", id.hyphenated());
    uuid
}

#[tauri::command]
pub async fn delete_facet_objects(
    db: State<'_, AppDatabase>,
    facet: TypeFacet,
) -> Result<(), Error> {
    let mut writer = db.index_writer.write().infer_err()?;
    let _opstamp = writer.delete_term(Term::from_facet(db.get_field(Fields::Type), &facet.facet()));
    let _opstamp = writer.commit().infer_err()?;
    Ok(())
}

#[tauri::command]
pub async fn enter_searchable(
    db: State<'_, AppDatabase>,
    data: Vec<SearchableEntry<serde_json::Map<String, serde_json::Value>>>,
    facet: TypeFacet,
) -> Result<(), Error> {
    let ctime = db.now_time().infer_err()?;
    let writer = db.index_writer.read().infer_err()?;
    data.into_iter().try_for_each(|e| {
        let mut doc = Document::new();
        let v = Meta {
            id: db.new_id(),
            facet: facet.clone(),
            data: Taggable {
                data: e,
                tags: vec![],
            },
            ctime,
            last_update: ctime,
            last_interaction: ctime,
        };
        v.add(db.inner(), &mut doc).look(|e| dbg!(e))?;

        let _opstamp = writer.add_document(doc).look(|e| dbg!(e)).infer_err()?;
        // TODO: if err, do i remove all those that succeeded?
        Ok(())
    })?;
    drop(writer);
    let _opstamp = db.index_writer.write().infer_err()?.commit().infer_err()?;
    Ok(())
}

#[tauri::command]
pub async fn search_jsml_object(
    db: State<'_, AppDatabase>,
    query: String,
    facet: TypeFacet,
    limit: usize,
    offset: usize,
) -> Result<Vec<Meta<Taggable<serde_json::Map<String, serde_json::Value>>, TypeFacet>>, Error> {
    tagged_search(
        db.inner(),
        facet,
        query,
        limit,
        offset,
        ObjectSearchScoreTweaker::new(db.inner())?,
    )
}

#[tauri::command]
pub async fn add_tag_to_object(
    db: State<'_, AppDatabase>,
    id: Id,
    tag_id: Id,
) -> Result<(), Error> {
    let mut doc = db.get_doc(id)?;

    let mut v: Meta<Taggable<SearchableEntry<serde_json::Map<String, serde_json::Value>>>, Facet> =
        DbAble::take(db.inner(), &mut doc)?;
    v.data.tags.push(tag_id);
    let mut doc = Document::new();
    v.add(db.inner(), &mut doc)?;

    let mut writer = db.index_writer.write().infer_err()?;
    let _opstamp = writer.delete_term(Term::from_field_u64(db.get_field(Fields::Id), id as _));
    let _opstamp = writer.add_document(doc).infer_err()?;
    let _opstamp = writer.commit().infer_err()?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_object(
    db: State<'_, AppDatabase>,
    id: Id,
    tag_id: Id,
) -> Result<(), Error> {
    let mut doc = db.get_doc(id)?;

    let mut j: Meta<Taggable<SearchableEntry<serde_json::Map<String, serde_json::Value>>>, Facet> =
        DbAble::take(db.inner(), &mut doc)?;
    j.data.tags.retain(|&t| t != tag_id);
    let mut doc = Document::new();
    j.add(db.inner(), &mut doc)?;

    let mut writer = db.index_writer.write().infer_err()?;
    let _opstamp = writer.delete_term(Term::from_field_u64(db.get_field(Fields::Id), id as _));
    let _opstamp = writer.add_document(doc).infer_err()?;
    let _opstamp = writer.commit().infer_err()?;
    Ok(())
}

#[tauri::command]
pub fn get_path(config: State<'_, AppConfig>, path: Path) -> std::path::PathBuf {
    crate::filesystem::get_path(&path, config.inner())
}

// NOTE:
// calling DbAble::take for Meta<Taggable<Map<_, _>>> returns different stuff than
// calling DbAble::take for Meta<Taggable<SearchableEntry<Map<_, _>>>>
// because each thing is associated with different varient of Fields enum
// so calling DbAble::take for Map<_, _> parses the Fields::Json and leaves out Fields::Text stuff
//   that is used for SearchableEntry
//
// so entering Meta<Taggable<SearchableEntry<Object>>> and calling
// DbAble::take for Object, Meta<Object>, Taggable<Object>, Meta<SearchableEntry<Object>>, Taggable<Meta<Object>>, etc
// should be fine. which is kinda nice and kinda aweful at the same time
// donno if i really want this behaviour, but imma keep it for now. (i realised all this while debugging :P)
//
// this also makes it possible to define multiple views of the same Document
// for example, a TaggableMeta<T> can be implimented such that it contains { id, data, tags }
// and ignore the rest of the stuff from a Meta<Taggable<T>>
// this will allow js stuff to avoid annoying ".data" chains
//
// MAYBE:
// a stricter version of this can be made by first entering stuff in different varients of Fields enum
// and then at last converting the entire thing into a Map<_, _>
// i.e. convert Meta<Taggable<...>> into a Map<_, _>  and save it in something like Fields::Json
// and while calling DbAble::take - just deserialize this and ignore all the other stuff from the Document
// to make this checked at compile time, (Taggable<Meta<..>> would still be valid acc to compiler, but would fail in DbAble::take)
//   an extra trait constraint can be added. (impl ExtraTrait for Meta<Taggable<T>>, Meta<Tag>, ...)
//   which would enumerate all the views that do not fail
pub trait DbAble
where
    Self: Sized,
{
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error>;
    fn take(db: &AppDatabase, doc: &mut Document) -> Result<Self, Error>;
}

pub trait AutoDbAble
where
    Self: Serialize + DeserializeOwned,
{
    // fn get_type() -> ObjectType; // TODO: use this to add facet to the database somhow??
}
impl AutoDbAble for Image {}
impl AutoDbAble for Bookmark {}
impl AutoDbAble for Tag {}
impl AutoDbAble for serde_json::Map<String, serde_json::Value> {}
impl AutoDbAble for tantivy::schema::Value {}

// MAYBE: split this into 2 traits. one to add, another to get
impl<T: DbAble> DbAble for Meta<T, Facet> {
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        doc.add_u64(db.get_field(Fields::Id), self.id as _);
        doc.add_facet(db.get_field(Fields::Type), self.facet);
        doc.add_u64(db.get_field(Fields::Ctime), self.ctime as _);
        doc.add_u64(db.get_field(Fields::Mtime), self.last_update as _);
        doc.add_u64(
            db.get_field(Fields::LastInteraction),
            self.last_interaction as _,
        );
        self.data.add(db, doc)
    }

    fn take(db: &AppDatabase, doc: &mut Document) -> Result<Self, Error> {
        let m = Meta {
            id: doc
                .get_first(db.get_field(Fields::Id))
                .and_then(|e| e.as_u64().map(|e| e as _))
                .bad_err("bad id")?,
            facet: doc
                .get_first(db.get_field(Fields::Type))
                .and_then(|f| f.as_facet().map(|f| f.to_owned()))
                .bad_err("bad facet")?,
            ctime: doc
                .get_first(db.get_field(Fields::Ctime))
                .and_then(|e| e.as_u64())
                .bad_err("bad ctime")?,
            last_update: doc
                .get_first(db.get_field(Fields::Mtime))
                .and_then(|e| e.as_u64())
                .bad_err("bad last_update")?,
            last_interaction: doc
                .get_first(db.get_field(Fields::LastInteraction))
                .and_then(|e| e.as_u64())
                .bad_err("bad last_interaction")?,
            data: DbAble::take(db, &mut *doc)?,
        };
        Ok(m)
    }
}
impl<T: DbAble> DbAble for Meta<T, TypeFacet> {
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        let m = Meta {
            id: self.id,
            facet: self.facet.facet(),
            data: self.data,
            ctime: self.ctime,
            last_update: self.last_update,
            last_interaction: self.last_interaction,
        };
        m.add(db, doc)
    }

    fn take(db: &AppDatabase, doc: &mut Document) -> Result<Self, Error> {
        let m: Meta<T, Facet> = DbAble::take(db, doc)?;
        Ok(Self {
            id: m.id,
            facet: TypeFacet::try_from(m.facet.to_path_string())
                .ok()
                .bad_err("could not convert to TypeFacet")?,
            data: m.data,
            ctime: m.ctime,
            last_update: m.last_update,
            last_interaction: m.last_interaction,
        })
    }
}

impl<T: DbAble> DbAble for Taggable<T> {
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        self.tags.into_iter().for_each(|t| {
            doc.add_u64(db.get_field(Fields::Tag), t as _);
        });
        self.data.add(db, doc)
    }

    fn take(db: &AppDatabase, doc: &mut Document) -> Result<Self, Error> {
        let t = Self {
            tags: doc
                .get_all(db.get_field(Fields::Tag))
                .map(|e| e.as_u64().map(|e| e as _))
                .collect::<Option<Vec<_>>>()
                .bad_err("bad tags")?,
            data: DbAble::take(db, &mut *doc)?,
        };
        Ok(t)
    }
}

impl<T> DbAble for T
where
    T: AutoDbAble,
{
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        let v = match serde_json::to_value(self).infer_err()? {
            serde_json::Value::Object(o) => o,
            _ => return None.bad_err("bad json object :/"),
        };
        doc.add_json_object(db.get_field(Fields::Json), v);
        Ok(())
    }

    fn take(db: &AppDatabase, doc: &mut Document) -> Result<Self, Error> {
        let j = doc
            .get_first(db.get_field(Fields::Json))
            .bad_err("no Json in document")?
            .as_json()
            .bad_err("value is not a Map")?;
        let j = serde_json::from_value(serde_json::Value::Object(j.to_owned())).infer_err()?;
        Ok(j)
    }
}

impl<T: DbAble> DbAble for SearchableEntry<T> {
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        self.searchable
            .into_iter()
            .try_for_each(|e| e.add(db, &mut *doc))?;
        self.data.add(db, doc)?;
        Ok(())
    }

    fn take(db: &AppDatabase, doc: &mut Document) -> Result<Self, Error> {
        let s = Self {
            data: DbAble::take(db, &mut *doc)?,
            searchable: doc
                .get_all(db.get_field(Fields::Text))
                .filter_map(|e| e.as_text().map(String::from))
                .map(|v| Indexed {
                    field: Fields::Text,
                    data: serde_json::Value::String(v),
                })
                .collect::<Vec<_>>(),
        };
        Ok(s)
    }
}

impl DbAble for Indexed {
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        match (self.field, self.data) {
            // (kolekk_types::objects::Fields::Id, serde_json::Value::Number(n)) if let Some(n) = n.as_u64() => {}
            // (kolekk_types::objects::Fields::Type, serde_json::Value::String(s)) => {}
            // (kolekk_types::objects::Fields::Ctime, serde_json::Value::Number(n)) if let Some(n) = n.as_u64() => {}
            // (kolekk_types::objects::Fields::Mtime, serde_json::Value::Number(n)) if let Some(n) = n.as_u64() => {}
            // (kolekk_types::objects::Fields::Chksum, serde_json::Value::Array(a)) if let Some(a) = a.into_iter().map(|e|
            //     e.as_u64().filter(|&e| e <= u8::MAX as _).map(|e| e as u8)
            // ).try_collect::<Vec<_>>() => {}
            (Fields::Text, serde_json::Value::String(s)) => {
                doc.add_text(db.get_field(Fields::Text), s);
            }
            _ => {
                return None.bad_err("invalid searchable entry");
            }
        }
        Ok(())
    }

    fn take(_db: &AppDatabase, _doc: &mut Document) -> Result<Self, Error> {
        unreachable!();
    }
}

// TODO:
// #[tauri::command]
// pub async fn update_object(data: Meta<Taggable<Object>>) {
//     match data.data.data {
//         Object::....
//     }
// }

// https://docs.rs/tantivy/0.19.2/tantivy/struct.IndexReader.html#method.reload
// docs say that automatic reloads may take a smol while to take effect.
#[tauri::command]
pub async fn reload_reader(db: State<'_, AppDatabase>) -> Result<(), Error> {
    db.index_reader.reload().infer_err()
}

pub fn direct_search<T, TScore, TScoreSegmentTweaker, TScoreTweaker>(
    db: &AppDatabase,
    ob_type: TypeFacet,
    query: impl AsRef<str>,
    limit: usize,
    offset: usize,
    search_tweaker: TScoreTweaker,
) -> Result<Vec<T>, Error>
where
    T: DbAble + Debug,
    TScore: 'static + Send + Sync + Clone + PartialOrd + Debug,
    TScoreSegmentTweaker: ScoreSegmentTweaker<TScore> + 'static,
    TScoreTweaker: ScoreTweaker<TScore, Child = TScoreSegmentTweaker> + Send + Sync,
{
    let searcher = db.get_searcher();
    let query = query.as_ref();

    let q = BooleanQuery::new(vec![
        (
            Occur::Must,
            Box::new(TermQuery::new(
                Term::from_facet(db.get_field(Fields::Type), &ob_type.facet()),
                IndexRecordOption::Basic,
            )) as _,
        ),
        (
            Occur::Should,
            Box::new(BooleanQuery::new(
                query
                    .split_whitespace()
                    .flat_map(|t| {
                        [
                            (
                                Occur::Should,
                                Box::new(FuzzyTermQuery::new(
                                    Term::from_field_text(db.get_field(Fields::Text), t),
                                    2,    // ?
                                    true, // what??
                                )) as _,
                            ),
                            (
                                Occur::Should,
                                Box::new(BoostQuery::new(
                                    Box::new(TermQuery::new(
                                        Term::from_field_text(db.get_field(Fields::Text), t),
                                        IndexRecordOption::Basic,
                                    )) as _,
                                    2.0,
                                )) as _,
                            ),
                        ]
                        .into_iter()
                    })
                    .collect(),
            )),
        ),
    ]);
    searcher
        .search(
            &q,
            &TopDocs::with_limit(limit)
                .and_offset(offset)
                .tweak_score(search_tweaker),
        )
        .infer_err()?
        .into_iter()
        .map(|(score, address)| {
            let mut doc = searcher.doc(address).infer_err()?;
            DbAble::take(db, &mut doc).look(|e| dbg!(score, e))
        })
        .collect::<Result<Vec<_>, _>>()
}

// TODO: most recent interations | total interactions
pub fn tagged_search<T, TScore, TScoreSegmentTweaker, TScoreTweaker>(
    db: &AppDatabase,
    ob_type: TypeFacet,
    query: impl AsRef<str>,
    limit: usize,
    offset: usize,
    search_tweaker: TScoreTweaker,
) -> Result<Vec<T>, Error>
where
    T: DbAble + Debug,
    TScore: 'static + Send + Sync + Clone + PartialOrd + Debug,
    TScoreSegmentTweaker: ScoreSegmentTweaker<TScore> + 'static,
    TScoreTweaker: ScoreTweaker<TScore, Child = TScoreSegmentTweaker> + Send + Sync,
{
    // TODO: look into using QueryParser, or maybe somehow including regex queries and stuff
    // TODO: is query is empty, return results by recently added or recently viewed or a mixture of this stuff
    //       - 2 fast values, 1 each for when added timestamp and for last viewed timestamp
    //       - timestamps are not really needed, so just an increasing counter will also do

    // facet:TermQuery AND (
    //   title:PhraseQuery(entire query) -> priority 0
    //   // OR tag(s):TermQuery(split at whiltespace map) -> priority 1
    //   OR tag(s):FuzzyTermQuery(split at whitespace map) -> priority 2
    //   OR title:FuzzyTermQuery(split at whitespace map) -> priority 3
    // )
    let searcher = db.get_searcher();
    let query = query.as_ref();

    let obj_type_query = Box::new(TermQuery::new(
        Term::from_facet(db.get_field(Fields::Type), &ob_type.facet()),
        IndexRecordOption::Basic,
    ));

    let phrase_query_terms = query
        .split_whitespace()
        .map(|t| Term::from_field_text(db.get_field(Fields::Text), t))
        .collect::<Vec<_>>();
    let title_query = if phrase_query_terms.len() < 2 {
        // PhraseQuery does not support less than 2 terms
        Box::new(TermQuery::new(
            Term::from_field_text(db.get_field(Fields::Text), query),
            IndexRecordOption::Basic,
        )) as _
    } else {
        Box::new(PhraseQuery::new(phrase_query_terms)) as _
    };

    // - search tags
    // - replace all alias tags by main
    // - map tags to ids
    // - dedup tag ids while keeping the ones that appear first
    // - only consider first n tags / tags above a certain score
    // - convert these into queries (term queries + boost with the score)
    // - search items for this query
    let mut tag_set = HashSet::new();
    let tag_prequery = BooleanQuery::new(vec![
        (
            Occur::Must,
            Box::new(TermQuery::new(
                Term::from_facet(db.get_field(Fields::Type), &TypeFacet::Tag.facet()),
                IndexRecordOption::Basic,
            )) as _,
        ),
        (
            Occur::Must,
            Box::new(BooleanQuery::new(
                query
                    .split_whitespace()
                    .flat_map(|t| {
                        [
                            (
                                Occur::Should,
                                Box::new(FuzzyTermQuery::new(
                                    Term::from_field_text(db.get_field(Fields::Text), t),
                                    2,    // ?
                                    true, // what??
                                )) as _,
                            ),
                            (
                                Occur::Should,
                                Box::new(BoostQuery::new(
                                    Box::new(TermQuery::new(
                                        Term::from_field_text(db.get_field(Fields::Text), t),
                                        IndexRecordOption::Basic,
                                    )) as _,
                                    2.0,
                                )) as _,
                            ),
                        ]
                        .into_iter()
                    })
                    .collect(),
            )),
        ),
    ]);
    let tags = searcher
        .search(&tag_prequery, &TopDocs::with_limit(20).and_offset(0))
        .infer_err()?
        .into_iter()
        .map(|(score, address)| {
            let mut doc = searcher.doc(address).infer_err()?;
            let t: Meta<Tag, Facet> = DbAble::take(db, &mut doc).look(|e| dbg!(e))?;
            let t = match t.data {
                Tag::Main { .. } => t.id,
                Tag::Alias { alias_to, .. } => alias_to,
            };
            Ok((score, t, tag_set.insert(t)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let tags = tags
        .into_iter()
        .enumerate()
        .filter(|(_, (_, _, f))| *f)
        .map(|(_i, (score, t, _))| (score, t))
        .map(|(score, t)| {
            (
                Occur::Should,
                Box::new(BoostQuery::new(
                    Box::new(TermQuery::new(
                        Term::from_field_u64(db.get_field(Fields::Tag), t as _),
                        IndexRecordOption::Basic,
                    )) as _,
                    score,
                )) as _,
            )
        })
        .collect();
    let tag_query = Box::new(BooleanQuery::new(tags));

    let title_fuzzy_query = Box::new(BooleanQuery::new(
        // TODO: implement these as methods of Fields and ObjectType
        query
            .split_whitespace()
            .flat_map(|t| {
                [
                    (
                        Occur::Should, // TODO: should this be Must instead?
                        Box::new(FuzzyTermQuery::new(
                            Term::from_field_text(db.get_field(Fields::Text), t),
                            2,    // ?
                            true, // what??
                        )) as _,
                    ),
                    (
                        Occur::Should,
                        Box::new(BoostQuery::new(
                            Box::new(TermQuery::new(
                                Term::from_field_text(db.get_field(Fields::Text), t),
                                IndexRecordOption::Basic,
                            )) as _,
                            2.0,
                        )) as _,
                    ),
                ]
                .into_iter()
            })
            .collect(),
    ));

    let search_query = Box::new(BooleanQuery::new(vec![
        // TODO: the priority stuff
        (Occur::Should, title_query),
        (Occur::Should, Box::new(BoostQuery::new(tag_query, 2.0))),
        (Occur::Should, title_fuzzy_query),
    ]));

    searcher
        .search(
            &BooleanQuery::new(vec![
                (Occur::Must, obj_type_query),
                (Occur::Should, Box::new(AllQuery)),
                (Occur::Should, search_query),
            ]),
            &TopDocs::with_limit(limit)
                .and_offset(offset)
                .tweak_score(search_tweaker),
        )
        .infer_err()?
        .into_iter()
        .map(move |(_score, address)| {
            let mut doc = searcher.doc(address).infer_err()?;
            DbAble::take(db, &mut doc).look(|e| dbg!((_score, e)))
        })
        .collect()
}

pub struct TagSearchScoreTweaker {
    pub now: u64,
    pub last_interaction: Field,
    pub id_field: Field,
}

impl TagSearchScoreTweaker {
    pub fn new(db: &AppDatabase) -> Result<Self, Error> {
        let id_field = db.get_field(Fields::Id);
        let last_interaction = db.get_field(Fields::LastInteraction);
        let now = db.now_time()?;
        let s = Self {
            now,
            last_interaction,
            id_field,
        };
        Ok(s)
    }
}

pub struct TagSearchScoreSegmentTweaker {
    pub last_interaction: Arc<dyn Column<u64>>,
    pub id_reader: Arc<dyn Column<u64>>,
    pub now: u64,
}

type TagSearchTweakedScore = (tantivy::Score, u64, u64);

impl ScoreSegmentTweaker<TagSearchTweakedScore> for TagSearchScoreSegmentTweaker {
    fn score(&mut self, doc: tantivy::DocId, score: tantivy::Score) -> TagSearchTweakedScore {
        let last_interaction = self.last_interaction.get_val(doc);
        let id = self.id_reader.get_val(doc);

        // PartialOrd on tuples: https://stackoverflow.com/a/61323034
        (score, last_interaction, id)
    }
}
impl ScoreTweaker<TagSearchTweakedScore> for TagSearchScoreTweaker {
    type Child = TagSearchScoreSegmentTweaker;

    fn segment_tweaker(&self, segment_reader: &SegmentReader) -> tantivy::Result<Self::Child> {
        let last_interaction = segment_reader.fast_fields().u64(self.last_interaction)?;
        let id_reader = segment_reader.fast_fields().u64(self.id_field)?;

        let tw = TagSearchScoreSegmentTweaker {
            last_interaction,
            id_reader,
            now: self.now,
        };
        Ok(tw)
    }
}

pub struct ObjectSearchScoreTweaker {
    pub now: u64,
    pub ctime_field: Field,
    pub id_field: Field,
}

impl ObjectSearchScoreTweaker {
    pub fn new(db: &AppDatabase) -> Result<Self, Error> {
        let ctime_field = db.get_field(Fields::Ctime);
        let id_field = db.get_field(Fields::Id);
        let now = db.now_time()?;
        let s = Self {
            now,
            ctime_field,
            id_field,
        };
        Ok(s)
    }
}

pub struct ObjectSearchScoreSegmentTweaker {
    pub ctime_reader: Arc<dyn Column<u64>>,
    pub id_reader: Arc<dyn Column<u64>>,
    pub now: u64,
}

type ObjectSearchTweakedScore = (tantivy::Score, u64, u64);

impl ScoreSegmentTweaker<ObjectSearchTweakedScore> for ObjectSearchScoreSegmentTweaker {
    fn score(&mut self, doc: tantivy::DocId, score: tantivy::Score) -> ObjectSearchTweakedScore {
        let ctime = self.ctime_reader.get_val(doc);
        let id = self.id_reader.get_val(doc);

        // https://www.desmos.com/calculator/nqrwqablae
        // let sub = (self.now - ctime) as f32;
        // let days = sub / (60.0 * 60.0 * 24.0);
        // let pow = -(days / 8.0);
        // let epow = std::f64::consts::E.powf(pow as _);
        // let sigmoid = 1.0 / (1.0 + epow);
        // let fin = 1.0 - (sigmoid - 0.5) * 2.0;

        // let s = format!("{now} {ctime} {days} {pow} {epow} {sigmoid} {fin}");
        // dbg!(s);

        // PartialOrd on tuples: https://stackoverflow.com/a/61323034
        (score, ctime, id) // id for - fallback order consistency
    }
}
impl ScoreTweaker<ObjectSearchTweakedScore> for ObjectSearchScoreTweaker {
    type Child = ObjectSearchScoreSegmentTweaker;

    fn segment_tweaker(&self, segment_reader: &SegmentReader) -> tantivy::Result<Self::Child> {
        let ctime_reader = segment_reader.fast_fields().u64(self.ctime_field)?;
        let id_reader = segment_reader.fast_fields().u64(self.id_field)?;

        let tw = ObjectSearchScoreSegmentTweaker {
            ctime_reader,
            id_reader,
            now: self.now,
        };
        Ok(tw)
    }
}

pub async fn init_database(app_handle: &AppHandle, conf: &AppConfig) -> Result<(), Error> {
    let handle = app_handle.app_handle();

    let mut db = AppDatabase::new(conf).await?;

    let searcher = db.get_searcher();
    let state: AppDatabaseState = searcher
        .search(
            &TermQuery::new(
                Term::from_facet(
                    db.get_field(Fields::Type),
                    &TypeFacet::Temp("/app_data/state".into()).facet(),
                ),
                IndexRecordOption::Basic,
            ),
            &TopDocs::with_limit(1),
        )
        .infer_err()?
        .first()
        .and_then(|&(_, add)| searcher.doc(add).ok())
        .and_then(|mut doc| DbAble::take(&db, &mut doc).ok())
        .unwrap_or_default();
    db.update_state(state);

    app_handle.manage(db);

    // TODO: ugly unwraps :(
    app_handle
        .windows()
        .into_values()
        .next()
        .expect("no window?")
        .on_window_event(move |e| match e {
            WindowEvent::Destroyed | WindowEvent::CloseRequested { .. } => {
                let db = handle.state::<AppDatabase>().inner();

                // TODO: Temp really?
                let facet = TypeFacet::Temp("/app_data/state".into()).facet();

                let mut writer = db.index_writer.write().unwrap();
                let _opstamp =
                    writer.delete_term(Term::from_facet(db.get_field(Fields::Type), &facet));

                let mut doc = Document::new();
                doc.add_facet(db.get_field(Fields::Type), facet);

                // TODO: this does not gurantee that ids won't be repeated after app reboot as
                //    some ids may get created after this thing is saved in the database
                //    not clear how to solve this problem as it is not possible to unhandle anything from tauri
                //    maybe just panic inside new_id()?
                db.get_state().add(db, &mut doc).unwrap();

                dbg!("saving appdatabase state");

                let _opstamp = writer
                    .add_document(doc)
                    .expect("eror: failed to add cache document to tantivy");
                let _opstamp = writer
                    .commit()
                    .expect("eror: failed to commit changes to tantivy");
            }
            _ => {}
        });
    Ok(())
}

pub struct AppDatabase {
    // sql: DatabaseConnection,
    index: Index,
    index_reader: IndexReader,
    pub index_writer: RwLock<IndexWriter>, // TODO: make commits explicit (don't commit in add_object functions. commit should be called when needed explicitly)
    fields: HashMap<Fields, Field>,
    id_gen: AtomicU32,
}

// the state that persists
#[derive(Serialize, Deserialize, Default)]
struct AppDatabaseState {
    id_gen: u32,
}

impl AutoDbAble for AppDatabaseState {}

impl AppDatabase {
    pub async fn new(config: &AppConfig) -> Result<Self, Error> {
        // - [create new db file sea-orm](https://github.com/SeaQL/sea-orm/discussions/283#discussioncomment-1564939)
        let db_dir = config.app_data_dir.join("database");
        if !db_dir.exists() {
            std::fs::create_dir(&db_dir).infer_err()?;
        }
        let tantivy_dir = db_dir.join("tantivy");
        if !tantivy_dir.exists() {
            std::fs::create_dir(&tantivy_dir).infer_err()?;
        }

        // let db_path = db_dir.join("kolekdb.db");
        // let new_db = !db_path.exists();

        // let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        // // let db_url = "sqlite::memory:";
        // let db = Database::connect(db_url)
        //     .await
        //     .look(|e| dbg!(e))
        //     .infer_err()?;

        // if new_db {
        //     let backend = db.get_database_backend();
        //     let schema = sea_orm::Schema::new(backend);
        //     // dbg!(backend.build(&table).to_string());
        //     // let table = schema.create_table_from_entity(kolekk_types::images::Entity);
        //     // let _ = db.execute(backend.build(&table)).await.unwrap();
        //     // let table = schema.create_table_from_entity(kolekk_types::tags::Entity);
        //     // let _ = db.execute(backend.build(&table)).await.unwrap();
        //     // let table = schema.create_table_from_entity(kolekk_types::urls::Entity);
        //     // let _ = db.execute(backend.build(&table)).await.unwrap();
        //     // let table = schema.create_table_from_entity(kolekk_types::metadata::Entity);
        //     // let _ = db.execute(backend.build(&table)).await.unwrap();
        // }

        let mut schema_builder = tantivy::schema::Schema::builder();

        let mut fields = HashMap::<Fields, Field>::new();

        let id = schema_builder.add_u64_field(&Fields::Id, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::Id, id);
        let object_type =
            schema_builder.add_facet_field(&Fields::Type, FacetOptions::default().set_stored());
        let _ = fields.insert(Fields::Type, object_type);
        let text = schema_builder.add_text_field(&Fields::Text, STORED | TEXT);
        let _ = fields.insert(Fields::Text, text);
        let chksum = schema_builder.add_bytes_field(&Fields::Chksum, STORED);
        let _ = fields.insert(Fields::Chksum, chksum);
        let tag = schema_builder.add_u64_field(&Fields::Tag, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::Tag, tag);
        let json = schema_builder.add_json_field(&Fields::Json, STORED);
        let _ = fields.insert(Fields::Json, json);
        let ctime = schema_builder.add_u64_field(&Fields::Ctime, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::Ctime, ctime);
        let mtime = schema_builder.add_u64_field(&Fields::Mtime, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::Mtime, mtime);
        let last_interaction =
            schema_builder.add_u64_field(&Fields::LastInteraction, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::LastInteraction, last_interaction);

        let schema = schema_builder.build();
        let dir = ManagedDirectory::wrap(Box::new(MmapDirectory::open(tantivy_dir).infer_err()?))
            .infer_err()?;
        let index = Index::open_or_create(dir, schema).infer_err()?;
        let index_writer = index.writer(50_000_000).infer_err()?;

        let index_reader = index.reader_builder().try_into().infer_err()?;

        Ok(AppDatabase {
            // sql: db,
            index_reader,
            index_writer: RwLock::new(index_writer),
            index,
            fields,
            id_gen: 0.into(),
        })
    }

    fn get_state(&self) -> AppDatabaseState {
        AppDatabaseState {
            id_gen: self.id_gen.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    fn update_state(&mut self, state: AppDatabaseState) {
        self.id_gen = state.id_gen.into();
    }

    pub fn get_field(&self, f: Fields) -> Field {
        *self.fields.get(&f).unwrap()
    }

    pub fn get_searcher(&self) -> Searcher {
        Searcher(self.index_reader.searcher())
    }

    pub fn get_doc_address(&self, id: Id) -> Result<DocAddress, Error> {
        let searcher = self.get_searcher();
        let id_term = Term::from_field_u64(self.get_field(Fields::Id), id as _);
        let top_docs = searcher
            .search(
                &TermQuery::new(id_term, IndexRecordOption::Basic),
                &TopDocs::with_limit(1),
            )
            .look(|e| dbg!(e))
            .infer_err()?;
        let (_score, doc_address) = top_docs.first().bad_err("object does not exist")?;
        Ok(*doc_address)
    }

    pub fn get_doc(&self, id: Id) -> Result<Document, Error> {
        let searcher = self.get_searcher();
        let doc = searcher.doc(self.get_doc_address(id)?).infer_err()?;
        Ok(doc)
    }

    pub fn new_id(&self) -> Id {
        self.id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn now_time(&self) -> Result<u64, Error> {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .infer_err()?
            .as_secs();
        Ok(secs)
    }
}

pub trait FacetFrom
where
    Self: AsRef<str>,
{
    fn facet(&self) -> Facet {
        self.as_ref().into()
    }
}
impl<T> FacetFrom for T where T: AsRef<str> {}

// newtype pattern so i can implement more methods on it
pub struct Searcher(tantivy::Searcher);

impl Deref for Searcher {
    type Target = tantivy::Searcher;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
