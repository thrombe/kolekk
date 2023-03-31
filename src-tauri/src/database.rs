#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::{atomic::AtomicU32, Mutex},
};

use kolekk_types::{Bookmark, Image, Tag};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use tantivy::{
    collector::TopDocs,
    query::{AllQuery, BooleanQuery, BoostQuery, FuzzyTermQuery, Occur, PhraseQuery, TermQuery},
    schema::{Facet, FacetOptions, Field, IndexRecordOption, FAST, INDEXED, STORED, TEXT},
    DocAddress, Document, Index, IndexReader, IndexWriter, Term,
};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    config::AppConfig,
};

// is async cuz the other db might require it in future
pub async fn add_image(db: &AppDatabase, img: Image) -> Result<(), Error> {
    let mut doc = Document::new();
    doc.add_facet(db.get_field(Fields::ObjectType), ObjectType::Image);
    doc.add_u64(db.get_field(Fields::Id), img.id as _);
    doc.add_text(db.get_field(Fields::Title), img.title);
    doc.add_text(db.get_field(Fields::SrcPath), img.src_path);
    doc.add_text(db.get_field(Fields::DbPath), img.db_path);
    doc.add_bytes(db.get_field(Fields::Chksum), img.chksum);
    doc.add_u64(db.get_field(Fields::Size), img.size as _);
    img.tags
        .into_iter()
        .for_each(|t| doc.add_u64(db.get_field(Fields::Tag), t as _));
    img.urls
        .into_iter()
        .for_each(|u| doc.add_text(db.get_field(Fields::Url), u));

    let mut writer = db.index_writer.lock().infer_err()?;
    let _opstamp = writer.add_document(doc).look(|e| dbg!(e)).infer_err()?;
    writer.commit().look(|e| dbg!(e)).infer_err()?;
    Ok(())
}

// is async cuz the other db might require it in future
pub async fn add_bookmark(db: &AppDatabase, bk: Bookmark) -> Result<(), Error> {
    let mut doc = Document::new();
    let json = serde_json::to_value(&bk).infer_err().look(|e| dbg!(e))?;
    doc.add_json_object(
        db.get_field(Fields::Json),
        json.as_object().bad_err("cannot fail")?.to_owned(),
    );
    doc.add_facet(db.get_field(Fields::ObjectType), ObjectType::Bookmark);
    doc.add_u64(db.get_field(Fields::Id), bk.id as _);
    let _ = bk
        .title
        .map(|t| doc.add_text(db.get_field(Fields::Title), t));
    let _ = bk
        .description
        .map(|d| doc.add_text(db.get_field(Fields::Description), d));
    bk.tags
        .into_iter()
        .for_each(|t| doc.add_u64(db.get_field(Fields::Tag), t as _));
    bk.related
        .into_iter()
        .for_each(|r| doc.add_u64(db.get_field(Fields::Related), r as _));

    let mut writer = db.index_writer.lock().infer_err()?;
    let _opstamp = writer.add_document(doc).look(|e| dbg!(e)).infer_err()?;
    writer.commit().look(|e| dbg!(e)).infer_err()?;
    Ok(())
}

// is async cuz the other db might require it in future
pub async fn add_tag(db: &AppDatabase, tag: Tag) -> Result<(), Error> {
    let mut doc = Document::new();
    let json = serde_json::to_value(&tag).infer_err().look(|e| dbg!(e))?;
    doc.add_json_object(
        db.get_field(Fields::Json),
        json.as_object().bad_err("cannot fail")?.to_owned(),
    );
    doc.add_facet(db.get_field(Fields::ObjectType), ObjectType::Tag);
    doc.add_u64(
        db.get_field(Fields::Id),
        match tag {
            Tag::Main { id, .. } => id,
            Tag::Alias { id, .. } => id,
        } as _,
    );
    doc.add_text(
        db.get_field(Fields::Title),
        match tag {
            Tag::Main { name, .. } => name,
            Tag::Alias { name, .. } => name,
        },
    );

    let mut writer = db.index_writer.lock().infer_err()?;
    let _opstamp = writer.add_document(doc).look(|e| dbg!(e)).infer_err()?;
    writer.commit().look(|e| dbg!(e)).infer_err()?;
    Ok(())
}

