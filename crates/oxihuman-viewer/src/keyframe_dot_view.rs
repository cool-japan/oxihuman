// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Keyframe dot overlay on mesh view stub.

/// Keyframe selection state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeDotState {
    Normal,
    Selected,
    Hovered,
    Locked,
}

/// A keyframe dot entry.
#[derive(Debug, Clone)]
pub struct KeyframeDot {
    pub frame: f32,
    pub position: [f32; 3],
    pub state: KeyframeDotState,
    pub radius: f32,
}

/// Keyframe dot overlay view.
#[derive(Debug, Clone)]
pub struct KeyframeDotView {
    pub dots: Vec<KeyframeDot>,
    pub dot_color: [f32; 4],
    pub selected_color: [f32; 4],
    pub show_frame_numbers: bool,
    pub enabled: bool,
}

impl KeyframeDotView {
    pub fn new() -> Self {
        KeyframeDotView {
            dots: Vec::new(),
            dot_color: [1.0, 0.8, 0.0, 1.0],
            selected_color: [1.0, 0.3, 0.0, 1.0],
            show_frame_numbers: false,
            enabled: true,
        }
    }
}

impl Default for KeyframeDotView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new keyframe dot view.
pub fn new_keyframe_dot_view() -> KeyframeDotView {
    KeyframeDotView::new()
}

/// Add a keyframe dot.
pub fn kdv_add_dot(view: &mut KeyframeDotView, dot: KeyframeDot) {
    view.dots.push(dot);
}

/// Clear all dots.
pub fn kdv_clear(view: &mut KeyframeDotView) {
    view.dots.clear();
}

/// Set dot color.
pub fn kdv_set_dot_color(view: &mut KeyframeDotView, color: [f32; 4]) {
    view.dot_color = color;
}

/// Toggle frame number labels.
pub fn kdv_show_frame_numbers(view: &mut KeyframeDotView, show: bool) {
    view.show_frame_numbers = show;
}

/// Enable or disable.
pub fn kdv_set_enabled(view: &mut KeyframeDotView, enabled: bool) {
    view.enabled = enabled;
}

/// Return dot count.
pub fn kdv_dot_count(view: &KeyframeDotView) -> usize {
    view.dots.len()
}

/// Serialize to JSON-like string.
pub fn kdv_to_json(view: &KeyframeDotView) -> String {
    format!(
        r#"{{"dot_count":{},"show_frame_numbers":{},"enabled":{}}}"#,
        view.dots.len(),
        view.show_frame_numbers,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dot(frame: f32) -> KeyframeDot {
        KeyframeDot {
            frame,
            position: [0.0, 1.0, 0.0],
            state: KeyframeDotState::Normal,
            radius: 4.0,
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_keyframe_dot_view();
        assert_eq!(kdv_dot_count(&v), 0 /* no dots initially */);
    }

    #[test]
    fn test_add_dot() {
        let mut v = new_keyframe_dot_view();
        kdv_add_dot(&mut v, make_dot(0.0));
        assert_eq!(kdv_dot_count(&v), 1 /* one dot after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_keyframe_dot_view();
        kdv_add_dot(&mut v, make_dot(1.0));
        kdv_clear(&mut v);
        assert_eq!(kdv_dot_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_set_dot_color() {
        let mut v = new_keyframe_dot_view();
        kdv_set_dot_color(&mut v, [0.0, 1.0, 0.0, 1.0]);
        assert!((v.dot_color[1] - 1.0).abs() < 1e-6 /* green channel must be 1.0 */);
    }

    #[test]
    fn test_show_frame_numbers() {
        let mut v = new_keyframe_dot_view();
        kdv_show_frame_numbers(&mut v, true);
        assert!(v.show_frame_numbers /* frame numbers must be shown */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_keyframe_dot_view();
        kdv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_dot_count() {
        let v = new_keyframe_dot_view();
        let j = kdv_to_json(&v);
        assert!(j.contains("\"dot_count\"") /* JSON must have dot_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_keyframe_dot_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_frame_numbers_default_false() {
        let v = new_keyframe_dot_view();
        assert!(!v.show_frame_numbers /* frame numbers must be hidden by default */);
    }
}
