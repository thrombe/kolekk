use kolekk_types::{Bookmark, Image};
use tauri::State;

use crate::{bad_error::Error, database::AppDatabase};

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
