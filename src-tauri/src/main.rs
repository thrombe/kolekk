#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod bad_error;
mod bookmarks;
mod config;
mod database;
mod filesystem;
mod logg;
mod mal;
mod orm;
mod player;
mod search;

use bad_error::Error;
use logg::init_logger;
use mal::{mal_init, MalClient};
use player::Player;
use tauri::Manager;

pub use logg::{debug, error};

use crate::{api::tmdb::TmdbClient, bad_error::InferBadError, database::AppDatabase};

#[derive(PartialEq, Eq)]
pub enum AppInitialisationStatus {
    Uninitialised,
    Initialised,
}

fn main() {
    tauri::Builder::default()
        .manage(std::sync::Mutex::new(
            AppInitialisationStatus::Uninitialised,
        ))
        .invoke_handler(tauri::generate_handler![
            initialise_app,
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
            orm::add_tag_to_image,
            orm::remove_tag_from_image,
            filesystem::save_images_in_appdir,
            bookmarks::save_bookmark,
            search::search_images,
            search::search_bookmarks,
            api::commands::search_tmdb_multi,
            api::commands::tmdb_get_external_ids,
        ])
        .setup(|app| {
            app.handle().manage(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn initialise_app(
    status: tauri::State<'_, std::sync::Mutex<AppInitialisationStatus>>,
    app: tauri::State<'_, tauri::AppHandle>,
) -> Result<(), Error> {
    if *status.inner().lock().unwrap() == AppInitialisationStatus::Initialised {
        return Ok(());
    }
    *status.inner().lock().unwrap() = AppInitialisationStatus::Initialised;

    println!("trying to initialise app!");

    setup(app.inner()).await?;
    Ok(())
}

async fn setup(app_handle: &tauri::AppHandle) -> Result<(), Error> {
    let path_res = app_handle.path_resolver();
    let conf = config::AppConfig::new(&path_res);
    conf.create_dirs()?;
    println!("{:?}", &conf);
    init_logger(&conf.app_log_dir).unwrap();

    let client = std::sync::Arc::new(reqwest::Client::new());

    app_handle
        .manage(TmdbClient::new(include_str!("../../cache/tmdb_v3_auth"), client.clone()).await?);
    app_handle.manage(client.clone());
    app_handle.manage(AppDatabase::new(&conf).await?);
    app_handle.manage(Player(std::sync::Mutex::new(
        musiplayer::Player::new().unwrap(),
    )));
    app_handle.manage(conf);
    Ok(())
}
