// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Variable store: named f32 variables with change tracking.

use std::collections::HashMap;

/// A variable entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VarEntry {
    pub name: String,
    pub value: f32,
    pub default: f32,
    pub changed: bool,
}

/// Variable store.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct VarStore {
    vars: HashMap<String, VarEntry>,
}

/// Create a new `VarStore`.
#[allow(dead_code)]
pub fn new_var_store() -> VarStore {
    VarStore::default()
}

/// Declare a variable with a default value.
#[allow(dead_code)]
pub fn vs_declare(vs: &mut VarStore, name: &str, default: f32) {
    vs.vars.entry(name.to_string()).or_insert(VarEntry {
        name: name.to_string(),
        value: default,
        default,
        changed: false,
    });
}

/// Set a variable value.
#[allow(dead_code)]
pub fn vs_set(vs: &mut VarStore, name: &str, value: f32) {
    let entry = vs.vars.entry(name.to_string()).or_insert(VarEntry {
        name: name.to_string(),
        value,
        default: value,
        changed: false,
    });
    entry.changed = (entry.value - value).abs() > f32::EPSILON;
    entry.value = value;
}

/// Get a variable value.
#[allow(dead_code)]
pub fn vs_get(vs: &VarStore, name: &str) -> Option<f32> {
    vs.vars.get(name).map(|e| e.value)
}

/// Whether the variable was changed since last flush.
#[allow(dead_code)]
pub fn vs_is_changed(vs: &VarStore, name: &str) -> bool {
    vs.vars.get(name).is_some_and(|e| e.changed)
}

/// Reset a variable to its default.
#[allow(dead_code)]
pub fn vs_reset(vs: &mut VarStore, name: &str) {
    if let Some(e) = vs.vars.get_mut(name) {
        e.value = e.default;
        e.changed = false;
    }
}

/// Flush change flags.
#[allow(dead_code)]
pub fn vs_flush(vs: &mut VarStore) {
    for e in vs.vars.values_mut() {
        e.changed = false;
    }
}

/// Number of declared variables.
#[allow(dead_code)]
pub fn vs_len(vs: &VarStore) -> usize {
    vs.vars.len()
}

/// Names of all changed variables.
#[allow(dead_code)]
pub fn vs_changed_names(vs: &VarStore) -> Vec<String> {
    vs.vars
        .values()
        .filter(|e| e.changed)
        .map(|e| e.name.clone())
        .collect()
}

/// Remove a variable.
#[allow(dead_code)]
pub fn vs_remove(vs: &mut VarStore, name: &str) {
    vs.vars.remove(name);
}

/// Clear all variables.
#[allow(dead_code)]
pub fn vs_clear(vs: &mut VarStore) {
    vs.vars.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declare_and_get() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "mass", 1.0);
        assert!((vs_get(&vs, "mass").unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_and_get() {
        let mut vs = new_var_store();
        vs_set(&mut vs, "speed", 5.0);
        assert!((vs_get(&vs, "speed").unwrap() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_change_flag() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "x", 0.0);
        vs_set(&mut vs, "x", 1.0);
        assert!(vs_is_changed(&vs, "x"));
    }

    #[test]
    fn test_flush_clears_change() {
        let mut vs = new_var_store();
        vs_set(&mut vs, "y", 3.0);
        vs_flush(&mut vs);
        assert!(!vs_is_changed(&vs, "y"));
    }

    #[test]
    fn test_reset_to_default() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "z", 10.0);
        vs_set(&mut vs, "z", 99.0);
        vs_reset(&mut vs, "z");
        assert!((vs_get(&vs, "z").unwrap() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_changed_names() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "a", 0.0);
        vs_declare(&mut vs, "b", 0.0);
        vs_set(&mut vs, "a", 1.0);
        let changed = vs_changed_names(&vs);
        assert!(changed.contains(&"a".to_string()));
        assert!(!changed.contains(&"b".to_string()));
    }

    #[test]
    fn test_remove() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "k", 5.0);
        vs_remove(&mut vs, "k");
        assert!(vs_get(&vs, "k").is_none());
    }

    #[test]
    fn test_len() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "a", 1.0);
        vs_declare(&mut vs, "b", 2.0);
        assert_eq!(vs_len(&vs), 2);
    }

    #[test]
    fn test_clear() {
        let mut vs = new_var_store();
        vs_declare(&mut vs, "x", 1.0);
        vs_clear(&mut vs);
        assert_eq!(vs_len(&vs), 0);
    }
}
