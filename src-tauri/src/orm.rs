use std::path::Path;

use sea_orm::{entity::prelude::*, ConnectionTrait, Database};
use serde::{Deserialize, Serialize};

type DB<'a> = tauri::State<'a, sea_orm::DatabaseConnection>;

#[tauri::command]
pub async fn add_image_from_path(db: DB<'_>, path: String) -> Result<(), ()> {
    let img = images::ActiveModel {
        path: sea_orm::Set(path),
        ..Default::default()
    };
    let r = img.save(db.inner()).await.unwrap();
    dbg!(r);
    Ok(())
}

#[tauri::command]
pub fn create_image_from_bytes(img: &[u8]) {}

#[tauri::command]
pub async fn get_images(db: DB<'_>) -> Result<Vec<Image>, ()> {
    let imgs = Image::all_from_db(db.inner()).await;
    Ok(imgs)
}

pub async fn setup_sea_orm() -> anyhow::Result<sea_orm::DatabaseConnection> {
    // - [create new db file sea-orm](https://github.com/SeaQL/sea-orm/discussions/283#discussioncomment-1564939)
    let db_path = "/home/issac/0Git/kolekk/cache/kolekdb.db";
    let db_url = format!("sqlite://{db_path}?mode=rwc");
    let new_db = !Path::new(db_path).exists();
    // let db_url = "sqlite::memory:";
    let db = Database::connect(db_url).await?;

    if new_db {
        let backend = db.get_database_backend();
        let schema = sea_orm::Schema::new(backend);
        let table = schema.create_table_from_entity(images::Entity);
        // dbg!(backend.build(&table).to_string());
        let _ = db.execute(backend.build(&table)).await.unwrap();
        let table = schema.create_table_from_entity(tags::Entity);
        let _ = db.execute(backend.build(&table)).await.unwrap();
    }

    Ok(db)
}

#[tauri::command]
pub async fn add_tag_to_image(db: DB<'_>, mut img: Image, tag: String) -> Result<Image, ()> {
    img.add_tag(db.inner(), tag).await;
    Ok(img)
}

#[tauri::command]
pub async fn remove_tag_from_image(db: DB<'_>, mut img: Image, tag: String) -> Result<Image, ()> {
    img.remove_tag(db.inner(), tag).await;
    Ok(img)
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub id: u32,
    pub title: String,
    pub path: String,
    pub urls: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: metadata::Model,
}

#[derive(Serialize, Deserialize)]
pub struct Bookmark {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
    pub metadata: metadata::Model,
}

impl Image {
    async fn all_from_db(db: &DatabaseConnection) -> Vec<Self> {
        let img = images::Entity::find().all(db).await.unwrap();
        let mut images = vec![];
        for e in img.into_iter() {
            let e = Self {
                id: e.id,
                title: e.title,
                path: e.path,
                urls: urls::Entity::find_by_id(e.id)
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|e| e.url)
                    .collect(),
                tags: tags::Entity::find_by_id(e.id)
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|e| e.tag)
                    .collect(),
                metadata: metadata::Entity::find_by_id(e.id)
                    .one(db)
                    .await
                    .unwrap()
                    .unwrap(),
            };
            images.push(e);
        }
        images
    }

    async fn add_tag(&mut self, db: &DatabaseConnection, tag: String) {
        let tag = tags::ActiveModel {
            id: sea_orm::Set(self.id),
            tag: sea_orm::Set(tag),
        };
        let tag = tag.insert(db).await.unwrap();
        self.tags.push(tag.tag);
    }

    async fn remove_tag(&mut self, db: &DatabaseConnection, tag: String) {
        self.tags
            .remove(self.tags.iter().position(|e| *e == tag).unwrap());
        let tag = tags::ActiveModel {
            id: sea_orm::Set(self.id),
            tag: sea_orm::Set(tag),
        };
        let _ = tags::Entity::delete(tag).exec(db).await.unwrap();
    }
}

mod bookmarks {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "bookmarks")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u32,
        pub title: String,
        pub url: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod images {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "images")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u32,
        // #[sea_orm(primary_key)]
        // #[sea_orm(unique)]
        // pub md5_or_somethin: String,
        pub title: String,
        pub path: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod tags {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tags")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u32,
        #[sea_orm(unique)]
        pub tag: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod urls {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "urls")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u32,
        #[sea_orm(unique)]
        pub url: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod metadata {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "metadata")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u32,
        // pub added_ts: datetime?,
        // pub last_edit: ts?,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/* too much boilerplate for the relations man. maybe i'll do it once everything db is finalised

mod images {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "images")]
    pub struct Model {
        #[sea_orm(primary_key)]
        id: u32,
        // #[sea_orm(primary_key)]
        // #[sea_orm(unique)]
        // md5_or_somethin: String,
        path: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_one = "super::urls::Entity")]
        URLs,
        #[sea_orm(has_one = "super::tags::Entity")]
        Tags,
        #[sea_orm(has_one = "super::metadata::Entity")]
        Metadata,
    }

    impl Related<super::urls::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::URLs.def()
        }
    }
    impl Related<super::tags::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Tags.def()
        }
    }
    impl Related<super::metadata::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Metadata.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod tags {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tags")]
    pub struct Model {
        #[sea_orm(primary_key)]
        id: u32,
        #[sea_orm(primary_key)]
        tag: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::images::Entity",
            from = "Column::Id",
            to = "super::images::Column::Id",
        )]
        Image,
    }
    impl Related<super::images::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Image.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod urls {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "urls")]
    pub struct Model {
        #[sea_orm(primary_key)]
        id: u32,
        #[sea_orm(primary_key)]
        url: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::images::Entity",
            from = "Column::Id",
            to = "super::images::Column::Id",
        )]
        Image,
    }
    impl Related<super::images::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Image.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod metadata {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "metadata")]
    pub struct Model {
        #[sea_orm(primary_key)]
        id: u32,
        // added_ts: datetime?,
        // last_edit: ts?,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::images::Entity",
            from = "Column::Id",
            to = "super::images::Column::Id",
        )]
        Image,
    }
    impl Related<super::images::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Image.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

*/
