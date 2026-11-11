mod coordinator;
#[cfg(test)]
mod mock;
mod netease;
mod session_provider;
mod windows;

use thiserror::Error;

#[cfg(test)]
pub use mock::MockMediaProvider;
#[cfg(test)]
pub(crate) use netease::{
    is_cloudmusic_executable, netease_control_point, parse_netease_window_title,
    APPCOMMAND_MEDIA_NEXTTRACK, APPCOMMAND_MEDIA_PLAY_PAUSE, APPCOMMAND_MEDIA_PREVIOUSTRACK,
};
pub use session_provider::MediaSessionProvider;
#[allow(unused_imports)]
pub use windows::WindowsMediaProvider;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum MediaProviderError {
    #[error("Windows system media controls are unavailable: {0}")]
    SystemUnavailable(String),
    #[error("Media session not found: {0}")]
    SessionNotFound(String),
    #[error("Windows media error 0x{code:08X}: {message}")]
    Windows { code: i32, message: String },
}

impl From<::windows::core::Error> for MediaProviderError {
    fn from(error: ::windows::core::Error) -> Self {
        let code = error.code().0;
        if code == 0x80070424u32 as i32 {
            Self::SystemUnavailable(error.message().to_string())
        } else {
            Self::Windows {
                code,
                message: error.message().to_string(),
            }
        }
    }
}

pub(crate) fn friendly_app_name(source: &str) -> String {
    let lower = source.to_ascii_lowercase();
    let known = [
        ("cloudmusic", "网易云音乐"),
        ("spotify", "Spotify"),
        ("qqmusic", "QQ 音乐"),
        ("kugou", "酷狗音乐"),
        ("kuwo", "酷我音乐"),
        ("foobar", "foobar2000"),
        ("aimp", "AIMP"),
        ("apple", "Apple Music"),
    ];

    if let Some((_, name)) = known.iter().find(|(needle, _)| lower.contains(needle)) {
        return (*name).to_owned();
    }

    let leaf = source
        .rsplit(['!', '\\', '/'])
        .next()
        .unwrap_or(source)
        .trim_end_matches(".exe")
        .trim_end_matches(".EXE")
        .replace(['_', '-'], " ");

    if leaf.is_empty() {
        "未知音源".to_owned()
    } else {
        leaf
    }
}

pub(crate) fn ticks_to_millis(ticks: i64) -> i64 {
    ticks.max(0) / 10_000
}

pub(crate) fn image_extension(mime: &str) -> Option<&'static str> {
    match mime.to_ascii_lowercase().as_str() {
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        "image/bmp" => Some("bmp"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;

pub use coordinator::MediaCoordinator;
