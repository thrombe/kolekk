#[allow(unused_imports)]
use crate::{dbg, debug, error};

use kolekk_types::{ByteArrayFile, DragDropPaste};
use std::{
    collections::HashSet,
    fmt::Debug,
    io::Write,
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
    bad_error::{Error, InferBadError, Inspectable},
    AppConfig,
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
            .look(|e| dbg!(e))
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
            let _ = p
                .as_path()
                .save_in_dir(&data_dir)
                .look(|e| dbg!(e))
                .ok()
                .map(|pb| image_paths.push(pb));
        } else {
            reqs.push(get_resp(u, &client));
        };
    }
    futures::future::join_all(reqs)
        .await
        .into_iter()
        .filter_map(|e| e.look(|e| dbg!(e)).ok())
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
        let mut path = dir.to_path_buf();
        path.push(buf);
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
            let mut path = dir.to_path_buf();
            path.push(buf);
            let _num_bytes_copied = std::fs::copy(self, &path).look(|e| dbg!(e)).infer_err()?;
            Ok(path)
        } else {
            Err(Error::new("the path is not a file"))
        }
    }
}
