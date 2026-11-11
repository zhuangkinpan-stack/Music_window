use std::{
    sync::mpsc::{self, SyncSender},
    time::Duration,
};

use ::windows::{
    core::{w, Interface, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM, WPARAM},
        Media::Audio::{
            eConsole, eRender, AudioSessionStateActive, IAudioSessionControl2,
            IAudioSessionManager2, IMMDeviceEnumerator, MMDeviceEnumerator,
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
        },
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            FindWindowExW, FindWindowW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
            GetWindowThreadProcessId, IsIconic, IsWindowVisible, PostMessageW, ShowWindow, SW_HIDE,
            SW_MINIMIZE, SW_SHOWNOACTIVATE, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
        },
    },
};

use crate::contracts::{MediaSessionSummary, PlaybackState, PlaybackStatus, SessionCapabilities};

use super::MediaProviderError;

pub const NET_EASE_SESSION_ID: &str = "netease:cloudmusic";
pub const APPCOMMAND_MEDIA_NEXTTRACK: u32 = 11;
pub const APPCOMMAND_MEDIA_PREVIOUSTRACK: u32 = 12;
pub const APPCOMMAND_MEDIA_PLAY_PAUSE: u32 = 14;

pub fn parse_netease_window_title(title: &str) -> Option<(String, String)> {
    let title = title.trim();
    if title.is_empty() || matches!(title, "网易云音乐" | "桌面歌词" | "迷你播放器") {
        return None;
    }

    let (track, artist) = title.rsplit_once(" - ")?;
    let track = track.trim();
    let artist = artist.trim();
    if track.is_empty() || artist.is_empty() {
        return None;
    }
    Some((track.to_owned(), artist.to_owned()))
}

pub fn netease_control_point(width: i32, height: i32, command: u32) -> (i32, i32) {
    let center = width / 2;
    let offset = match command {
        APPCOMMAND_MEDIA_PREVIOUSTRACK => -61,
        APPCOMMAND_MEDIA_NEXTTRACK => 56,
        _ => 0,
    };
    ((center + offset).max(0), (height - 43).max(0))
}

pub(crate) fn is_cloudmusic_executable(path: &str) -> bool {
    path.rsplit(['\\', '/'])
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("cloudmusic.exe"))
}

pub struct NeteaseClient {
    audio_probe: AudioSessionProbe,
}

impl NeteaseClient {
    pub fn new() -> Self {
        Self {
            audio_probe: AudioSessionProbe::new(),
        }
    }

    pub fn session(&self) -> Option<MediaSessionSummary> {
        let window = find_netease_window()?;
        let playback_status = if self.audio_probe.has_active_session(window.process_id) {
            PlaybackStatus::Playing
        } else {
            PlaybackStatus::Paused
        };

        Some(MediaSessionSummary {
            id: NET_EASE_SESSION_ID.to_owned(),
            app_name: "网易云音乐".to_owned(),
            title: window.track,
            artist: window.artist,
            album: String::new(),
            artwork_url: None,
            playback_status,
            capabilities: SessionCapabilities {
                play_pause: true,
                next: true,
                previous: true,
                seek: false,
            },
        })
    }

    pub fn current_if_playing(&self) -> Option<String> {
        self.session()
            .filter(|session| session.playback_status == PlaybackStatus::Playing)
            .map(|session| session.id)
    }

    pub fn playback_state(&self, session_id: &str) -> Result<PlaybackState, MediaProviderError> {
        ensure_netease_id(session_id)?;
        let session = self
            .session()
            .ok_or_else(|| MediaProviderError::SessionNotFound(session_id.to_owned()))?;
        Ok(PlaybackState {
            session_id: session.id,
            playback_status: session.playback_status,
            position_ms: 0,
            duration_ms: 0,
        })
    }

    pub fn send_command(&self, session_id: &str, command: u32) -> Result<bool, MediaProviderError> {
        ensure_netease_id(session_id)?;
        let window = find_netease_window()
            .ok_or_else(|| MediaProviderError::SessionNotFound(session_id.to_owned()))?;
        Ok(unsafe { click_netease_control(window.hwnd, command) }.unwrap_or(false))
    }
}

fn ensure_netease_id(session_id: &str) -> Result<(), MediaProviderError> {
    if session_id == NET_EASE_SESSION_ID {
        Ok(())
    } else {
        Err(MediaProviderError::SessionNotFound(session_id.to_owned()))
    }
}

