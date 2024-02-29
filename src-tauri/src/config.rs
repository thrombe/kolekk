use std::path::PathBuf;

use tauri::PathResolver;

use crate::bad_error::{Error, InferBadError};

#[derive(Debug)]
pub struct AppConfig {
    pub app_data_dir: PathBuf,
    pub app_config_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_log_dir: PathBuf,
    pub home_dir: PathBuf,
}

impl AppConfig {
    pub fn new(path_res: &PathResolver) -> Self {
        AppConfig {
            app_data_dir: path_res.app_data_dir().unwrap(),
            app_config_dir: path_res.app_config_dir().unwrap(),
            app_cache_dir: path_res.app_cache_dir().unwrap(),
            app_log_dir: path_res.app_log_dir().unwrap(),
            home_dir: tauri::api::path::home_dir().unwrap(),
        }
    }

    pub fn create_dirs(&self) -> Result<(), Error> {
        for dir in [
            &self.app_data_dir,
            &self.app_config_dir,
            &self.app_cache_dir,
            &self.app_log_dir,
        ] {
            if !dir.exists() {
                std::fs::create_dir_all(dir).infer_err()?;
            }
        }
        Ok(())
    }
}
