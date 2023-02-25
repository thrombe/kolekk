#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod bad_error;
mod filesystem;
mod logg;
mod mal;
mod orm;
mod player;

use std::path::PathBuf;

use bad_error::Error;
use logg::init_logger;
use mal::{mal_init, MalClient};
use player::Player;
use tauri::Manager;

pub use logg::{debug, error};

use crate::bad_error::InferBadError;

#[derive(Debug)]
pub struct AppConfig {
    pub app_data_dir: PathBuf,
    pub app_config_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_log_dir: PathBuf,
    pub home_dir: PathBuf,
}

impl AppConfig {
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

fn main() {
    let client = tauri::async_runtime::block_on(mal_init()).unwrap();
    let db = tauri::async_runtime::block_on(orm::setup_sea_orm()).unwrap();

    tauri::Builder::default()
        .manage(Player(std::sync::Mutex::new(
            musiplayer::Player::new().unwrap(),
        )))
        .manage(MalClient(std::sync::Arc::new(client)))
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            player::get_folder,
            player::play_song,
            player::get_song_progress,
            player::seek_perc,
            player::set_stat,
            player::stop_song,
            mal::get_seasonal_anime,
            mal::mal_auth_needed,
            mal::mal_auth,
            orm::add_image_from_path,
            orm::get_images,
            orm::create_image_from_bytes,
            orm::add_tag_to_image,
            orm::remove_tag_from_image,
        ])
        .setup(|app| {
            let path_res = app.path_resolver();
            dbg!(path_res.resource_dir());
            let conf = AppConfig {
                app_data_dir: path_res.app_data_dir().unwrap(),
                app_config_dir: path_res.app_config_dir().unwrap(),
                app_cache_dir: path_res.app_cache_dir().unwrap(),
                app_log_dir: path_res.app_log_dir().unwrap(),
                home_dir: tauri::api::path::home_dir().unwrap(),
            };
            conf.create_dirs()?;
            init_logger(&conf.app_log_dir).unwrap();

            dbg!(&conf);
            app.handle().manage(conf);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
