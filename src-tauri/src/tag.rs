use tauri::State;

use crate::{
    bad_error::Error,
    database::{add_tag, AppDatabase, ObjectType},
};

#[tauri::command]
pub async fn search_tags(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    crate::database::search_object(db.inner(), ObjectType::Tag, query, limit, offset)
}

#[tauri::command]
pub async fn save_tag(db: State<'_, AppDatabase>, name: String) -> Result<u32, Error> {
    let id = db.new_id();
    add_tag(db.inner(), kolekk_types::Tag::Main { id, name }).await?;
    Ok(id)
}

#[tauri::command]
pub async fn save_alias_tag(
    db: State<'_, AppDatabase>,
    name: String,
    alias_to: u32,
) -> Result<u32, Error> {
    let id = db.new_id();
    add_tag(db.inner(), kolekk_types::Tag::Alias { id, name, alias_to }).await?;
    Ok(id)
}
