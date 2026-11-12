use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use crate::contracts::{LyricsSource, LyricsState, LyricsStatus};

use super::{
    parse_lrc, result_from_candidate, select_candidate, LyricsCache, LyricsError, LyricsFetcher,
    LyricsResult, TrackQuery,
};

pub struct LyricsService {
    fetchers: Vec<Arc<dyn LyricsFetcher>>,
    cache: LyricsCache,
    generation: AtomicU64,
    fetcher_timeout: Duration,
}

impl LyricsService {
    pub fn new(fetchers: Vec<Arc<dyn LyricsFetcher>>, cache: LyricsCache) -> Self {
        Self::with_timeout(fetchers, cache, Duration::from_secs(4))
    }

    pub fn with_timeout(
        fetchers: Vec<Arc<dyn LyricsFetcher>>,
        cache: LyricsCache,
        fetcher_timeout: Duration,
    ) -> Self {
        Self {
            fetchers,
            cache,
            generation: AtomicU64::new(0),
            fetcher_timeout,
        }
    }

    pub fn begin_request(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn is_current(&self, token: u64) -> bool {
        self.generation.load(Ordering::Acquire) == token
    }

    pub async fn load(&self, track: &TrackQuery, manual_lrc: Option<&str>) -> LyricsState {
        let token = self.begin_request();

        if let Some(lrc) = manual_lrc {
            return match parse_lrc(lrc) {
                Ok(lines) => {
                    let result = LyricsResult {
                        source: LyricsSource::Manual,
                        lines,
                        duration_ms: 0,
                    };
                    if let Err(error) = self.cache.save(track, &result) {
                        return error_state(error);
                    }
                    ready_state(result)
                }
                Err(error) => LyricsState {
                    status: LyricsStatus::Error,
                    source: LyricsSource::Manual,
                    lines: Vec::new(),
                    duration_ms: 0,
                    error: Some(error.to_string()),
                },
            };
        }

        if let Some(cached) = self.cache.load(track) {
            return ready_state(cached);
        }

        let mut last_error = None;
        let mut candidate = None;
        for fetcher in &self.fetchers {
            let lookup = tokio::time::timeout(self.fetcher_timeout, fetcher.lookup(track)).await;
            match lookup {
                Ok(Ok(candidates)) => {
                    if let Some(found) = select_candidate(track, &candidates) {
                        candidate = Some(found);
                        break;
                    }
                }
                Ok(Err(error)) => last_error = Some(error),
                Err(_) => {
                    last_error = Some(LyricsError::Network("歌词源请求超时".to_owned()));
                }
            }
            if !self.is_current(token) {
                return LyricsState {
                    status: LyricsStatus::Idle,
                    source: LyricsSource::None,
                    lines: Vec::new(),
                    duration_ms: 0,
                    error: None,
                };
            }
        }
        if let Some(error) = last_error.filter(|_| candidate.is_none()) {
            return error_state(error);
        }
        let Some(candidate) = candidate else {
            return LyricsState {
                status: LyricsStatus::NotFound,
                source: LyricsSource::None,
                lines: Vec::new(),
                duration_ms: 0,
                error: None,
            };
        };
        let result = match result_from_candidate(candidate) {
            Ok(result) => result,
            Err(error) => return error_state(error),
        };
        if let Err(error) = self.cache.save(track, &result) {
            return error_state(error);
        }
        ready_state(result)
    }
}

fn ready_state(result: LyricsResult) -> LyricsState {
    LyricsState {
        status: LyricsStatus::Ready,
        source: result.source,
        duration_ms: result.duration_ms,
        lines: result.lines,
        error: None,
    }
}

fn error_state(error: LyricsError) -> LyricsState {
    LyricsState {
        status: LyricsStatus::Error,
        source: LyricsSource::None,
        lines: Vec::new(),
        duration_ms: 0,
        error: Some(error.to_string()),
    }
}
