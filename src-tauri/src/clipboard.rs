use std::{borrow::Cow, sync::Mutex};

use arboard::ImageData;
use image::io::Reader;
use kolekk_types::utility::Path;
use tauri::State;

use crate::{
    bad_error::{Error, InferBadError},
    config::AppConfig,
    filesystem::get_path,
};

#[tauri::command]
pub async fn copy_image_to_clipboard(
    clp: State<'_, Clipboard>,
    conf: State<'_, AppConfig>,
    img_path: Path,
) -> Result<(), Error> {
    clp.copy_image_to_clipboard(img_path, conf.inner())
}

#[tauri::command]
pub async fn copy_text(
    clp: State<'_, Clipboard>,
    text: String,
) -> Result<(), Error> {
    clp.copy_text(text)
}

pub struct Clipboard {
    inner: Mutex<arboard::Clipboard>,
}

impl Clipboard {
    pub fn new() -> Result<Self, Error> {
        let clp = Self {
            inner: Mutex::new(arboard::Clipboard::new().infer_err()?),
        };
        Ok(clp)
    }

    pub fn copy_text(&self, text: String) -> Result<(), Error> {
        self.inner.lock().infer_err()?.set_text(text).infer_err()?;
        Ok(())
    }

    pub fn copy_image_to_clipboard(&self, img_path: Path, conf: &AppConfig) -> Result<(), Error> {
        let path = get_path(&img_path, conf);
        let reader = Reader::open(path)
            .infer_err()?
            .with_guessed_format()
            .infer_err()?;
        let img = reader.decode().infer_err()?;
        let img = img.into_rgba8();
        let img = ImageData {
            width: img.width() as _,
            height: img.height() as _,
            bytes: Cow::from(img.into_raw()),
        };
        self.inner.lock().infer_err()?.set_image(img).infer_err()?;
        Ok(())
    }
}
