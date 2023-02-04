use std::path::Path;

use sea_orm::{entity::prelude::*, ConnectionTrait, Database};

use kolekk_types::Image;
pub use kolekk_types::{images, tags, urls, metadata};

type DB<'a> = tauri::State<'a, sea_orm::DatabaseConnection>;

#[tauri::command]
pub async fn add_image_from_path(db: DB<'_>, path: String, title: String) -> Result<(), ()> {
    let img = images::ActiveModel {
        path: sea_orm::Set(path),
        title: sea_orm::Set(title),
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
