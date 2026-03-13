// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dope sheet animation view.

/// A keyframe marker in the dope sheet.
#[derive(Debug, Clone)]
pub struct DopeKey {
    pub track_id: u32,
    pub frame: i32,
    pub selected: bool,
}

/// State for the dope sheet view.
#[derive(Debug, Clone)]
pub struct DopeSheetView {
    pub keys: Vec<DopeKey>,
    pub current_frame: i32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub enabled: bool,
}

/// Create a new dope sheet view.
pub fn new_dope_sheet_view() -> DopeSheetView {
    DopeSheetView {
        keys: Vec::new(),
        current_frame: 0,
        start_frame: 0,
        end_frame: 250,
        enabled: true,
    }
}

/// Add a key.
pub fn dsv_add_key(v: &mut DopeSheetView, track_id: u32, frame: i32) {
    v.keys.push(DopeKey {
        track_id,
        frame,
        selected: false,
    });
}

/// Select all keys on a given frame.
pub fn dsv_select_frame(v: &mut DopeSheetView, frame: i32) {
    for k in &mut v.keys {
        k.selected = k.frame == frame;
    }
}

/// Count selected keys.
pub fn dsv_selected_count(v: &DopeSheetView) -> usize {
    v.keys.iter().filter(|k| k.selected).count()
}

/// Set current frame.
pub fn dsv_set_current_frame(v: &mut DopeSheetView, frame: i32) {
    v.current_frame = frame;
}

/// Serialise to JSON.
pub fn dsv_to_json(v: &DopeSheetView) -> String {
    format!(
        r#"{{"key_count":{},"current_frame":{},"enabled":{}}}"#,
        v.keys.len(),
        v.current_frame,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty_keys() {
        let v = new_dope_sheet_view();
        assert!(v.keys.is_empty() /* no keys */);
    }

    #[test]
    fn add_key() {
        let mut v = new_dope_sheet_view();
        dsv_add_key(&mut v, 1, 10);
        assert_eq!(v.keys.len(), 1 /* one key */);
    }

    #[test]
    fn select_frame_marks_correct_keys() {
        let mut v = new_dope_sheet_view();
        dsv_add_key(&mut v, 1, 10);
        dsv_add_key(&mut v, 2, 20);
        dsv_select_frame(&mut v, 10);
        assert_eq!(dsv_selected_count(&v), 1 /* one key selected */);
    }

    #[test]
    fn selected_count_zero_initially() {
        let mut v = new_dope_sheet_view();
        dsv_add_key(&mut v, 1, 5);
        assert_eq!(dsv_selected_count(&v), 0 /* none selected */);
    }

    #[test]
    fn set_current_frame() {
        let mut v = new_dope_sheet_view();
        dsv_set_current_frame(&mut v, 100);
        assert_eq!(v.current_frame, 100 /* frame set */);
    }

    #[test]
    fn json_has_key_count() {
        let v = new_dope_sheet_view();
        assert!(dsv_to_json(&v).contains("key_count") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_dope_sheet_view();
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn end_frame_default() {
        let v = new_dope_sheet_view();
        assert_eq!(v.end_frame, 250 /* default 250 */);
    }
}
