#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, str::FromStr};

use futures::{future::OptionFuture, stream::FuturesUnordered, StreamExt};
use kolekk_types::{Bookmark, ByteArrayFile, DragDropPaste};
use reqwest::Client;
use tauri::{http::Uri, Manager, State};

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    config::AppConfig,
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
        .filter_map(|u| Uri::from_str(u).ok().map(|_| u))
        .map(ToOwned::to_owned)
        .map(|u| async {
            let title: OptionFuture<_> = client
                .inner()
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
            let id = 0;
            let b = Bookmark {
                url: u,
                title: title.await.unwrap_or_default(),
                description: None,
                id,
                tags: vec![],
                related: vec![],
            };
            crate::database::add_bookmark(db.inner(), b)
                .await
                .infer_err()?;
            app.emit_all("item-added", id).infer_err()?;
            Ok(id)
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<_, _>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}
