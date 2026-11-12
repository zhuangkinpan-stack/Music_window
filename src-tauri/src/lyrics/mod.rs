mod cache;
mod lrclib;
mod netease;
mod parser;
mod service;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use cache::LyricsCache;
pub use lrclib::LrclibFetcher;
pub use netease::NeteaseLyricsFetcher;
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
    #[serde(default)]
    pub duration_ms: i64,
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
        .filter_map(|candidate| {
            title_match_score(&title, &normalize_text(&candidate.track_name))
                .map(|score| (score, candidate))
        })
        .filter(|(_, candidate)| {
            artists_match(&artist, &normalize_text(&candidate.artist_name))
                && (track.duration_ms <= 0
                    || (candidate.duration_ms > 0
                        && (candidate.duration_ms - track.duration_ms).abs() <= 4_000))
        })
        .min_by_key(|(title_score, candidate)| {
            (
                *title_score,
                (candidate.duration_ms - track.duration_ms).abs(),
                normalize_text(&candidate.album_name) != normalize_text(&track.album),
            )
        })
        .map(|(_, candidate)| candidate.clone())
}

fn title_match_score(expected: &str, candidate: &str) -> Option<u8> {
    if expected == candidate {
        Some(0)
    } else if expected.len() >= 2
        && candidate.len() >= 2
        && (expected.contains(candidate) || candidate.contains(expected))
    {
        Some(1)
    } else if expected.len() >= 2 && candidate.len() >= 2 && levenshtein(expected, candidate) <= 2 {
        Some(2)
    } else {
        None
    }
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_char) in right_chars.iter().enumerate() {
            current.push(
                (previous[right_index + 1] + 1)
                    .min(current[right_index] + 1)
                    .min(previous[right_index] + usize::from(left_char != *right_char)),
            );
        }
        previous = current;
    }
    previous[right_chars.len()]
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

pub(crate) fn result_from_candidate(
    candidate: LyricsCandidate,
) -> Result<LyricsResult, LyricsError> {
    let lrc = candidate
        .synced_lyrics
        .ok_or_else(|| LyricsError::Parse("同步歌词为空".to_owned()))?;
    Ok(LyricsResult {
        source: LyricsSource::Automatic,
        duration_ms: candidate.duration_ms,
        lines: parse_lrc(&lrc).map_err(|error| LyricsError::Parse(error.to_string()))?,
    })
}

#[cfg(test)]
mod tests;
