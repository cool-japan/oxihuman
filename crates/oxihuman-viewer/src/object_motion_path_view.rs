// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Object motion path overlay view.

/// Motion path display settings.
#[derive(Debug, Clone)]
pub struct ObjectMotionPathView {
    pub enabled: bool,
    pub frame_start: i32,
    pub frame_end: i32,
    pub show_frame_numbers: bool,
    pub line_width: f32,
}

impl Default for ObjectMotionPathView {
    fn default() -> Self {
        Self {
            enabled: false,
            frame_start: 0,
            frame_end: 100,
            show_frame_numbers: false,
            line_width: 1.0,
        }
    }
}

/// Create a new ObjectMotionPathView.
pub fn new_object_motion_path_view() -> ObjectMotionPathView {
    ObjectMotionPathView::default()
}

/// Set the frame range for the path display.
pub fn motion_path_set_range(view: &mut ObjectMotionPathView, start: i32, end: i32) {
    view.frame_start = start;
    view.frame_end = end.max(start);
}

/// Enable or disable path overlay.
pub fn motion_path_set_enabled(view: &mut ObjectMotionPathView, enabled: bool) {
    view.enabled = enabled;
}

/// Toggle frame number labels.
pub fn motion_path_show_frames(view: &mut ObjectMotionPathView, show: bool) {
    view.show_frame_numbers = show;
}

/// Set line width for path rendering.
pub fn motion_path_set_line_width(view: &mut ObjectMotionPathView, w: f32) {
    view.line_width = w.clamp(0.5, 10.0);
}

/// Compute frame count in range.
pub fn motion_path_frame_count(view: &ObjectMotionPathView) -> u32 {
    (view.frame_end - view.frame_start).max(0) as u32
}

/// Serialize to JSON.
pub fn object_motion_path_to_json(view: &ObjectMotionPathView) -> String {
    format!(
        r#"{{"enabled":{},"start":{},"end":{},"show_frames":{},"line_width":{}}}"#,
        view.enabled, view.frame_start, view.frame_end, view.show_frame_numbers, view.line_width,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_object_motion_path_view();
        assert!(!v.enabled /* disabled by default */);
    }

    #[test]
    fn test_set_range() {
        let mut v = new_object_motion_path_view();
        motion_path_set_range(&mut v, 10, 50);
        assert_eq!(v.frame_start, 10 /* stored */);
        assert_eq!(v.frame_end, 50 /* stored */);
    }

    #[test]
    fn test_range_end_min() {
        let mut v = new_object_motion_path_view();
        motion_path_set_range(&mut v, 50, 10);
        assert_eq!(v.frame_end, 50 /* end >= start */);
    }

    #[test]
    fn test_enable() {
        let mut v = new_object_motion_path_view();
        motion_path_set_enabled(&mut v, true);
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn test_show_frames() {
        let mut v = new_object_motion_path_view();
        motion_path_show_frames(&mut v, true);
        assert!(v.show_frame_numbers /* enabled */);
    }

    #[test]
    fn test_line_width_clamp() {
        let mut v = new_object_motion_path_view();
        motion_path_set_line_width(&mut v, 20.0);
        assert!((v.line_width - 10.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_frame_count() {
        let mut v = new_object_motion_path_view();
        motion_path_set_range(&mut v, 0, 100);
        assert_eq!(motion_path_frame_count(&v), 100 /* 100 frames */);
    }

    #[test]
    fn test_frame_count_inverted() {
        let mut v = new_object_motion_path_view();
        v.frame_start = 100;
        v.frame_end = 50;
        assert_eq!(motion_path_frame_count(&v), 0 /* inverted = 0 */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_object_motion_path_view();
        let j = object_motion_path_to_json(&v);
        assert!(j.contains("enabled") /* key */);
    }
}
