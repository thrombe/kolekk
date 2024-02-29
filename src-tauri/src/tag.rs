#[allow(unused_imports)]
use crate::{dbg, debug, error};

use kolekk_types::objects::{Fields, Id, Indexed, Meta, SearchableEntry, Tag, TypeFacet};
use tantivy::{collector::TopDocs, query::{BooleanQuery, Occur, TermQuery}, schema::IndexRecordOption, Document, Term};
use tauri::State;

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    database::{AppDatabase, DbAble, FacetFrom, TagSearchScoreTweaker},
};

#[tauri::command]
pub async fn search_tags(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<Meta<serde_json::Map<String, serde_json::Value>, TypeFacet>>, Error> {
    crate::database::direct_search(
        db.inner(),
        TypeFacet::Tag,
        query,
        limit,
        offset,
        TagSearchScoreTweaker::new(db.inner())?,
    )
}

#[tauri::command]
pub async fn save_new_tag(db: State<'_, AppDatabase>, tag: Tag) -> Result<Id, Error> {
    let mut doc = Document::new();

    let ctime = db.now_time().infer_err()?;
    let id = db.new_id();
    let v = Meta {
        id,
        facet: TypeFacet::Tag,
        data: SearchableEntry {
            searchable: vec![Indexed {
                field: Fields::Text,
                data: match &tag {
                    Tag::Main { name } | Tag::Alias { name, .. } => {
                        serde_json::Value::String(name.to_string())
                    }
                },
            }],
            data: tag,
        },
        ctime,
        last_update: ctime,
        last_interaction: ctime,
    };
    v.add(db.inner(), &mut doc)?;
    let mut writer = db.index_writer.write().infer_err()?;
    let _opstamp = writer.add_document(doc).infer_err()?;
    let _opstamp = writer.commit().infer_err()?;
    Ok(id)
}

#[tauri::command]
pub async fn get_tags_from_ids(
    ids: Vec<u32>,
    db: State<'_, AppDatabase>,
) -> Result<Vec<Meta<serde_json::Map<String, serde_json::Value>, TypeFacet>>, Error> {
    let searcher = db.get_searcher();
    ids.into_iter()
        .map(|id| Term::from_field_u64(db.get_field(Fields::Id), id as _))
        .map(|t| {
            let td = searcher
                .search(
                    &BooleanQuery::new(vec![
                        (Occur::Must, Box::new(TermQuery::new(t, IndexRecordOption::Basic)) as _),
                        (
                            Occur::Must,
                            Box::new(TermQuery::new(
                                Term::from_facet(db.get_field(Fields::Type), &TypeFacet::Tag.facet()),
                                IndexRecordOption::Basic,
                            )) as _,
                        ),
                    ]),
                    &TopDocs::with_limit(1),
                )
                .look(|e| dbg!(e))
                .infer_err()?;
            let (_score, address) = td.first().bad_err("no tag found")?;
            Ok(*address)
        })
        .filter_map(|e: Result<_, Error>| e.ok())
        .map(|address| {
            let mut doc = searcher.doc(address).look(|e| dbg!(e)).infer_err()?;
            let t = DbAble::take(db.inner(), &mut doc).look(|e| dbg!(e))?;
            Ok(t)
        })
        .collect()
}
