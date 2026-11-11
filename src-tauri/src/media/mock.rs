use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use crate::contracts::{MediaSessionSummary, PlaybackState};

use super::{MediaProviderError, MediaSessionProvider};

#[derive(Clone, Default)]
pub struct MockMediaProvider {
    sessions: Arc<RwLock<BTreeMap<String, MediaSessionSummary>>>,
    current_session: Arc<RwLock<Option<String>>>,
}

impl MockMediaProvider {
    pub fn from_sessions(sessions: Vec<MediaSessionSummary>) -> Self {
        let current_session = sessions
            .iter()
            .find(|session| session.playback_status == crate::contracts::PlaybackStatus::Playing)
            .or_else(|| sessions.first())
            .map(|session| session.id.clone());
        let sessions = sessions
            .into_iter()
            .map(|session| (session.id.clone(), session))
            .collect();
        Self {
            sessions: Arc::new(RwLock::new(sessions)),
            current_session: Arc::new(RwLock::new(current_session)),
        }
    }

    pub fn set_current_session(&self, session_id: Option<&str>) {
        *self
            .current_session
            .write()
            .expect("mock current session lock poisoned") = session_id.map(str::to_owned);
    }

    pub fn remove_session(&self, session_id: &str) {
        self.sessions
            .write()
            .expect("mock session lock poisoned")
            .remove(session_id);
    }

    fn session(
        &self,
        session_id: &str,
    ) -> Result<MediaSessionSummary, MediaProviderError> {
        self.sessions
            .read()
            .expect("mock session lock poisoned")
            .get(session_id)
            .cloned()
            .ok_or_else(|| MediaProviderError::SessionNotFound(session_id.to_owned()))
    }
}

#[async_trait]
impl MediaSessionProvider for MockMediaProvider {
    async fn current_session_id(
        &self,
    ) -> Result<Option<String>, MediaProviderError> {
        Ok(self
            .current_session
            .read()
            .expect("mock current session lock poisoned")
            .clone())
    }

    async fn sessions(&self) -> Result<Vec<MediaSessionSummary>, MediaProviderError> {
        Ok(self
            .sessions
            .read()
            .expect("mock session lock poisoned")
            .values()
            .cloned()
            .collect())
    }

    async fn playback_state(
        &self,
        session_id: &str,
    ) -> Result<PlaybackState, MediaProviderError> {
        let session = self.session(session_id)?;
        Ok(PlaybackState {
            session_id: session.id,
            playback_status: session.playback_status,
            position_ms: 0,
            duration_ms: 0,
        })
    }

    async fn toggle_play_pause(
        &self,
        session_id: &str,
    ) -> Result<bool, MediaProviderError> {
        Ok(self.session(session_id)?.capabilities.play_pause)
    }

    async fn next_track(&self, session_id: &str) -> Result<bool, MediaProviderError> {
        Ok(self.session(session_id)?.capabilities.next)
    }

    async fn previous_track(
        &self,
        session_id: &str,
    ) -> Result<bool, MediaProviderError> {
        Ok(self.session(session_id)?.capabilities.previous)
    }

    async fn seek(&self, session_id: &str, _position_ms: i64) -> Result<bool, MediaProviderError> {
        Ok(self.session(session_id)?.capabilities.seek)
    }
}

