#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, str::FromStr};

use futures::{future::OptionFuture, stream::FuturesUnordered, StreamExt};
use kolekk_types::{
    objects::{Bookmark, TypeFacet},
    utility::{ByteArrayFile, DragDropPaste},
};
use reqwest::Client;
use tauri::{http::Uri, State};

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    database::AppDatabase,
};

#[tauri::command]
pub async fn search_bookmarks(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    crate::database::search_object(db.inner(), TypeFacet::Bookmark, query, limit, offset)
}

#[tauri::command]
pub async fn get_bookmarks(
    data: DragDropPaste<ByteArrayFile>,
    client: State<'_, Client>,
    db: State<'_, AppDatabase>,
) -> Result<Vec<Bookmark>, Error> {
    let bks = bookmarks_from_ddp(data, client.inner()).await;
    Ok(bks)
}

pub async fn bookmarks_from_ddp(
    data: DragDropPaste<ByteArrayFile>,
    client: &Client,
) -> Vec<Bookmark> {
    data.file_uris
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect::<HashSet<_>>())
        .or_else(|| data.text.as_ref().map(|t| t.lines().collect()))
        .or_else(|| data.text_html.as_ref().map(|h| get_urls_from_hrefs(h)))
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim_matches(&['.', ' '][..]))
        .map(ToOwned::to_owned)
        .map(|u| async {
            match bookmark_from_markdown_url(&u) {
                Some(u) => Some(u),
                None => bookmark_from_url(u, client).await.ok(),
            }
        })
        .collect::<FuturesUnordered<_>>() // TODO: this still only runs on a single thread. source: https://youtu.be/ThjvMReOXYM?t=4767 use tokio::task::spawn
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
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

pub fn bookmark_from_markdown_url(u: impl AsRef<str>) -> Option<Bookmark> {
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
                };
                return Some(b);
            }
        }
    }
    None
}

pub async fn bookmark_from_url(u: String, client: &Client) -> Result<Bookmark, Error> {
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
    };
    Ok(b)
}
