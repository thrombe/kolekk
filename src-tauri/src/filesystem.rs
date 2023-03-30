#[allow(unused_imports)]
use crate::{dbg, debug, error};

use kolekk_types::{ByteArrayFile, FileMetadata};
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use tauri::{
    api::http::{Client, HttpRequestBuilder},
    http::Uri,
};

use crate::bad_error::{BadError, Error, InferBadError, Inspectable};

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
    pub async fn save_in_dir(self, dir: &Path) -> Result<FiledResult, Error> {
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
