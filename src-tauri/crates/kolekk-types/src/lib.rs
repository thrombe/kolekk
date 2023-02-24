
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
pub use ts_rs::TS;
use std::fmt::Debug;


#[derive(Serialize, Deserialize, TS, Debug)]
pub struct ByteArrayFile {
    name: String,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, TS, Debug)]
pub struct DragDropPaste<F: Debug> {
    // priority in the same order
    file_uris: Option<Vec<String>>, // "http://" "ftp://" "smb://" "/home/"
    text: Option<String>, // anything. links, just text, whatever
    text_html: Option<String>, // <img href=""> <span>
    files: Option<Vec<F>>, // File data as name, bytes
    
    uri_list: Option<String>, // link drops. (link is also available in self.text)    
}


#[derive(Serialize, Deserialize, TS)]
pub struct Image {
    pub id: u32,
    pub title: String,
    pub path: String,
    pub urls: Vec<String>,
    pub tags: Vec<String>,
    // pub metadata: metadata::Model,
}

#[derive(Serialize, Deserialize, TS)]
pub struct Bookmark {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
    // pub metadata: metadata::Model,
}


impl Image {
    pub async fn all_from_db(db: &DatabaseConnection) -> Vec<Self> {
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
                tags: tags::Entity::find()
                    .filter(tags::Column::Id.eq(e.id))
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|e| e.tag)
                    .collect(),
                // metadata: metadata::Entity::find_by_id(e.id)
                //     .one(db)
                //     .await
                //     .unwrap()
                //     .unwrap(),
            };
            images.push(e);
        }
        images
    }

    pub async fn add_tag(&mut self, db: &DatabaseConnection, tag: String) {
        if tags::Entity::find()
        .filter(tags::Column::Id.eq(self.id))
        .filter(tags::Column::Tag.eq(tag.clone()))
        .one(db).await.unwrap().is_some() {
            return;
        }

        let tag = tags::ActiveModel {
            id: sea_orm::Set(self.id),
            tag: sea_orm::Set(tag),
        };
        let tag = tag
        .insert(db)
        .await
        .unwrap();
        self.tags.push(tag.tag);
    }

    pub async fn remove_tag(&mut self, db: &DatabaseConnection, tag: String) {
        self.tags
            .remove(self.tags.iter().position(|e| *e == tag).unwrap());
        let tag = tags::ActiveModel {
            id: sea_orm::Set(self.id),
            tag: sea_orm::Set(tag),
        };
        let _ = tags::Entity::delete(tag).exec(db).await.unwrap();
    }
}

pub mod bookmarks {
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

pub mod images {
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

pub mod tags {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tags")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u32,
        #[sea_orm(primary_key)]
        pub tag: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod urls {
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

pub mod metadata {
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
