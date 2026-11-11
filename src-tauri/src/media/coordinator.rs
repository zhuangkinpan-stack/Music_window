use std::sync::Arc;

use tokio::sync::RwLock;

use crate::contracts::{MediaDiagnostic, MediaSnapshot, PlaybackState};

use super::{MediaProviderError, MediaSessionProvider};

pub struct MediaCoordinator {
    provider: Arc<dyn MediaSessionProvider>,
    selected_session_id: RwLock<Option<String>>,
}

impl MediaCoordinator {
    pub fn new(provider: Arc<dyn MediaSessionProvider>) -> Self {
        Self {
            provider,
            selected_session_id: RwLock::new(None),
        }
    }

    pub async fn snapshot(&self) -> MediaSnapshot {
        let sessions = match self.provider.sessions().await {
            Ok(sessions) => sessions,
            Err(error) => {
                return MediaSnapshot {
                    current_session_id: None,
                    sessions: Vec::new(),
                    playback: None,
                    diagnostic: Some(error_to_diagnostic(&error)),
                };
            }
        };

        let current_session_id = self.provider.current_session_id().await.ok().flatten();
        let mut selected = self.selected_session_id.write().await;
        if selected
            .as_ref()
            .is_none_or(|id| !sessions.iter().any(|session| &session.id == id))
        {
            *selected = current_session_id
                .clone()
                .filter(|id| sessions.iter().any(|session| &session.id == id))
                .or_else(|| {
                    sessions
                        .iter()
                        .find(|session| {
                            session.playback_status == crate::contracts::PlaybackStatus::Playing
                        })
                        .or_else(|| sessions.first())
                        .map(|session| session.id.clone())
                });
        }

        let playback = match selected.as_deref() {
            Some(id) => self.playback_state(id).await,
            None => None,
        };

        MediaSnapshot {
            current_session_id,
            sessions,
            playback,
            diagnostic: None,
        }
    }

    pub async fn select_session(
        &self,
        session_id: &str,
    ) -> Result<MediaSnapshot, MediaProviderError> {
        let sessions = self.provider.sessions().await?;
        if !sessions.iter().any(|session| session.id == session_id) {
            return Err(MediaProviderError::SessionNotFound(session_id.to_owned()));
        }
        *self.selected_session_id.write().await = Some(session_id.to_owned());
        Ok(self.snapshot_for_sessions(sessions).await)
    }

    pub async fn toggle_play_pause(
        &self,
        session_id: &str,
    ) -> Result<MediaSnapshot, MediaProviderError> {
        self.provider.toggle_play_pause(session_id).await?;
        Ok(self.snapshot().await)
    }

    pub async fn next_track(&self, session_id: &str) -> Result<MediaSnapshot, MediaProviderError> {
        self.provider.next_track(session_id).await?;
        Ok(self.snapshot().await)
    }

    pub async fn previous_track(
        &self,
        session_id: &str,
    ) -> Result<MediaSnapshot, MediaProviderError> {
        self.provider.previous_track(session_id).await?;
        Ok(self.snapshot().await)
    }

    pub async fn seek(
        &self,
        session_id: &str,
        position_ms: i64,
    ) -> Result<MediaSnapshot, MediaProviderError> {
        self.provider.seek(session_id, position_ms).await?;
        Ok(self.snapshot().await)
    }

    async fn playback_state(&self, session_id: &str) -> Option<PlaybackState> {
        self.provider.playback_state(session_id).await.ok()
    }

    async fn snapshot_for_sessions(
        &self,
        sessions: Vec<crate::contracts::MediaSessionSummary>,
    ) -> MediaSnapshot {
        let selected = self.selected_session_id.read().await.clone();
        let playback = match selected.as_deref() {
            Some(id) => self.playback_state(id).await,
            None => None,
        };
        MediaSnapshot {
            current_session_id: self.provider.current_session_id().await.ok().flatten(),
            sessions,
            playback,
            diagnostic: None,
        }
    }
}

fn error_to_diagnostic(error: &MediaProviderError) -> MediaDiagnostic {
    match error {
        MediaProviderError::SystemUnavailable(message) => MediaDiagnostic {
            code: "smtc_unavailable".to_owned(),
            message: format!("Windows 系统媒体控制不可用：{message}"),
            recoverable: true,
        },
        MediaProviderError::SessionNotFound(id) => MediaDiagnostic {
            code: "session_not_found".to_owned(),
            message: format!("音乐会话已关闭：{id}"),
            recoverable: true,
        },
        MediaProviderError::Windows { code, message } => MediaDiagnostic {
            code: format!("windows_{code:08x}"),
            message: format!("Windows 媒体控制失败：{message}"),
            recoverable: true,
        },
    }
}
