mod native;

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

pub use native::{
    apply_geometry, configure, create, schedule_auto_relock, set_lock_state, start_hover_monitor,
};

pub const AUTO_LOCK_AFTER: Duration = Duration::from_secs(5);

struct Inner {
    locked: bool,
    last_interaction: Option<Instant>,
}

pub struct InteractionLock {
    inner: Mutex<Inner>,
}

impl Default for InteractionLock {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                locked: true,
                last_interaction: None,
            }),
        }
    }
}

impl InteractionLock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_locked(&self) -> bool {
        self.inner.lock().expect("interaction lock poisoned").locked
    }

    pub fn lock(&self) {
        let mut inner = self.inner.lock().expect("interaction lock poisoned");
        inner.locked = true;
        inner.last_interaction = None;
    }

    pub fn unlock(&self, now: Instant) {
        let mut inner = self.inner.lock().expect("interaction lock poisoned");
        inner.locked = false;
        inner.last_interaction = Some(now);
    }

    pub fn touch(&self, now: Instant) {
        let mut inner = self.inner.lock().expect("interaction lock poisoned");
        if !inner.locked {
            inner.last_interaction = Some(now);
        }
    }

    pub fn should_relock(&self, now: Instant) -> bool {
        let inner = self.inner.lock().expect("interaction lock poisoned");
        !inner.locked
            && inner
                .last_interaction
                .is_some_and(|last| now.duration_since(last) >= AUTO_LOCK_AFTER)
    }
}

pub fn point_inside(x: i32, y: i32, left: i32, top: i32, right: i32, bottom: i32) -> bool {
    x >= left && x < right && y >= top && y < bottom
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuickBarZone {
    Drag,
    Previous,
    PlayPause,
    Next,
    Locked,
}

pub fn classify_quick_bar_zone(x: i32, width: i32) -> QuickBarZone {
    if width <= 0 {
        return QuickBarZone::Drag;
    }
    match x.max(0) * 100 / width {
        0..=54 => QuickBarZone::Drag,
        55..=64 => QuickBarZone::Previous,
        65..=77 => QuickBarZone::PlayPause,
        78..=90 => QuickBarZone::Next,
        _ => QuickBarZone::Locked,
    }
}

pub fn dragged_origin(
    start_window_x: i32,
    start_window_y: i32,
    start_cursor_x: i32,
    start_cursor_y: i32,
    cursor_x: i32,
    cursor_y: i32,
) -> (i32, i32) {
    (
        start_window_x + cursor_x - start_cursor_x,
        start_window_y + cursor_y - start_cursor_y,
    )
}

pub fn should_relock_after_hover(locked: bool, hovered: bool, hovered_since_unlock: bool) -> bool {
    !locked && hovered_since_unlock && !hovered
}

#[cfg(test)]
mod tests;
