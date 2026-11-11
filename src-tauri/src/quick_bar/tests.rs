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
