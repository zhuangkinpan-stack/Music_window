use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::contracts::{LyricsSource, LyricsState, LyricsStatus};

use super::{
    parse_lrc, result_from_candidate, select_candidate, LyricsCache, LyricsError,
    LyricsFetcher, LyricsResult, TrackQuery,
};

pub struct LyricsService {
    fetcher: Arc<dyn LyricsFetcher>,
    cache: LyricsCache,
    generation: AtomicU64,
}

impl LyricsService {
    pub fn new(fetcher: Arc<dyn LyricsFetcher>, cache: LyricsCache) -> Self {
        Self {
            fetcher,
            cache,
            generation: AtomicU64::new(0),
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
                    error: Some(error.to_string()),
                },
            };
        }

        if let Some(cached) = self.cache.load(track) {
            return ready_state(cached);
        }

        let candidates = match self.fetcher.lookup(track).await {
            Ok(candidates) => candidates,
            Err(error) => return error_state(error),
        };
        if !self.is_current(token) {
            return LyricsState {
                status: LyricsStatus::Idle,
                source: LyricsSource::None,
                lines: Vec::new(),
                error: None,
            };
        }

        let Some(candidate) = select_candidate(track, &candidates) else {
            return LyricsState {
                status: LyricsStatus::NotFound,
                source: LyricsSource::None,
                lines: Vec::new(),
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
        lines: result.lines,
        error: None,
    }
}

fn error_state(error: LyricsError) -> LyricsState {
    LyricsState {
        status: LyricsStatus::Error,
        source: LyricsSource::None,
        lines: Vec::new(),
        error: Some(error.to_string()),
    }
}

