use super::*;

#[test]
fn maps_tray_menu_ids_to_actions() {
    assert_eq!(TrayAction::from_id("open"), Some(TrayAction::Open));
    assert_eq!(
        TrayAction::from_id("play_pause"),
        Some(TrayAction::PlayPause)
    );
    assert_eq!(TrayAction::from_id("unknown"), None);
}

