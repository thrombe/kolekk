#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    collections::HashSet,
    fmt::Debug,
    fs::File,
    io::{BufWriter, Cursor, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Mutex,
    time::Duration,
};

use image::io::Reader;
use kolekk_types::{ByteArrayFile, DragDropPaste, Image, ThumbnailSize};
use lru::LruCache;
use reqwest::{header::HeaderValue, Client, Url};
use stretto::AsyncCache;
use tauri::{http::Uri, State};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
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
    todo!();
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

// fetching thumbnail for an uri
// - check if the uri already has a an associated dir using LruCache<uri, _> else create one and copy original image in it
//   - can store id for thumbnail Object in the lru cache
//     - fetch thumbnail object from db and get the uuid from object
//   - can store uuid for thumbnail dir in the lru cache
//     - fetch the thumbnail object from db using uuid
//   - store thumbnail object in lru cache
// - check if the required size is smaller than the original image, create a thumbnail if required. else return original img
// #[allow(clippy::await_holding_lock)] // clippy does not detect the drop(cache) calls?
#[tauri::command]
pub async fn image_thumbnail(
    db: State<'_, AppDatabase>,
    thumbnailer: State<'_, Thumbnailer>,
    client: State<'_, Client>,
    uri: String,
    width: f64,
) -> Result<String, Error> {
    // - [Tokio decide how many threads](https://github.com/tokio-rs/tokio/discussions/3858)
    let width = width.round() as u32;

    let dir = thumbnailer.dir.clone();
    let tmb = { thumbnailer.cache.lock().infer_err()?.get(&uri).cloned() };
    if let Some(tmb) = tmb {
        let img = tokio::task::spawn_blocking(move || {
            let img = tmb.get_image(ThumbnailSize::get_appropriate_size(width), &dir)?;
            Ok(img)
        })
        .await
        .infer_err()??;
        return Ok(img.to_string_lossy().to_string());
    }

    let client = client.inner().clone();
    let u = uri.clone();
    let (tmb, img) = tokio::task::spawn_blocking(move || async move {
        let tmb = Thumbnail::new(&u, &dir, &client)
            .await?
            .look(|e| dbg!(e))
            .bad_err("coule not get an image from the uri")?;
        let img = tmb.get_image(ThumbnailSize::get_appropriate_size(width), &dir)?;
        Ok((tmb, img))
    })
    .await
    .infer_err()?
    .await?;

    let old = { thumbnailer.cache.lock().infer_err()?.push(uri, tmb) };
    if let Some((_uri, tmb)) = old {
        tmb.delete(&thumbnailer.dir)?;
    }

    Ok(img.to_string_lossy().to_string())
}

pub struct Thumbnailer {
    dir: PathBuf,
    cache: Mutex<LruCache<String, Thumbnail>>,
}
impl Thumbnailer {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref().join("thumbnails");
        if !dir.exists() {
            std::fs::create_dir(&dir).infer_err()?;
        }
        Ok(Self {
            dir,
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(5000).unwrap())),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Thumbnail {
    width: u32,
    uuid: String,
}
impl Thumbnail {
    pub async fn new(
        uri: impl AsRef<str>,
        thumbnail_dir: impl AsRef<Path>,
        client: &Client,
    ) -> Result<Option<Self>, Error> {
        let uri = uri.as_ref();
        dbg!("uri => ", uri);
        let id = uuid::Uuid::new_v4();
        let uuid = id.hyphenated().to_string();
        let dir = thumbnail_dir.as_ref().join(&uuid);
        std::fs::create_dir(&dir).infer_err()?;

        match PathBuf::from_str(uri).ok() {
            Some(p) if p.exists() => {
                let dest_path = dir.join(ThumbnailSize::Original.as_ref());

                // TODO:? sync or async file io ?
                std::fs::copy(p, &dest_path).infer_err()?;
                let img_dimensions = Reader::open(dest_path)
                    .infer_err()?
                    .with_guessed_format()
                    .infer_err()?
                    .into_dimensions()
                    .infer_err()?;

                return Ok(Some(Self {
                    width: img_dimensions.0,
                    uuid,
                }));
            }
            _ => {
                // check if uri
                if Url::parse(uri).is_ok() {
                    let p = dir.join(ThumbnailSize::Original.as_ref()).look(|e| dbg!(e));
                    let resp = client.get(uri).send().await.infer_err()?;
                    resp.headers()
                        .look(|e| dbg!(e))
                        .get(reqwest::header::CONTENT_TYPE)
                        .bad_err("no content type in response")?
                        .to_str()
                        .infer_err()?
                        .contains("image")
                        .then_some(())
                        .bad_err("response is not an image")?; // bail out if not an image
                    dbg!("fetching image!!!!!");
                    let bytes = resp.bytes().await.infer_err()?;
                    dbg!("got image!!!!");

                    // TODO:? sync or async file io ?
                    let mut file = BufWriter::new(File::create(&p).infer_err()?);
                    file.write(&bytes).infer_err().look(|e| dbg!(e))?;
                    let img_dimensions = Reader::new(Cursor::new(&bytes))
                        .with_guessed_format()
                        .expect("Cursor io never fails")
                        .into_dimensions()
                        .infer_err()?;

                    return Ok(Some(Self {
                        width: img_dimensions.0,
                        uuid,
                    }));
                }
            }
        }
        Ok(None)
    }

    pub fn get_image(
        &self,
        size: ThumbnailSize,
        thumbnail_dir: impl AsRef<Path>,
    ) -> Result<PathBuf, Error> {
        let path = thumbnail_dir.as_ref().join(&self.uuid);
        let original_img = path.join(ThumbnailSize::Original.as_ref());

        if size.value().map(|v| v < self.width).unwrap_or(false) {
            // thumbnail
            let thumbnail = path.join(size.as_ref());
            if !thumbnail.exists() {
                let s = size
                    .value()
                    .expect("this should have a value as it is not Thumnail::Original");

                let reader = Reader::open(&original_img)
                    .infer_err()?
                    .with_guessed_format()
                    .infer_err()?;
                let img = reader.decode().infer_err()?;
                img.thumbnail(s, u32::MAX)
                    .save_with_format(&thumbnail, image::ImageFormat::Jpeg)
                    .infer_err()?;
            }
            Ok(thumbnail)
        } else {
            // original image
            Ok(original_img)
        }
    }

    pub fn delete(self, thumbnail_dir: impl AsRef<Path>) -> Result<(), Error> {
        let path = thumbnail_dir.as_ref().join(self.uuid);
        std::fs::remove_dir_all(path).infer_err()?;
        Ok(())
    }
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
