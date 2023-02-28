
use sea_orm::entity::prelude::*;

use kolekk_types::Image;
pub use kolekk_types::{images, tags, urls, metadata};

type DB<'a> = tauri::State<'a, sea_orm::DatabaseConnection>;

#[tauri::command]
pub async fn add_image_from_path(db: DB<'_>, path: String, title: String) -> Result<(), ()> {
    let img = images::ActiveModel {
        src_path: sea_orm::Set(path),
        title: sea_orm::Set(title),
        ..Default::default()
    };
    let r = img.save(db.inner()).await.unwrap();
    dbg!(r);
    Ok(())
}

#[tauri::command]
pub async fn get_images(db: DB<'_>) -> Result<Vec<Image>, ()> {
    let imgs = Image::all_from_db(db.inner()).await;
    Ok(imgs)
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
