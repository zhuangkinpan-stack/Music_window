use std::time::{Duration, Instant};

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

#[test]
fn netease_command_override_survives_until_observation_confirms() {
    let started = Instant::now();
    let mut runtime = NeteaseRuntimeState::new("song-a".to_owned());
    runtime.observe_track("song-a", started, PlaybackStatus::Paused);
    runtime.set_command_override(PlaybackStatus::Playing, started);

    // 音频会话还没跟上（仍报告暂停），按钮保持用户刚触发的状态
    assert_eq!(
        runtime.status_at(
            started + Duration::from_millis(5_000),
            PlaybackStatus::Paused
        ),
        PlaybackStatus::Playing
    );
    // 观测确认后立即交回真实状态，并由 override 退出
    assert_eq!(
        runtime.status_at(
            started + Duration::from_millis(5_500),
            PlaybackStatus::Playing
        ),
        PlaybackStatus::Playing
    );
    assert_eq!(
        runtime.status_at(
            started + Duration::from_millis(6_000),
            PlaybackStatus::Paused
        ),
        PlaybackStatus::Paused
    );
}

#[test]
fn netease_command_override_expires_when_player_never_follows() {
    let started = Instant::now();
    let mut runtime = NeteaseRuntimeState::new("song-a".to_owned());
    runtime.observe_track("song-a", started, PlaybackStatus::Playing);
    runtime.set_command_override(PlaybackStatus::Paused, started);

    assert_eq!(
        runtime.status_at(started + Duration::from_secs(7), PlaybackStatus::Playing),
        PlaybackStatus::Paused
    );
    assert_eq!(
        runtime.status_at(started + Duration::from_secs(9), PlaybackStatus::Playing),
        PlaybackStatus::Playing
    );
}

#[test]
fn netease_track_change_clears_timeline_and_override() {
    let started = Instant::now();
    let mut runtime = NeteaseRuntimeState::new("song-a".to_owned());
    runtime.observe_track("song-a", started, PlaybackStatus::Playing);
    runtime.set_command_override(PlaybackStatus::Playing, started);
    runtime.set_timeline(1_000, 200_000, started);

    assert!(runtime.observe_track(
        "song-b",
        started + Duration::from_secs(1),
        PlaybackStatus::Paused
    ));

    assert_eq!(runtime.timeline(), Some((0, 0)));
    assert_eq!(
        runtime.status_at(started + Duration::from_secs(1), PlaybackStatus::Paused),
        PlaybackStatus::Paused
    );
}

#[test]
fn netease_timeline_freezes_at_live_position_on_pause() {
    let captured = Instant::now();
    let mut runtime = NeteaseRuntimeState::new(String::new());
    runtime.observe_track("song-a", captured, PlaybackStatus::Playing);
    runtime.set_timeline(10_000, 200_000, captured);

    let paused_at = captured + Duration::from_millis(1_500);
    runtime.sync_status(PlaybackStatus::Paused, paused_at);

    // 暂停后停在暂停那一刻的真实位置，而不是上次采样或 0
    assert_eq!(
        runtime.timeline_at(paused_at + Duration::from_secs(30)),
        (11_500, 200_000)
    );
}

#[test]
fn netease_timeline_resumes_from_frozen_position() {
    let captured = Instant::now();
    let mut runtime = NeteaseRuntimeState::new(String::new());
    runtime.observe_track("song-a", captured, PlaybackStatus::Playing);
    runtime.set_timeline(10_000, 200_000, captured);

    runtime.sync_status(PlaybackStatus::Paused, captured + Duration::from_secs(2));
    runtime.sync_status(PlaybackStatus::Playing, captured + Duration::from_secs(30));

    assert_eq!(
        runtime.timeline_at(captured + Duration::from_secs(31)),
        (13_000, 200_000)
    );
}

#[test]
fn netease_timeline_keeps_wall_clock_position_without_ocr() {
    let started = Instant::now();
    let mut runtime = NeteaseRuntimeState::new(String::new());
    runtime.observe_track("song-a", started, PlaybackStatus::Playing);

    assert!(runtime.timeline_estimated());
    assert_eq!(
        runtime.timeline_at(started + Duration::from_secs(12)),
        (12_000, 0)
    );

    runtime.sync_status(PlaybackStatus::Paused, started + Duration::from_secs(12));
    assert_eq!(
        runtime.timeline_at(started + Duration::from_secs(60)),
        (12_000, 0)
    );
}

