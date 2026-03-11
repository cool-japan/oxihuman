// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Action clip timeline view stub.

/// Clip playback state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipPlayState {
    Stopped,
    Playing,
    Paused,
    Looping,
}

/// An action clip entry.
#[derive(Debug, Clone)]
pub struct ActionClip {
    pub name: String,
    pub start_frame: f32,
    pub end_frame: f32,
    pub speed: f32,
    pub play_state: ClipPlayState,
}

/// Action clip view configuration.
#[derive(Debug, Clone)]
pub struct ActionClipView {
    pub clips: Vec<ActionClip>,
    pub current_frame: f32,
    pub pixels_per_frame: f32,
    pub enabled: bool,
}

impl ActionClipView {
    pub fn new() -> Self {
        ActionClipView {
            clips: Vec::new(),
            current_frame: 0.0,
            pixels_per_frame: 4.0,
            enabled: true,
        }
    }
}

impl Default for ActionClipView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new action clip view.
pub fn new_action_clip_view() -> ActionClipView {
    ActionClipView::new()
}

/// Add a clip.
pub fn acv_add_clip(view: &mut ActionClipView, clip: ActionClip) {
    view.clips.push(clip);
}

/// Clear all clips.
pub fn acv_clear(view: &mut ActionClipView) {
    view.clips.clear();
}

/// Set current playhead frame.
pub fn acv_set_current_frame(view: &mut ActionClipView, frame: f32) {
    view.current_frame = frame.max(0.0);
}

/// Set pixels per frame zoom.
pub fn acv_set_zoom(view: &mut ActionClipView, pixels_per_frame: f32) {
    view.pixels_per_frame = pixels_per_frame.max(0.5);
}

/// Enable or disable.
pub fn acv_set_enabled(view: &mut ActionClipView, enabled: bool) {
    view.enabled = enabled;
}

/// Return clip count.
pub fn acv_clip_count(view: &ActionClipView) -> usize {
    view.clips.len()
}

/// Serialize to JSON-like string.
pub fn acv_to_json(view: &ActionClipView) -> String {
    format!(
        r#"{{"clip_count":{},"current_frame":{},"pixels_per_frame":{},"enabled":{}}}"#,
        view.clips.len(),
        view.current_frame,
        view.pixels_per_frame,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clip() -> ActionClip {
        ActionClip {
            name: "walk".to_string(),
            start_frame: 0.0,
            end_frame: 30.0,
            speed: 1.0,
            play_state: ClipPlayState::Stopped,
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_action_clip_view();
        assert_eq!(acv_clip_count(&v), 0 /* no clips initially */);
    }

    #[test]
    fn test_add_clip() {
        let mut v = new_action_clip_view();
        acv_add_clip(&mut v, make_clip());
        assert_eq!(acv_clip_count(&v), 1 /* one clip after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_action_clip_view();
        acv_add_clip(&mut v, make_clip());
        acv_clear(&mut v);
        assert_eq!(acv_clip_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_current_frame_min() {
        let mut v = new_action_clip_view();
        acv_set_current_frame(&mut v, -5.0);
        assert!(v.current_frame.abs() < 1e-6 /* current_frame must not be negative */);
    }

    #[test]
    fn test_set_current_frame() {
        let mut v = new_action_clip_view();
        acv_set_current_frame(&mut v, 25.0);
        assert!((v.current_frame - 25.0).abs() < 1e-6 /* current_frame must be set */);
    }

    #[test]
    fn test_zoom_min() {
        let mut v = new_action_clip_view();
        acv_set_zoom(&mut v, 0.0);
        assert!((v.pixels_per_frame - 0.5).abs() < 1e-6 /* minimum zoom must be 0.5 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_action_clip_view();
        acv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_clip_count() {
        let v = new_action_clip_view();
        let j = acv_to_json(&v);
        assert!(j.contains("\"clip_count\"") /* JSON must have clip_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_action_clip_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
