// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Keyboard shortcut registry.

#![allow(dead_code)]

/// A keyboard shortcut binding.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Shortcut {
    /// Key code.
    pub key: u32,
    /// Modifier bitmask (e.g. Ctrl=1, Shift=2, Alt=4).
    pub modifiers: u8,
    /// Action identifier triggered by this shortcut.
    pub action_id: u32,
    /// Human-readable description.
    pub description: String,
}

/// Registry of all keyboard shortcuts.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShortcutRegistry {
    /// Registered shortcuts.
    pub shortcuts: Vec<Shortcut>,
}

/// Create a new empty shortcut registry.
#[allow(dead_code)]
pub fn new_shortcut_registry() -> ShortcutRegistry {
    ShortcutRegistry { shortcuts: Vec::new() }
}

/// Register a shortcut in the registry.
#[allow(dead_code)]
pub fn register_shortcut(
    reg: &mut ShortcutRegistry,
    key: u32,
    mods: u8,
    action: u32,
    desc: &str,
) {
    reg.shortcuts.push(Shortcut {
        key,
        modifiers: mods,
        action_id: action,
        description: desc.to_string(),
    });
}

/// Find the action id for the given key + modifier combination.
/// Returns `None` if no matching shortcut is registered.
#[allow(dead_code)]
pub fn find_shortcut(reg: &ShortcutRegistry, key: u32, mods: u8) -> Option<u32> {
    reg.shortcuts
        .iter()
        .find(|s| s.key == key && s.modifiers == mods)
        .map(|s| s.action_id)
}

/// Return the number of registered shortcuts.
#[allow(dead_code)]
pub fn shortcut_count(reg: &ShortcutRegistry) -> usize {
    reg.shortcuts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let reg = new_shortcut_registry();
        assert_eq!(shortcut_count(&reg), 0);
    }

    #[test]
    fn test_register_and_count() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 83, 1, 10, "Save (Ctrl+S)");
        assert_eq!(shortcut_count(&reg), 1);
    }

    #[test]
    fn test_find_existing() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 90, 1, 20, "Undo (Ctrl+Z)");
        let action = find_shortcut(&reg, 90, 1);
        assert_eq!(action, Some(20));
    }

    #[test]
    fn test_find_not_found() {
        let reg = new_shortcut_registry();
        assert!(find_shortcut(&reg, 65, 0).is_none());
    }

    #[test]
    fn test_different_mods_no_match() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 65, 1, 5, "Ctrl+A");
        // same key, different modifier
        assert!(find_shortcut(&reg, 65, 2).is_none());
    }

    #[test]
    fn test_multiple_shortcuts() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 1, 0, 1, "A");
        register_shortcut(&mut reg, 2, 0, 2, "B");
        register_shortcut(&mut reg, 3, 0, 3, "C");
        assert_eq!(shortcut_count(&reg), 3);
    }

    #[test]
    fn test_find_second_shortcut() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 1, 0, 100, "First");
        register_shortcut(&mut reg, 2, 0, 200, "Second");
        assert_eq!(find_shortcut(&reg, 2, 0), Some(200));
    }

    #[test]
    fn test_shortcut_description() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 70, 0, 30, "Focus view");
        assert_eq!(reg.shortcuts[0].description, "Focus view");
    }

    #[test]
    fn test_no_mods_zero() {
        let mut reg = new_shortcut_registry();
        register_shortcut(&mut reg, 70, 0, 99, "F key");
        assert_eq!(find_shortcut(&reg, 70, 0), Some(99));
    }
}
