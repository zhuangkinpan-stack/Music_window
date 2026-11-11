use std::sync::Arc;

use tauri::State;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::contracts::{AppSettings, WindowPosition};
use crate::shortcuts;
use crate::{
    contracts::{LyricsState, MediaSnapshot},
    lyrics::{LyricsService, TrackQuery},
    media::{MediaCoordinator, MediaProviderError},
    quick_bar::{schedule_auto_relock, set_lock_state, InteractionLock},
    settings::SettingsService,
};

pub struct AppState {
    pub media: Arc<MediaCoordinator>,
    pub lyrics: Arc<LyricsService>,
    pub interaction: Arc<InteractionLock>,
    pub settings: Arc<SettingsService>,
}

impl AppState {
    pub fn new(
        media: Arc<MediaCoordinator>,
        lyrics: Arc<LyricsService>,
        interaction: Arc<InteractionLock>,
        settings: Arc<SettingsService>,
    ) -> Self {
        Self {
            media,
            lyrics,
            interaction,
            settings,
        }
    }
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<MediaSnapshot, String> {
    Ok(state.media.snapshot().await)
}

#[tauri::command]
pub async fn select_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    state
        .media
        .select_session(&session_id)
        .await
        .map_err(error_message)
}

#[tauri::command]
pub async fn toggle_play_pause(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    state
        .media
        .toggle_play_pause(&session_id)
        .await
        .map_err(error_message)
}

#[tauri::command]
pub async fn next_track(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    state
        .media
        .next_track(&session_id)
        .await
        .map_err(error_message)
}

#[tauri::command]
pub async fn previous_track(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    state
        .media
        .previous_track(&session_id)
        .await
        .map_err(error_message)
}

#[tauri::command]
pub async fn seek(
    session_id: String,
    position_ms: i64,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    state
        .media
        .seek(&session_id, position_ms)
        .await
        .map_err(error_message)
}

#[tauri::command]
pub async fn load_lyrics(
    title: String,
    artist: String,
    album: String,
    duration_ms: i64,
    manual_lrc: Option<String>,
    state: State<'_, AppState>,
) -> Result<LyricsState, String> {
    Ok(state
        .lyrics
        .load(
            &TrackQuery {
                title,
                artist,
                album,
                duration_ms,
            },
            manual_lrc.as_deref(),
        )
        .await)
}

#[tauri::command]
pub fn quick_bar_lock_state(state: State<'_, AppState>) -> bool {
    state.interaction.is_locked()
}

#[tauri::command]
pub fn set_quick_bar_locked(
    locked: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    if locked {
        state.interaction.lock();
    } else {
        state.interaction.unlock(std::time::Instant::now());
    }
    set_lock_state(&app, locked)?;
    if !locked {
        schedule_auto_relock(app);
    }
    Ok(locked)
}

#[tauri::command]
pub fn touch_quick_bar(state: State<'_, AppState>) {
    state.interaction.touch(std::time::Instant::now());
}

#[tauri::command]
pub fn toggle_quick_bar(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("quick")
        .ok_or_else(|| "便捷条窗口不存在".to_owned())?;
    let visible = window.is_visible().map_err(|error| error.to_string())?;
    if visible {
        window.hide().map_err(|error| error.to_string())?;
    } else {
        window.show().map_err(|error| error.to_string())?;
        window
            .set_always_on_top(true)
            .map_err(|error| error.to_string())?;
    }
    Ok(!visible)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.get()
}

#[tauri::command]
pub fn update_settings(
    settings: AppSettings,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let previous = state.settings.get();
    crate::settings::validate_settings(&settings).map_err(|error| error.to_string())?;
    let shortcuts_changed = settings.unlock_shortcut != previous.unlock_shortcut
        || settings.toggle_quick_bar_shortcut != previous.toggle_quick_bar_shortcut;
    if shortcuts_changed {
        if let Err(error) = shortcuts::rebind(&app, &settings) {
            let _ = shortcuts::rebind(&app, &previous);
            return Err(error);
        }
    }
    if let Err(error) = state.settings.update(settings.clone()) {
        if shortcuts_changed {
            let _ = shortcuts::rebind(&app, &previous);
        }
        return Err(error.to_string());
    }
    if settings.start_with_windows {
        app.autolaunch()
            .enable()
            .map_err(|error| error.to_string())?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| error.to_string())?;
    }
    if let Some(window) = app.get_webview_window("quick") {
        crate::quick_bar::apply_geometry(
            &window,
            settings.quick_bar_width_cm,
            settings.quick_bar_position.as_ref(),
        )?;
    }
    Ok(settings)
}

#[tauri::command]
pub fn save_quick_bar_position(
    x: i32,
    y: i32,
    monitor: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.get();
    settings.quick_bar_position = Some(WindowPosition { x, y, monitor });
    state
        .settings
        .update(settings)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_quick_bar_visible(visible: bool, app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("quick") {
        if visible {
            window.show().map_err(|error| error.to_string())?;
        } else {
            window.hide().map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn error_message(error: MediaProviderError) -> String {
    error.to_string()
}