// TODO: when query is empty, sort by recently added / most recent interations / total interactions + most recent interactions
//       collect this stat inside the objects
pub fn search_object(
    db: &AppDatabase,
    ob_type: ObjectType,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
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
        Term::from_facet(db.get_field(Fields::ObjectType), &ob_type.into()),
        IndexRecordOption::Basic,
    ));

    let phrase_query_terms = query
        .split_whitespace()
        .map(|t| Term::from_field_text(db.get_field(Fields::Title), t))
        .collect::<Vec<_>>();
    let title_query = if phrase_query_terms.len() < 2 {
        // PhraseQuery does not support less than 2 terms
        Box::new(TermQuery::new(
            Term::from_field_text(db.get_field(Fields::Title), &query),
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
                Term::from_facet(db.get_field(Fields::ObjectType), &ObjectType::Tag.into()),
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
                                Term::from_field_text(db.get_field(Fields::Title), t),
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
        .infer_err()
        .look(|e| dbg!(e))?
        .into_iter()
        .map(|(score, address)| {
            let doc = searcher.doc(address).look(|e| dbg!(e)).infer_err()?;
            let t = doc
                .get_first(db.get_field(Fields::Json))
                .bad_err("no value")?
                .as_json()
                .bad_err("not an object")?
                .to_owned();
            let t = serde_json::from_value::<Tag>(serde_json::Value::Object(t)).infer_err()?;
            let t = match t {
                Tag::Main { id, .. } => id,
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
                        Term::from_field_text(db.get_field(Fields::Title), t),
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
        .infer_err()
        .look(|e| dbg!(e))?
        .into_iter()
        .map(move |(_score, address)| {
            let doc = searcher.doc(address).look(|e| dbg!(e)).infer_err()?;
            Ok(doc
                .get_first(db.get_field(Fields::Json))
                .bad_err("no value")?
                .as_json()
                .bad_err("not an object")?
                .to_owned())
        })
        .collect()
}

pub struct AppDatabase {
    sql: DatabaseConnection,
    index: Index,
    index_reader: IndexReader,
    pub index_writer: Mutex<IndexWriter>, // TODO: RWlock + make commits explicit (don't commit in add_object functions. commit should be called when needed explicitly)
    fields: HashMap<Fields, Field>,
    id_gen: AtomicU32,
}

impl AppDatabase {
    pub async fn new(config: &AppConfig) -> Result<Self, Error> {
        // - [create new db file sea-orm](https://github.com/SeaQL/sea-orm/discussions/283#discussioncomment-1564939)
        let db_dir = config.app_data_dir.join("database");
        if !db_dir.exists() {
            std::fs::create_dir(&db_dir).infer_err().look(|e| dbg!(e))?;
        }

        let db_path = db_dir.join("kolekdb.db");
        let new_db = !db_path.exists();

        let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        // let db_url = "sqlite::memory:";
        let db = Database::connect(db_url)
            .await
            .look(|e| dbg!(e))
            .infer_err()?;

        if new_db {
            let backend = db.get_database_backend();
            let schema = sea_orm::Schema::new(backend);
            // dbg!(backend.build(&table).to_string());
            // let table = schema.create_table_from_entity(kolekk_types::images::Entity);
            // let _ = db.execute(backend.build(&table)).await.unwrap();
            // let table = schema.create_table_from_entity(kolekk_types::tags::Entity);
            // let _ = db.execute(backend.build(&table)).await.unwrap();
            // let table = schema.create_table_from_entity(kolekk_types::urls::Entity);
            // let _ = db.execute(backend.build(&table)).await.unwrap();
            // let table = schema.create_table_from_entity(kolekk_types::metadata::Entity);
            // let _ = db.execute(backend.build(&table)).await.unwrap();
        }

        let mut schema_builder = tantivy::schema::Schema::builder();

        let mut fields = HashMap::<Fields, Field>::new();

        let id = schema_builder.add_u64_field(&Fields::Id, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::Id, id);
        let src_path = schema_builder.add_text_field(&Fields::SrcPath, STORED);
        let _ = fields.insert(Fields::SrcPath, src_path);
        let db_path = schema_builder.add_text_field(&Fields::DbPath, STORED);
        let _ = fields.insert(Fields::DbPath, db_path);
        let url = schema_builder.add_text_field(&Fields::Url, STORED);
        let _ = fields.insert(Fields::Url, url);
        let title = schema_builder.add_text_field(&Fields::Title, STORED | TEXT);
        let _ = fields.insert(Fields::Title, title);
        let description = schema_builder.add_text_field(&Fields::Description, STORED | TEXT);
        let _ = fields.insert(Fields::Description, description);
        let tag = schema_builder.add_u64_field(&Fields::Tag, STORED | FAST | INDEXED);
        let _ = fields.insert(Fields::Tag, tag);
        let object_type =
            schema_builder.add_facet_field(&Fields::ObjectType, FacetOptions::default());
        let _ = fields.insert(Fields::ObjectType, object_type);
        let chksum = schema_builder.add_bytes_field(&Fields::Chksum, STORED);
        let _ = fields.insert(Fields::Chksum, chksum);
        let size = schema_builder.add_u64_field(&Fields::Size, STORED);
        let _ = fields.insert(Fields::Size, size);
        let related = schema_builder.add_u64_field(&Fields::Related, STORED);
        let _ = fields.insert(Fields::Related, related);
        let json = schema_builder.add_json_field(&Fields::Json, STORED);
        let _ = fields.insert(Fields::Json, json);

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let index_writer = index.writer(50_000_000).infer_err()?;

        let index_reader = index.reader_builder().try_into().infer_err()?;

        Ok(AppDatabase {
            sql: db,
            index_reader,
            index_writer: Mutex::new(index_writer),
            index,
            fields,
            id_gen: 0.into(),
        })
    }

    pub fn get_field(&self, f: Fields) -> Field {
        *self.fields.get(&f).unwrap()
    }

    pub fn get_searcher(&self) -> Searcher {
        Searcher(self.index_reader.searcher())
    }

    pub fn get_doc_address(&self, id: u32) -> Result<DocAddress, Error> {
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

    pub fn get_doc(&self, id: u32) -> Result<Document, Error> {
        let searcher = self.get_searcher();
        let doc = searcher.doc(self.get_doc_address(id)?).infer_err()?;
        Ok(doc)
    }

    pub fn new_id(&self) -> u32 {
        self.id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Fields {
    Id,
    Json,
    Related,
    SrcPath,
    DbPath,
    Url,
    Title,
    Description,
    Tag,
    ObjectType,
    Chksum,
    Size,
}

impl Deref for Fields {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Title => "title",
            Self::Tag => "tag",
            Self::ObjectType => "object_type",
            Self::Id => "id",
            Self::SrcPath => "src_path",
            Self::DbPath => "db_path",
            Self::Url => "url",
            Self::Chksum => "chksum",
            Self::Size => "size",
            Self::Json => "json",
            Self::Description => "description",
            Self::Related => "related",
        }
    }
}

impl AsRef<str> for Fields {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

pub enum ObjectType {
    Image,
    Bookmark,
    Tag,
    Group,
}

impl Deref for ObjectType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Image => "/image",
            Self::Bookmark => "/bookmark",
            Self::Tag => "/tag",
            Self::Group => "/group",
        }
    }
}

impl From<ObjectType> for Facet {
    fn from(value: ObjectType) -> Self {
        value.deref().into()
    }
}

// newtype pattern so i can implement more methods on it
pub struct Searcher(tantivy::Searcher);

impl Deref for Searcher {
    type Target = tantivy::Searcher;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
