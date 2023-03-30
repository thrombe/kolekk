use kolekk_types::Image;
use tauri::State;

use crate::{
    bad_error::Error,
    database::{AppDatabase, ObjectType},
};

#[tauri::command]
pub async fn search_images(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<Image>, Error> {
    crate::database::search_images(db.inner(), query, limit, offset)
}

#[tauri::command]
pub async fn search_bookmarks(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    crate::database::search_object(db.inner(), ObjectType::Bookmark, query, limit, offset)
}
