#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shape key list UI state.
//!
//! Note: `ShapeKeyEntry_` uses a trailing underscore to avoid name conflicts
//! with other `ShapeKeyEntry` exports in the crate.

/// A single shape key entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeKeyEntry_ {
    pub name: String,
    pub value: f32,
    pub muted: bool,
}

/// The shape key list panel state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ShapeKeyView {
    pub keys: Vec<ShapeKeyEntry_>,
    pub active_key: Option<usize>,
}

/// Create a new empty `ShapeKeyView`.
#[allow(dead_code)]
pub fn new_shape_key_view() -> ShapeKeyView {
    ShapeKeyView::default()
}

/// Add a shape key with the given name (value defaults to 0, not muted).
#[allow(dead_code)]
pub fn add_shape_key_entry(view: &mut ShapeKeyView, name: &str) {
    view.keys.push(ShapeKeyEntry_ {
        name: name.to_string(),
        value: 0.0,
        muted: false,
    });
}

/// Set the value of the shape key at `idx`.
#[allow(dead_code)]
pub fn set_key_value(view: &mut ShapeKeyView, idx: usize, val: f32) {
    if let Some(key) = view.keys.get_mut(idx) {
        key.value = val.clamp(0.0, 1.0);
    }
}

/// Toggle the muted state of the shape key at `idx`.
#[allow(dead_code)]
pub fn toggle_mute(view: &mut ShapeKeyView, idx: usize) {
    if let Some(key) = view.keys.get_mut(idx) {
        key.muted = !key.muted;
    }
}

/// Return the number of shape keys.
#[allow(dead_code)]
pub fn key_count(view: &ShapeKeyView) -> usize {
    view.keys.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_view_is_empty() {
        let v = new_shape_key_view();
        assert_eq!(key_count(&v), 0);
    }

    #[test]
    fn add_shape_key_increments_count() {
        let mut v = new_shape_key_view();
        add_shape_key_entry(&mut v, "Smile");
        assert_eq!(key_count(&v), 1);
    }

    #[test]
    fn added_key_defaults() {
        let mut v = new_shape_key_view();
        add_shape_key_entry(&mut v, "Frown");
        assert!((v.keys[0].value).abs() < 1e-6);
        assert!(!v.keys[0].muted);
    }

    #[test]
    fn set_key_value_valid() {
        let mut v = new_shape_key_view();
        add_shape_key_entry(&mut v, "A");
        set_key_value(&mut v, 0, 0.7);
        assert!((v.keys[0].value - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_key_value_clamps() {
        let mut v = new_shape_key_view();
        add_shape_key_entry(&mut v, "A");
        set_key_value(&mut v, 0, 2.5);
        assert!((v.keys[0].value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_key_value_out_of_range_no_panic() {
        let mut v = new_shape_key_view();
        set_key_value(&mut v, 99, 0.5); // must not panic
    }

    #[test]
    fn toggle_mute_mutes() {
        let mut v = new_shape_key_view();
        add_shape_key_entry(&mut v, "B");
        toggle_mute(&mut v, 0);
        assert!(v.keys[0].muted);
    }

    #[test]
    fn toggle_mute_unmutes() {
        let mut v = new_shape_key_view();
        add_shape_key_entry(&mut v, "B");
        toggle_mute(&mut v, 0);
        toggle_mute(&mut v, 0);
        assert!(!v.keys[0].muted);
    }

    #[test]
    fn toggle_mute_out_of_range_no_panic() {
        let mut v = new_shape_key_view();
        toggle_mute(&mut v, 5); // must not panic
    }

    #[test]
    fn active_key_defaults_none() {
        let v = new_shape_key_view();
        assert!(v.active_key.is_none());
    }
}
