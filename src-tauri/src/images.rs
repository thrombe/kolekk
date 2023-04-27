#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, fmt::Debug, path::PathBuf, str::FromStr};

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
        sync::Mutex,
    };

    use caches::{AdaptiveCache, Cache};
    use derivative::Derivative;
    use image::io::Reader;
    use kolekk_types::utility::ThumbnailSize;
    use reqwest::{Client, Url};
    use serde::{Deserialize, Serialize};
    use tauri::{AppHandle, Manager, State, WindowEvent};
    use tokio::select;

    use crate::{
        bad_error::{BadError, Error, InferBadError, Inspectable},
        config::AppConfig,
        database::AppDatabase,
    };

    pub async fn init_thumbnailer(
        app_handle: &AppHandle,
        conf: &AppConfig,
        client: Client,
    ) -> Result<(), Error> {
        let handle = app_handle.app_handle();
        app_handle.manage(Thumbnailer::new(&conf.app_data_dir, client).await?);
        // TODO: ugly unwraps :(
        app_handle
            // .get_window("kolekk")
            .windows()
            .into_values()
            .next()
            .expect("no window?")
            .on_window_event(move |e| match e {
                WindowEvent::Destroyed | WindowEvent::CloseRequested { .. } => {
                    let thumbnailer = handle.state::<Thumbnailer>().inner();
                    let db = handle.state::<AppDatabase>().inner();
                    let cache = thumbnailer.shut_down().unwrap();
                    let v = cache.frequent_iter().collect::<Vec<_>>();
                    // let v = LruCacheStore {
                    //     v: v.into_iter().map(|e| (e.0.clone(), e.1.clone())).collect(),
                    // };
                    // let mut doc = Document::new();
                    // doc.add_facet(db.get_field(kolekk_types::objects::Fields::Type), TypeFacet::Temp("/cache/".into()).facet());
                    // v.add(db, &mut doc).unwrap();
                }
                _ => {}
            });
        Ok(())
    }

    #[tauri::command]
    pub fn get_thumbnail_size(width: f64) -> ThumbnailSize {
        ThumbnailSize::get_appropriate_size(width.round() as _)
    }

    #[tauri::command]
    pub async fn image_thumbnail(
        db: State<'_, AppDatabase>,
        thumbnailer: State<'_, Thumbnailer>,
        client: State<'_, Client>,
        uri: String,
        thumbnail_size: ThumbnailSize,
    ) -> Result<PathBuf, Error> {
        // - [Tokio decide how many threads](https://github.com/tokio-rs/tokio/discussions/3858)
        dbg!(&uri, &thumbnail_size);
        thumbnailer.image_thumbnail(thumbnail_size, uri).await
    }

    // thumbnailer using single threaded cache without mutex
    // spawn a tokio task for doing the cache + channel stuff
    // spawn a threadpool for the processing itself
    // use async channels to communicate to all the requesters that a thumbnail has been created, and these requesters can wait on these
    //   channels to respond to javascript
    // when threadpool creates thumbnails, the main task can fill in the cache entry and respond via channels
    // if an entry is already found, just return the required thumbnail
    // benefits:
    //   - easy to delete requests
    //     - delete_thumbnail_requests can just decrement a counter of a url in a fifo queue
    //   - multiple requests -> single action (multiple requests for the same url in quick succession do not create
    //     a bunch of different thumbnails)
    //   - can save thumbnails to a db
    // cons:
    //   - no stretto's LFU cache
    //   - thumbnail weights should be based on their size (which stretto can handle nicely)
    //
    // - a ton of requests get made from the frontend
    // - must not save dulpicate thumbnails
    // - must not do duplicate work to create thumbnails
    // - must return the appropriate thumbnail path to each call (no matter if it is dulpicate or no)
    //
    // - need to cache what thumbnail sizes are created
    // - need to be able to wait for results of computation of other requests
    //
    // - thumbnail stores what sizes already have an image
    // - if i see a new uri, immediately insert some object in cache then start processing on it
    //   - Cache<_, Tmb>, enum Tmb { WaitingMultipleNew(Vec<Sender<_>>), WaitingNew(Sender<_>), Completed(Thumbnail) }
    //   - if WaitingNew or WaitingMultipleNew, swap the varient with Completed, and send the appropriate path from these senders
    // - if old uri new size
    //   - Completed(Thumbnail, Box<[Size, Status]>), enum Status { None, Waiting(Sender<_>), WaitingMultiple(Sender<_>), Completed }
    //
    // - have a oneshot channel in Thumbnailer to send Cache from this seperate task to somewhere - where it can be shoved in db
    //
    // - [tokio::sync - Rust](https://docs.rs/tokio/latest/tokio/sync/index.html#mpsc-channel)

    #[derive(Derivative)]
    #[derivative(Debug)]
    pub struct ThumbnailRequest {
        size: ThumbnailSize,
        uri: String,
        #[derivative(Debug = "ignore")]
        tx: tokio::sync::oneshot::Sender<Result<PathBuf, Error>>,
    }

    // #[derive(Deserialize, Serialize)]
    pub enum ThumbnailStatus {
        // #[serde(skip)]
        Waiting(Vec<ThumbnailRequest>),
        Completed {
            tmb: Thumbnail,
            sizes: Box<[ThumbnailSizeStatus; 8]>,
        },
    }
    // #[derive(Deserialize, Serialize)]
    #[derive(Derivative)]
    #[derivative(Debug)]
    pub enum ThumbnailSizeStatus {
        None,
        // #[serde(deserialize_with = "none_status", skip_serializing)]
        Waiting(
            #[derivative(Debug = "ignore")]
            Vec<tokio::sync::oneshot::Sender<Result<PathBuf, Error>>>,
        ),
        Completed,
    }
    impl Default for ThumbnailSizeStatus {
        fn default() -> Self {
            Self::None
        }
    }
    // fn none_status<'de, D>(deserializer: D) -> Result<, D::Error>
    // where
    //     D: serde::Deserializer<'de>,
    // {
    //     let value = i64::deserialize(deserializer)?;
    //     let v = (value > 0).then_some(value as _);
    //     Ok(v)
    // }

    pub struct Thumbnailer {
        tx: tokio::sync::mpsc::UnboundedSender<ThumbnailRequest>,

        close_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
        cache_rx:
            Mutex<Option<tokio::sync::oneshot::Receiver<AdaptiveCache<String, ThumbnailStatus>>>>,
    }

    // #[derive(Deserialize, Serialize)]
    pub struct LruCacheStore {
        v: Vec<(String, ThumbnailStatus)>,
    }
    // impl AutoDbAble for LruCacheStore {}
    // impl AutoDbAble for ThumbnailStatus {}

    #[derive(Debug)]
    pub enum ThumbnailWorkResult {
        NewThumbnail {
            tmb: Thumbnail,
            uri: String,
        },
        NewSize {
            uri: String,
            size: ThumbnailSize,
            path: PathBuf,
        },
    }

    impl Thumbnailer {
        pub async fn new(dir: impl AsRef<Path>, client: Client) -> Result<Self, Error> {
            let dir = dir.as_ref().join("thumbnails");
            if !dir.exists() {
                std::fs::create_dir(&dir).infer_err()?;
            }

            // receive requests through this
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ThumbnailRequest>();

            // exit signal
            let (close_tx, mut close_rx) = tokio::sync::oneshot::channel::<()>();

            // send cache through this
            let (cache_tx, cache_rx) =
                tokio::sync::oneshot::channel::<AdaptiveCache<String, ThumbnailStatus>>(); // exit signal
            let mut cache: AdaptiveCache<String, ThumbnailStatus> =
                AdaptiveCache::new(10_000).infer_err()?;

            // receive work results from work tasks through this
            let (work_tx, mut work_rx) =
                tokio::sync::mpsc::unbounded_channel::<ThumbnailWorkResult>();

            let requests_tx = tx.clone();
            let _r = tokio::task::spawn(async move {
                loop {
                    select! {
                        biased;
                        // another channel to see if app is to be shut down
                        _ = &mut close_rx => {
                            dbg!("shutting down thumbnailer!");
                            let _ = cache_tx.send(cache);
                            break;
                        }

                        // results of tasks
                        c = work_rx.recv() => {
                            dbg!(&c);
                            match c {
                                Some(t) => {
                                    // TODO: bad unwrap
                                    Self::handle_results(t, &requests_tx, &mut cache, &dir, &work_tx, &client).await.unwrap();
                                }
                                None => {
                                    // channel closed
                                    todo!();
                                }
                            }
                        }
                        // new requests from Thumbnailer
                        r = rx.recv() => {
                            dbg!(&r);
                            match r {
                                Some(r) => {
                                    // TODO: bad unwrap
                                    Self::handle_requests(r, &mut cache, &dir, &work_tx, &client).await.unwrap();
                                }
                                None => {
                                    // channel closed
                                    todo!();
                                }
                            }
                        }
                    }
                }
                Result::<_, Error>::Ok(())
            });

            Ok(Self {
                tx,
                close_tx: Mutex::new(Some(close_tx)),
                cache_rx: Mutex::new(Some(cache_rx)),
            })
        }

        async fn handle_results(
            t: ThumbnailWorkResult,
            request_tx: &tokio::sync::mpsc::UnboundedSender<ThumbnailRequest>,
            cache: &mut AdaptiveCache<String, ThumbnailStatus>,
            dir: &std::path::Path,
            work_tx: &tokio::sync::mpsc::UnboundedSender<ThumbnailWorkResult>,
            client: &Client,
        ) -> Result<(), Error> {
            match t {
                ThumbnailWorkResult::NewThumbnail { tmb, uri } => {
                    let k = cache.get_mut(&uri).expect("unreachable!");
                    match k {
                        ThumbnailStatus::Waiting(v) => {
                            // it is okay to send first and then set cache, as this part is not really concurrent
                            while let Some(r) = v.pop() {
                                // send the same request again, as the requested size might not be available yet
                                request_tx.send(r).expect("cannot send through channel");
                            }
                            cache.put(
                                uri,
                                ThumbnailStatus::Completed {
                                    tmb,
                                    // TODO: can mark completed for the ones with higher resolution than the original image
                                    sizes: Box::new([
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                        ThumbnailSizeStatus::None,
                                    ]),
                                },
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                ThumbnailWorkResult::NewSize { uri, size, path } => {
                    let k = cache.get_mut(&uri).expect("unreachable");
                    match k {
                        ThumbnailStatus::Completed { tmb, sizes } => {
                            match &mut sizes[size as usize] {
                                ThumbnailSizeStatus::Waiting(v) => {
                                    while let Some(s) = v.pop() {
                                        s.send(Ok(path.clone()))
                                            .expect("cannot send through channel");
                                    }
                                    sizes[size as usize] = ThumbnailSizeStatus::Completed;
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Ok(())
        }

        async fn handle_requests(
            r: ThumbnailRequest,
            cache: &mut AdaptiveCache<String, ThumbnailStatus>,
            dir: &std::path::Path,
            work_tx: &tokio::sync::mpsc::UnboundedSender<ThumbnailWorkResult>,
            client: &Client,
        ) -> Result<(), Error> {
            let uri = r.uri.clone();
            let t = cache.get_mut(&uri);
            match t {
                Some(t) => {
                    match t {
                        ThumbnailStatus::Waiting(v) => {
                            v.push(r);
                        }
                        ThumbnailStatus::Completed { tmb, .. }
                            if r.size == ThumbnailSize::Original =>
                        {
                            r.tx.send(Ok(dir.join(&tmb.uuid).join(r.size.as_ref())))
                                .expect("could not send through channel");
                        }
                        ThumbnailStatus::Completed { tmb, sizes } => {
                            // check if the required size is available or not and spawn a task if required
                            match &mut sizes[r.size as usize] {
                                ThumbnailSizeStatus::None => {
                                    sizes[r.size as usize] =
                                        ThumbnailSizeStatus::Waiting(vec![r.tx]);
                                    let work_tx = work_tx.clone();
                                    let uri = r.uri.clone();
                                    let dir = dir.to_path_buf();
                                    let size = r.size;
                                    let tmb = tmb.clone();
                                    let _r = tokio::task::spawn(async move {
                                        let _r = tokio::task::spawn_blocking(move || {
                                            // TODO: bad unwrap
                                            let path = tmb.get_image(size, dir).unwrap();
                                            work_tx
                                                .send(ThumbnailWorkResult::NewSize {
                                                    uri,
                                                    size,
                                                    path,
                                                })
                                                .expect("channel closed or something");
                                        })
                                        .await;
                                    });
                                }
                                ThumbnailSizeStatus::Waiting(v) => {
                                    v.push(r.tx);
                                }
                                ThumbnailSizeStatus::Completed => {
                                    r.tx.send(Ok(dir
                                        .join(&tmb.uuid)
                                        .join(tmb.get_appropriate_size(r.size).as_ref())))
                                        .expect("could not send");
                                }
                            }
                        }
                    }
                }
                None => {
                    dbg!(&r);
                    let uri = r.uri.clone();
                    // TODO: delete the evicted thumbnails
                    match cache.put(r.uri.clone(), ThumbnailStatus::Waiting(vec![r])) {
                        caches::PutResult::Put => (),
                        caches::PutResult::Update(_) => todo!(),
                        caches::PutResult::Evicted { key, value } => todo!(),
                        caches::PutResult::EvictedAndUpdate { evicted, update } => todo!(),
                    }
                    let work_tx = work_tx.clone();

                    // download | copy image to thumbnail dir
                    let dir = dir.to_path_buf();
                    let client = client.clone();
                    let _r = tokio::task::spawn(async move {
                        dbg!(&uri);
                        // TODO: bad unwraps
                        let tmb = Thumbnail::new(&uri, &dir, &client)
                            .await
                            .look(|e| dbg!(e))
                            .unwrap()
                            .bad_err("coule not get an image from the uri")
                            .unwrap();
                        // let img = tmb.get_image(thumbnail_size, &dir)?;
                        work_tx
                            .send(ThumbnailWorkResult::NewThumbnail { tmb, uri })
                            .expect("channel closed or something");
                    });
                }
            }
            Ok(())
        }

        pub async fn image_thumbnail(
            &self,
            size: ThumbnailSize,
            uri: String,
        ) -> Result<PathBuf, Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.tx
                .send(ThumbnailRequest { size, uri, tx }.look(|e| dbg!(e)))
                .infer_err()?;
            rx.await.infer_err()?.look(|e| dbg!(e, size))
            // TODO: maybe some kinda timeout?
        }

        // TODO: how do i enforce that calling image_thumbnail after calling this method fails
        pub fn shut_down(&self) -> Result<AdaptiveCache<String, ThumbnailStatus>, Error> {
            // TODO: unwrap??
            self.close_tx
                .lock()
                .infer_err()?
                .take()
                .unwrap()
                .send(())
                .unwrap();
            let cache = self
                .cache_rx
                .lock()
                .infer_err()?
                .take()
                .unwrap()
                .blocking_recv()
                .infer_err()?;
            Ok(cache)
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
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

        fn needs_new_thumbnail(&self, size: ThumbnailSize) -> bool {
            size.value().map(|v| v < self.width).unwrap_or(false)
        }

        pub fn get_appropriate_size(&self, size: ThumbnailSize) -> ThumbnailSize {
            if self.needs_new_thumbnail(size) {
                size
            } else {
                ThumbnailSize::Original
            }
        }

        pub fn get_image(
            &self,
            size: ThumbnailSize,
            thumbnail_dir: impl AsRef<Path>,
        ) -> Result<PathBuf, Error> {
            let path = thumbnail_dir.as_ref().join(&self.uuid);
            let original_img = path.join(ThumbnailSize::Original.as_ref());

            if self.needs_new_thumbnail(size) {
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
