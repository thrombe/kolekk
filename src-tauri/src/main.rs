#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unused_variables, dead_code)]
// #![allow(unused_imports)]

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
mod tag;

#[cfg(feature = "music")]
mod player;

use bad_error::Error;
use logg::init_logger;
use tauri::Manager;

pub use logg::{debug, error};

use crate::{
    api::{lastfm::LastFmClient, tmdb::TmdbClient},
    database::AppDatabase,
};

#[derive(PartialEq, Eq)]
pub enum AppInitialisationStatus {
    Uninitialised,
    Initialised,
}

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(feature = "music")]
    let builder = builder.plugin(player::init());

    let app = builder
        .manage(std::sync::Mutex::new(
            AppInitialisationStatus::Uninitialised,
        ))
        .invoke_handler(tauri::generate_handler![
            initialise_app,
            bookmarks::get_tagged_bookmarks_from_text,
            bookmarks::bookmarks_from_html,
            bookmarks::get_bookmarks,
            bookmarks::search_bookmarks,
            images::get_images,
            images::get_ddp_info,
            images::get_image_paths_from_dirs,
            images::save_images_from_paths,
            images::save_images_from_uris,
            images::save_images_from_bytes,
            images::thumbnails::image_thumbnail,
            images::thumbnails::whatever_thumbnail,
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
            api::commands::lfm_search_track,
            api::commands::lfm_search_album,
            api::commands::lfm_search_artist,
            api::commands::lfm_get_track_info,
            api::commands::lfm_get_album_info,
            api::commands::lfm_get_artist_info,
            database::exact_search,
            database::exact_search_taggable,
            database::delete_from_id,
            database::enter_searchable,
            database::enter_searchable_item,
            database::search_jsml_object,
            database::add_tag_to_object,
            database::remove_tag_from_object,
            database::delete_facet_objects,
            database::new_temp_facet,
            database::get_path,
            database::reload_reader,
            clipboard::copy_image_to_clipboard,
            clipboard::copy_text,
            logg_string,
            logg_jsml,
        ])
        .setup(|app| {
            app.handle().manage(app.handle());
            // app.windows().look(|e| println!("{:?}", e)).into_keys().look(|e| println!("{:?}", e));
            // app.get_window("main")
            // .bad_err("no window found")?
            // .eval("import { invoke } from '@tauri-apps/api/tauri'; invoke('initialise_app')")
            // .eval("window.__TAURI__.invoke('initialise_app')")
            // .eval("window.location.replace('http://my-oauth-login-page.com')")
            // .eval("setInterval(() => {console.log('sasas')}, 100)")
            // .look(|e| println!("{:?}", e))
            // .infer_err()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, e| {});
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

#[tauri::command]
async fn logg_string(string: String) {
    debug!("{}", string);
}
#[tauri::command]
async fn logg_jsml(jsml: serde_json::Value) {
    dbg!(jsml);
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
    app_handle.manage(
        LastFmClient::new(include_str!("../../cache/lastfm_api_key"), client.clone())
            .test()
            .await,
    );
    app_handle.manage(client.clone());
    app_handle.manage(clipboard::Clipboard::new()?);

    database::init_database(app_handle, &conf).await?;
    let db = app_handle.state::<AppDatabase>().inner();

    images::thumbnails::init_thumbnailer(app_handle, &conf, db, client.clone()).await?;

    app_handle.manage(conf);
    Ok(())
}
