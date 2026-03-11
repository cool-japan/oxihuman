// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Node editor viewport state (shader/compositor/geometry nodes).

/// State for the node editor viewport.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEditorView {
    /// Pan offset in node space [x, y].
    pub pan: [f32; 2],
    /// Zoom factor (1.0 = 1:1).
    pub zoom: f32,
    /// Currently active node id, if any.
    pub active_node: Option<u32>,
}

/// Return the default node editor view.
#[allow(dead_code)]
pub fn default_node_editor_view() -> NodeEditorView {
    NodeEditorView {
        pan: [0.0, 0.0],
        zoom: 1.0,
        active_node: None,
    }
}

/// Pan the node editor by a delta.
#[allow(dead_code)]
pub fn node_editor_pan(view: &mut NodeEditorView, delta: [f32; 2]) {
    view.pan[0] += delta[0];
    view.pan[1] += delta[1];
}

/// Multiply the current zoom by a factor (clamped to 0.05..20.0).
#[allow(dead_code)]
pub fn node_editor_zoom(view: &mut NodeEditorView, factor: f32) {
    view.zoom = (view.zoom * factor).clamp(0.05, 20.0);
}

/// Convert node-space position to screen position.
#[allow(dead_code)]
pub fn node_to_screen(view: &NodeEditorView, node_pos: [f32; 2]) -> [f32; 2] {
    [
        (node_pos[0] - view.pan[0]) * view.zoom,
        (node_pos[1] - view.pan[1]) * view.zoom,
    ]
}

/// Convert screen position to node-space position.
#[allow(dead_code)]
pub fn screen_to_node(view: &NodeEditorView, screen: [f32; 2]) -> [f32; 2] {
    [
        screen[0] / view.zoom + view.pan[0],
        screen[1] / view.zoom + view.pan[1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_no_active_node() {
        let v = default_node_editor_view();
        assert_eq!(v.active_node, None);
    }

    #[test]
    fn default_zoom_one() {
        let v = default_node_editor_view();
        assert_eq!(v.zoom, 1.0);
    }

    #[test]
    fn pan_accumulates() {
        let mut v = default_node_editor_view();
        node_editor_pan(&mut v, [5.0, -3.0]);
        node_editor_pan(&mut v, [2.0, 1.0]);
        assert!((v.pan[0] - 7.0).abs() < 1e-6);
        assert!((v.pan[1] - (-2.0)).abs() < 1e-6);
    }

    #[test]
    fn zoom_clamps_min() {
        let mut v = default_node_editor_view();
        node_editor_zoom(&mut v, 0.0);
        assert!(v.zoom >= 0.05);
    }

    #[test]
    fn zoom_clamps_max() {
        let mut v = default_node_editor_view();
        node_editor_zoom(&mut v, 1_000.0);
        assert!(v.zoom <= 20.0);
    }

    #[test]
    fn node_to_screen_round_trip() {
        let v = NodeEditorView { pan: [10.0, 5.0], zoom: 2.0, active_node: None };
        let pos = [20.0, 15.0];
        let screen = node_to_screen(&v, pos);
        let back = screen_to_node(&v, screen);
        assert!((back[0] - pos[0]).abs() < 1e-5);
        assert!((back[1] - pos[1]).abs() < 1e-5);
    }

    #[test]
    fn node_at_pan_origin_maps_to_zero_screen() {
        let v = NodeEditorView { pan: [10.0, 5.0], zoom: 1.0, active_node: None };
        let screen = node_to_screen(&v, [10.0, 5.0]);
        assert!((screen[0]).abs() < 1e-6);
        assert!((screen[1]).abs() < 1e-6);
    }

    #[test]
    fn set_active_node() {
        let mut v = default_node_editor_view();
        v.active_node = Some(42);
        assert_eq!(v.active_node, Some(42));
    }

    #[test]
    fn clear_active_node() {
        let mut v = NodeEditorView { pan: [0.0, 0.0], zoom: 1.0, active_node: Some(7) };
        v.active_node = None;
        assert_eq!(v.active_node, None);
    }
}
