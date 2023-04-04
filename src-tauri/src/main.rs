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
// mod orm;
mod images;
mod player;
mod tag;

use bad_error::Error;
use logg::init_logger;
use player::Player;
use tauri::Manager;

pub use logg::{debug, error};

use crate::{api::tmdb::TmdbClient, database::AppDatabase};

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
            // orm::add_image_from_path,
            // orm::get_images,
            // orm::add_tag_to_image,
            // orm::remove_tag_from_image,
            bookmarks::save_bookmarks_from_drop,
            bookmarks::save_bookmarks,
            bookmarks::get_bookmarks,
            bookmarks::add_tag_to_bookmark,
            bookmarks::remove_tag_from_bookmark,
            bookmarks::search_bookmarks,
            images::search_images,
            images::save_images_in_appdir,
            tag::search_tags,
            tag::save_tag,
            tag::save_alias_tag,
            tag::get_tags_from_ids,
            api::commands::search_tmdb_multi,
            api::commands::tmdb_get_external_ids,
            api::commands::init_tachidesk_client,
            api::commands::tachidesk_get_all_extensions,
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

    let client = reqwest::Client::new();

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
