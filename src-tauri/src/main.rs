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
mod clipboard;
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
            bookmarks::get_bookmarks,
            bookmarks::search_bookmarks,
            images::get_images,
            images::thumbnails::image_thumbnail,
            images::thumbnails::get_thumbnail_size,
            tag::search_tags,
            tag::save_new_tag,
            tag::get_tags_from_ids,
            api::commands::search_tmdb_multi,
            api::commands::tmdb_get_external_ids,
            api::commands::init_tachidesk_client,
            api::commands::tachidesk_get_all_extensions,
            api::commands::tachidesk_get_extension_icon_url,
            api::commands::tachidesk_get_manga_chapter_list,
            api::commands::tachidesk_get_manga_page_url,
            api::commands::tachidesk_get_manga_thumbnail_url,
            api::commands::tachidesk_get_manga,
            api::commands::tachidesk_get_source_list,
            api::commands::tachidesk_get_latest_manga_list,
            api::commands::tachidesk_get_popular_manga_list,
            api::commands::tachidesk_extension_action,
            api::commands::tachidesk_search_manga_in,
            api::commands::tachidesk_get_chapter,
            api::commands::tachidesk_get_source_filters,
            database::enter_searchable,
            database::search_jsml_object,
            database::add_tag_to_object,
            database::remove_tag_from_object,
            database::delete_facet_objects,
            database::new_temp_facet,
            database::get_path,
            clipboard::copy_image_to_clipboard,
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
    app_handle.manage(clipboard::Clipboard::new()?);

    database::init_database(app_handle, &conf).await?;
    let db = app_handle.state::<AppDatabase>().inner();

    images::thumbnails::init_thumbnailer(app_handle, &conf, db, client.clone()).await?;

    app_handle.manage(Player(std::sync::Mutex::new(
        musiplayer::Player::new().unwrap(),
    )));
    app_handle.manage(conf);
    Ok(())
}
