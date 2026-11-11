use std::time::Duration;

use tauri::{Emitter, Manager, Runtime};

use crate::commands::AppState;

pub fn start<R: Runtime>(app: tauri::AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let media = app.state::<AppState>().media.clone();
            let snapshot = media.snapshot().await;
            let _ = app.emit("media://snapshot", snapshot);
        }
    });
}
