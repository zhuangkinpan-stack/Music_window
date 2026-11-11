mod commands;
mod contracts;
mod media;

use std::sync::Arc;

use commands::AppState;
use media::{MediaCoordinator, WindowsMediaProvider};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new(Arc::new(MediaCoordinator::new(Arc::new(
            WindowsMediaProvider::new(),
        )))))
        .invoke_handler(tauri::generate_handler![
            commands::list_sessions,
            commands::select_session,
            commands::toggle_play_pause,
            commands::next_track,
            commands::previous_track,
            commands::seek
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Lumi");
}

