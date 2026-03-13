#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Key binding definitions and helpers.

/// Key modifier flags.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum KeyModifier {
    None,
    Ctrl,
    Shift,
    Alt,
    CtrlShift,
    CtrlAlt,
}

/// Action associated with a key binding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum KeyAction {
    Press,
    Release,
    Repeat,
}

/// A key binding combining a key code, modifier, and action.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KeyBinding {
    pub key_code: u32,
    pub modifier: KeyModifier,
    pub action: KeyAction,
    pub name: String,
}

/// Create a new key binding.
#[allow(dead_code)]
pub fn new_key_binding(key_code: u32, modifier: KeyModifier, action: KeyAction, name: &str) -> KeyBinding {
    KeyBinding { key_code, modifier, action, name: name.to_string() }
}

/// Check if two key bindings match (same code + modifier).
#[allow(dead_code)]
pub fn binding_matches(a: &KeyBinding, b: &KeyBinding) -> bool {
    a.key_code == b.key_code && a.modifier == b.modifier
}

/// Convert a key binding to a human-readable string.
#[allow(dead_code)]
pub fn binding_to_string(kb: &KeyBinding) -> String {
    let mod_str = match &kb.modifier {
        KeyModifier::None => "".to_string(),
        KeyModifier::Ctrl => "Ctrl+".to_string(),
        KeyModifier::Shift => "Shift+".to_string(),
        KeyModifier::Alt => "Alt+".to_string(),
        KeyModifier::CtrlShift => "Ctrl+Shift+".to_string(),
        KeyModifier::CtrlAlt => "Ctrl+Alt+".to_string(),
    };
    format!("{}{} ({})", mod_str, kb.key_code, kb.name)
}

/// Add a modifier to an existing binding (returns new binding).
#[allow(dead_code)]
pub fn add_modifier(kb: &KeyBinding, modifier: KeyModifier) -> KeyBinding {
    KeyBinding { modifier, ..kb.clone() }
}

/// Check if a binding has a specific modifier.
#[allow(dead_code)]
pub fn has_modifier(kb: &KeyBinding, modifier: &KeyModifier) -> bool {
    &kb.modifier == modifier
}

/// Get the key code from a binding.
#[allow(dead_code)]
pub fn binding_key_code(kb: &KeyBinding) -> u32 {
    kb.key_code
}

/// Return a set of default key bindings.
#[allow(dead_code)]
pub fn default_key_bindings() -> Vec<KeyBinding> {
    vec![
        new_key_binding(27, KeyModifier::None, KeyAction::Press, "escape"),
        new_key_binding(13, KeyModifier::None, KeyAction::Press, "enter"),
        new_key_binding(9, KeyModifier::None, KeyAction::Press, "tab"),
        new_key_binding(90, KeyModifier::Ctrl, KeyAction::Press, "undo"),
        new_key_binding(89, KeyModifier::Ctrl, KeyAction::Press, "redo"),
    ]
}

/// Count the number of bindings in a list.
#[allow(dead_code)]
pub fn key_binding_count(bindings: &[KeyBinding]) -> usize {
    bindings.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_key_binding() {
        let kb = new_key_binding(65, KeyModifier::Ctrl, KeyAction::Press, "select_all");
        assert_eq!(kb.key_code, 65);
        assert_eq!(kb.name, "select_all");
    }

    #[test]
    fn test_binding_matches() {
        let a = new_key_binding(65, KeyModifier::Ctrl, KeyAction::Press, "a");
        let b = new_key_binding(65, KeyModifier::Ctrl, KeyAction::Release, "b");
        assert!(binding_matches(&a, &b));
    }

    #[test]
    fn test_binding_no_match() {
        let a = new_key_binding(65, KeyModifier::Ctrl, KeyAction::Press, "a");
        let b = new_key_binding(65, KeyModifier::Shift, KeyAction::Press, "b");
        assert!(!binding_matches(&a, &b));
    }

    #[test]
    fn test_binding_to_string() {
        let kb = new_key_binding(65, KeyModifier::Ctrl, KeyAction::Press, "test");
        let s = binding_to_string(&kb);
        assert!(s.contains("Ctrl+"));
    }

    #[test]
    fn test_add_modifier() {
        let kb = new_key_binding(65, KeyModifier::None, KeyAction::Press, "a");
        let kb2 = add_modifier(&kb, KeyModifier::Shift);
        assert_eq!(kb2.modifier, KeyModifier::Shift);
    }

    #[test]
    fn test_has_modifier() {
        let kb = new_key_binding(65, KeyModifier::Alt, KeyAction::Press, "a");
        assert!(has_modifier(&kb, &KeyModifier::Alt));
        assert!(!has_modifier(&kb, &KeyModifier::Ctrl));
    }

    #[test]
    fn test_binding_key_code() {
        let kb = new_key_binding(88, KeyModifier::None, KeyAction::Press, "x");
        assert_eq!(binding_key_code(&kb), 88);
    }

    #[test]
    fn test_default_key_bindings() {
        let bindings = default_key_bindings();
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_key_binding_count() {
        let bindings = default_key_bindings();
        assert_eq!(key_binding_count(&bindings), bindings.len());
    }
}
