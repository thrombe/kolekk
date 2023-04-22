#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    collections::HashSet,
    fmt::Debug,
    path::PathBuf,
    str::FromStr,
};

use kolekk_types::{
    objects::Image,
    utility::{BasePath, ByteArrayFile, DragDropPaste, Path},
};
use reqwest::Client;
use tauri::State;

use crate::{
    bad_error::{Error, InferBadError, Inspectable},
    config::AppConfig,
    database::AppDatabase,
    filesystem::{file_mdata, get_path, path_is_in_dir, Filable, FilableUri, FiledResult},
};

pub mod thumbnails {
    #[allow(unused_imports)]
    use crate::{dbg, debug, error};

    use std::{
        fs::File,
        io::{BufWriter, Cursor, Write},
        path::{Path, PathBuf},
        str::FromStr,
    };

    use image::io::Reader;
    use kolekk_types::utility::ThumbnailSize;
    use reqwest::{Client, Url};
    use stretto::AsyncCache;
    use tauri::State;

    use crate::{
        bad_error::{BadError, Error, InferBadError, Inspectable},
        database::AppDatabase,
    };

    #[tauri::command]
    pub fn get_thumbnail_size(width: f64) -> ThumbnailSize {
        ThumbnailSize::get_appropriate_size(width.round() as _)
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
        thumbnail_size: ThumbnailSize,
    ) -> Result<String, Error> {
        // - [Tokio decide how many threads](https://github.com/tokio-rs/tokio/discussions/3858)
        let dir = thumbnailer.dir.clone();

        // let tmb = { thumbnailer.cache.lock().infer_err()?.get(&uri).cloned()  };
        // dbg!(&tmb);
        let tmb = { thumbnailer.cache.get(&uri).map(|v| v.value().clone()) };
        dbg!(&tmb, thumbnailer.cache.len());

        if let Some(tmb) = tmb {
            let img = tokio::task::spawn_blocking(move || {
                let img = tmb.get_image(thumbnail_size, &dir)?;
                Ok(img)
            })
            .await
            .infer_err()??;
            return Ok(img.to_string_lossy().to_string());
        }

        let client = client.inner().clone();
        let u = uri.clone();
        // TODO: no thumbnail new img if the folder already exists. if fail, remove the folder
        let (tmb, img) = tokio::task::spawn_blocking(move || async move {
            let tmb = Thumbnail::new(&u, &dir, &client)
                .await
                .look(|e| dbg!(e))?
                .bad_err("coule not get an image from the uri")?;
            let img = tmb.get_image(thumbnail_size, &dir)?;
            Ok((tmb, img))
        })
        .await
        .infer_err()?
        .await?;

        // { thumbnailer.cache.lock().infer_err()?.push(uri, tmb) }.look(|e| dbg!(e));
        { thumbnailer.cache.insert(uri.clone(), tmb, 1).await }.look(|e| dbg!(e));
        thumbnailer.cache.wait().await.infer_err()?;
        {
            thumbnailer
                .cache
                .get(&uri)
                .map(|v| v.value().clone())
                .look(|e| dbg!(e, thumbnailer.cache.len()));
        }

        Ok(img.to_string_lossy().to_string())
    }

    pub struct Thumbnailer {
        dir: PathBuf,
        // cache: Mutex<LruCache<String, Thumbnail>>,
        cache: AsyncCache<String, Thumbnail>,
    }
    impl Thumbnailer {
        pub fn new(dir: impl AsRef<Path>) -> Result<Self, Error> {
            let dir = dir.as_ref().join("thumbnails");
            if !dir.exists() {
                std::fs::create_dir(&dir).infer_err()?;
            }
            Ok(Self {
                dir,
                // cache: Mutex::new(LruCache::new(NonZeroUsize::new(5000).unwrap())),
                cache: AsyncCache::builder(50000, 5000)
                    .set_ignore_internal_cost(true)
                    .finalize(tokio::spawn)
                    .infer_err()?,
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
                        .infer_err()
                        .look(|e| dbg!(e))?;

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
                            .bad_err("no content type in response")
                            .look(|e| dbg!(e))?
                            .to_str()
                            .infer_err()?
                            .contains("image")
                            .then_some(())
                            .bad_err("response is not an image")
                            .look(|e| dbg!(e))?; // bail out if not an image
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
                            .infer_err()
                            .look(|e| dbg!(e))?;

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
                    dbg!("creating new thumbnail!!!");
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
}

#[tauri::command]
pub async fn get_images(
    data: DragDropPaste<ByteArrayFile>,
    config: State<'_, AppConfig>,
    db: State<'_, AppDatabase>,
    client: State<'_, Client>,
) -> Result<Vec<Image>, Error> {
    let res = save_images(&data, client.inner(), &config)
        .await
        .look(|e| dbg!(e))?;

    // TODO: if file is already in database, then remove the file that was just saved

    let imgs = res
        .into_iter()
        .map(|file| {
            let mdata = file_mdata(get_path(&file.dest, config.inner()))?;
            let img = Image {
                src: file.src,
                title: file.title,
                path: file.dest,
                chksum: mdata.chksum.into(),
                size: mdata.size as _,
            };
            Ok(img)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(imgs)
}

pub async fn save_images<F: Debug + Filable>(
    data: &DragDropPaste<F>,
    client: &Client,
    config: &AppConfig,
) -> Result<Vec<FiledResult>, Error> {
    let images_path = Path {
        path: PathBuf::from("images"),
        base: BasePath::AppDataDir,
    };
    let images_dir = get_path(&images_path, config);

    if !images_dir.exists() {
        std::fs::create_dir(&images_dir).infer_err()?;
    }

    let mut saved_files = data
        .files
        .as_ref()
        .map(|v| &v[..])
        .unwrap_or(&[])
        .iter()
        .filter_map(|f| f.save_in_dir(&images_path, config).ok())
        .collect::<Vec<_>>();
    let potential_links: HashSet<_> = data
        .file_uris
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect())
        .or_else(|| data.text.as_ref().map(|t| t.lines().collect()))
        .or_else(|| data.text_html.as_ref().map(|h| todo!())) // parse all potential urls if the text does not exist
        // .or(data.uri_list.as_ref().map(|u| todo!())) // donno if including this does any good
        .unwrap_or_default();

    let mut reqs = vec![];
    for u in potential_links {
        if let Some(p) = PathBuf::from_str(u).ok().filter(|p| p.is_file()) {
            if !path_is_in_dir(&p, &images_dir).unwrap_or(false) {
                let _ = p
                    .as_path()
                    .save_in_dir(&images_path, config)
                    .ok()
                    .map(|pb| saved_files.push(pb));
            }
        } else {
            reqs.push(
                FilableUri {
                    title: None,
                    src: u,
                    client,
                    content_type_contains: "image",
                }
                .save_in_dir(&images_path, config),
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
