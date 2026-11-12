use std::time::Duration;

use tauri::{
    Emitter, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        Shell::{SHAppBarMessage, ABM_GETTASKBARPOS, APPBARDATA},
        WindowsAndMessaging::{
            GetCursorPos, GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
        },
    },
};

use crate::commands::AppState;
use crate::contracts::{AppSettings, WindowPosition};

use super::{
    classify_quick_bar_zone, dragged_origin, point_inside, should_relock_after_hover,
    InteractionLock, QuickBarZone,
};

pub fn create<R: Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &AppSettings,
) -> tauri::Result<WebviewWindow<R>> {
    WebviewWindowBuilder::new(
        app,
        "quick",
        WebviewUrl::App("index.html?window=quick".into()),
    )
    .title("流光便捷条")
    .inner_size(width_dips(settings.quick_bar_width_cm), 48.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .focusable(false)
    .visible(false)
    .build()
}

pub fn configure<R: Runtime>(
    window: &WebviewWindow<R>,
    settings: &AppSettings,
) -> Result<(), String> {
    apply_geometry(
        window,
        settings.quick_bar_width_cm,
        settings.quick_bar_position.as_ref(),
    )?;
    window
        .set_ignore_cursor_events(true)
        .map_err(|error| error.to_string())
}

pub fn apply_geometry<R: Runtime>(
    window: &WebviewWindow<R>,
    width_cm: f64,
    saved_position: Option<&WindowPosition>,
) -> Result<(), String> {
    let scale = window.scale_factor().unwrap_or(1.0);
    let taskbar = taskbar_rect();
    let height = taskbar
        .map(|rect| (rect.bottom - rect.top) as f64 / scale)
        .unwrap_or(48.0)
        .clamp(40.0, 58.0);
    let width = width_dips(width_cm);
    let _ = window.set_size(tauri::LogicalSize::new(width, height));

    if let Some(position) = saved_position {
        return window
            .set_position(PhysicalPosition::new(position.x, position.y))
            .map_err(|error| error.to_string());
    }
    position_near_taskbar(window, taskbar)?;
    Ok(())
}

pub fn set_click_through<R: Runtime>(
    window: &WebviewWindow<R>,
    locked: bool,
) -> Result<(), String> {
    window
        .set_ignore_cursor_events(locked)
        .map_err(|error| error.to_string())
}

fn taskbar_rect() -> Option<RECT> {
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    let has_taskbar = unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut data) } != 0;

    (has_taskbar && data.rc.right > data.rc.left && data.rc.bottom > data.rc.top).then_some(data.rc)
}

fn position_near_taskbar<R: Runtime>(
    window: &WebviewWindow<R>,
    taskbar: Option<RECT>,
) -> Result<(), String> {
    let Some(taskbar) = taskbar else {
        return Ok(());
    };

    let size = window.outer_size().map_err(|error| error.to_string())?;
    let width = size.width as i32;
    let height = size.height as i32;
    let horizontal = taskbar.right - taskbar.left >= taskbar.bottom - taskbar.top;
    let position = if horizontal {
        if taskbar.top > 0 {
            PhysicalPosition::new(taskbar.left + 16, taskbar.top - height - 8)
        } else {
            PhysicalPosition::new(taskbar.left + 16, taskbar.bottom + 8)
        }
    } else if taskbar.left > 0 {
        PhysicalPosition::new(taskbar.left - width - 8, taskbar.top + 16)
    } else {
        PhysicalPosition::new(taskbar.right + 8, taskbar.top + 16)
    };
    window
        .set_position(position)
        .map_err(|error| error.to_string())
}

fn width_dips(width_cm: f64) -> f64 {
    width_cm / 2.54 * 96.0
}

pub fn set_lock_state<R: Runtime>(app: &tauri::AppHandle<R>, locked: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("quick") {
        set_click_through(&window, locked)?;
    }
    app.emit("quickbar://lock-state", locked)
        .map_err(|error| error.to_string())
}

