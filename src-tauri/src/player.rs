// use ts_rs::TS;
use serde::Serialize;
use std::fs;

// #[derive(Serialize, TS)]
// #[ts(export, export_to = "../bindings/FOrDir.ts")]
#[derive(Serialize, Debug, Clone)]
pub enum FilderKind {
    File,
    Folder,
}

#[derive(Serialize, Debug, Clone)]
pub struct Filder {
    name: String,
    files: Option<Vec<Filder>>,
    kind: FilderKind,
}

pub struct Player(pub std::sync::Mutex<musiplayer::Player>);

#[tauri::command]
pub fn play_song(path: &str, player: tauri::State<'_, Player>) {
    let mut player = player.0.lock().unwrap();
    dbg!(path);
    player.play(path.into()).unwrap();
}

#[tauri::command]
pub fn seek_perc(t: f64, player: tauri::State<'_, Player>) {
    let mut player = player.0.lock().unwrap();
    let d = player.duration().unwrap();
    let abs_time = t * d;
    let pos = player.position().unwrap();
    player.seek(abs_time - pos).unwrap();
}

#[tauri::command]
pub fn get_song_progress(player: tauri::State<'_, Player>) -> Option<f64> {
    player.0.lock().unwrap().progress().ok()
}

#[tauri::command]
pub fn stop_song(player: tauri::State<'_, Player>) {
    player.0.lock().unwrap().stop().unwrap();
}

#[tauri::command]
pub fn set_stat(pause: bool, player: tauri::State<'_, Player>) {
    let mut player = player.0.lock().unwrap();
    if (player.is_paused().unwrap() && !pause) || (!player.is_paused().unwrap() && pause) {
        player.toggle_pause().unwrap();
    }
}

#[tauri::command]
pub fn get_folder(path: &str) -> Option<Filder> {
    let files = fs::read_dir(path)
        .ok()?
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
