use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use tempfile::tempdir;

use super::*;
use crate::contracts::{LyricLine, LyricsStatus};

#[test]
fn parses_offset_crlf_and_multiple_timestamps() {
    let input = "[offset:500]\r\n[00:01.00][00:03.25]Hello\r\n[00:02.5]World";

    let lines = parse_lrc(input).unwrap();

    assert_eq!(
        lines,
        vec![
            LyricLine {
                time_ms: 1_500,
                text: "Hello".to_owned(),
            },
            LyricLine {
                time_ms: 3_000,
                text: "World".to_owned(),
            },
            LyricLine {
                time_ms: 3_750,
                text: "Hello".to_owned(),
            },
        ]
    );
}

#[test]
fn rejects_lrc_without_timed_lines() {
    let error = parse_lrc("[ar:Artist]\nnot a lyric timestamp").unwrap_err();

    assert!(matches!(error, LyricsParseError::NoTimedLines));
}

#[test]
fn rejects_candidates_with_wrong_artist_or_duration() {
    let track = track_query();
    let wrong_artist = candidate("晴天", "其他歌手", 269_000);
    let wrong_duration = candidate("晴天", "周杰伦", 255_000);

    assert!(select_candidate(&track, &[wrong_artist, wrong_duration]).is_none());
}

#[test]
fn selects_synced_candidate_with_closest_duration() {
    let track = track_query();
    let exact = candidate("晴天", "周杰伦", 268_000);
    let farther = candidate("晴天", "周杰伦", 271_000);

    let selected = select_candidate(&track, &[farther, exact]).unwrap();

    assert_eq!(selected.duration_ms, 268_000);
}

#[tokio::test]
async fn offline_load_uses_cached_automatic_lyrics() {
    let directory = tempdir().unwrap();
    let cache = LyricsCache::new(directory.path().to_path_buf());
    cache
        .save(
            &track_query(),
            &LyricsResult {
                source: crate::contracts::LyricsSource::Automatic,
                lines: vec![LyricLine {
                    time_ms: 1_000,
                    text: "缓存歌词".to_owned(),
                }],
            },
        )
        .unwrap();
    let service = LyricsService::new(
        Arc::new(FailingFetcher),
        cache,
    );

    let state = service.load(&track_query(), None).await;

    assert_eq!(state.status, LyricsStatus::Ready);
    assert_eq!(state.lines[0].text, "缓存歌词");
}

#[tokio::test]
async fn stale_request_does_not_publish_old_result() {
    let directory = tempdir().unwrap();
    let cache = LyricsCache::new(directory.path().to_path_buf());
    let fetcher = Arc::new(DelayedFetcher::new(Duration::from_millis(50)));
    let service = LyricsService::new(fetcher, cache);
    let first_token = service.begin_request();
    let _second_token = service.begin_request();

    assert!(!service.is_current(first_token));
}

fn track_query() -> TrackQuery {
    TrackQuery {
        title: "晴天".to_owned(),
        artist: "周杰伦".to_owned(),
        album: "叶惠美".to_owned(),
        duration_ms: 269_000,
    }
}

fn candidate(title: &str, artist: &str, duration_ms: i64) -> LyricsCandidate {
    LyricsCandidate {
        track_name: title.to_owned(),
        artist_name: artist.to_owned(),
        album_name: "叶惠美".to_owned(),
        duration_ms,
        synced_lyrics: Some("[00:01.00]故事的小黄花".to_owned()),
    }
}

struct FailingFetcher;

#[async_trait]
impl LyricsFetcher for FailingFetcher {
    async fn lookup(&self, _track: &TrackQuery) -> Result<Vec<LyricsCandidate>, LyricsError> {
        Err(LyricsError::Network("offline".to_owned()))
    }
}

struct DelayedFetcher {
    delay: Duration,
}

impl DelayedFetcher {
    fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

#[async_trait]
impl LyricsFetcher for DelayedFetcher {
    async fn lookup(&self, _track: &TrackQuery) -> Result<Vec<LyricsCandidate>, LyricsError> {
        tokio::time::sleep(self.delay).await;
        Ok(Vec::new())
    }
}

