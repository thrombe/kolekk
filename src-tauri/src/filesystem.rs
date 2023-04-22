#[allow(unused_imports)]
use crate::{dbg, debug, error};

use kolekk_types::utility::{BasePath, ByteArrayFile, FileMetadata, Path, Source};
use reqwest::Client;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use crate::{
    bad_error::{BadError, Error, InferBadError, Inspectable},
    config::AppConfig,
};

pub trait Filable {
    fn save_in_dir(&self, dir: &Path, config: &AppConfig) -> Result<FiledResult, Error>;
}

trait AutoFilable
where
    Self: AsRef<[u8]>,
{
}
impl AutoFilable for Vec<u8> {}
struct FilableBytes<'a>(&'a [u8]);
impl AsRef<[u8]> for FilableBytes<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}
impl AutoFilable for FilableBytes<'_> {}
// impl AutoFilable for reqwest::Bytes {} // :(

#[derive(Clone, Debug)]
pub struct FiledResult {
    pub title: Option<String>,
    pub src: Option<Source>,
    pub dest: Path,
}

impl Filable for ByteArrayFile {
    fn save_in_dir(&self, dir: &Path, config: &AppConfig) -> Result<FiledResult, Error> {
        self.data.save_in_dir(dir, config).map(|r| FiledResult {
            title: Some(self.name.to_owned()),
            ..r
        })
    }
}

impl<T> Filable for T
where
    T: AutoFilable,
{
    fn save_in_dir(&self, dir: &Path, config: &AppConfig) -> Result<FiledResult, Error> {
        let id = uuid::Uuid::new_v4();
        let buf = id.hyphenated().to_string();
        let path = dir.join(buf);

        let mut file =
            std::io::BufWriter::new(std::fs::File::create(get_path(&path, config)).infer_err()?);
        file.write(self.as_ref()).infer_err()?;
        Ok(FiledResult {
            src: None,
            title: None,
            dest: path,
        })
    }
}

impl Filable for &std::path::Path {
    fn save_in_dir(&self, dir: &Path, config: &AppConfig) -> Result<FiledResult, Error> {
        if self.is_file() {
            let id = uuid::Uuid::new_v4();
            let buf = id.hyphenated().to_string();
            let path = dir.join(buf);

            let _num_bytes_copied = std::fs::copy(self, get_path(&path, config)).infer_err()?;
            Ok(FiledResult {
                title: self.file_stem().map(|f| f.to_string_lossy().into_owned()),
                src: Some(Source::Path(Path {
                    base: BasePath::AbsolutePath,
                    path: self.to_path_buf(),
                })),
                dest: path,
            })
        } else {
            Err(Error::new("the path is not a file"))
        }
    }
}

pub struct FilableUri<'a, 'b, 'c> {
    pub title: Option<String>,
    pub src: &'a str,
    pub client: &'b Client,
    pub content_type_contains: &'c str,
}

impl FilableUri<'_, '_, '_> {
    pub async fn save_in_dir(self, dir: &Path, config: &AppConfig) -> Result<FiledResult, Error> {
        // TODO: ckeck how big the file is before downloading
        let resp = self.client.get(self.src).send().await.infer_err()?;
        resp.headers()
            .look(|e| dbg!(e))
            .get(reqwest::header::CONTENT_TYPE)
            .bad_err("no content type in response")
            .look(|e| dbg!(e))?
            .to_str()
            .infer_err()?
            .contains(self.content_type_contains)
            .then_some(())
            .bad_err("response type is not as required")
            .look(|e| dbg!(e))?; // bail out
        let bytes = resp.bytes().await.infer_err()?;

        let res = FilableBytes(bytes.as_ref()).save_in_dir(dir, config)?;

        Ok(FiledResult {
            title: self.title,
            src: Some(Source::Url(self.src.to_owned())),
            dest: res.dest,
        })
    }
}

pub fn get_path(path: &Path, config: &AppConfig) -> PathBuf {
    match path.base {
        BasePath::AppDataDir => config.app_data_dir.join(&path.path),
        BasePath::AppCacheDir => config.app_cache_dir.join(&path.path),
        BasePath::AbsolutePath => path.path.to_owned(),
    }
}

pub fn path_is_in_dir(path: impl Into<PathBuf>, dir: impl Into<PathBuf>) -> Result<bool, Error> {
    let path = path.into().canonicalize().infer_err()?;
    let parent = if path.is_file() {
        path.parent().bad_err("no parent")?.into()
    } else {
        path
    };
    let dir = dir.into().canonicalize().infer_err()?;
    Ok(parent.starts_with(dir))
}

pub fn file_mdata(path: impl AsRef<std::path::Path>) -> Result<FileMetadata, Error> {
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
    new_file: impl AsRef<std::path::Path>,
    data: &FileMetadata,
) -> Result<Option<FileMetadata>, Error> {
    let fd = file_mdata(new_file.as_ref())?;
    Ok(Some(fd).filter(|fd| *fd != *data))
}
