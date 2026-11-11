use tauri::{Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{commands::AppState, contracts::AppSettings, quick_bar};

pub fn rebind<R: Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &AppSettings,
) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|error| error.to_string())?;

    app.global_shortcut()
        .on_shortcut(settings.unlock_shortcut.as_str(), |app, _, event| {
            if event.state == ShortcutState::Pressed {
                let state = app.state::<AppState>();
                state
                    .interaction
                    .unlock(std::time::Instant::now());
                let _ = quick_bar::set_lock_state(app, false);
                quick_bar::schedule_auto_relock(app.clone());
            }
        })
        .map_err(|error| error.to_string())?;

    app.global_shortcut()
        .on_shortcut(
            settings.toggle_quick_bar_shortcut.as_str(),
            |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("quick") {
                        let visible = window.is_visible().unwrap_or(true);
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                        }
                    }
                }
            },
        )
        .map_err(|error| error.to_string())
}

