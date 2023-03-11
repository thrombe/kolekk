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

    let client = ClientBuilder::new()
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

pub trait Filable {
    fn save_in_dir(&self, dir: &Path) -> Result<FiledResult, Error>;
}

#[derive(Clone, Debug)]
pub struct FiledResult {
    pub title: String,
    pub src: FileSource,
    pub dest_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum FileSource {
    Path(PathBuf),
    Uri(Uri),
    ByteArray,
}

impl Filable for ByteArrayFile {
    fn save_in_dir(&self, dir: &Path) -> Result<FiledResult, Error> {
        let id = uuid::Uuid::new_v4();
        let buf = id.hyphenated().to_string();
        let path = dir.join(buf);
        let mut file =
            std::io::BufWriter::new(std::fs::File::create(&path).look(|e| dbg!(e)).infer_err()?);
        file.write(&self.data).look(|e| dbg!(e)).infer_err()?;
        Ok(FiledResult {
            src: FileSource::ByteArray,
            title: self.name.clone(),
            dest_path: path,
        })
    }
}

impl Filable for &Path {
    fn save_in_dir(&self, dir: &Path) -> Result<FiledResult, Error> {
        if self.is_file() {
            let id = uuid::Uuid::new_v4();
            let buf = id.hyphenated().to_string();
            let path = dir.join(buf);
            let _num_bytes_copied = std::fs::copy(self, &path).look(|e| dbg!(e)).infer_err()?;
            Ok(FiledResult {
                title: self
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                src: FileSource::Path(self.to_path_buf()),
                dest_path: path,
            })
        } else {
            Err(Error::new("the path is not a file"))
        }
    }
}

pub struct FilableUri<'a> {
    pub title: String,
    pub src: Uri,
    pub client: &'a Client,
}

impl FilableUri<'_> {
    async fn save_in_dir(self, dir: &Path) -> Result<FiledResult, Error> {
        // TODO: check content-type in headers before downloading the file
        // TODO: ckeck how big the file is before downloading
        let req = HttpRequestBuilder::new("GET", self.src.to_string())
            .look(|e| dbg!(e))
            .infer_err()?;
        let bytes = self
            .client
            .send(req)
            .await
            .look(|e| dbg!(e))
            .infer_err()?
            .bytes()
            .await
            .infer_err()?
            .data;

        let res = ByteArrayFile {
            name: "".into(),
            data: bytes,
        }
        .save_in_dir(dir)?;

        Ok(FiledResult {
            title: self.title,
            src: FileSource::Uri(self.src),
            dest_path: res.dest_path,
        })
    }
}

pub fn path_is_in_dir(path: impl Into<PathBuf>, dir: impl Into<PathBuf>) -> Result<bool, Error> {
    let path = path.into().canonicalize().look(|e| dbg!(e)).infer_err()?;
    let parent = if path.is_file() {
        path.parent().bad_err("no parent")?.into()
    } else {
        path
    };
    let dir = dir.into().canonicalize().look(|e| dbg!(e)).infer_err()?;
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

