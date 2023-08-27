#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{collections::HashSet, fmt::Debug, path::PathBuf, str::FromStr};

use kolekk_types::{
    objects::Image,
    utility::{BasePath, ByteArrayFile, DdpInfo, DirFiles, DragDropPaste, Path},
};
use reqwest::Client;
use tauri::State;

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
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
        num::NonZeroUsize,
        path::{Path, PathBuf},
        str::FromStr,
        sync::Mutex,
    };

    use derivative::Derivative;
    use image::io::Reader;
    use kolekk_types::{
        objects::{Fields, TypeFacet},
        utility::ThumbnailSize,
    };
    use lru::LruCache;
    use reqwest::{Client, Url};
    use serde::{Deserialize, Serialize};
    use tantivy::{
        collector::TopDocs, query::TermQuery, schema::IndexRecordOption, Document, Term,
    };
    use tauri::{AppHandle, Manager, State, WindowEvent};
    use tokio::select;

    use crate::{
        bad_error::{BadError, Error, InferBadError, Inspectable, InspectableErr},
        config::AppConfig,
        database::{AppDatabase, AutoDbAble, DbAble, FacetFrom},
    };

    pub async fn init_thumbnailer(
        app_handle: &AppHandle,
        conf: &AppConfig,
        db: &AppDatabase,
        client: Client,
    ) -> Result<(), Error> {
        let handle = app_handle.app_handle();
        app_handle.manage(Thumbnailer::new(&conf.app_data_dir, client, db).await?);
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
                    let Some(cache) = thumbnailer.shut_down().unwrap() else {
                        // return if already shut down
                        return;
                    };
                    let v = cache.into_iter(); // mru order
                    let v = LruCacheStore {
                        v: v.filter_map(|e| e.1.kinda_clone().map(|t| (e.0, t)))
                            .collect(),
                    };

                    let facet = TypeFacet::Temp("/cache/thumbnails_cache".into()).facet();

                    let mut writer = db.index_writer.write().unwrap();
                    let _opstamp =
                        writer.delete_term(Term::from_facet(db.get_field(Fields::Type), &facet));

                    let mut doc = Document::new();
                    doc.add_facet(db.get_field(Fields::Type), facet);
                    v.add(db, &mut doc)
                        .expect("failed add thumbnail cache to Document");

                    dbg!("saving cache");

                    let _opstamp = writer
                        .add_document(doc)
                        .expect("eror: failed to add cache document to tantivy");
                    let _opstamp = writer
                        .commit()
                        .expect("eror: failed to commit changes to tantivy");
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

    #[derive(Deserialize, Serialize)]
    pub enum ThumbnailStatus {
        #[serde(skip)]
        Waiting(Vec<ThumbnailRequest>),
        Completed {
            tmb: Thumbnail,
            sizes: Box<[ThumbnailSizeStatus; 9]>,
        },
    }
    impl ThumbnailStatus {
        pub fn is_completed(&self) -> bool {
            matches!(self, Self::Completed { .. })
        }
        pub fn kinda_clone(&self) -> Option<Self> {
            match self {
                ThumbnailStatus::Waiting(_) => None,
                ThumbnailStatus::Completed { tmb, sizes } => {
                    let s = Self::Completed {
                        tmb: tmb.clone(),
                        sizes: Box::new([
                            sizes[0].kinda_clone(),
                            sizes[1].kinda_clone(),
                            sizes[2].kinda_clone(),
                            sizes[3].kinda_clone(),
                            sizes[4].kinda_clone(),
                            sizes[5].kinda_clone(),
                            sizes[6].kinda_clone(),
                            sizes[7].kinda_clone(),
                            sizes[8].kinda_clone(),
                        ]),
                    };
                    Some(s)
                }
            }
        }
    }

    #[derive(Derivative, Deserialize, Serialize)]
    #[derivative(Debug)]
    pub enum ThumbnailSizeStatus {
        None,
        #[serde(skip)]
        Waiting(
            #[derivative(Debug = "ignore")]
            Vec<tokio::sync::oneshot::Sender<Result<PathBuf, Error>>>,
        ),
        Completed,
        Original,
    }

    impl ThumbnailSizeStatus {
        pub fn kinda_clone(&self) -> Self {
            match self {
                ThumbnailSizeStatus::Waiting(_) => ThumbnailSizeStatus::None,
                ThumbnailSizeStatus::None => ThumbnailSizeStatus::None,
                ThumbnailSizeStatus::Completed => ThumbnailSizeStatus::Completed,
                ThumbnailSizeStatus::Original => ThumbnailSizeStatus::Original,
            }
        }
    }

    pub struct Thumbnailer {
        tx: tokio::sync::mpsc::UnboundedSender<ThumbnailRequest>,

        close_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
        cache_rx: Mutex<Option<tokio::sync::oneshot::Receiver<LruCache<String, ThumbnailStatus>>>>,
    }

    #[derive(Deserialize, Serialize, Default)]
    pub struct LruCacheStore {
        v: Vec<(String, ThumbnailStatus)>,
    }
    impl AutoDbAble for LruCacheStore {}

    #[derive(Debug)]
    #[allow(clippy::enum_variant_names)]
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
        NewSizeError {
            err: Error,
            uri: String,
            size: ThumbnailSize,
        },
        NewThumbnailError {
            err: Error,
            uri: String,
        },
        NewThumbnailNone {
            uri: String,
        },
    }

    impl Thumbnailer {
        pub async fn new(
            dir: impl AsRef<Path>,
            client: Client,
            db: &AppDatabase,
        ) -> Result<Self, Error> {
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
                tokio::sync::oneshot::channel::<LruCache<String, ThumbnailStatus>>(); // exit signal
            let mut cache: LruCache<String, ThumbnailStatus> =
                LruCache::new(NonZeroUsize::new(2000).unwrap());
            let searcher = db.get_searcher();
            let cache_store: LruCacheStore = searcher
                .search(
                    &TermQuery::new(
                        Term::from_facet(
                            db.get_field(Fields::Type),
                            &TypeFacet::Temp("/cache/thumbnails_cache".into()).facet(),
                        ),
                        IndexRecordOption::Basic,
                    ),
                    &TopDocs::with_limit(1),
                )
                .infer_err()?
                .first()
                .and_then(|&(_, add)| searcher.doc(add).ok())
                .and_then(|mut doc| DbAble::take(db, &mut doc).look_err(|e| dbg!(e)).ok())
                .unwrap_or_default();
            cache_store.v.into_iter().rev().for_each(|(k, v)| {
                if !matches!(cache.put(k, v), None) {
                    unreachable!();
                }
            });

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
                            // NOTE: this also executes if this channel is dead - which is fine
                            dbg!("shutting down thumbnailer!");
                            let _ = cache_tx.send(cache);
                            break;
                        }

                        // results of tasks
                        c = work_rx.recv() => {
                            dbg!(&c);
                            let t = c.expect("dead channel");
                            Self::handle_results(t, &requests_tx, &mut cache, &dir, &work_tx, &client).await;
                        }
                        // new requests from Thumbnailer
                        r = rx.recv() => {
                            dbg!(&r);
                            let r = r.expect("dead channel");
                            Self::handle_requests(r, &mut cache, &dir, &work_tx, &client).await;
                        }
                    }
                }
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
            cache: &mut LruCache<String, ThumbnailStatus>,
            dir: &std::path::Path,
            work_tx: &tokio::sync::mpsc::UnboundedSender<ThumbnailWorkResult>,
            client: &Client,
        ) {
            match t {
                ThumbnailWorkResult::NewThumbnail { tmb, uri } => {
                    let k = cache.get_mut(&uri).expect("unreachable!");
                    match k {
                        ThumbnailStatus::Waiting(v) => {
                            // it is okay to send first and then set cache, as this part is not really concurrent
                            while let Some(r) = v.pop() {
                                // send the same request again, as the requested size might not be available yet
                                request_tx.send(r).expect("dead channel");
                            }
                            let init_size = |size| {
                                if tmb.get_appropriate_size(size).eq(&ThumbnailSize::Original) {
                                    ThumbnailSizeStatus::Original
                                } else {
                                    ThumbnailSizeStatus::None
                                }
                            };
                            match cache.put(
                                uri,
                                ThumbnailStatus::Completed {
                                    // don't try to generate thumbnails for images with less pixels than what the size asks for
                                    sizes: Box::new([
                                        init_size(ThumbnailSize::W50),
                                        init_size(ThumbnailSize::W100),
                                        init_size(ThumbnailSize::W150),
                                        init_size(ThumbnailSize::W200),
                                        init_size(ThumbnailSize::W350),
                                        init_size(ThumbnailSize::W500),
                                        init_size(ThumbnailSize::W750),
                                        init_size(ThumbnailSize::W1000),
                                        init_size(ThumbnailSize::W1920),
                                    ]),
                                    tmb,
                                },
                            ) {
                                Some(_) => (),
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                ThumbnailWorkResult::NewSize { uri, size, path } => {
                    let k = cache.get_mut(&uri).expect("unreachable!");
                    match k {
                        ThumbnailStatus::Completed { tmb, sizes } => {
                            match &mut sizes[size as usize] {
                                ThumbnailSizeStatus::Waiting(v) => {
                                    while let Some(s) = v.pop() {
                                        s.send(Ok(path.clone())).expect("dead channel");
                                    }
                                    sizes[size as usize] = ThumbnailSizeStatus::Completed;
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                ThumbnailWorkResult::NewSizeError { uri, size, err } => {
                    let k = cache.get_mut(&uri).expect("unreachable!");
                    match k {
                        ThumbnailStatus::Completed { tmb, sizes } => {
                            match &mut sizes[size as usize] {
                                ThumbnailSizeStatus::Waiting(v) => {
                                    while let Some(s) = v.pop() {
                                        s.send(Err(err.clone())).expect("dead channel");
                                    }
                                    sizes[size as usize] = ThumbnailSizeStatus::None;
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                ThumbnailWorkResult::NewThumbnailError { uri, err } => {
                    let k = cache.pop(&uri).expect("unreachable!");
                    match k {
                        ThumbnailStatus::Waiting(mut v) => {
                            while let Some(s) = v.pop() {
                                s.tx.send(Err(err.clone())).expect("dead channel");
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                ThumbnailWorkResult::NewThumbnailNone { uri } => {
                    let k = cache.pop(&uri).expect("unreachable!");
                    match k {
                        ThumbnailStatus::Waiting(mut v) => {
                            while let Some(s) = v.pop() {
                                s.tx.send(None.bad_err("could not get an image from the uri"))
                                    .expect("dead channel");
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        async fn handle_requests(
            r: ThumbnailRequest,
            cache: &mut LruCache<String, ThumbnailStatus>,
            dir: &std::path::Path,
            work_tx: &tokio::sync::mpsc::UnboundedSender<ThumbnailWorkResult>,
            client: &Client,
        ) {
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
                                .expect("dead channel");
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
                                        let uri_for_err = uri.clone();
                                        let res = match tokio::task::spawn_blocking(move || {
                                            match tmb.create_thumbnail(size, &dir) {
                                                Ok(()) => ThumbnailWorkResult::NewSize {
                                                    uri,
                                                    size,
                                                    path: dir.join(&tmb.uuid).join(size.as_ref()),
                                                },
                                                Err(err) => ThumbnailWorkResult::NewSizeError {
                                                    err,
                                                    uri,
                                                    size,
                                                },
                                            }
                                        })
                                        .await
                                        .infer_err()
                                        {
                                            Ok(res) => res,
                                            Err(err) => ThumbnailWorkResult::NewSizeError {
                                                err,
                                                uri: uri_for_err,
                                                size,
                                            },
                                        };
                                        work_tx.send(res).expect("dead channel");
                                    });
                                }
                                ThumbnailSizeStatus::Waiting(v) => {
                                    v.push(r.tx);
                                }
                                ThumbnailSizeStatus::Completed => {
                                    r.tx.send(Ok(dir.join(&tmb.uuid).join(r.size.as_ref())))
                                        .expect("dead channel");
                                }
                                ThumbnailSizeStatus::Original => {
                                    r.tx.send(Ok(dir
                                        .join(&tmb.uuid)
                                        .join(ThumbnailSize::Original.as_ref())))
                                        .expect("dead channel");
                                }
                            }
                        }
                    }
                }
                None => {
                    dbg!(&r);
                    let uri = r.uri.clone();
                    // TODO: bad bad bad. all the .expect("unreachable!") used above can actually fail because of these evictions.
                    //       but it should be quite rare, as the items should get bumped up in the LRU cache when a new request is put up
                    //       for it. also the limit is high enough that requests will hopefully get resolved quicker than the ThumbnailStatus
                    //       associated with it gets evicted
                    //   - this can be dealt with by
                    //     - sending some kinda Evicted error through the channels in the evicted items
                    //     - ignoring the ThumbnailWorkResult for the evicted items
                    //       - NOTE: make sure to delete any files that might have been created or undo anything that needs to be undone
                    if let Some((k, v)) = cache.push(r.uri.clone(), ThumbnailStatus::Waiting(vec![r])) {
                        match v {
                            ThumbnailStatus::Completed { tmb, sizes } => {
                                let _ = std::fs::remove_dir_all(dir.join(tmb.uuid)).look(|e| dbg!(e));
                            }
                            ThumbnailStatus::Waiting(_) => (),
                        }
                    }
                    let work_tx = work_tx.clone();

                    // download | copy image to thumbnail dir
                    let dir = dir.to_path_buf();
                    let client = client.clone();
                    let _r = tokio::task::spawn(async move {
                        dbg!(&uri);
                        let res = match Thumbnail::new(&uri, &dir, &client).await.look(|e| dbg!(e))
                        {
                            Ok(t) => match t {
                                Some(tmb) => ThumbnailWorkResult::NewThumbnail { tmb, uri },
                                None => ThumbnailWorkResult::NewThumbnailNone { uri },
                            },
                            Err(err) => ThumbnailWorkResult::NewThumbnailError { err, uri },
                        };
                        work_tx.send(res).expect("dead channel");
                    });
                }
            }
        }

        // TODO:
        // - seperate lru cache for thumbnails of different types of objects
        //  - store HashMap<TypeFacet, LruCache>
        //    - max number of items is passed in when creating new cache object ig
        //  - multiple prioritiy of thumbnails
        //    - store HashMap<Priority, LruCache>
        // TODO: use semaphores to allow only 'n' concurrent downloads. rest can wait till some of the current downloads
        //   are complete. (too many concurrent downloads take up too much memory sometimes)
        // TODO: also look into cancelling these requests.
        //  - maybe require passing a unique id for each request.
        //    - id generated by a command
        //  - another command can take this id and add it to some set for cancellation
        //  - before any request is handled, check if it is cancelled.
        //  - after any requst is handled, check if there are any cancel requests and remove those from the cancel set (saving memory)
        //    - then return None even though the task is completed successfully
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
        /// returns None if it is already shut down
        pub fn shut_down(&self) -> Result<Option<LruCache<String, ThumbnailStatus>>, Error> {
            let close_tx = match self.close_tx.lock().infer_err()?.take() {
                Some(c) => c,
                None => return Ok(None),
            };
            close_tx.send(()).ok().bad_err("dead channel")?;
            let cache = self
                .cache_rx
                .lock()
                .infer_err()?
                .take()
                .bad_err("no cache channel found")?
                .blocking_recv()
                .infer_err()?;
            Ok(Some(cache))
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Thumbnail {
        width: u32,
        uuid: String,
    }
    impl Thumbnail {
        // TODO: maybe these should be concrete errors :/
        /// returns None if
        ///  - passed string was not a Path | Url
        ///  - path was not an image / unsupported image format
        ///  - url was not an image
        ///  - url does not return a CONTENT_TYPE header
        ///    - assume that url was not an image
        /// returns Err if
        ///  - some IO error
        ///  - url with CONTENT_TYPE containing image is unsupported
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
                    match Reader::open(dest_path)
                        .infer_err()?
                        .with_guessed_format()
                        .infer_err()?
                        .into_dimensions()
                    {
                        Ok(img_dimensions) => {
                            return Ok(Some(Self {
                                width: img_dimensions.0,
                                uuid,
                            }));
                        }
                        Err(image::ImageError::Unsupported(e)) => {
                            return Ok(None);
                        }
                        Err(e) => {
                            return Err(e).infer_err();
                        }
                    }
                }
                _ => {
                    // check if uri
                    if Url::parse(uri).is_ok() {
                        let p = dir.join(ThumbnailSize::Original.as_ref()).look(|e| dbg!(e));
                        let resp = client.get(uri).send().await.infer_err()?;
                        let Some(h) = resp
                            .headers()
                            .look(|e| dbg!(e))
                            .get(reqwest::header::CONTENT_TYPE)
                        else {
                           return Ok(None);
                        };

                        if !h.to_str().infer_err()?.contains("image") {
                            return Ok(None);
                        }

                        dbg!("fetching image!!!!!");
                        let bytes = resp.bytes().await.infer_err()?;
                        dbg!("got image!!!!");

                        // TODO:? sync or async file io ?
                        let mut file = BufWriter::new(File::create(&p).infer_err()?);
                        file.write(&bytes).infer_err().look(|e| dbg!(e))?;
                        let img_dimensions = Reader::new(Cursor::new(&bytes))
                            .with_guessed_format()
                            .expect("Cursor io never fails")
                            .into_dimensions() // NOTE: return error here, as CONTENT_TYPE was supposed to be an image
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

        pub fn create_thumbnail(
            &self,
            size: ThumbnailSize,
            thumbnail_dir: impl AsRef<Path>,
        ) -> Result<(), Error> {
            let path = thumbnail_dir.as_ref().join(&self.uuid);
            let original_img = path.join(ThumbnailSize::Original.as_ref());
            let thumbnail = path.join(size.as_ref());

            dbg!("creating new thumbnail!!!");
            let s = size
                .value()
                .bad_err("cannot create a thumbnail for size Thumnail::Original")?;

            let reader = Reader::open(original_img)
                .infer_err()?
                .with_guessed_format()
                .infer_err()?;
            let img = reader.decode().infer_err()?;
            img.thumbnail(s, u32::MAX)
                .save_with_format(thumbnail, image::ImageFormat::Jpeg)
                .infer_err()?;
            Ok(())
        }

        pub fn delete(self, thumbnail_dir: impl AsRef<Path>) -> Result<(), Error> {
            let path = thumbnail_dir.as_ref().join(self.uuid);
            std::fs::remove_dir_all(path).infer_err()?;
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn get_ddp_info(
    data: DragDropPaste<ByteArrayFile>,
    client: State<'_, Client>,
) -> Result<DdpInfo<ByteArrayFile>, Error> {
    let potential_links: HashSet<&str> = data
        .file_uris
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect())
        .or_else(|| data.text.as_ref().map(|t| t.lines().collect()))
        .or_else(|| data.text_html.as_ref().map(|h| todo!())) // parse all potential urls if the text does not exist
        // .or(data.uri_list.as_ref().map(|u| todo!())) // donno if including this does any good
        .unwrap_or_default();

    let mut dirs = Vec::new();
    let mut image_paths = Vec::new();
    let mut image_uris = Vec::new();

    let mut reqs = Vec::new();
    let mut file_tasks = Vec::new();

    for link in potential_links.iter().copied() {
        let p = PathBuf::from(link);
        if p.is_dir() {
            dirs.push(p);
        } else if p.is_file() {
            let req = tokio::task::spawn_blocking(move || {
                if tree_magic_mini::from_filepath(p.as_path())
                    .look(|d| {
                        dbg!(&d);
                    })
                    .map(|t| t.contains("image"))
                    .unwrap_or(false)
                {
                    Some(p)
                } else {
                    None
                }
            });
            file_tasks.push(req);
        } else {
            let link = link.to_owned();
            let client = client.inner();
            let req = async move {
                let resp = client.get(&link).send().await.infer_err()?;
                resp.headers()
                    .look(|e| dbg!(e))
                    .get(reqwest::header::CONTENT_TYPE)
                    .bad_err("no content type in response")
                    .look(|e| dbg!(e))?
                    .to_str()
                    .infer_err()?
                    .contains("image")
                    .then_some(())
                    .bad_err("response type is not as required")
                    .look(|e| dbg!(e))?; // bail out
                Ok::<_, Error>(link)
            };
            reqs.push(req);
        }
    }
    futures::future::join_all(reqs)
        .await
        .into_iter()
        .filter_map(|e| e.ok())
        .for_each(|l| {
            image_uris.push(l);
        });
    futures::future::join_all(file_tasks)
        .await
        .into_iter()
        .filter_map(|e| e.ok())
        .flatten()
        .for_each(|l| {
            image_paths.push(l);
        });

    Ok(DdpInfo {
        files: data.files.unwrap_or_default(),
        image_uris,
        image_paths,
        dirs,
    })
}

// adding entire dirs at once
// - add all images from the dir path + add a tag with name of that dir
// - recursively add images from the dir path + add a tag for each nested dir

#[tauri::command]
pub async fn get_image_paths_from_dirs(
    paths: Vec<PathBuf>,
    recursive: bool,
) -> Result<Vec<DirFiles>, Error> {
    let mut path_tasks = Vec::new();

    for path in paths {
        if !path.is_dir() {
            return None.bad_err(format!("path {:?} is not a dir", &path));
        }

        let mut file_tasks = Vec::new();
        for entry in walkdir::WalkDir::new(&path)
            .min_depth(1)
            .max_depth(if recursive { 1 << 10 } else { 1 })
            .follow_links(true)
        {
            let e = entry.infer_err()?;
            let p = e.into_path();
            if !p.is_file() {
                continue;
            }

            let stripped_path = p.strip_prefix(&path).unwrap().to_path_buf();
            let req = tokio::task::spawn_blocking(move || {
                if tree_magic_mini::from_filepath(&p)
                    .look(|d| {
                        dbg!(&d);
                    })
                    .map(|t| t.contains("image"))
                    .unwrap_or(false)
                {
                    Some(stripped_path)
                } else {
                    None
                }
            });
            file_tasks.push(req);
        }

        let task = async move {
            let paths = futures::future::join_all(file_tasks)
                .await
                .into_iter()
                .filter_map(|e| e.ok())
                .flatten()
                .collect();

            Ok::<_, Error>(DirFiles {
                dir_name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                dir: Path {
                    base: BasePath::AbsolutePath,
                    path,
                },
                files: paths,
            })
        };
        path_tasks.push(task);
    }

    let res = futures::future::join_all(path_tasks)
        .await
        .into_iter()
        .collect::<Result<_, Error>>()?;
    Ok(res)
}

#[tauri::command]
pub async fn save_images_from_paths(
    paths: Vec<PathBuf>,
    config: State<'_, AppConfig>,
) -> Result<Vec<Image>, Error> {
    let images_path = Path {
        path: PathBuf::from("images"),
        base: BasePath::AppDataDir,
    };
    let images_dir = get_path(&images_path, config.inner());

    if !images_dir.exists() {
        std::fs::create_dir(&images_dir).infer_err()?;
    }

    let mut images = Vec::new();
    for path in paths {
        if !path_is_in_dir(&path, &images_dir).unwrap_or(false) {
            let file = path.as_path().save_in_dir(&images_path, config.inner())?;

            // TODO: if file is already in database, then remove the file that was just saved

            let mdata = file_mdata(get_path(&file.dest, config.inner()))?;
            let img = Image {
                src: file.src,
                title: file.title,
                path: file.dest,
                chksum: mdata.chksum.into(),
                size: mdata.size as _,
            };

            images.push(img);
        }
    }

    Ok(images)
}

#[tauri::command]
pub async fn save_images_from_uris(
    links: Vec<String>,
    config: State<'_, AppConfig>,
    client: State<'_, Client>,
) -> Result<Vec<Image>, Error> {
    let images_path = Path {
        path: PathBuf::from("images"),
        base: BasePath::AppDataDir,
    };
    let images_dir = get_path(&images_path, config.inner());

    if !images_dir.exists() {
        std::fs::create_dir(&images_dir).infer_err()?;
    }

    futures::future::join_all(links.iter().map(|l| {
        FilableUri {
            title: None,
            src: l.as_ref(),
            client: client.inner(),
            content_type_contains: "image",
        }
        .save_in_dir(&images_path, config.inner())
    }))
    .await
    .into_iter()
    .map(|file| {
        let file = file?;
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
    .collect()
}

#[tauri::command]
pub async fn save_images_from_bytes(
    files: Vec<ByteArrayFile>,
    config: State<'_, AppConfig>,
    client: State<'_, Client>,
) -> Result<Vec<Image>, Error> {
    let images_path = Path {
        path: PathBuf::from("images"),
        base: BasePath::AppDataDir,
    };
    let images_dir = get_path(&images_path, config.inner());

    if !images_dir.exists() {
        std::fs::create_dir(&images_dir).infer_err()?;
    }

    files
        .into_iter()
        .map(|f| f.save_in_dir(&images_path, config.inner()))
        .into_iter()
        .map(|file| {
            let file = file?;
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
        .collect()
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
