#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, fmt::Debug, path::PathBuf, str::FromStr, time::Duration};

use kolekk_types::{ByteArrayFile, DragDropPaste, Image};
use tauri::{http::Uri, State};

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    config::AppConfig,
    database::{AppDatabase, ObjectType},
    filesystem::{file_mdata, path_is_in_dir, Filable, FilableUri, FileSource, FiledResult},
};

#[tauri::command]
pub async fn search_images(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Error> {
    crate::database::search_object(db.inner(), ObjectType::Image, query, limit, offset)
}

#[tauri::command]
pub async fn save_images_in_appdir(
    data: DragDropPaste<ByteArrayFile>,
    config: State<'_, AppConfig>,
    db: State<'_, AppDatabase>,
) -> Result<(), Error> {
    let res = save_images(&data, &config.app_data_dir)
        .await
        .look(|e| dbg!(e))?;

    // TODO: if file is already in database, then remove the file that was just saved

    let futs = res
        .into_iter()
        .map(|file| -> Result<Image, Error> {
            let mdata = file_mdata(&file.dest_path).look(|e| dbg!(e))?;
            let (src_path, uri) = match file.src {
                FileSource::Path(p) => (Some(p), None),
                FileSource::Uri(u) => (None, Some(u)),
                FileSource::ByteArray => (None, None),
            };
            let img = Image {
                id: 0,
                src_path: src_path.unwrap_or_default().to_string_lossy().to_string(),
                title: file.title,
                urls: uri.into_iter().map(|e| e.to_string()).collect(),
                tags: vec![],
                db_path: file.dest_path.to_string_lossy().to_string(),
                chksum: mdata.chksum.into(),
                size: mdata.size as _,
            };
            Ok(img)
        })
        .map(Result::unwrap) // TODO: ?
        .map(|img| crate::database::add_image(db.inner(), img))
        .collect::<Vec<_>>();

    futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_, Error>>()?;
    Ok(())
}

pub async fn save_images<'a, F: Debug + Filable>(
    data: &DragDropPaste<F>,
    data_dir: impl Into<PathBuf>,
) -> Result<Vec<FiledResult>, Error> {
    let data_dir = data_dir.into().join("images");

    if !data_dir.exists() {
        std::fs::create_dir(&data_dir)
            .look(|e| dbg!(e))
            .infer_err()?;
    }

    let mut saved_files = data
        .files
        .as_ref()
        .map(|v| &v[..])
        .unwrap_or(&[])
        .iter()
        .filter_map(|f| f.save_in_dir(&data_dir).look(|e| dbg!(e)).ok())
        .collect::<Vec<_>>();
    let potential_links: HashSet<_> = data
        .file_uris
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect())
        .or_else(|| data.text.as_ref().map(|t| t.lines().collect()))
        .or_else(|| data.text_html.as_ref().map(|h| todo!())) // parse all potential urls if the text does not exist
        // .or(data.uri_list.as_ref().map(|u| todo!())) // donno if including this does any good
        .unwrap_or_default();

    let client = tauri::api::http::ClientBuilder::new()
        .max_redirections(5)
        .connect_timeout(Duration::new(5, 0))
        .build()
        .look(|e| dbg!(e))
        .infer_err()?;

    let mut reqs = vec![];
    for u in potential_links {
        if let Some(p) = PathBuf::from_str(u)
            .look(|e| dbg!(e))
            .ok()
            .filter(|p| p.is_file())
        {
            if !path_is_in_dir(&p, &data_dir).unwrap_or(false) {
                let _ = p
                    .as_path()
                    .save_in_dir(&data_dir)
                    .look(|e| dbg!(e))
                    .ok()
                    .map(|pb| saved_files.push(pb));
            }
        } else {
            reqs.push(
                FilableUri {
                    title: "".into(),
                    src: Uri::from_str(u).look(|e| dbg!(e)).infer_err()?,
                    client: &client,
                }
                .save_in_dir(&data_dir),
            );
        };
    }
    futures::future::join_all(reqs)
        .await
        .into_iter()
        .filter_map(|e| e.ok())
        .for_each(|pb| saved_files.push(pb));
    Ok(saved_files)
}
