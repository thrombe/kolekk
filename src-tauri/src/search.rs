use kolekk_types::{Image, Bookmark};
use tauri::State;

use crate::{database::AppDatabase, bad_error::Error};

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
) -> Result<Vec<Bookmark>, Error> {
    todo!()
}

