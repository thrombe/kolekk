#[allow(unused_imports)]
use crate::{dbg, debug, error};

use kolekk_types::{ByteArrayFile, DragDropPaste, FileMetadata, Image};
use std::{
    collections::HashSet,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tauri::{
    api::http::{Client, ClientBuilder, HttpRequestBuilder},
    http::Uri,
    State,
};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    config::AppConfig,
    database::AppDatabase,
};

pub async fn save_images<'a, F: Debug + Filable>(
    data: &DragDropPaste<F>,
    data_dir: impl Into<PathBuf>,
) -> Result<Vec<PathBuf>, Error> {
    let mut data_dir = data_dir.into();
    data_dir.push("images");

    if !data_dir.exists() {
        std::fs::create_dir(&data_dir)
            .look(|e| dbg!(e))
            .infer_err()?;
    }

    let client = ClientBuilder::new()
        .max_redirections(5)
        .connect_timeout(Duration::new(5, 0))
        .build()
        .look(|e| dbg!(e))
        .infer_err()?;

    let mut image_paths = data
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
    // let potential_links = data
    //     .file_uris
    //     .iter()
    //     .flatten()
    //     .map(String::as_str)
    //     .chain(
    //         data.text
    //             .iter()
    //             .map(|t| t.lines().collect::<Vec<_>>())
    //             .flatten(),
    //     )
    //     .chain(data.text_html.iter().map(String::as_str)) // parse all potential urls if the text does not exist
    //     // .chain(data.uri_list.iter()) // donno if including this does any good
    //     .collect::<HashSet<_>>();
    async fn get_resp(uri: &str, client: &Client) -> Result<Vec<u8>, Error> {
        let u = Uri::from_str(uri).look(|e| dbg!(e)).infer_err()?;
        // TODO: check content-type in headers before downloading the file
        // TODO: ckeck how big the file is before downloading
        let req = HttpRequestBuilder::new("GET", u.to_string())
            .look(|e| dbg!(e))
            .infer_err()?;
        let bytes = client
            .send(req)
            .await
            .look(|e| dbg!(e))
            .infer_err()?
            .bytes()
            .await
            // .look(|e| dbg!(e))
            .infer_err()?
            .data;
        Ok(bytes)
    }

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
                    .map(|pb| image_paths.push(pb));
            }
        } else {
            reqs.push(get_resp(u, &client));
        };
    }
    futures::future::join_all(reqs)
        .await
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|bytes| {
            ByteArrayFile {
                name: "".into(),
                data: bytes,
            }
            .save_in_dir(&data_dir)
            .look(|e| dbg!(e))
            .ok()
        })
        .for_each(|pb| image_paths.push(pb));
    Ok(image_paths)
}

pub trait Filable {
    fn save_in_dir(&self, dir: &Path) -> Result<PathBuf, Error>;
}

impl Filable for ByteArrayFile {
    fn save_in_dir(&self, dir: &Path) -> Result<PathBuf, Error> {
        let id = uuid::Uuid::new_v4();
        let buf = id.hyphenated().to_string();
        let path = dir.join(buf);
        let mut file =
            std::io::BufWriter::new(std::fs::File::create(&path).look(|e| dbg!(e)).infer_err()?);
        file.write(&self.data).look(|e| dbg!(e)).infer_err()?;
        Ok(path)
    }
}

impl Filable for &Path {
    fn save_in_dir(&self, dir: &Path) -> Result<PathBuf, Error> {
        if self.is_file() {
            let id = uuid::Uuid::new_v4();
            let buf = id.hyphenated().to_string();
            let path = dir.join(buf);
            let _num_bytes_copied = std::fs::copy(self, &path).look(|e| dbg!(e)).infer_err()?;
            Ok(path)
        } else {
            Err(Error::new("the path is not a file"))
        }
    }
}

pub fn path_is_in_dir(path: impl Into<PathBuf>, dir: impl Into<PathBuf>) -> Result<bool, Error> {
    let path = path.into().canonicalize().look(|e| dbg!(e)).infer_err()?;
    let parent = if path.is_file() {
        path.parent().bad_err("no parent")?.into()
    } else {
        path.clone()
    };
    let dir = dir.into().canonicalize().look(|e| dbg!(e)).infer_err()?;
    // debug!(
    //     "skipping file {} as it is already in data path",
    //     path.display()
    // );
    Ok(parent.starts_with(dir).look(|e| dbg!(e)))
}

pub fn file_mdata(path: impl AsRef<Path>) -> Result<FileMetadata, Error> {
    let mut ctx = md5::Context::new();
    let f = File::open(path.as_ref()).infer_err()?;
    let len = f.metadata().unwrap().len();
    let buf_len = len.min(1_000_000) as usize;
    let mut buf = BufReader::with_capacity(buf_len, f);
    loop {
        let part = buf.fill_buf().infer_err()?;
        if part.is_empty() {
            break;
        }
        ctx.consume(part);
        let part_len = part.len();
        buf.consume(part_len);
    }
    let digest = ctx.compute();
    Ok(FileMetadata {
        chksum: digest.0,
        size: len,
    })
}

// Some() if different, None if same
pub fn is_file_same(
    new_file: impl AsRef<Path>,
    data: &FileMetadata,
) -> Result<Option<FileMetadata>, Error> {
    let fd = file_mdata(new_file.as_ref())?;
    Ok(Some(fd).filter(|fd| *fd != *data))
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

    let futs = res
        .into_iter()
        .map(|db_path| -> Result<Image, Error> {
            let mdata = file_mdata(&db_path)?;
            let img = Image {
                id: 0,
                src_path: "".into(),
                title: "".into(),
                urls: vec![],
                tags: vec![],
                db_path: db_path.to_string_lossy().to_string(),
                chksum: mdata.chksum.into(),
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

#[tauri::command]
pub async fn search_images(
    db: State<'_, AppDatabase>,
    query: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<Image>, Error> {
    crate::database::search_images(db.inner(), query, limit, offset)
}
