#[allow(unused_imports)]
use crate::{dbg, debug, error};

use tantivy::{collector::TopDocs, query::TermQuery, schema::IndexRecordOption, Term};
use tauri::State;

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    database::{add_tag, AppDatabase, Fields, ObjectType},
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

#[tauri::command]
pub async fn get_tags_from_ids(
    ids: Vec<u32>,
    db: State<'_, AppDatabase>,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    let searcher = db.get_searcher();
    ids.into_iter()
        .map(|id| Term::from_field_u64(db.get_field(Fields::Id), id as _))
        .map(|t| {
            let td = searcher
                .search(
                    &TermQuery::new(t, IndexRecordOption::Basic),
                    &TopDocs::with_limit(1),
                )
                .look(|e| dbg!(e))
                .infer_err()?;
            let (_score, address) = td.first().bad_err("no tag found")?;
            Ok(*address)
        })
        .filter_map(|e: Result<_, Error>| e.ok())
        .map(|address| {
            let tag = searcher.doc(address).infer_err().map(|doc| {
                doc.get_first(db.get_field(Fields::Json))
                    .bad_err("no tag found")
                    .map(|t| t.as_json().bad_err("no tag found").map(|t| t.to_owned()))
            })???;
            Ok(tag)
        })
        .collect()
}
