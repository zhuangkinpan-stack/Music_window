use std::fs;

use tempfile::tempdir;

use super::*;
use crate::contracts::AppSettings;

#[test]
fn missing_settings_file_uses_defaults() {
    let directory = tempdir().unwrap();
    let service = SettingsService::load(directory.path().join("settings.json")).unwrap();

    assert_eq!(service.get(), AppSettings::default());
}

#[test]
fn partial_settings_file_keeps_default_values_for_missing_fields() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(&path, r#"{"theme":"light"}"#).unwrap();

    let service = SettingsService::load(path).unwrap();

    assert_eq!(service.get().theme, "light");
    assert_eq!(service.get().quick_bar_width_cm, 6.5);
}

#[test]
fn corrupt_settings_fall_back_to_defaults() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(&path, "{not-json").unwrap();

    let service = SettingsService::load(path).unwrap();

    assert_eq!(service.get(), AppSettings::default());
}

#[test]
fn persisted_settings_round_trip() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let service = SettingsService::load(path.clone()).unwrap();
    let mut settings = service.get();
    settings.quick_bar_width_cm = 7.0;
    settings.reduce_transparency = true;

    service.update(settings.clone()).unwrap();
    let reloaded = SettingsService::load(path).unwrap();

    assert_eq!(reloaded.get(), settings);
}

#[test]
fn rejects_invalid_or_duplicate_shortcuts() {
    assert!(validate_settings(&AppSettings::default()).is_ok());

    let mut settings = AppSettings::default();
    settings.unlock_shortcut = "not a shortcut".to_owned();

    assert!(validate_settings(&settings).is_err());

    settings = AppSettings::default();
    settings.toggle_quick_bar_shortcut = settings.unlock_shortcut.clone();

    assert!(validate_settings(&settings).is_err());
}

