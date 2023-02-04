#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod mal;
mod orm;
mod player;

use mal::{mal_init, MalClient};
use orm::setup_sea_orm;
use player::Player;

fn main() {
    let client = tauri::async_runtime::block_on(mal_init()).unwrap();
    let db = tauri::async_runtime::block_on(setup_sea_orm()).unwrap();

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
