#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    collections::HashSet, fs, os::unix::fs::MetadataExt, path::PathBuf, str::FromStr,
    time::Duration,
};

use futures::{future::OptionFuture, stream::FuturesUnordered, StreamExt};
use kolekk_types::{
    objects::{
        Bookmark, BookmarkSource, Fields, Indexed, Meta, SearchableEntry, Taggable, Tagged,
        TypeFacet, WithContext,
    },
    utility::{ByteArrayFile, DragDropPaste},
};
use reqwest::Client;
use serde_json::Value;
use tantivy::{
    collector::TopDocs,
    query::{Occur, TermQuery},
    schema::IndexRecordOption,
    Document, Term,
};
use tauri::{http::Uri, State};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    config::AppConfig,
    database::{AppDatabase, AutoDbAble, DbAble, FacetFrom, IntoRObject, ObjectSearchScoreTweaker},
    filesystem::get_path,
};

#[tauri::command]
pub async fn refresh_bookmark_sources(
    db: State<'_, AppDatabase>,
    config: State<'_, AppConfig>,
    client: State<'_, Client>,
) -> Result<(), Error> {
    let dbi = db.inner();
    let searcher = db.get_searcher();
    let obj_type_query = TermQuery::new(
        Term::from_facet(
            db.get_field(Fields::Type),
            &TypeFacet::BookmarkSource.facet(),
        ),
        IndexRecordOption::Basic,
    );
    let sources: Result<Vec<Meta<BookmarkSource, TypeFacet>>, _> = searcher
        .search(&obj_type_query, &TopDocs::with_limit(10000))
        .infer_err()?
        .into_iter()
        .map(move |(_score, address)| {
            let mut doc = searcher.doc(address).infer_err()?;
            DbAble::take(dbi, &mut doc).look(|e| dbg!((_score, e)))
        })
        .collect();
    let sources = sources.infer_err()?;

    let time = db.now_time()?;
    for source in sources {
        let mdata = fs::metadata(get_path(&source.data.path, config.inner())).infer_err()?;
        if source.data.mtime == mdata.mtime() {
            continue;
        }

        {
            let writer = db.index_writer.write().infer_err()?;
            writer.delete_term(Term::from_field_u64(
                db.get_field(Fields::SourceId),
                source.id as _,
            ));
        }
        add_bookmark_source(
            db.clone(),
            config.clone(),
            client.clone(),
            source.data.title,
            source.data.path,
        )
        .await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn add_bookmark_source(
    db: State<'_, AppDatabase>,
    config: State<'_, AppConfig>,
    client: State<'_, Client>,
    title: String,
    path: kolekk_types::utility::Path,
) -> Result<u32, Error> {
    let db = db.inner();
    let pb = get_path(&path, config.inner());

    let mut file = File::open(&pb).await.infer_err()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.infer_err()?;
    let mdata = file.metadata().await.infer_err()?;
    let source_id = db.new_id();
    let time = db.now_time().infer_err()?;
    let source = Meta {
        data: SearchableEntry {
            searchable: vec![
                Indexed {
                    field: Fields::Text,
                    data: title.clone().into(),
                },
                Indexed {
                    field: Fields::Text,
                    data: pb.to_string_lossy().to_string().into(),
                },
            ],
            data: BookmarkSource {
                title: title.clone(),
                path,
                last_checked: time,
                mtime: mdata.mtime(),
            },
        },
        facet: TypeFacet::BookmarkSource,
        ctime: time,
        last_update: time,
        last_interaction: time,
        id: source_id,
    };

    let mut res = _get_tagged_bookmarks_from_text(&contents, client.inner())
        .await
        .infer_err()?;
    for bk in res.0.iter_mut() {
        bk.data.source = Some(source_id);
        bk.tags.push(title.clone());
    }

    {
        let mut writer = db.index_writer.write().infer_err()?;
        let _opstamp = writer.delete_term(Term::from_field_u64(
            db.get_field(Fields::Id),
            source_id as _,
        ));
    }
    let mut doc = Document::new();
    source.add(db, &mut doc)?;

    for bk in res.0 {
        // TODO: BAD: OOF: IntoRObject trait implementations lock the writer. which is bad. write better code
        let bk = bk.into_robject(db)?;
        let id = bk.id;
        let mut doc = Document::new();
        bk.add(db, &mut doc)?;

        {
            let mut writer = db.index_writer.write().infer_err()?;
            let _opstamp =
                writer.delete_term(Term::from_field_u64(db.get_field(Fields::Id), id as _));
            writer.add_document(doc).infer_err()?;
        }
    }

    let mut writer = db.index_writer.write().infer_err()?;
    let _opstamp = writer.add_document(doc).infer_err()?;
    let _opstamp = writer.commit().infer_err()?;

    Ok(source_id)
}

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
pub async fn bookmarks_from_html(
    html: String,
    client: State<'_, Client>,
) -> Result<Vec<Bookmark>, Error> {
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

type BookmarkFromTextResult = (
    Vec<Tagged<Bookmark>>,
    Vec<WithContext<Tagged<String>, String>>,
);

#[tauri::command]
pub async fn get_tagged_bookmarks_from_text(
    text: String,
    client: State<'_, Client>,
) -> Result<BookmarkFromTextResult, Error> {
    _get_tagged_bookmarks_from_text(text, client.inner()).await
}

pub async fn _get_tagged_bookmarks_from_text(
    text: impl AsRef<str>,
    client: &Client,
) -> Result<BookmarkFromTextResult, Error> {
    let text = text.as_ref();
    let mut bks = vec![];
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
pub fn tagged_strings_from_text(
    text: impl AsRef<str>,
) -> (
    Vec<Tagged<String>>,
    Vec<WithContext<Tagged<String>, String>>,
) {
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
                    },
                });
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
                    source: None,
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
        source: None,
    };
    Ok(b)
}
