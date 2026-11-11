use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::RwLock,
};

use thiserror::Error;
use tauri_plugin_global_shortcut::Shortcut;

use crate::contracts::AppSettings;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("设置读写失败：{0}")]
    Io(String),
    #[error("设置格式无效：{0}")]
    Serialize(String),
    #[error("快捷键无效或重复：{0}")]
    InvalidShortcut(String),
}

pub struct SettingsService {
    path: PathBuf,
    settings: RwLock<AppSettings>,
}

impl SettingsService {
    pub fn load(path: PathBuf) -> Result<Self, SettingsError> {
        let settings = load_settings(&path);
        Ok(Self {
            path,
            settings: RwLock::new(settings),
        })
    }

    pub fn get(&self) -> AppSettings {
        self.settings
            .read()
            .expect("settings lock poisoned")
            .clone()
    }

    pub fn update(&self, settings: AppSettings) -> Result<(), SettingsError> {
        validate_settings(&settings)?;
        save_settings(&self.path, &settings)?;
        *self
            .settings
            .write()
            .expect("settings lock poisoned") = settings;
        Ok(())
    }
}

pub fn validate_settings(settings: &AppSettings) -> Result<(), SettingsError> {
    if !(5.0..=7.0).contains(&settings.quick_bar_width_cm) {
        return Err(SettingsError::InvalidShortcut(
            "便捷条宽度必须在 5–7 cm 之间".to_owned(),
        ));
    }
    for shortcut in [
        &settings.unlock_shortcut,
        &settings.toggle_quick_bar_shortcut,
    ] {
        if Shortcut::from_str(shortcut).is_err() {
            return Err(SettingsError::InvalidShortcut(shortcut.clone()));
        }
    }
    if settings
        .unlock_shortcut
        .eq_ignore_ascii_case(&settings.toggle_quick_bar_shortcut)
    {
        return Err(SettingsError::InvalidShortcut(
            "两个全局快捷键不能相同".to_owned(),
        ));
    }
    Ok(())
}

fn load_settings(path: &Path) -> AppSettings {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<AppSettings>(&bytes).ok())
        .filter(|settings| validate_settings(settings).is_ok())
        .unwrap_or_default()
}

fn save_settings(path: &Path, settings: &AppSettings) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| SettingsError::Io(error.to_string()))?;
    }
    let bytes = serde_json::to_vec_pretty(settings)
        .map_err(|error| SettingsError::Serialize(error.to_string()))?;
    fs::write(path, bytes).map_err(|error| SettingsError::Io(error.to_string()))
}

#[cfg(test)]
mod tests;

