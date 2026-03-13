// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Symmetry plane guide overlay view stub.

/// Symmetry plane orientation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymmetryAxis {
    X,
    Y,
    Z,
}

/// Symmetry plane view configuration.
#[derive(Debug, Clone)]
pub struct SymmetryPlaneView {
    pub axis: SymmetryAxis,
    pub grid_color: [f32; 4],
    pub line_width: f32,
    pub show_mirror_mesh: bool,
    pub enabled: bool,
}

impl SymmetryPlaneView {
    pub fn new() -> Self {
        SymmetryPlaneView {
            axis: SymmetryAxis::X,
            grid_color: [0.2, 0.6, 1.0, 0.5],
            line_width: 1.0,
            show_mirror_mesh: false,
            enabled: true,
        }
    }
}

impl Default for SymmetryPlaneView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new symmetry plane view.
pub fn new_symmetry_plane_view() -> SymmetryPlaneView {
    SymmetryPlaneView::new()
}

/// Set symmetry axis.
pub fn spv_set_axis(view: &mut SymmetryPlaneView, axis: SymmetryAxis) {
    view.axis = axis;
}

/// Set grid line color (RGBA).
pub fn spv_set_color(view: &mut SymmetryPlaneView, color: [f32; 4]) {
    view.grid_color = color;
}

/// Set line width in pixels.
pub fn spv_set_line_width(view: &mut SymmetryPlaneView, width: f32) {
    view.line_width = width.max(0.1);
}

/// Show or hide the mirrored mesh ghost.
pub fn spv_show_mirror_mesh(view: &mut SymmetryPlaneView, show: bool) {
    view.show_mirror_mesh = show;
}

/// Enable or disable.
pub fn spv_set_enabled(view: &mut SymmetryPlaneView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn spv_to_json(view: &SymmetryPlaneView) -> String {
    let ax = match view.axis {
        SymmetryAxis::X => "x",
        SymmetryAxis::Y => "y",
        SymmetryAxis::Z => "z",
    };
    format!(
        r#"{{"axis":"{}","line_width":{},"show_mirror_mesh":{},"enabled":{}}}"#,
        ax, view.line_width, view.show_mirror_mesh, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_axis() {
        let v = new_symmetry_plane_view();
        assert_eq!(v.axis, SymmetryAxis::X /* default axis must be X */);
    }

    #[test]
    fn test_set_axis() {
        let mut v = new_symmetry_plane_view();
        spv_set_axis(&mut v, SymmetryAxis::Z);
        assert_eq!(v.axis, SymmetryAxis::Z /* axis must be set */);
    }

    #[test]
    fn test_set_color() {
        let mut v = new_symmetry_plane_view();
        spv_set_color(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.grid_color[0] - 1.0).abs() < 1e-6 /* red channel must be 1.0 */);
    }

    #[test]
    fn test_line_width_min() {
        let mut v = new_symmetry_plane_view();
        spv_set_line_width(&mut v, -1.0);
        assert!((v.line_width - 0.1).abs() < 1e-6 /* line_width minimum must be 0.1 */);
    }

    #[test]
    fn test_show_mirror_mesh() {
        let mut v = new_symmetry_plane_view();
        spv_show_mirror_mesh(&mut v, true);
        assert!(v.show_mirror_mesh /* mirror mesh must be shown */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_symmetry_plane_view();
        spv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_axis() {
        let v = new_symmetry_plane_view();
        let j = spv_to_json(&v);
        assert!(j.contains("\"axis\"") /* JSON must have axis */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_symmetry_plane_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_mirror_default_false() {
        let v = new_symmetry_plane_view();
        assert!(!v.show_mirror_mesh /* mirror mesh must be hidden by default */);
    }
}
