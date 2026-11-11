use ::windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use ::windows::Storage::Streams::DataReader;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::OnceCell;

use crate::contracts::{MediaSessionSummary, PlaybackState, PlaybackStatus, SessionCapabilities};

use super::{
    friendly_app_name, image_data_uri, ticks_to_millis, MediaProviderError, MediaSessionProvider,
};

pub struct WindowsMediaProvider {
    manager: OnceCell<GlobalSystemMediaTransportControlsSessionManager>,
    artwork_cache: Mutex<HashMap<String, Option<String>>>,
}

impl Default for WindowsMediaProvider {
    fn default() -> Self {
        Self {
            manager: OnceCell::new(),
            artwork_cache: Mutex::new(HashMap::new()),
        }
    }
}

impl WindowsMediaProvider {
    pub fn new() -> Self {
        Self::default()
    }

    async fn manager(
        &self,
    ) -> Result<&GlobalSystemMediaTransportControlsSessionManager, MediaProviderError> {
        self.manager
            .get_or_try_init(|| async {
                GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?
                    .get()
                    .map_err(Into::into)
            })
            .await
    }

    async fn find_session(
        &self,
        session_id: &str,
    ) -> Result<GlobalSystemMediaTransportControlsSession, MediaProviderError> {
        let manager = self.manager().await?;
        let sessions = manager.GetSessions()?;
        for index in 0..sessions.Size()? {
            let session = sessions.GetAt(index)?;
            let source = session.SourceAppUserModelId()?.to_string();
            if source == session_id {
                return Ok(session);
            }
        }
        Err(MediaProviderError::SessionNotFound(session_id.to_owned()))
    }

    fn summarize(
        &self,
        session: GlobalSystemMediaTransportControlsSession,
    ) -> Result<MediaSessionSummary, MediaProviderError> {
        let id = session.SourceAppUserModelId()?.to_string();
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;
        let playback = session.GetPlaybackInfo()?;
        let controls = playback.Controls()?;

        let artwork_url = self.artwork_url(&id, &properties)?;
        Ok(MediaSessionSummary {
            app_name: friendly_app_name(&id),
            id,
            title: properties.Title().unwrap_or_default().to_string(),
            artist: properties.Artist().unwrap_or_default().to_string(),
            album: properties.AlbumTitle().unwrap_or_default().to_string(),
            artwork_url,
            playback_status: map_playback_status(playback.PlaybackStatus()?),
            capabilities: SessionCapabilities {
                play_pause: controls.IsPlayPauseToggleEnabled().unwrap_or(false),
                next: controls.IsNextEnabled().unwrap_or(false),
                previous: controls.IsPreviousEnabled().unwrap_or(false),
                seek: controls.IsPlaybackPositionEnabled().unwrap_or(false),
            },
        })
    }

    fn artwork_url(
        &self,
        source: &str,
        properties: &::windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
    ) -> Result<Option<String>, MediaProviderError> {
        let title = properties.Title().unwrap_or_default().to_string();
        let album = properties.AlbumTitle().unwrap_or_default().to_string();
        let key = format!("{source}\u{1f}{title}\u{1f}{album}");
        if let Some(cached) = self
            .artwork_cache
            .lock()
            .expect("artwork cache poisoned")
            .get(&key)
        {
            return Ok(cached.clone());
        }

        let artwork = properties
            .Thumbnail()
            .ok()
            .and_then(|reference| reference.OpenReadAsync().ok())
            .and_then(|operation| operation.get().ok())
            .and_then(|stream| {
                let mime = stream.ContentType().ok()?.to_string();
                let size = stream.Size().ok()?;
                if size == 0 || size > 5 * 1024 * 1024 {
                    return None;
                }
                let input = stream.GetInputStreamAt(0).ok()?;
                let reader = DataReader::CreateDataReader(&input).ok()?;
                reader.LoadAsync(size as u32).ok()?.get().ok()?;
                let length = reader.UnconsumedBufferLength().ok()? as usize;
                let mut bytes = vec![0; length];
                reader.ReadBytes(&mut bytes).ok()?;
                image_data_uri(&mime, &bytes)
            });
        self.artwork_cache
            .lock()
            .expect("artwork cache poisoned")
            .insert(key, artwork.clone());
        Ok(artwork)
    }
}

#[async_trait]
impl MediaSessionProvider for WindowsMediaProvider {
    async fn current_session_id(&self) -> Result<Option<String>, MediaProviderError> {
        let manager = self.manager().await?;
        Ok(manager
            .GetCurrentSession()
            .ok()
            .and_then(|session| session.SourceAppUserModelId().ok())
            .map(|id| id.to_string()))
    }

    async fn sessions(&self) -> Result<Vec<MediaSessionSummary>, MediaProviderError> {
        let manager = self.manager().await?;
        let sessions = manager.GetSessions()?;
        let mut result = Vec::with_capacity(sessions.Size()? as usize);
        for index in 0..sessions.Size()? {
            if let Ok(summary) = self.summarize(sessions.GetAt(index)?) {
                result.push(summary);
            }
        }
        Ok(result)
    }

    async fn playback_state(&self, session_id: &str) -> Result<PlaybackState, MediaProviderError> {
        let session = self.find_session(session_id).await?;
        let timeline = session.GetTimelineProperties()?;
        let playback = session.GetPlaybackInfo()?;
        let start = timeline.StartTime()?.Duration;
        let end = timeline.EndTime()?.Duration;
        let position = timeline.Position()?.Duration;

        Ok(PlaybackState {
            session_id: session_id.to_owned(),
            playback_status: map_playback_status(playback.PlaybackStatus()?),
            position_ms: ticks_to_millis(position.saturating_sub(start)),
            duration_ms: ticks_to_millis(end.saturating_sub(start)),
        })
    }

    async fn toggle_play_pause(&self, session_id: &str) -> Result<bool, MediaProviderError> {
        Ok(self
            .find_session(session_id)
            .await?
            .TryTogglePlayPauseAsync()?
            .get()?)
    }

    async fn next_track(&self, session_id: &str) -> Result<bool, MediaProviderError> {
        Ok(self
            .find_session(session_id)
            .await?
            .TrySkipNextAsync()?
            .get()?)
    }

    async fn previous_track(&self, session_id: &str) -> Result<bool, MediaProviderError> {
        Ok(self
            .find_session(session_id)
            .await?
            .TrySkipPreviousAsync()?
            .get()?)
    }

    async fn seek(&self, session_id: &str, position_ms: i64) -> Result<bool, MediaProviderError> {
        Ok(self
            .find_session(session_id)
            .await?
            .TryChangePlaybackPositionAsync(position_ms.max(0).saturating_mul(10_000))?
            .get()?)
    }
}

fn map_playback_status(
    status: GlobalSystemMediaTransportControlsSessionPlaybackStatus,
) -> PlaybackStatus {
    match status {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => PlaybackStatus::Opened,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => {
            PlaybackStatus::Changing
        }
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => PlaybackStatus::Stopped,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => PlaybackStatus::Playing,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => PlaybackStatus::Paused,
        _ => PlaybackStatus::Closed,
    }
}
