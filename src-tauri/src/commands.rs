use std::sync::Arc;

use tauri::State;
use tauri::{AppHandle, Emitter, Manager};

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
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    let snapshot = state
        .media
        .select_session(&session_id)
        .await
        .map_err(error_message)?;
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn toggle_play_pause(
    session_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    let snapshot = state
        .media
        .toggle_play_pause(&session_id)
        .await
        .map_err(error_message)?;
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn next_track(
    session_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    let snapshot = state
        .media
        .next_track(&session_id)
        .await
        .map_err(error_message)?;
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn previous_track(
    session_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    let snapshot = state
        .media
        .previous_track(&session_id)
        .await
        .map_err(error_message)?;
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn seek(
    session_id: String,
    position_ms: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MediaSnapshot, String> {
    let snapshot = state
        .media
        .seek(&session_id, position_ms)
        .await
        .map_err(error_message)?;
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

/// 播放控制由任一窗口触发时，立刻把结果推给所有窗口，
/// 避免另一个窗口只能等下一次轮询、出现按钮状态错位。
fn broadcast(app: &AppHandle, snapshot: &MediaSnapshot) {
    let _ = app.emit("media://snapshot", snapshot);
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
    let settings = crate::settings::merge_settings(previous.clone(), settings);
    crate::settings::validate_settings(&settings).map_err(|error| error.to_string())?;
    let shortcuts_changed = settings.unlock_shortcut != previous.unlock_shortcut
        || settings.toggle_quick_bar_shortcut != previous.toggle_quick_bar_shortcut;
    if shortcuts_changed {
        shortcuts::rebind(&app, &previous, &settings)?;
    }
    let autostart_changed = settings.start_with_windows != previous.start_with_windows;
    if autostart_changed {
        if let Err(error) = crate::autostart::set_enabled(settings.start_with_windows) {
            if shortcuts_changed {
                let _ = shortcuts::rebind(&app, &settings, &previous);
            }
            return Err(error);
        }
    }
    if let Err(error) = state.settings.update(settings.clone()) {
        if shortcuts_changed {
            let _ = shortcuts::rebind(&app, &settings, &previous);
        }
        if autostart_changed {
            let _ = crate::autostart::set_enabled(previous.start_with_windows);
        }
        return Err(error.to_string());
    }
    if let Some(window) = app.get_webview_window("quick") {
        crate::quick_bar::apply_geometry(
            &window,
            settings.quick_bar_width_cm,
            settings.quick_bar_position.as_ref(),
        )?;
    }
    let _ = app.emit("settings://changed", &settings);
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
