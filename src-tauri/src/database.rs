#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashMap, ops::Deref, sync::Mutex};

use kolekk_types::{images, metadata, tags, urls, Image};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use tantivy::{
    collector::TopDocs,
    query::{AllQuery, BooleanQuery, EmptyQuery, FuzzyTermQuery, Occur, PhraseQuery, TermQuery},
    schema::{Facet, FacetOptions, Field, IndexRecordOption, STORED, TEXT},
    Document, Index, IndexReader, IndexWriter, Term,
};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    config::AppConfig,
};

pub async fn add_image(db: &AppDatabase, img: Image) -> Result<(), Error> {
    let mut doc = Document::new();
    doc.add_facet(db.get_field(&Fields::ObjectType), ObjectType::Image);
    doc.add_u64(db.get_field(&Fields::Id), img.id as _);
    doc.add_text(db.get_field(&Fields::Title), img.title);
    doc.add_text(db.get_field(&Fields::SrcPath), img.src_path);
    doc.add_text(db.get_field(&Fields::DbPath), img.db_path);
    doc.add_bytes(db.get_field(&Fields::Chksum), img.chksum);
    doc.add_u64(db.get_field(&Fields::Size), img.size as _);
    img.tags
        .into_iter()
        .for_each(|t| doc.add_text(db.get_field(&Fields::Tag), t));
    img.urls
        .into_iter()
        .for_each(|u| doc.add_text(db.get_field(&Fields::Url), u));

    let mut writer = db.index_writer.lock().infer_err()?;
    let opstamp = writer.add_document(doc).look(|e| dbg!(e)).infer_err()?;
    writer.commit().look(|e| dbg!(e)).infer_err()?;
    Ok(())
}

pub fn search_images(
    db: &AppDatabase,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<Image>, Error> {
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
        Term::from_facet(db.get_field(&Fields::ObjectType), &ObjectType::Image.into()),
        IndexRecordOption::Basic,
    ));

    let phrase_query_terms = query
        .split_whitespace()
        .map(|t| Term::from_field_text(db.get_field(&Fields::Title), t))
        .collect::<Vec<_>>();
    let title_query = if phrase_query_terms.len() < 2 {
        // PhraseQuery does not support less than 2 terms
        Box::new(TermQuery::new(
            Term::from_field_text(db.get_field(&Fields::Title), &query),
            IndexRecordOption::Basic,
        )) as _
    } else {
        Box::new(PhraseQuery::new(phrase_query_terms)) as _
    };

    let tag_query = Box::new(BooleanQuery::new(
        query
            .split_whitespace()
            .map(|t| {
                (
                    Occur::Should,
                    Box::new(FuzzyTermQuery::new(
                        Term::from_field_text(db.get_field(&Fields::Tag), t),
                        2,    // ?
                        true, // what??
                    )) as _,
                )
            })
            .collect(),
    ));

    let title_fuzzy_query = Box::new(BooleanQuery::new(
        // TODO: implement these  a methods of Fields and ObjectType
        query
            .split_whitespace()
            .map(|t| {
                (
                    Occur::Should, // TODO: should this be Must instead?
                    Box::new(FuzzyTermQuery::new(
                        Term::from_field_text(db.get_field(&Fields::Title), t),
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
        (Occur::Should, tag_query),
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
            Ok(Image {
                // these unwraps should probably be fine
                id: doc
                    .get_first(db.get_field(&Fields::Id))
                    .bad_err("no id")?
                    .as_u64()
                    .unwrap() as _,
                title: doc
                    .get_first(db.get_field(&Fields::Title))
                    .bad_err("no title")?
                    .as_text()
                    .unwrap()
                    .to_owned(),
                src_path: doc
                    .get_first(db.get_field(&Fields::SrcPath))
                    .bad_err("no src_path")?
                    .as_text()
                    .unwrap()
                    .to_owned(),
                db_path: doc
                    .get_first(db.get_field(&Fields::DbPath))
                    .bad_err("no db_path")?
                    .as_text()
                    .unwrap()
                    .to_owned(),
                chksum: doc
                    .get_first(db.get_field(&Fields::Chksum))
                    .bad_err("no chksum")?
                    .as_bytes()
                    .unwrap()
                    .to_owned(),
                size: doc
                    .get_first(db.get_field(&Fields::Size))
                    .bad_err("no size")?
                    .as_u64()
                    .unwrap() as _,
                urls: doc
                    .get_all(db.get_field(&Fields::Url))
                    .map(|u| u.as_text().unwrap().to_owned())
                    .collect(),
                tags: doc
                    .get_all(db.get_field(&Fields::Tag))
                    .map(|t| t.as_text().unwrap().to_owned())
                    .collect(),
            })
        })
        .collect()
}

pub struct AppDatabase {
    sql: DatabaseConnection,
    index: Index,
    index_reader: IndexReader,
    index_writer: Mutex<IndexWriter>,
    fields: HashMap<&'static str, Field>,
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
            let table = schema.create_table_from_entity(images::Entity);
            let _ = db.execute(backend.build(&table)).await.unwrap();
            let table = schema.create_table_from_entity(tags::Entity);
            let _ = db.execute(backend.build(&table)).await.unwrap();
            let table = schema.create_table_from_entity(urls::Entity);
            let _ = db.execute(backend.build(&table)).await.unwrap();
            let table = schema.create_table_from_entity(metadata::Entity);
            let _ = db.execute(backend.build(&table)).await.unwrap();
        }

        let mut schema_builder = tantivy::schema::Schema::builder();

        let mut fields = HashMap::<&str, Field>::new();

        let id = schema_builder.add_text_field(&Fields::Id, STORED);
        let _ = fields.insert(&Fields::Id, id);
        let src_path = schema_builder.add_text_field(&Fields::SrcPath, STORED);
        let _ = fields.insert(&Fields::SrcPath, src_path);
        let db_path = schema_builder.add_text_field(&Fields::DbPath, STORED);
        let _ = fields.insert(&Fields::DbPath, db_path);
        let url = schema_builder.add_text_field(&Fields::Url, STORED);
        let _ = fields.insert(&Fields::Url, url);
        let title = schema_builder.add_text_field(&Fields::Title, STORED | TEXT);
        let _ = fields.insert(&Fields::Title, title);
        let tag = schema_builder.add_text_field(&Fields::Tag, STORED | TEXT);
        let _ = fields.insert(&Fields::Tag, tag);
        let object_type =
            schema_builder.add_facet_field(&Fields::ObjectType, FacetOptions::default());
        let _ = fields.insert(&Fields::ObjectType, object_type);
        let chksum = schema_builder.add_bytes_field(&Fields::Chksum, STORED);
        let _ = fields.insert(&Fields::Chksum, chksum);
        let size = schema_builder.add_u64_field(&Fields::Size, STORED);
        let _ = fields.insert(&Fields::Size, size);

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
        })
    }

    pub fn get_field(&self, ident: &str) -> Field {
        *self
            .fields
            .get(ident.look(|e| dbg!(e)))
            .look(|e| dbg!(e))
            .unwrap()
    }

    pub fn get_searcher(&self) -> Searcher {
        Searcher(self.index_reader.searcher())
    }
}

pub enum Fields {
    Id,
    SrcPath,
    DbPath,
    Url,
    Title,
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
}

impl Deref for ObjectType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            ObjectType::Image => "/image",
            ObjectType::Bookmark => "/bookmark",
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
