mod cache;
mod lrclib;
mod parser;
mod service;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use cache::LyricsCache;
pub use lrclib::LrclibFetcher;
pub use parser::parse_lrc;
#[cfg(test)]
pub use parser::LyricsParseError;
pub use service::LyricsService;

use crate::contracts::{LyricLine, LyricsSource};

#[derive(Clone, Debug)]
pub struct TrackQuery {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsCandidate {
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub duration_ms: i64,
    pub synced_lyrics: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsResult {
    pub source: LyricsSource,
    pub lines: Vec<LyricLine>,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum LyricsError {
    #[error("歌词网络请求失败：{0}")]
    Network(String),
    #[error("歌词服务返回 HTTP {0}")]
    Http(u16),
    #[error("歌词解析失败：{0}")]
    Parse(String),
    #[error("歌词缓存读写失败：{0}")]
    Io(String),
}

#[async_trait]
pub trait LyricsFetcher: Send + Sync {
    async fn lookup(&self, track: &TrackQuery) -> Result<Vec<LyricsCandidate>, LyricsError>;
}

pub fn select_candidate(
    track: &TrackQuery,
    candidates: &[LyricsCandidate],
) -> Option<LyricsCandidate> {
    let title = normalize_text(&track.title);
    let artist = normalize_text(&track.artist);

    candidates
        .iter()
        .filter(|candidate| {
            !candidate
                .synced_lyrics
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        })
        .filter(|candidate| normalize_text(&candidate.track_name) == title)
        .filter(|candidate| artists_match(&artist, &normalize_text(&candidate.artist_name)))
        .filter(|candidate| {
            track.duration_ms <= 0
                || (candidate.duration_ms > 0
                    && (candidate.duration_ms - track.duration_ms).abs() <= 4_000)
        })
        .min_by_key(|candidate| {
            (
                (candidate.duration_ms - track.duration_ms).abs(),
                normalize_text(&candidate.album_name) != normalize_text(&track.album),
            )
        })
        .cloned()
}

fn artists_match(expected: &str, candidate: &str) -> bool {
    expected == candidate
        || (expected.len() >= 2
            && candidate.len() >= 2
            && (expected.contains(candidate) || candidate.contains(expected)))
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn result_from_candidate(candidate: LyricsCandidate) -> Result<LyricsResult, LyricsError> {
    let lrc = candidate
        .synced_lyrics
        .ok_or_else(|| LyricsError::Parse("同步歌词为空".to_owned()))?;
    Ok(LyricsResult {
        source: LyricsSource::Automatic,
        lines: parse_lrc(&lrc).map_err(|error| LyricsError::Parse(error.to_string()))?,
    })
}

#[cfg(test)]
mod tests;

