mod commands;
mod contracts;
mod lyrics;
mod media;
mod quick_bar;
mod settings;
mod shortcuts;
mod tray;

use std::sync::Arc;

use commands::AppState;
use lyrics::{LrclibFetcher, LyricsCache, LyricsService};
use media::{MediaCoordinator, WindowsMediaProvider};
use quick_bar::InteractionLock;
use settings::SettingsService;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("Lumi")
                .build(),
        )
        .setup(|app| {
            let media = Arc::new(MediaCoordinator::new(Arc::new(WindowsMediaProvider::new())));
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
            let settings_path = app
                .path()
                .app_config_dir()
                .map_err(|error| error.to_string())?
                .join("settings.json");
            let settings =
                Arc::new(SettingsService::load(settings_path).map_err(|error| error.to_string())?);
            let initial_settings = settings.get();
            app.manage(AppState::new(media, lyrics, interaction.clone(), settings));
            tray::build(app)?;

            if let Some(main_window) = app.get_webview_window("main") {
                let _ = window_vibrancy::apply_mica(&main_window, Some(true));
            }
            let quick_window = quick_bar::create(app.handle(), &initial_settings)?;
            quick_bar::configure(&quick_window, &initial_settings)
                .map_err(std::io::Error::other)?;
            quick_bar::start_hover_monitor(app.handle().clone(), interaction);

            let _ = shortcuts::rebind(app.handle(), &initial_settings);
            if initial_settings.start_with_windows {
                let _ = app.autolaunch().enable();
            } else {
                let _ = app.autolaunch().disable();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
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
            commands::toggle_quick_bar,
            commands::get_settings,
            commands::update_settings,
            commands::save_quick_bar_position,
            commands::set_quick_bar_visible
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Lumi");
}
