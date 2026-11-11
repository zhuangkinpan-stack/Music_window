use ::windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use ::windows::Storage::Streams::DataReader;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tokio::sync::OnceCell;

use crate::contracts::{MediaSessionSummary, PlaybackState, PlaybackStatus, SessionCapabilities};

use super::netease::{
    NeteaseClient, APPCOMMAND_MEDIA_NEXTTRACK, APPCOMMAND_MEDIA_PLAY_PAUSE,
    APPCOMMAND_MEDIA_PREVIOUSTRACK, NET_EASE_SESSION_ID,
};
use super::{
    friendly_app_name, image_extension, ticks_to_millis, MediaProviderError, MediaSessionProvider,
};

pub struct WindowsMediaProvider {
    manager: OnceCell<GlobalSystemMediaTransportControlsSessionManager>,
    artwork_cache: Mutex<HashMap<String, Option<String>>>,
    artwork_cache_dir: PathBuf,
    netease: NeteaseClient,
}

impl WindowsMediaProvider {
    pub fn new(artwork_cache_dir: PathBuf) -> Self {
        Self {
            manager: OnceCell::new(),
            artwork_cache: Mutex::new(HashMap::new()),
            artwork_cache_dir,
            netease: NeteaseClient::new(),
        }
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
                let extension = image_extension(&mime)?;
                let mut bytes = vec![0; length];
                reader.ReadBytes(&mut bytes).ok()?;
                self.cache_artwork(&key, extension, &bytes).ok()
            });
        let mut cache = self.artwork_cache.lock().expect("artwork cache poisoned");
        if cache.len() >= 16 {
            cache.clear();
        }
        cache.insert(key, artwork.clone());
        Ok(artwork)
    }

    fn cache_artwork(&self, key: &str, extension: &str, bytes: &[u8]) -> std::io::Result<String> {
        std::fs::create_dir_all(&self.artwork_cache_dir)?;
        let digest = Sha256::digest(key.as_bytes());
        let filename = format!(
            "{}.{}",
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            extension
        );
        let path = self.artwork_cache_dir.join(filename);
        write_if_changed(&path, bytes)?;
        Ok(path.to_string_lossy().into_owned())
    }
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|existing| existing == bytes) {
        return Ok(());
    }
    std::fs::write(path, bytes)
}

#[async_trait]
impl MediaSessionProvider for WindowsMediaProvider {
    async fn current_session_id(&self) -> Result<Option<String>, MediaProviderError> {
        if let Some(session_id) = self.netease.current_if_playing() {
            return Ok(Some(session_id));
        }
        let manager = match self.manager().await {
            Ok(manager) => manager,
            Err(_) if self.netease.session().is_some() => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(manager
            .GetCurrentSession()
            .ok()
            .and_then(|session| session.SourceAppUserModelId().ok())
            .map(|id| id.to_string()))
    }

    async fn sessions(&self) -> Result<Vec<MediaSessionSummary>, MediaProviderError> {
        let mut result = Vec::new();
        let manager_result = self.manager().await;
        if let Ok(manager) = manager_result {
            let sessions = manager.GetSessions()?;
            result.reserve(sessions.Size()? as usize + 1);
            for index in 0..sessions.Size()? {
                if let Ok(summary) = self.summarize(sessions.GetAt(index)?) {
                    result.push(summary);
                }
            }
        }
        if let Some(session) = self.netease.session() {
            result.push(session);
        } else {
            manager_result?;
        }
        Ok(result)
    }

    async fn playback_state(&self, session_id: &str) -> Result<PlaybackState, MediaProviderError> {
        if session_id == NET_EASE_SESSION_ID {
            return self.netease.playback_state(session_id);
        }
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
        if session_id == NET_EASE_SESSION_ID {
            return self
                .netease
                .send_command(session_id, APPCOMMAND_MEDIA_PLAY_PAUSE);
        }
        Ok(self
            .find_session(session_id)
            .await?
            .TryTogglePlayPauseAsync()?
            .get()?)
    }

    async fn next_track(&self, session_id: &str) -> Result<bool, MediaProviderError> {
        if session_id == NET_EASE_SESSION_ID {
            return self
                .netease
                .send_command(session_id, APPCOMMAND_MEDIA_NEXTTRACK);
        }
        Ok(self
            .find_session(session_id)
            .await?
            .TrySkipNextAsync()?
            .get()?)
    }

    async fn previous_track(&self, session_id: &str) -> Result<bool, MediaProviderError> {
        if session_id == NET_EASE_SESSION_ID {
            return self
                .netease
                .send_command(session_id, APPCOMMAND_MEDIA_PREVIOUSTRACK);
        }
        Ok(self
            .find_session(session_id)
            .await?
            .TrySkipPreviousAsync()?
            .get()?)
    }

    async fn seek(&self, session_id: &str, position_ms: i64) -> Result<bool, MediaProviderError> {
        if session_id == NET_EASE_SESSION_ID {
            return Ok(false);
        }
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
