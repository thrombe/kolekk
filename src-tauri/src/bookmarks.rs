#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, str::FromStr};

use futures::{future::OptionFuture, stream::FuturesUnordered, StreamExt};
use kolekk_types::{
    objects::{Bookmark, Meta, Taggable, Tagged, TypeFacet, WithContext},
    utility::{ByteArrayFile, DragDropPaste},
};
use reqwest::Client;
use tauri::{http::Uri, State};

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    database::{AppDatabase, ObjectSearchScoreTweaker},
};

#[tauri::command]
pub async fn search_bookmarks(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    crate::database::tagged_search(
        db.inner(),
        TypeFacet::Bookmark,
        query,
        limit,
        offset,
        ObjectSearchScoreTweaker::new(db.inner())?,
    )
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


#[tauri::command]
pub async fn bookmarks_from_html(html: String, client: State<'_, Client>) -> Result<Vec<Bookmark>, Error> {
    let client = client.inner();

    let res = get_urls_from_hrefs(&html)
        .into_iter()
        .map(|s| s.trim_matches(&['.', ' '][..]))
        .map(ToOwned::to_owned)
        .map(|u| async { bookmark_from_url(u, client).await.ok() })
        .collect::<FuturesUnordered<_>>() // TODO: this still only runs on a single thread. source: https://youtu.be/ThjvMReOXYM?t=4767 use tokio::task::spawn
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    Ok(res)
}

#[tauri::command]
pub async fn get_tagged_bookmarks_from_text(
    text: String,
    client: State<'_, Client>,
) -> Result<
    (
        Vec<Tagged<Bookmark>>,
        Vec<WithContext<Tagged<String>, String>>,
    ),
    Error,
> {
    let mut bks = vec![];
    let client = client.inner();
    let (pot_bks, mut errored) = tagged_strings_from_text(text);

    let results = pot_bks
        .into_iter()
        .map(|bk| async {
            match bookmark_from_markdown_url(&bk.data) {
                Some(u) => (
                    Some(Tagged {
                        tags: bk.tags,
                        data: u,
                    }),
                    None,
                ),
                None => match bookmark_from_url(bk.data.clone(), client).await {
                    Ok(b) => (
                        Some(Tagged {
                            tags: bk.tags,
                            data: b,
                        }),
                        None,
                    ),
                    Err(e) => (
                        None,
                        Some(WithContext {
                            data: bk,
                            context: e.to_string(),
                        }),
                    ),
                },
            }
        })
        .collect::<FuturesUnordered<_>>() // TODO: this still only runs on a single thread. source: https://youtu.be/ThjvMReOXYM?t=4767 use tokio::task::spawn
        .collect::<Vec<_>>()
        .await;

    for res in results.into_iter() {
        match res {
            (Some(bk), None) => {
                bks.push(bk);
            }
            (None, Some(err)) => {
                errored.push(err);
            }
            (None, None) | (Some(_), Some(_)) => unreachable!(),
        }
    }

    Ok((bks, errored))
}

/*
# tag
  - tag + tag / tag
    # tag
      - [link title](link url)
        - https://somelink
      https://somelink
*/
pub fn tagged_strings_from_text(text: impl AsRef<str>) -> (Vec<Tagged<String>>, Vec<WithContext<Tagged<String>, String>>) {
    let text = text.as_ref();
    let mut tags = Vec::<(_, Vec<String>)>::new();
    let mut potential_bks = vec![];
    let mut donno = vec![];

    let get_tags = |tag: &str| {
        let new_tags = tag
            .split('+')
            .flat_map(|t| t.trim().split('/'))
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        new_tags
    };

    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let mut indent = 0;
        for c in line.chars() {
            if c == ' ' {
                indent += 1;
            } else {
                break;
            }
        }
        while !tags.is_empty() && tags.last().unwrap().0 >= indent {
            let _ = tags.pop();
        }

        let line = &line[indent..];
        match (
            line.starts_with('#'),
            line.starts_with("- "),
            line.trim_start_matches(['-', ' ']).starts_with("http://")
                || line.trim_start_matches(['-', ' ']).starts_with("https://"),
            line.starts_with("- ["),
        ) {
            (true, _, _, _) | (false, true, false, false) => {
                let tag = line.trim_start_matches(['#', ' ', '-']).trim_end();
                tags.push((indent, get_tags(tag)));
            }
            (false, _, true, _) | (false, _, false, true) => {
                potential_bks.push(Tagged {
                    tags: tags.iter().flat_map(|(_, v)| v).cloned().collect(),
                    data: line.to_owned(),
                });
            }
            (false, false, false, false) => {
                // donno what this is :/
                // passing it in potential (for context in frontend), but making it non-parsable uri

                donno.push(WithContext {
                    context: "don't know how to parse this line".to_owned(),
                    data: Tagged {
                    tags: tags.iter().flat_map(|(_, v)| v).cloned().collect(),
                    data: line.trim().to_owned(),
                }});
            }
        }

        // if line[indent..].starts_with('#') {
        //     let tag = line[indent..]
        //         .trim_start_matches(['#', ' '])
        //         .trim_end();
        //     tags.push((indent, get_tags(tag)));
        // } else if line[indent..].starts_with("- ") {
        //     if line[indent..].starts_with("- [") || line[indent..].starts_with("- http") {
        //         potential_bks.push(Tagged {
        //             tags: tags.iter().flat_map(|(_, v)| v).cloned().collect(),
        //             data: line[indent..].to_owned(),
        //         });
        //     } else {
        //         let tag = line[indent + 2..].trim();
        //         tags.push((indent, get_tags(tag)));
        //     }
        // }
    }
    (potential_bks, donno)
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
