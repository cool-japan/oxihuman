// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! F-curve graph editor view state.

/// State for the F-curve graph editor.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GraphEditorView {
    /// First visible frame.
    pub frame_start: f32,
    /// Last visible frame.
    pub frame_end: f32,
    /// Minimum visible value (y-axis bottom).
    pub value_min: f32,
    /// Maximum visible value (y-axis top).
    pub value_max: f32,
    /// Horizontal zoom factor.
    pub zoom_x: f32,
    /// Vertical zoom factor.
    pub zoom_y: f32,
}

/// Return the default graph editor view.
#[allow(dead_code)]
pub fn default_graph_editor_view() -> GraphEditorView {
    GraphEditorView {
        frame_start: 0.0,
        frame_end: 250.0,
        value_min: -1.0,
        value_max: 1.0,
        zoom_x: 1.0,
        zoom_y: 1.0,
    }
}

/// Convert a frame number to an x pixel within the given pixel width.
#[allow(dead_code)]
pub fn frame_to_screen_x(view: &GraphEditorView, frame: f32, width: f32) -> f32 {
    let range = (view.frame_end - view.frame_start).max(1.0);
    ((frame - view.frame_start) / range) * width * view.zoom_x
}

/// Convert a curve value to a y pixel within the given pixel height (y increases downward).
#[allow(dead_code)]
pub fn value_to_screen_y(view: &GraphEditorView, val: f32, height: f32) -> f32 {
    let range = (view.value_max - view.value_min).abs().max(1e-6);
    ((view.value_max - val) / range) * height * view.zoom_y
}

/// Convert an x pixel to a frame number.
#[allow(dead_code)]
pub fn screen_to_frame(view: &GraphEditorView, x: f32, width: f32) -> f32 {
    let range = (view.frame_end - view.frame_start).max(1.0);
    x / (width * view.zoom_x) * range + view.frame_start
}

/// Convert a y pixel to a curve value.
#[allow(dead_code)]
pub fn screen_to_value(view: &GraphEditorView, y: f32, height: f32) -> f32 {
    let range = (view.value_max - view.value_min).abs().max(1e-6);
    view.value_max - y / (height * view.zoom_y) * range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value_range() {
        let v = default_graph_editor_view();
        assert_eq!(v.value_min, -1.0);
        assert_eq!(v.value_max, 1.0);
    }

    #[test]
    fn frame_to_x_at_start_is_zero() {
        let v = default_graph_editor_view();
        let x = frame_to_screen_x(&v, v.frame_start, 500.0);
        assert!(x.abs() < 1e-5);
    }

    #[test]
    fn frame_to_x_at_end_is_width() {
        let v = default_graph_editor_view();
        let x = frame_to_screen_x(&v, v.frame_end, 500.0);
        assert!((x - 500.0).abs() < 1e-3);
    }

    #[test]
    fn frame_round_trip() {
        let v = default_graph_editor_view();
        let frame = 75.0;
        let x = frame_to_screen_x(&v, frame, 600.0);
        let back = screen_to_frame(&v, x, 600.0);
        assert!((back - frame).abs() < 1e-4);
    }

    #[test]
    fn value_round_trip() {
        let v = default_graph_editor_view();
        let val = 0.3;
        let y = value_to_screen_y(&v, val, 400.0);
        let back = screen_to_value(&v, y, 400.0);
        assert!((back - val).abs() < 1e-4);
    }

    #[test]
    fn value_max_maps_to_zero_y() {
        let v = default_graph_editor_view();
        let y = value_to_screen_y(&v, v.value_max, 400.0);
        assert!(y.abs() < 1e-5);
    }

    #[test]
    fn zoom_x_doubles_x() {
        let mut v = default_graph_editor_view();
        let x1 = frame_to_screen_x(&v, 125.0, 500.0);
        v.zoom_x = 2.0;
        let x2 = frame_to_screen_x(&v, 125.0, 500.0);
        assert!((x2 - x1 * 2.0).abs() < 1e-3);
    }

    #[test]
    fn zoom_y_doubles_y() {
        let mut v = default_graph_editor_view();
        let y1 = value_to_screen_y(&v, 0.0, 400.0);
        v.zoom_y = 2.0;
        let y2 = value_to_screen_y(&v, 0.0, 400.0);
        assert!((y2 - y1 * 2.0).abs() < 1e-3);
    }

    #[test]
    fn default_zoom_is_one() {
        let v = default_graph_editor_view();
        assert_eq!(v.zoom_x, 1.0);
        assert_eq!(v.zoom_y, 1.0);
    }
}
