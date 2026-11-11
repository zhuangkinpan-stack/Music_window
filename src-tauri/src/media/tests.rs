use super::*;
use crate::contracts::{MediaSessionSummary, PlaybackState, PlaybackStatus, SessionCapabilities};

#[tokio::test]
async fn coordinator_uses_current_session_when_selection_is_empty() {
    let provider = MockMediaProvider::from_sessions(vec![
        mock_session("spotify.exe", SessionCapabilities::default()),
        mock_session("cloudmusic.exe", SessionCapabilities::default()),
    ]);
    provider.set_current_session(Some("cloudmusic.exe"));
    let coordinator = MediaCoordinator::new(std::sync::Arc::new(provider));

    let snapshot = coordinator.snapshot().await;

    assert_eq!(
        snapshot.current_session_id.as_deref(),
        Some("cloudmusic.exe")
    );
}

#[tokio::test]
async fn coordinator_exposes_manual_selection_in_snapshot() {
    let provider = MockMediaProvider::from_sessions(vec![
        mock_session("spotify.exe", SessionCapabilities::default()),
        mock_session("cloudmusic.exe", SessionCapabilities::default()),
    ]);
    let coordinator = MediaCoordinator::new(std::sync::Arc::new(provider));
    coordinator.select_session("cloudmusic.exe").await.unwrap();

    let snapshot = coordinator.snapshot().await;

    assert_eq!(
        snapshot.selected_session_id.as_deref(),
        Some("cloudmusic.exe")
    );
}

struct FailingProvider;

#[async_trait::async_trait]
impl MediaSessionProvider for FailingProvider {
    async fn current_session_id(&self) -> Result<Option<String>, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }

    async fn sessions(&self) -> Result<Vec<MediaSessionSummary>, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }

    async fn playback_state(&self, _session_id: &str) -> Result<PlaybackState, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }

    async fn toggle_play_pause(&self, _session_id: &str) -> Result<bool, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }

    async fn next_track(&self, _session_id: &str) -> Result<bool, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }

    async fn previous_track(&self, _session_id: &str) -> Result<bool, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }

    async fn seek(&self, _session_id: &str, _position_ms: i64) -> Result<bool, MediaProviderError> {
        Err(MediaProviderError::SystemUnavailable(
            "audio service missing".to_owned(),
        ))
    }
}

#[tokio::test]
async fn coordinator_converts_provider_failure_to_diagnostic_snapshot() {
    let coordinator = MediaCoordinator::new(std::sync::Arc::new(FailingProvider));

    let snapshot = coordinator.snapshot().await;

    assert!(snapshot.sessions.is_empty());
    assert_eq!(
        snapshot
            .diagnostic
            .as_ref()
            .map(|diagnostic| diagnostic.code.as_str()),
        Some("smtc_unavailable")
    );
}

#[test]
fn maps_known_source_apps_to_friendly_names() {
    assert_eq!(friendly_app_name("CloudMusic.exe"), "网易云音乐");
    assert_eq!(friendly_app_name("Spotify.exe"), "Spotify");
    assert_eq!(friendly_app_name("QQMusic.exe"), "QQ 音乐");
    assert_eq!(
        friendly_app_name("com.example.UnknownPlayer_123!App"),
        "App"
    );
}

#[test]
fn converts_windows_time_ticks_to_milliseconds() {
    assert_eq!(ticks_to_millis(10_000_000), 1_000);
    assert_eq!(ticks_to_millis(-10_000_000), 0);
}

#[test]
fn maps_only_supported_image_types_to_extensions() {
    assert_eq!(image_extension("image/png"), Some("png"));
    assert_eq!(image_extension("image/jpeg"), Some("jpg"));
    assert_eq!(image_extension("text/html"), None);
}

#[test]
fn parses_netease_main_window_title() {
    assert_eq!(
        parse_netease_window_title("恋人 - 李荣浩"),
        Some(("恋人".to_owned(), "李荣浩".to_owned()))
    );
    assert_eq!(parse_netease_window_title("桌面歌词"), None);
}

#[test]
fn maps_netease_commands_to_renderer_control_points() {
    assert_eq!(
        netease_control_point(1_057, 752, APPCOMMAND_MEDIA_PREVIOUSTRACK),
        (467, 709)
    );
    assert_eq!(
        netease_control_point(1_057, 752, APPCOMMAND_MEDIA_PLAY_PAUSE),
        (528, 709)
    );
    assert_eq!(
        netease_control_point(1_057, 752, APPCOMMAND_MEDIA_NEXTTRACK),
        (584, 709)
    );
}

#[test]
fn identifies_cloudmusic_audio_process_paths() {
    assert!(is_cloudmusic_executable(
        r"C:\Program Files\Netease\CloudMusic\cloudmusic.exe"
    ));
    assert!(is_cloudmusic_executable(r"D:\Apps\CloudMusic.EXE"));
    assert!(!is_cloudmusic_executable(
        r"C:\Program Files\Spotify\Spotify.exe"
    ));
}

#[tokio::test]
async fn mock_provider_never_advertises_unavailable_controls() {
    let provider = MockMediaProvider::from_sessions(vec![mock_session(
        "player.exe",
        SessionCapabilities {
            play_pause: true,
            next: false,
            previous: false,
            seek: false,
        },
    )]);

    let sessions = provider.sessions().await.unwrap();

    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].capabilities.play_pause);
    assert!(!sessions[0].capabilities.next);
    assert!(!provider.next_track("player.exe").await.unwrap());
}

#[tokio::test]
async fn mock_provider_reports_removed_sessions() {
    let provider = MockMediaProvider::from_sessions(vec![mock_session(
        "player.exe",
        SessionCapabilities::default(),
    )]);

    provider.remove_session("player.exe");

    let error = provider.playback_state("player.exe").await.unwrap_err();
    assert!(matches!(error, MediaProviderError::SessionNotFound(_)));
}

fn mock_session(app_id: &str, capabilities: SessionCapabilities) -> MediaSessionSummary {
    MediaSessionSummary {
        id: app_id.to_owned(),
        app_name: friendly_app_name(app_id),
        title: "Test Track".to_owned(),
        artist: "Test Artist".to_owned(),
        album: "Test Album".to_owned(),
        artwork_url: None,
        playback_status: PlaybackStatus::Playing,
        capabilities,
    }
}
