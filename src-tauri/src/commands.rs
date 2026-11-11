use std::sync::Arc;

use tauri::State;

use crate::{
    contracts::MediaSnapshot,
    media::{MediaCoordinator, MediaProviderError},
};

pub struct AppState {
    pub media: Arc<MediaCoordinator>,
}

impl AppState {
    pub fn new(media: Arc<MediaCoordinator>) -> Self {
        Self { media }
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

fn error_message(error: MediaProviderError) -> String {
    error.to_string()
}

