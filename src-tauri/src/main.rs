#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  tauri::Builder::default()
    .manage(Player(std::sync::Mutex::new(musiplayer::Player::new().unwrap())))
    .invoke_handler(tauri::generate_handler![
      get_folder,
      play_song,
      get_song_progress,
      seek_perc,
      set_stat,
      stop_song,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// use ts_rs::TS;
use serde::Serialize;

// #[derive(Serialize, TS)]
// #[ts(export, export_to = "../bindings/FOrDir.ts")]
#[derive(Serialize, Debug, Clone)]
pub enum FilderKind {
  File,
  Folder,
}

#[derive(Serialize, Debug, Clone)]
struct Filder {
  name: String,
  files: Option<Vec<Filder>>,
  kind: FilderKind,
}

use std::fs;
use anyhow::Result;

struct Player(std::sync::Mutex<musiplayer::Player>);

#[tauri::command]
fn play_song(path: &str, player: tauri::State<'_, Player>) {
  let mut player = player.0.lock().unwrap();
  dbg!(path);
  player.play(path.into()).unwrap();
}

#[tauri::command]
fn seek_perc(t: f64, player: tauri::State<'_, Player>) {
  let mut player = player.0.lock().unwrap();
  let d = player.duration().unwrap();
  let abs_time = t*d;
  let pos = player.position().unwrap();
  player.seek(abs_time - pos).unwrap();
}

#[tauri::command]
fn get_song_progress(player: tauri::State<'_, Player>) -> Option<f64> {
  player.0.lock().unwrap().progress().ok()
}

#[tauri::command]
fn stop_song(player: tauri::State<'_, Player>) {
  player.0.lock().unwrap().stop().unwrap();
}

#[tauri::command]
fn set_stat(pause: bool, player: tauri::State<'_, Player>) {
  let mut player = player.0.lock().unwrap();
  if player.is_paused().unwrap() && !pause {
    player.toggle_pause().unwrap();
  } else if !player.is_paused().unwrap() && pause {
    player.toggle_pause().unwrap();
  }
}

#[tauri::command]
fn get_folder(path: &str) -> Option<Filder> {
   let files = fs::read_dir(path).ok()?
    .filter_map(|e| {
        let e = e.ok()?;
        let ft = e.file_type().ok()?;
        let f = if ft.is_dir() || ft.is_symlink() {
          Filder {
            name: e.path().to_str().unwrap().into(),
            files: None,
            kind: FilderKind::Folder,
          }
        } else if ft.is_file() {
          Filder {
            name: e.path().to_str().unwrap().into(),
            files: None,
            kind: FilderKind::File,
          }
        } else {
          return None;
        };
        Some(f)
    })
    .collect();
   let res = Filder {
      name: path.into(),
      files: Some(files),
      kind: FilderKind::Folder,
   };
   dbg!(&res);
   Some(res)
}
