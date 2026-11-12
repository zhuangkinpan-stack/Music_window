use std::{collections::BTreeSet, sync::Mutex};

use super::*;

#[derive(Default)]
struct FakeBackend {
    registered: Mutex<BTreeSet<String>>,
    fail_toggle: bool,
}

impl FakeBackend {
    fn with_failing_toggle() -> Self {
        Self {
            fail_toggle: true,
            ..Default::default()
        }
    }

    fn register(&self, shortcut: &str) -> Result<(), String> {
        if self.fail_toggle && shortcut == "Ctrl+Alt+M" {
            return Err("HotKey already registered".to_owned());
        }
        self.registered.lock().unwrap().insert(shortcut.to_owned());
        Ok(())
    }

    fn unregister_all(&self) {
        self.registered.lock().unwrap().clear();
    }
}

impl ShortcutBackend for FakeBackend {
    fn unregister_all(&self) -> Result<(), String> {
        FakeBackend::unregister_all(self);
        Ok(())
    }

    fn register(&self, shortcut: &str, _action: ShortcutAction) -> Result<(), String> {
        FakeBackend::register(self, shortcut)
    }
}

#[test]
fn failed_rebind_rolls_back_previous_shortcuts() {
    let backend = FakeBackend::with_failing_toggle();
    backend.register("Ctrl+Alt+L").unwrap();

    let error = apply_shortcut_bindings(
        &backend,
        ("Ctrl+Alt+L", "Ctrl+Alt+M"),
        ("Ctrl+Alt+B", "Ctrl+Alt+M"),
    )
    .unwrap_err();

    assert!(error.contains("Ctrl+Alt+M"));
    let registered = backend.registered.lock().unwrap();
    assert!(registered.contains("Ctrl+Alt+L"));
    assert!(!registered.contains("Ctrl+Alt+B"));
}
