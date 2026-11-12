use tauri::{menu::MenuBuilder, tray::TrayIconBuilder, Manager, Runtime};

use crate::{autostart, commands::AppState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayAction {
    Open,
    ToggleQuickBar,
    PlayPause,
    Next,
    Previous,
    ToggleAutostart,
    Quit,
}

impl TrayAction {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "open" => Some(Self::Open),
            "quick" => Some(Self::ToggleQuickBar),
            "play_pause" => Some(Self::PlayPause),
            "next" => Some(Self::Next),
            "previous" => Some(Self::Previous),
            "autostart" => Some(Self::ToggleAutostart),
            "quit" => Some(Self::Quit),
            _ => None,
        }
    }
}

pub fn build<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("open", "打开流光")
        .separator()
        .text("quick", "显示/隐藏便捷条")
        .text("play_pause", "播放 / 暂停")
        .text("previous", "上一首")
        .text("next", "下一首")
        .separator()
        .text("autostart", "切换开机启动")
        .separator()
        .text("quit", "退出流光")
        .build()?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("流光")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if let Some(action) = TrayAction::from_id(event.id().as_ref()) {
                handle_action(app, action);
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    let tray = builder.build(app)?;
    app.manage(tray);
    Ok(())
}

fn handle_action<R: Runtime>(app: &tauri::AppHandle<R>, action: TrayAction) {
    match action {
        TrayAction::Open => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }
        TrayAction::ToggleQuickBar => {
            if let Some(window) = app.get_webview_window("quick") {
                let visible = window.is_visible().unwrap_or(true);
                if visible {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                }
            }
        }
        TrayAction::PlayPause | TrayAction::Next | TrayAction::Previous => {
            let media = app.state::<AppState>().media.clone();
            tauri::async_runtime::spawn(async move {
                let snapshot = media.snapshot().await;
                if let Some(session_id) = snapshot.playback.map(|state| state.session_id) {
                    let _ = match action {
                        TrayAction::PlayPause => media.toggle_play_pause(&session_id).await,
                        TrayAction::Next => media.next_track(&session_id).await,
                        TrayAction::Previous => media.previous_track(&session_id).await,
                        _ => unreachable!(),
                    };
                }
            });
        }
        TrayAction::ToggleAutostart => {
            let state = app.state::<AppState>();
            let mut settings = state.settings.get();
            settings.start_with_windows = !settings.start_with_windows;
            let autostart_result = autostart::set_enabled(settings.start_with_windows);
            if autostart_result.is_ok() {
                let _ = state.settings.update(settings);
            }
        }
        TrayAction::Quit => app.exit(0),
    }
}

#[cfg(test)]
mod tests;
