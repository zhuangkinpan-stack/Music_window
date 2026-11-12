use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCapabilities {
    pub play_pause: bool,
    pub next: bool,
    pub previous: bool,
    pub seek: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackStatus {
    #[default]
    Closed,
    Opened,
    Changing,
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSessionSummary {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub artwork_url: Option<String>,
    pub playback_status: PlaybackStatus,
    pub capabilities: SessionCapabilities,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackState {
    pub session_id: String,
    pub playback_status: PlaybackStatus,
    pub position_ms: i64,
    pub duration_ms: i64,
    #[serde(default)]
    pub timeline_estimated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDiagnostic {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSnapshot {
    pub current_session_id: Option<String>,
    pub selected_session_id: Option<String>,
    pub sessions: Vec<MediaSessionSummary>,
    pub playback: Option<PlaybackState>,
    pub diagnostic: Option<MediaDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_ms: i64,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LyricsSource {
    Automatic,
    Manual,
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LyricsStatus {
    #[default]
    Idle,
    Loading,
    Ready,
    NotFound,
    Error,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsState {
    pub status: LyricsStatus,
    pub source: LyricsSource,
    pub lines: Vec<LyricLine>,
    #[serde(default)]
    pub duration_ms: i64,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
    pub monitor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub quick_bar_width_cm: f64,
    pub quick_bar_position: Option<WindowPosition>,
    pub show_quick_bar_on_launch: bool,
    pub start_with_windows: bool,
    pub reduce_transparency: bool,
    pub unlock_shortcut: String,
    pub toggle_quick_bar_shortcut: String,
    pub lyric_offset_ms: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_owned(),
            quick_bar_width_cm: 6.5,
            quick_bar_position: None,
            show_quick_bar_on_launch: true,
            start_with_windows: false,
            reduce_transparency: false,
            unlock_shortcut: "Ctrl+Alt+L".to_owned(),
            toggle_quick_bar_shortcut: "Ctrl+Alt+Q".to_owned(),
            lyric_offset_ms: 0,
        }
    }
}
