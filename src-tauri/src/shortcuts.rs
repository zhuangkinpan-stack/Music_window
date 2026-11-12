use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{commands::AppState, contracts::AppSettings, quick_bar};

#[derive(Clone, Copy)]
pub(crate) enum ShortcutAction {
    Unlock,
    Toggle,
}

pub(crate) trait ShortcutBackend {
    fn unregister_all(&self) -> Result<(), String>;
    fn register(&self, shortcut: &str, action: ShortcutAction) -> Result<(), String>;
}

pub(crate) fn apply_shortcut_bindings(
    backend: &impl ShortcutBackend,
    previous: (&str, &str),
    next: (&str, &str),
) -> Result<(), String> {
    backend.unregister_all()?;
    let result = backend
        .register(next.0, ShortcutAction::Unlock)
        .map_err(|error| format!("解锁快捷键 {} 注册失败：{error}", next.0))
        .and_then(|_| {
            backend
                .register(next.1, ShortcutAction::Toggle)
                .map_err(|error| format!("显示/隐藏快捷键 {} 已被占用或无效：{error}", next.1))
        });
    if let Err(error) = result {
        backend.unregister_all().ok();
        let _ = backend.register(previous.0, ShortcutAction::Unlock);
        let _ = backend.register(previous.1, ShortcutAction::Toggle);
        return Err(error);
    }
    Ok(())
}

pub fn rebind<R: Runtime>(
    app: &AppHandle<R>,
    previous: &AppSettings,
    settings: &AppSettings,
) -> Result<(), String> {
    let backend = TauriShortcutBackend { app };
    apply_shortcut_bindings(
        &backend,
        (
            &previous.unlock_shortcut,
            &previous.toggle_quick_bar_shortcut,
        ),
        (
            &settings.unlock_shortcut,
            &settings.toggle_quick_bar_shortcut,
        ),
    )
}

struct TauriShortcutBackend<'a, R: Runtime> {
    app: &'a AppHandle<R>,
}

impl<R: Runtime> ShortcutBackend for TauriShortcutBackend<'_, R> {
    fn unregister_all(&self) -> Result<(), String> {
        self.app
            .global_shortcut()
            .unregister_all()
            .map_err(|error| error.to_string())
    }

    fn register(&self, shortcut: &str, action: ShortcutAction) -> Result<(), String> {
        self.app
            .global_shortcut()
            .on_shortcut(shortcut, move |app, _, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                match action {
                    ShortcutAction::Unlock => {
                        let state = app.state::<AppState>();
                        state.interaction.unlock(std::time::Instant::now());
                        let _ = quick_bar::set_lock_state(app, false);
                        quick_bar::schedule_auto_relock(app.clone());
                    }
                    ShortcutAction::Toggle => {
                        if let Some(window) = app.get_webview_window("quick") {
                            let visible = window.is_visible().unwrap_or(true);
                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                            }
                        }
                    }
                }
            })
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests;
