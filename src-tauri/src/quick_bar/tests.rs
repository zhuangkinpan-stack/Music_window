use std::time::{Duration, Instant};

use super::*;

#[test]
fn interaction_lock_starts_locked() {
    let lock = InteractionLock::new();

    assert!(lock.is_locked());
}

#[test]
fn interaction_lock_relocks_after_five_idle_seconds() {
    let start = Instant::now();
    let lock = InteractionLock::new();
    lock.unlock(start);

    assert!(!lock.should_relock(start + Duration::from_millis(4_999)));
    assert!(lock.should_relock(start + Duration::from_secs(5)));
}

#[test]
fn interaction_touch_extends_unlock_window() {
    let start = Instant::now();
    let lock = InteractionLock::new();
    lock.unlock(start);
    lock.touch(start + Duration::from_secs(4));

    assert!(!lock.should_relock(start + Duration::from_secs(8)));
    assert!(lock.should_relock(start + Duration::from_secs(9)));
}

#[test]
fn detects_cursor_inside_window_rectangle() {
    assert!(point_inside(12, 20, 0, 0, 100, 50));
    assert!(!point_inside(120, 20, 0, 0, 100, 50));
}

#[test]
fn separates_control_zone_from_drag_zone() {
    assert_eq!(classify_quick_bar_zone(60, 246), QuickBarZone::Drag);
    assert_eq!(classify_quick_bar_zone(145, 246), QuickBarZone::Previous);
    assert_eq!(classify_quick_bar_zone(177, 246), QuickBarZone::PlayPause);
    assert_eq!(classify_quick_bar_zone(211, 246), QuickBarZone::Next);
    assert_eq!(classify_quick_bar_zone(234, 246), QuickBarZone::Locked);
}

#[test]
fn derives_dragged_window_origin_from_cursor_delta() {
    assert_eq!(dragged_origin(16, 976, 196, 1000, 296, 850), (116, 826));
}

#[test]
fn relocks_after_hover_only_after_pointer_entered() {
    assert!(!should_relock_after_hover(false, false, false));
    assert!(!should_relock_after_hover(false, true, true));
    assert!(should_relock_after_hover(false, false, true));
    assert!(!should_relock_after_hover(true, false, true));
}