pub fn schedule_auto_relock<R: Runtime>(app: tauri::AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let state = app.state::<AppState>();
            if state.interaction.should_relock(std::time::Instant::now()) {
                state.interaction.lock();
                let _ = set_lock_state(&app, true);
                break;
            }
        }
    });
}

pub fn start_hover_monitor<R: Runtime>(
    app: tauri::AppHandle<R>,
    interaction: std::sync::Arc<InteractionLock>,
) {
    std::thread::spawn(move || {
        let mut last = None;
        let mut last_button_down = false;
        let mut drag: Option<((i32, i32), (i32, i32))> = None;
        let mut hovered_since_unlock = false;
        loop {
            let locked = interaction.is_locked();
            std::thread::sleep(Duration::from_millis(if locked { 100 } else { 16 }));
            let Some(window) = app.get_webview_window("quick") else {
                break;
            };
            if !window.is_visible().unwrap_or(false) {
                continue;
            }
            let mut point = POINT::default();
            let mut rect = RECT::default();
            let Ok(hwnd) = window.hwnd() else {
                continue;
            };
            let hwnd = HWND(hwnd.0 as isize as *mut std::ffi::c_void);
            unsafe {
                if GetCursorPos(&mut point).is_err() || GetWindowRect(hwnd, &mut rect).is_err() {
                    continue;
                }
            }
            let hovered = point_inside(
                point.x,
                point.y,
                rect.left,
                rect.top,
                rect.right,
                rect.bottom,
            );
            if last != Some(hovered) {
                last = Some(hovered);
                let _ = app.emit("quickbar://hover-state", hovered);
            }
            if hovered && !locked {
                hovered_since_unlock = true;
            }
            if drag.is_none() && should_relock_after_hover(locked, hovered, hovered_since_unlock) {
                interaction.lock();
                let _ = set_lock_state(&app, true);
            }

            if locked {
                hovered_since_unlock = false;
                last_button_down = false;
                drag = None;
                continue;
            }

            let button_down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
            if button_down && !last_button_down && hovered {
                interaction.touch(std::time::Instant::now());
                let width = rect.right - rect.left;
                match classify_quick_bar_zone(point.x - rect.left, width) {
                    QuickBarZone::Drag => {
                        drag = Some(((rect.left, rect.top), (point.x, point.y)));
                    }
                    zone => dispatch_media_command(&app, zone),
                }
            }

            if button_down {
                if let Some((window_origin, cursor_origin)) = drag {
                    let (x, y) = dragged_origin(
                        window_origin.0,
                        window_origin.1,
                        cursor_origin.0,
                        cursor_origin.1,
                        point.x,
                        point.y,
                    );
                    unsafe {
                        let _ = SetWindowPos(
                            hwnd,
                            None,
                            x,
                            y,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                    interaction.touch(std::time::Instant::now());
                }
            } else if drag.take().is_some() {
                let mut final_rect = RECT::default();
                unsafe {
                    if GetWindowRect(hwnd, &mut final_rect).is_err() {
                        continue;
                    }
                }
                let mut settings = app.state::<AppState>().settings.get();
                settings.quick_bar_position = Some(WindowPosition {
                    x: final_rect.left,
                    y: final_rect.top,
                    monitor: None,
                });
                let _ = app.state::<AppState>().settings.update(settings);
            }
            last_button_down = button_down;
        }
    });
}

fn dispatch_media_command<R: Runtime>(app: &tauri::AppHandle<R>, zone: QuickBarZone) {
    let media = app.state::<AppState>().media.clone();
    tauri::async_runtime::spawn(async move {
        let snapshot = media.snapshot().await;
        let Some(session_id) = snapshot.selected_session_id else {
            return;
        };
        let _ = match zone {
            QuickBarZone::Previous => media.previous_track(&session_id).await,
            QuickBarZone::PlayPause => media.toggle_play_pause(&session_id).await,
            QuickBarZone::Next => media.next_track(&session_id).await,
            QuickBarZone::Drag | QuickBarZone::Locked => return,
        };
    });
}
