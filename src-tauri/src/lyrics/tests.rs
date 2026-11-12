use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

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

#[test]
fn selects_containment_match_when_exact_title_has_decorations() {
    let track = TrackQuery {
        title: "晴天 (Live)".to_owned(),
        artist: "周杰伦".to_owned(),
        album: String::new(),
        duration_ms: 0,
    };
    let live = candidate("晴天", "周杰伦", 270_000);

    let selected = select_candidate(&track, &[live]).unwrap();

    assert_eq!(selected.track_name, "晴天");
}

#[test]
fn legacy_lyrics_cache_defaults_duration_to_zero() {
    let result: LyricsResult =
        serde_json::from_str(r#"{"source":"automatic","lines":[{"timeMs":1000,"text":"旧缓存"}]}"#)
            .unwrap();

    assert_eq!(result.duration_ms, 0);
}

#[test]
fn default_lyrics_timeout_allows_eight_seconds_per_source() {
    let directory = tempdir().unwrap();
    let service = LyricsService::new(Vec::new(), LyricsCache::new(directory.path().to_path_buf()));

    assert_eq!(service.fetcher_timeout_for_test(), Duration::from_secs(8));
}

#[tokio::test]
async fn lyrics_service_tries_fetchers_in_order() {
    let directory = tempdir().unwrap();
    let empty = Arc::new(CountingFetcher::new("empty", Vec::new()));
    let found = Arc::new(CountingFetcher::new(
        "found",
        vec![candidate("晴天", "周杰伦", 269_000)],
    ));
    let service = LyricsService::new(
        vec![empty.clone(), found.clone()],
        LyricsCache::new(directory.path().to_path_buf()),
    );

    let state = service.load(&track_query(), None).await;

    assert_eq!(state.status, LyricsStatus::Ready);
    assert_eq!(state.duration_ms, 269_000);
    assert_eq!(empty.calls(), 1);
    assert_eq!(found.calls(), 1);
}

#[tokio::test]
async fn lyrics_service_times_out_stalled_fetcher_before_fallback() {
    let directory = tempdir().unwrap();
    let slow = Arc::new(DelayedFetcher::new(Duration::from_millis(50)));
    let found = Arc::new(CountingFetcher::new(
        "found",
        vec![candidate("晴天", "周杰伦", 269_000)],
    ));
    let service = LyricsService::with_timeout(
        vec![slow, found],
        LyricsCache::new(directory.path().to_path_buf()),
        Duration::from_millis(10),
    );

    let state = service.load(&track_query(), None).await;

    assert_eq!(state.status, LyricsStatus::Ready);
}

#[test]
fn builds_anonymous_netease_search_and_lyric_urls() {
    let search = super::netease::netease_search_url("You Are All I Need", "FLY COAST/二宮愛");
    let query: BTreeMap<_, _> = search.query_pairs().into_owned().collect();
    assert_eq!(
        query.get("s").map(String::as_str),
        Some("You Are All I Need FLY COAST/二宮愛")
    );
    assert_eq!(query.get("type").map(String::as_str), Some("1"));

    let lyric = super::netease::netease_lyric_url("12345");
    assert_eq!(lyric.path(), "/api/song/lyric");
    assert_eq!(lyric.query(), Some("id=12345&lv=-1&kv=-1&tv=-1"));
}

#[test]
fn selects_best_netease_search_result() {
    let response: super::netease::NeteaseSearchResponse = serde_json::from_str(
        r#"{"result":{"songs":[
          {"id":1,"name":"晴天 (Live)","duration":271000,"artists":[{"name":"周杰伦"}],"album":{"name":"演唱会"}},
          {"id":2,"name":"晴天","duration":269000,"artists":[{"name":"周杰伦"}],"album":{"name":"叶惠美"}}
        ]}}"#,
    )
    .unwrap();

    let selected = super::netease::select_netease_song(&track_query(), &response).unwrap();

    assert_eq!(selected.id, 2);
    assert_eq!(selected.duration, 269_000);
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
                duration_ms: 269_000,
                lines: vec![LyricLine {
                    time_ms: 1_000,
                    text: "缓存歌词".to_owned(),
                }],
            },
        )
        .unwrap();
    let service = LyricsService::new(vec![Arc::new(FailingFetcher)], cache);

    let state = service.load(&track_query(), None).await;

    assert_eq!(state.status, LyricsStatus::Ready);
    assert_eq!(state.lines[0].text, "缓存歌词");
}

#[tokio::test]
async fn stale_request_does_not_publish_old_result() {
    let directory = tempdir().unwrap();
    let cache = LyricsCache::new(directory.path().to_path_buf());
    let fetcher = Arc::new(DelayedFetcher::new(Duration::from_millis(50)));
    let service = LyricsService::new(vec![fetcher], cache);
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

struct CountingFetcher {
    calls: AtomicUsize,
    candidates: Vec<LyricsCandidate>,
}

impl CountingFetcher {
    fn new(_name: &str, candidates: Vec<LyricsCandidate>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            candidates,
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl LyricsFetcher for CountingFetcher {
    async fn lookup(&self, _track: &TrackQuery) -> Result<Vec<LyricsCandidate>, LyricsError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.candidates.clone())
    }
}

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
