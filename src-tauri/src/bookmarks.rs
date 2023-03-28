#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, str::FromStr};

use futures::{future::OptionFuture, stream::FuturesUnordered, StreamExt};
use kolekk_types::{Bookmark, ByteArrayFile, DragDropPaste};
use reqwest::Client;
use tauri::{http::Uri, Manager, State};

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    database::AppDatabase,
};

#[tauri::command]
pub async fn save_bookmark(
    data: DragDropPaste<ByteArrayFile>,
    // config: State<'_, AppConfig>,
    client: State<'_, Client>,
    db: State<'_, AppDatabase>,
    app: tauri::State<'_, tauri::AppHandle>,
) -> Result<(), Error> {
    data.file_uris
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect::<HashSet<_>>())
        .or_else(|| data.text.as_ref().map(|t| t.lines().collect()))
        .or_else(|| data.text_html.as_ref().map(|h| todo!())) // parse all potential urls if the text does not exist
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim_matches(&['.', ' '][..]))
        .map(ToOwned::to_owned)
        .map(|u| async {
            let id = db.new_id();
            let b = match try_bookmark_from_markdown_url(id, &u) {
                Some(u) => Some(u),
                None => try_bookmark_from_url(id, u, client.inner()).await.ok(),
            };

            if let Some(b) = b {
                crate::database::add_bookmark(db.inner(), b)
                    .await
                    .infer_err()?;
                app.emit_all("item-added", id).infer_err()?;
            }
            Ok(())
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<_, _>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

pub fn try_bookmark_from_markdown_url(id: u32, u: impl AsRef<str>) -> Option<Bookmark> {
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

pub async fn try_bookmark_from_url(id: u32, u: String, client: &Client) -> Result<Bookmark, Error> {
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