struct NeteaseWindow {
    hwnd: HWND,
    process_id: u32,
    track: String,
    artist: String,
}

fn find_netease_window() -> Option<NeteaseWindow> {
    let hwnd = unsafe { FindWindowW(w!("OrpheusBrowserHost"), PCWSTR::null()) }.ok()?;
    if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return None;
    }

    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length <= 0 {
        return None;
    }
    let mut title = vec![0u16; length as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut title) };
    if copied <= 0 {
        return None;
    }
    let title = String::from_utf16_lossy(&title[..copied as usize]);
    let (track, artist) = parse_netease_window_title(&title)?;

    let mut process_id = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }
    if process_id == 0 {
        return None;
    }

    Some(NeteaseWindow {
        hwnd,
        process_id,
        track,
        artist,
    })
}

unsafe fn click_netease_control(hwnd: HWND, command: u32) -> ::windows::core::Result<bool> {
    unsafe {
        let was_minimized = IsIconic(hwnd).as_bool();
        let was_visible = IsWindowVisible(hwnd).as_bool();
        if was_minimized || !was_visible {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            std::thread::sleep(Duration::from_millis(80));
        }

        let result = (|| {
            let browser = FindWindowExW(Some(hwnd), None, w!("CefBrowserWindow"), PCWSTR::null())?;
            let chrome = FindWindowExW(
                Some(browser),
                None,
                w!("Chrome_WidgetWin_0"),
                PCWSTR::null(),
            )?;
            let renderer = FindWindowExW(
                Some(chrome),
                None,
                w!("Chrome_RenderWidgetHostHWND"),
                PCWSTR::null(),
            )?;
            let mut rect = Default::default();
            GetWindowRect(renderer, &mut rect)?;
            let (x, y) =
                netease_control_point(rect.right - rect.left, rect.bottom - rect.top, command);
            let lparam = LPARAM(((y as isize & 0xffff) << 16) | (x as isize & 0xffff));
            PostMessageW(Some(renderer), WM_MOUSEMOVE, WPARAM(0), lparam)?;
            PostMessageW(Some(renderer), WM_LBUTTONDOWN, WPARAM(1), lparam)?;
            std::thread::sleep(Duration::from_millis(60));
            PostMessageW(Some(renderer), WM_LBUTTONUP, WPARAM(0), lparam)?;
            std::thread::sleep(Duration::from_millis(80));
            Ok(true)
        })();

        if was_minimized {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        } else if !was_visible {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        result
    }
}

struct AudioProbeRequest {
    process_id: u32,
    reply: SyncSender<bool>,
}

struct AudioSessionProbe {
    sender: mpsc::Sender<AudioProbeRequest>,
}

impl AudioSessionProbe {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<AudioProbeRequest>();
        std::thread::Builder::new()
            .name("lumi-netease-audio-probe".to_owned())
            .spawn(move || {
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                }
                while let Ok(request) = receiver.recv() {
                    let active = unsafe { has_active_audio_session(request.process_id) };
                    let _ = request.reply.send(active);
                }
                unsafe {
                    CoUninitialize();
                }
            })
            .expect("failed to start NetEase audio probe");
        Self { sender }
    }

    fn has_active_session(&self, process_id: u32) -> bool {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.sender
            .send(AudioProbeRequest { process_id, reply })
            .is_ok()
            && receiver
                .recv_timeout(Duration::from_millis(250))
                .unwrap_or(false)
    }
}

unsafe fn has_active_audio_session(process_id: u32) -> bool {
    unsafe {
        let Ok(enumerator) =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
        else {
            return false;
        };
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else {
            return false;
        };
        let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
            return false;
        };
        let Ok(sessions) = manager.GetSessionEnumerator() else {
            return false;
        };
        let Ok(count) = sessions.GetCount() else {
            return false;
        };

        for index in 0..count {
            let Ok(control) = sessions.GetSession(index) else {
                continue;
            };
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let Ok(session_process_id) = control2.GetProcessId() else {
                continue;
            };
            if control.GetState().ok() != Some(AudioSessionStateActive) {
                continue;
            }
            if session_process_id == process_id {
                return true;
            }
            if process_image_path(session_process_id)
                .as_deref()
                .is_some_and(is_cloudmusic_executable)
            {
                return true;
            }
        }
        false
    }
}

unsafe fn process_image_path(process_id: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
        let mut buffer = vec![0u16; 32_768];
        let mut length = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(process);
        result.ok()?;
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }
}
