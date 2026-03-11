// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! UV editor viewport state.

/// State for the UV editor viewport.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct UvEditorView {
    /// Pan offset in UV space [u, v].
    pub pan: [f32; 2],
    /// Zoom factor (1.0 = 1:1).
    pub zoom: f32,
    /// Whether the grid is shown.
    pub show_grid: bool,
    /// Whether UV seams are shown.
    pub show_seams: bool,
    /// Currently active UV layer index.
    pub active_layer: u32,
}

/// Return the default UV editor view state.
#[allow(dead_code)]
pub fn default_uv_editor_view() -> UvEditorView {
    UvEditorView {
        pan: [0.0, 0.0],
        zoom: 1.0,
        show_grid: true,
        show_seams: true,
        active_layer: 0,
    }
}

/// Pan the UV editor by a delta in UV space.
#[allow(dead_code)]
pub fn uv_editor_pan(view: &mut UvEditorView, delta: [f32; 2]) {
    view.pan[0] += delta[0];
    view.pan[1] += delta[1];
}

/// Multiply the current zoom by a factor (clamped to 0.01..100.0).
#[allow(dead_code)]
pub fn uv_editor_zoom(view: &mut UvEditorView, factor: f32) {
    view.zoom = (view.zoom * factor).clamp(0.01, 100.0);
}

/// Convert a UV coordinate to a screen pixel position.
#[allow(dead_code)]
pub fn uv_to_screen(view: &UvEditorView, uv: [f32; 2], viewport: [f32; 2]) -> [f32; 2] {
    let x = (uv[0] - view.pan[0]) * view.zoom * viewport[0];
    let y = (uv[1] - view.pan[1]) * view.zoom * viewport[1];
    [x, y]
}

/// Convert a screen pixel position to a UV coordinate.
#[allow(dead_code)]
pub fn screen_to_uv(view: &UvEditorView, screen: [f32; 2], viewport: [f32; 2]) -> [f32; 2] {
    let u = screen[0] / (view.zoom * viewport[0]) + view.pan[0];
    let v = screen[1] / (view.zoom * viewport[1]) + view.pan[1];
    [u, v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zoom_is_one() {
        let v = default_uv_editor_view();
        assert_eq!(v.zoom, 1.0);
    }

    #[test]
    fn default_pan_is_zero() {
        let v = default_uv_editor_view();
        assert_eq!(v.pan, [0.0, 0.0]);
    }

    #[test]
    fn default_show_grid_is_true() {
        let v = default_uv_editor_view();
        assert!(v.show_grid);
    }

    #[test]
    fn pan_adds_delta() {
        let mut v = default_uv_editor_view();
        uv_editor_pan(&mut v, [0.1, 0.2]);
        assert!((v.pan[0] - 0.1).abs() < 1e-6);
        assert!((v.pan[1] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn zoom_multiplies() {
        let mut v = default_uv_editor_view();
        uv_editor_zoom(&mut v, 2.0);
        assert!((v.zoom - 2.0).abs() < 1e-6);
    }

    #[test]
    fn zoom_clamps_minimum() {
        let mut v = default_uv_editor_view();
        uv_editor_zoom(&mut v, 0.0);
        assert!(v.zoom >= 0.01);
    }

    #[test]
    fn zoom_clamps_maximum() {
        let mut v = default_uv_editor_view();
        uv_editor_zoom(&mut v, 1_000_000.0);
        assert!(v.zoom <= 100.0);
    }

    #[test]
    fn uv_to_screen_and_back_round_trips() {
        let v = default_uv_editor_view();
        let uv = [0.3, 0.7];
        let viewport = [512.0, 512.0];
        let screen = uv_to_screen(&v, uv, viewport);
        let uv2 = screen_to_uv(&v, screen, viewport);
        assert!((uv2[0] - uv[0]).abs() < 1e-5);
        assert!((uv2[1] - uv[1]).abs() < 1e-5);
    }

    #[test]
    fn uv_to_screen_at_origin() {
        let v = default_uv_editor_view();
        let screen = uv_to_screen(&v, [0.0, 0.0], [100.0, 100.0]);
        assert_eq!(screen, [0.0, 0.0]);
    }
}
