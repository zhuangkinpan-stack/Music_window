use std::sync::Arc;

use tauri::State;
use tauri::{AppHandle, Manager};

use crate::{
    contracts::{LyricsState, MediaSnapshot},
    lyrics::{LyricsService, TrackQuery},
    media::{MediaCoordinator, MediaProviderError},
    quick_bar::{
        schedule_auto_relock, set_lock_state, InteractionLock,
    },
};

pub struct AppState {
    pub media: Arc<MediaCoordinator>,
    pub lyrics: Arc<LyricsService>,
    pub interaction: Arc<InteractionLock>,
}

impl AppState {
    pub fn new(
        media: Arc<MediaCoordinator>,
        lyrics: Arc<LyricsService>,
        interaction: Arc<InteractionLock>,
    ) -> Self {
        Self {
            media,
            lyrics,
            interaction,
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
        state
            .interaction
            .unlock(std::time::Instant::now());
    }
    set_lock_state(&app, locked)?;
    if !locked {
        schedule_auto_relock(app);
    }
    Ok(locked)
}

#[tauri::command]
pub fn touch_quick_bar(state: State<'_, AppState>) {
    state
        .interaction
        .touch(std::time::Instant::now());
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

fn error_message(error: MediaProviderError) -> String {
    error.to_string()
}

