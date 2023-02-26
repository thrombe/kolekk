#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::ops::Deref;

use kolekk_types::{images, metadata, tags, urls};
use sea_orm::{ConnectionTrait, Database};
use tantivy::schema::{STORED, TEXT};

use crate::{bad_error::{Error, InferBadError, Inspectable}, config::AppConfig};
pub struct AppDatabase {
    sql: sea_orm::DatabaseConnection,
    index: tantivy::Index,
    index_reader: tantivy::IndexReader,
    index_writer: std::sync::Mutex<tantivy::IndexWriter>,
}

pub enum Fields {
    Id,
    SrcPath,
    DbPath,
    Url,
    Title,
    Tag,
    ObjectType,
}

impl Deref for Fields {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Title => "title",
            Self::Tag => "tag",
            Self::ObjectType => "object_type",
            Fields::Id => "id",
            Fields::SrcPath => "src_path",
            Fields::DbPath => "db_path",
            Fields::Url => "url",
        }
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

        let _id = schema_builder.add_text_field(&Fields::Id, STORED);
        let _src_path = schema_builder.add_text_field(&Fields::SrcPath, STORED);
        let _db_path = schema_builder.add_text_field(&Fields::DbPath, STORED);
        let _url = schema_builder.add_text_field(&Fields::Url, STORED);
        let _title = schema_builder.add_text_field(&Fields::Title, STORED | TEXT);
        let _tag = schema_builder.add_text_field(&Fields::Tag, STORED | TEXT);
        let _ingredient = schema_builder.add_facet_field(
            &Fields::ObjectType,
            tantivy::schema::FacetOptions::default(),
        );

        let schema = schema_builder.build();
        let index = tantivy::Index::create_in_ram(schema);
        let index_writer = index.writer(50_000_000).infer_err()?;

        let index_reader = index.reader_builder().try_into().infer_err()?;

        Ok(AppDatabase {
            sql: db,
            index_reader,
            index_writer: std::sync::Mutex::new(index_writer),
            index,
        })
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
