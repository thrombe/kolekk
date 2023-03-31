#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, str::FromStr};

use futures::{future::OptionFuture, stream::FuturesUnordered, StreamExt};
use kolekk_types::{Bookmark, ByteArrayFile, DragDropPaste};
use reqwest::Client;
use tantivy::Term;
use tauri::{http::Uri, Manager, State};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    database::{add_bookmark, AppDatabase, Fields, ObjectType},
};

#[tauri::command]
pub async fn search_bookmarks(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    crate::database::search_object(db.inner(), ObjectType::Bookmark, query, limit, offset)
}

#[tauri::command]
pub async fn get_bookmarks(
    data: DragDropPaste<ByteArrayFile>,
    client: State<'_, Client>,
    db: State<'_, AppDatabase>,
) -> Result<Vec<Bookmark>, Error> {
    let bks = bookmarks_from_ddp(data, client.inner(), db.inner()).await;
    Ok(bks)
}

#[tauri::command]
pub async fn save_bookmarks_from_drop(
    data: DragDropPaste<ByteArrayFile>,
    client: State<'_, Client>,
    db: State<'_, AppDatabase>,
    app: State<'_, tauri::AppHandle>,
) -> Result<(), Error> {
    let data = bookmarks_from_ddp(data, client.inner(), db.inner()).await;
    save_bookmarks(data, db, app).await
}

#[tauri::command]
pub async fn save_bookmarks(
    data: Vec<Bookmark>,
    db: State<'_, AppDatabase>,
    app: State<'_, tauri::AppHandle>,
) -> Result<(), Error> {
    data.into_iter()
        .map(|b| async {
            let id = b.id;
            crate::database::add_bookmark(db.inner(), b)
                .await
                .infer_err()?;
            app.emit_all("item-added", id).infer_err()?;
            Ok(())
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<_, _>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

#[tauri::command]
pub async fn add_tag_to_bookmark(
    id: u32,
    tag_id: u32,
    db: State<'_, AppDatabase>,
) -> Result<(), Error> {
    let doc = db.get_doc(id)?;
    let bk = doc
        .get_first(db.get_field(Fields::Json))
        .and_then(|j| j.as_json())
        .and_then(|j| {
            serde_json::from_value::<Bookmark>(serde_json::Value::Object(j.to_owned())).ok()
        })
        .look(|b| dbg!(b))
        .map(|mut b| {
            b.tags.push(tag_id);
            b
        });

    {
        let writer = db.index_writer.lock().infer_err()?;
        let _opstamp = writer.delete_term(Term::from_field_u64(db.get_field(Fields::Id), id as _));
    }

    bk.map(|b| add_bookmark(db.inner(), b))
        .bad_err("eror")?
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_bookmark(
    id: u32,
    tag_id: u32,
    db: State<'_, AppDatabase>,
) -> Result<(), Error> {
    let doc = db.get_doc(id)?;
    let bk = doc
        .get_first(db.get_field(Fields::Json))
        .and_then(|j| j.as_json())
        .and_then(|j| {
            serde_json::from_value::<Bookmark>(serde_json::Value::Object(j.to_owned())).ok()
        })
        .look(|b| dbg!(b))
        .map(|mut b| {
            b.tags.retain(|t| *t != tag_id);
            b
        });

    {
        let writer = db.index_writer.lock().infer_err()?;
        let _opstamp = writer.delete_term(Term::from_field_u64(db.get_field(Fields::Id), id as _));
    }

    bk.map(|b| add_bookmark(db.inner(), b))
        .bad_err("eror")?
        .await?;
    Ok(())
}

pub async fn bookmarks_from_ddp(
    data: DragDropPaste<ByteArrayFile>,
    client: &Client,
    db: &AppDatabase,
) -> Vec<Bookmark> {
    let bks = data
        .file_uris
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect::<HashSet<_>>())
        .or_else(|| data.text.as_ref().map(|t| t.lines().collect()))
        .or_else(|| data.text_html.as_ref().map(|h| get_urls_from_hrefs(h)))
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim_matches(&['.', ' '][..]))
        .map(ToOwned::to_owned)
        .map(|u| async {
            let id = db.new_id();
            let b = match bookmark_from_markdown_url(id, &u) {
                Some(u) => Some(u),
                None => bookmark_from_url(id, u, client).await.ok(),
            };
            b
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    bks
}

pub fn get_urls_from_hrefs(mut h: &str) -> HashSet<&str> {
    // the html is broken for some reason. so cannot parse using scraper

    // dbg!(scraper::Html::parse_fragment(h)
    //     .select(&scraper::Selector::parse("*").unwrap())
    //     .map(|e| e.value().attrs().collect::<Vec<_>>())
    //     .collect::<Vec<_>>());

    let mut urls = HashSet::new();
    loop {
        let url = h
            .find("href")
            .map(|start| {
                h = &h[start + 1..];
                h
            })
            .and_then(|h| {
                h.find('"')
                    .map(|s| &h[s + 1..])
                    .map(|h| h.find('"').map(|e| &h[..e]))
            })
            .flatten();
        match url {
            Some(u) => {
                urls.insert(u);
            }
            None => {
                break;
            }
        }
    }
    urls
}

pub fn bookmark_from_markdown_url(id: u32, u: impl AsRef<str>) -> Option<Bookmark> {
    let u = u.as_ref();
    if u.starts_with("- [") {
        let title_start = u.find('[').unwrap();
        if let Some(title_end) = u.find("](") {
            let url_start = u[title_end..].find('(');
            let url_end = u[title_end..].find(')');
            if let (Some(start), Some(end)) = (url_start, url_end) {
                let url = &u[title_end..][start + 1..end];
                let _ = Uri::from_str(url).ok()?;
                let b = Bookmark {
                    url: url.into(),
                    title: Some(u[title_start + 1..title_end].into()),
                    description: None,
                    id,
                    tags: vec![],
                    related: vec![],
                };
                return Some(b);
            }
        }
    }
    None
}

pub async fn bookmark_from_url(id: u32, u: String, client: &Client) -> Result<Bookmark, Error> {
    let _ = Uri::from_str(&u).infer_err()?;
    let title: OptionFuture<_> = client
        .get(&u)
        .send()
        .await
        .look(|e| dbg!(e))
        .ok()
        .map(|page| async {
            if page
                .headers()
                .get("content-type")
                .and_then(|e| e.to_str().look(|e| dbg!(e)).ok())
                .unwrap_or_default()
                .contains("text/html")
            {
                page.text()
                    .await
                    .look(|e| dbg!(e))
                    .ok()
                    .as_ref()
                    .and_then(|t| {
                        scraper::Html::parse_document(t)
                            .select(&scraper::Selector::parse("title").ok()?)
                            .next()
                            .map(|e| e.inner_html())
                            .look(|e| dbg!(e))
                    })
            } else {
                None
            }
        })
        .into();
    let b = Bookmark {
        url: u,
        title: title.await.unwrap_or_default(),
        description: None,
        id,
        tags: vec![],
        related: vec![],
    };
    Ok(b)
}
