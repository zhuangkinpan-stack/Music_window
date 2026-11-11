use std::time::Duration;

use tauri::{
    Emitter, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    UI::{
        Shell::{ABM_GETTASKBARPOS, APPBARDATA, SHAppBarMessage},
        WindowsAndMessaging::{
            GetCursorPos, GetWindowRect,
        },
    },
};

use crate::commands::AppState;

use super::{point_inside, InteractionLock};

pub fn create<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<WebviewWindow<R>> {
    WebviewWindowBuilder::new(
        app,
        "quick",
        WebviewUrl::App("index.html?window=quick".into()),
    )
    .title("流光便捷条")
    .inner_size(248.0, 48.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .focusable(false)
    .visible(true)
    .build()
}

pub fn configure<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    position_near_taskbar(window)?;
    window
        .set_ignore_cursor_events(true)
        .map_err(|error| error.to_string())
}

pub fn set_click_through<R: Runtime>(
    window: &WebviewWindow<R>,
    locked: bool,
) -> Result<(), String> {
    window
        .set_ignore_cursor_events(locked)
        .map_err(|error| error.to_string())
}

fn position_near_taskbar<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    let has_taskbar = unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut data) } != 0;

    if !has_taskbar || data.rc.right <= data.rc.left || data.rc.bottom <= data.rc.top {
        return Ok(());
    }

    let scale = window.scale_factor().unwrap_or(1.0);
    let taskbar_height = (data.rc.bottom - data.rc.top) as f64 / scale;
    let height = taskbar_height.clamp(40.0, 58.0);
    let _ = window.set_size(tauri::LogicalSize::new(248.0, height));

    let size = window.outer_size().map_err(|error| error.to_string())?;
    let width = size.width as i32;
    let height = size.height as i32;
    let horizontal = data.rc.right - data.rc.left >= data.rc.bottom - data.rc.top;
    let position = if horizontal {
        if data.rc.top > 0 {
            PhysicalPosition::new(data.rc.left + 16, data.rc.top - height - 8)
        } else {
            PhysicalPosition::new(data.rc.left + 16, data.rc.bottom + 8)
        }
    } else if data.rc.left > 0 {
        PhysicalPosition::new(data.rc.left - width - 8, data.rc.top + 16)
    } else {
        PhysicalPosition::new(data.rc.right + 8, data.rc.top + 16)
    };
    window
        .set_position(position)
        .map_err(|error| error.to_string())
}

pub fn cursor_inside<R: Runtime>(window: &WebviewWindow<R>) -> bool {
    let Ok(hwnd) = window.hwnd() else {
        return false;
    };
    let hwnd = HWND(hwnd.0 as isize as *mut std::ffi::c_void);
    let mut point = POINT::default();
    let mut rect = RECT::default();
    unsafe {
        if GetCursorPos(&mut point).is_err() || GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
    }
    point_inside(
        point.x,
        point.y,
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
    )
}

pub fn set_lock_state<R: Runtime>(
    app: &tauri::AppHandle<R>,
    locked: bool,
) -> Result<(), String> {
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
        loop {
            std::thread::sleep(Duration::from_millis(100));
            let Some(window) = app.get_webview_window("quick") else {
                break;
            };
            if !window.is_visible().unwrap_or(false) {
                continue;
            }
            let hovered = cursor_inside(&window);
            if last != Some(hovered) {
                last = Some(hovered);
                let _ = app.emit("quickbar://hover-state", hovered);
                if !hovered && !interaction.is_locked() {
                    interaction.lock();
                    let _ = set_lock_state(&app, true);
                }
            }
        }
    });
}

