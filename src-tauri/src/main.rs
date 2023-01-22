#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod mal;
mod player;

use mal::{mal_init, MalClient};
use player::{get_folder, get_song_progress, play_song, seek_perc, set_stat, stop_song, Player};

use crate::mal::{get_seasonal_anime, mal_auth, mal_auth_needed};

fn main() {
    let client = tauri::async_runtime::block_on(mal_init()).unwrap();

    tauri::Builder::default()
        .manage(Player(std::sync::Mutex::new(
            musiplayer::Player::new().unwrap(),
        )))
        .manage(MalClient(std::sync::Arc::new(client)))
        .invoke_handler(tauri::generate_handler![
            get_folder,
            play_song,
            get_song_progress,
            seek_perc,
            set_stat,
            stop_song,
            get_seasonal_anime,
            mal_auth_needed,
            mal_auth,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
