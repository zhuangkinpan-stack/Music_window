mod commands;
mod contracts;
mod lyrics;
mod media;

use std::sync::Arc;

use commands::AppState;
use lyrics::{LrclibFetcher, LyricsCache, LyricsService};
use media::{MediaCoordinator, WindowsMediaProvider};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let media = Arc::new(MediaCoordinator::new(Arc::new(
                WindowsMediaProvider::new(),
            )));
            let cache_dir = app
                .path()
                .app_cache_dir()
                .map_err(|error| error.to_string())?
                .join("lyrics");
            let lyrics = Arc::new(LyricsService::new(
                Arc::new(LrclibFetcher::new().map_err(|error| error.to_string())?),
                LyricsCache::new(cache_dir),
            ));
            app.manage(AppState::new(media, lyrics));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_sessions,
            commands::select_session,
            commands::toggle_play_pause,
            commands::next_track,
            commands::previous_track,
            commands::seek,
            commands::load_lyrics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Lumi");
}

