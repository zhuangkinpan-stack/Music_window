mod commands;
mod contracts;
mod lyrics;
mod media;
mod quick_bar;

use std::sync::Arc;

use commands::AppState;
use lyrics::{LrclibFetcher, LyricsCache, LyricsService};
use media::{MediaCoordinator, WindowsMediaProvider};
use quick_bar::InteractionLock;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
            let interaction = Arc::new(InteractionLock::new());
            app.manage(AppState::new(
                media,
                lyrics,
                interaction.clone(),
            ));

            if let Some(main_window) = app.get_webview_window("main") {
                let _ = window_vibrancy::apply_mica(&main_window, Some(true));
            }
            let quick_window = quick_bar::create(app.handle())?;
            quick_bar::configure(&quick_window)
                .map_err(std::io::Error::other)?;
            quick_bar::start_hover_monitor(app.handle().clone(), interaction);

            let _ = app.global_shortcut().on_shortcut(
                "ctrl+alt+l",
                |app, _, event| {
                    if event.state == ShortcutState::Pressed {
                        let state = app.state::<AppState>();
                        state
                            .interaction
                            .unlock(std::time::Instant::now());
                        let _ = quick_bar::set_lock_state(app, false);
                        quick_bar::schedule_auto_relock(app.clone());
                    }
                },
            );
            let _ = app.global_shortcut().on_shortcut(
                "ctrl+alt+m",
                |app, _, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("quick") {
                            let visible = window.is_visible().unwrap_or(true);
                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                            }
                        }
                    }
                },
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_sessions,
            commands::select_session,
            commands::toggle_play_pause,
            commands::next_track,
            commands::previous_track,
            commands::seek,
            commands::load_lyrics,
            commands::quick_bar_lock_state,
            commands::set_quick_bar_locked,
            commands::touch_quick_bar,
            commands::toggle_quick_bar
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Lumi");
}