#[test]
fn parses_netease_progress_text() {
    assert_eq!(
        super::netease_timeline::parse_netease_progress_text("01:23 / 04:56"),
        Some((83_000, 296_000))
    );
    assert_eq!(
        super::netease_timeline::parse_netease_progress_text("1:05/3:20"),
        Some((65_000, 200_000))
    );
    assert_eq!(
        super::netease_timeline::parse_netease_progress_text("01：23 / 04：56"),
        Some((83_000, 296_000))
    );
    assert_eq!(
        super::netease_timeline::parse_netease_progress_text("01:23"),
        None
    );
}

#[test]
fn crops_the_bottom_progress_region() {
    assert_eq!(
        super::netease_timeline::netease_progress_crop(1_057, 752),
        (0, 682, 1_057, 752)
    );
}

#[test]
fn maps_bottom_up_dib_rows_to_visual_rows() {
    assert_eq!(super::netease_timeline::dib_source_row(752, 682), 69);
    assert_eq!(super::netease_timeline::dib_source_row(752, 751), 0);
}

#[test]
fn isolates_left_and_right_time_labels() {
    assert_eq!(
        super::netease_timeline::netease_time_label_crops(1_057, 752),
        [(0, 692, 180, 752), (877, 692, 1_057, 752),]
    );
}

#[test]
fn enters_estimated_timeline_after_three_consecutive_ocr_failures() {
    let start = Instant::now();
    let mut failures = super::netease_timeline::TimelineFailureTracker::new();

    assert!(!failures.record_failure(start));
    assert!(!failures.record_failure(start));
    assert!(failures.record_failure(start));
    assert!(failures.is_estimated());
    // 估算期间按间隔重试，而不是永久放弃采样
    assert!(!failures.should_attempt(start));
    assert!(failures.should_attempt(start + Duration::from_secs(21)));
    failures.record_success();
    assert!(failures.should_attempt(start + Duration::from_secs(22)));
    assert!(!failures.record_failure(start + Duration::from_secs(22)));
    assert!(!failures.record_failure(start + Duration::from_secs(22)));
    assert!(failures.record_failure(start + Duration::from_secs(22)));
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

/// 需要本机正在运行网易云音乐，手动执行：
/// `cargo test --lib netease_live_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn netease_live_probe() {
    let client = super::netease::NeteaseClient::new();
    for round in 0..40 {
        let session = client.session();
        let state = client.playback_state(super::netease::NET_EASE_SESSION_ID);
        match (session, state) {
            (Some(session), Ok(state)) => println!(
                "#{round} {} - {} | {:?} {}ms/{}ms estimated={}",
                session.title,
                session.artist,
                state.playback_status,
                state.position_ms,
                state.duration_ms,
                state.timeline_estimated
            ),
            (None, _) => println!("#{round} no netease session"),
            (_, Err(error)) => println!("#{round} error: {error}"),
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// 端到端验证暂停/恢复时钟：需要本机正在运行网易云音乐。
/// `cargo test --lib netease_live_pause_cycle -- --ignored --nocapture`
#[test]
#[ignore]
fn netease_live_pause_cycle() {
    let client = super::netease::NeteaseClient::new();
    let id = super::netease::NET_EASE_SESSION_ID;
    let session = client.session().expect("netease session");
    println!("track: {} - {}", session.title, session.artist);

    if session.playback_status != PlaybackStatus::Playing {
        println!(
            "start playback: {}",
            client
                .send_command(id, APPCOMMAND_MEDIA_PLAY_PAUSE)
                .unwrap()
        );
    }
    for round in 0..12 {
        let state = client.playback_state(id).unwrap();
        println!(
            "play #{round} {:?} {}ms/{}ms estimated={}",
            state.playback_status, state.position_ms, state.duration_ms, state.timeline_estimated
        );
        std::thread::sleep(Duration::from_millis(500));
    }

    println!(
        "pause: {}",
        client
            .send_command(id, APPCOMMAND_MEDIA_PLAY_PAUSE)
            .unwrap()
    );
    let mut paused_positions = Vec::new();
    for round in 0..6 {
        let state = client.playback_state(id).unwrap();
        println!(
            "paused #{round} {:?} {}ms",
            state.playback_status, state.position_ms
        );
        paused_positions.push(state.position_ms);
        std::thread::sleep(Duration::from_millis(500));
    }

    let first = paused_positions.first().copied().unwrap_or(0);
    let last = paused_positions.last().copied().unwrap_or(0);
    assert_eq!(first, last, "暂停后位置必须冻结不漂移");
}
