#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Deref,
    sync::{atomic::AtomicU32, Mutex},
};

use kolekk_types::{
    objects::{
        Bookmark, Fields, Id, Image, Indexed, Meta, SearchableEntry, Tag, Taggable, TypeFacet,
    },
    utility::Path,
};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use tantivy::{
    collector::TopDocs,
    directory::{ManagedDirectory, MmapDirectory},
    query::{AllQuery, BooleanQuery, BoostQuery, FuzzyTermQuery, Occur, PhraseQuery, TermQuery},
    schema::{Facet, FacetOptions, Field, IndexRecordOption, FAST, INDEXED, STORED, TEXT},
    DocAddress, Document, Index, IndexReader, IndexWriter, Term,
};
use tauri::{State, WindowEvent, AppHandle, Manager};

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
    let mut writer = db.index_writer.lock().infer_err()?;
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
    data.into_iter().try_for_each(|e| {
        let mut doc = Document::new();
        doc.add_facet(db.get_field(Fields::Type), facet.facet());
        let v = Meta {
            id: db.new_id(),
            data: Taggable {
                data: e,
                tags: vec![],
            },
            ctime,
            last_update: ctime,
            last_interaction: ctime,
        };
        v.add(db.inner(), &mut doc).look(|e| dbg!(e))?;

        // TODO: too much locking and unlocking too quickly
        let _opstamp = db
            .index_writer
            .lock()
            .infer_err()?
            .add_document(doc)
            .look(|e| dbg!(e))
            .infer_err()?;
        // TODO: if err, do i remove all those that succeeded?
        Ok(())
    })?;
    let _opstamp = db.index_writer.lock().infer_err()?.commit().infer_err()?;
    Ok(())
}

#[tauri::command]
pub async fn search_jsml_object(
    db: State<'_, AppDatabase>,
    query: String,
    facet: TypeFacet,
    limit: usize,
    offset: usize,
) -> Result<Vec<Meta<Taggable<serde_json::Map<String, serde_json::Value>>>>, Error> {
    crate::database::search_object(db.inner(), facet, query, limit, offset)
}

#[tauri::command]
pub async fn add_tag_to_object(
    db: State<'_, AppDatabase>,
    id: Id,
    tag_id: Id,
) -> Result<(), Error> {
    let mut doc = db.get_doc(id)?;

    let mut j: Meta<Taggable<tantivy::schema::Value>> = DbAble::take(db.inner(), &mut doc)?;
    j.data.tags.push(tag_id);
    let mut doc = Document::new();
    j.add(db.inner(), &mut doc)?;

    let mut writer = db.index_writer.lock().infer_err()?;
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

    let mut j: Meta<Taggable<tantivy::schema::Value>> = DbAble::take(db.inner(), &mut doc)?;
    j.data.tags.retain(|&t| t != tag_id);
    let mut doc = Document::new();
    j.add(db.inner(), &mut doc)?;

    let mut writer = db.index_writer.lock().infer_err()?;
    let _opstamp = writer.delete_term(Term::from_field_u64(db.get_field(Fields::Id), id as _));
    let _opstamp = writer.add_document(doc).infer_err()?;
    let _opstamp = writer.commit().infer_err()?;
    Ok(())
}

#[tauri::command]
pub fn get_path(config: State<'_, AppConfig>, path: Path) -> std::path::PathBuf {
    crate::filesystem::get_path(&path, config.inner())
}

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

impl<T: DbAble> DbAble for Meta<T> {
    fn add(self, db: &AppDatabase, doc: &mut Document) -> Result<(), Error> {
        doc.add_u64(db.get_field(Fields::Id), self.id as _);
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

// TODO: when query is empty, sort by recently added / most recent interations / total interactions + most recent interactions
//       collect this stat inside the objects
pub fn search_object<T: DbAble + Debug>(
    db: &AppDatabase,
    ob_type: TypeFacet,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<T>, Error> {
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
            Term::from_field_text(db.get_field(Fields::Text), &query),
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
                    .map(|t| {
                        (
                            Occur::Should,
                            Box::new(FuzzyTermQuery::new(
                                Term::from_field_text(db.get_field(Fields::Text), t),
                                2,    // ?
                                true, // what??
                            )) as _,
                        )
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
            let mut doc = searcher.doc(address).look(|e| dbg!(e)).infer_err()?;
            let t: Meta<Tag> = DbAble::take(db, &mut doc)?;
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
            .map(|t| {
                (
                    Occur::Should, // TODO: should this be Must instead?
                    Box::new(FuzzyTermQuery::new(
                        Term::from_field_text(db.get_field(Fields::Text), t),
                        2,    // ?
                        true, // what??
                    )) as _,
                )
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
            &TopDocs::with_limit(limit).and_offset(offset),
        )
        .infer_err()?
        .into_iter()
        .map(move |(_score, address)| {
            let mut doc = searcher.doc(address).infer_err()?;
            DbAble::take(db, &mut doc).look(|e| dbg!((_score, e)))
        })
        .collect()
}

pub async fn init_database(
    app_handle: &AppHandle,
    conf: &AppConfig,
) -> Result<(), Error> {
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

                let mut writer = db.index_writer.lock().unwrap();
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
    pub index_writer: Mutex<IndexWriter>, // TODO: RWlock + make commits explicit (don't commit in add_object functions. commit should be called when needed explicitly)
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
        let object_type = schema_builder.add_facet_field(&Fields::Type, FacetOptions::default());
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
            index_writer: Mutex::new(index_writer),
            index,
            fields,
            id_gen: 0.into(),
        })
    }

    fn get_state(&self) -> AppDatabaseState {
        AppDatabaseState { id_gen: self.id_gen.load(std::sync::atomic::Ordering::Relaxed) }
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
