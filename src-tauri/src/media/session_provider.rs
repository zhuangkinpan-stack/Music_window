use async_trait::async_trait;

use crate::contracts::{MediaSessionSummary, PlaybackState};

use super::MediaProviderError;

#[async_trait]
pub trait MediaSessionProvider: Send + Sync {
    async fn current_session_id(&self) -> Result<Option<String>, MediaProviderError>;
    async fn sessions(&self) -> Result<Vec<MediaSessionSummary>, MediaProviderError>;
    async fn playback_state(&self, session_id: &str) -> Result<PlaybackState, MediaProviderError>;
    async fn toggle_play_pause(&self, session_id: &str) -> Result<bool, MediaProviderError>;
    async fn next_track(&self, session_id: &str) -> Result<bool, MediaProviderError>;
    async fn previous_track(&self, session_id: &str) -> Result<bool, MediaProviderError>;
    async fn seek(&self, session_id: &str, position_ms: i64) -> Result<bool, MediaProviderError>;
}

