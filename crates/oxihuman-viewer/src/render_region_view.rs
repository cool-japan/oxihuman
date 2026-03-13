// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Render region border overlay for viewport display.

/// Render region view configuration.
#[derive(Debug, Clone)]
pub struct RenderRegionView {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub enabled: bool,
    pub border_color: [f32; 4],
}

impl RenderRegionView {
    pub fn new() -> Self {
        Self {
            x_min: 0.0,
            y_min: 0.0,
            x_max: 1.0,
            y_max: 1.0,
            enabled: false,
            border_color: [1.0, 0.5, 0.0, 1.0],
        }
    }
}

impl Default for RenderRegionView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new render region view.
pub fn new_render_region_view() -> RenderRegionView {
    RenderRegionView::new()
}

/// Set region bounds in normalized [0, 1] viewport coordinates.
pub fn rrv_set_bounds(view: &mut RenderRegionView, x_min: f32, y_min: f32, x_max: f32, y_max: f32) {
    view.x_min = x_min.clamp(0.0, 1.0);
    view.y_min = y_min.clamp(0.0, 1.0);
    view.x_max = x_max.clamp(0.0, 1.0);
    view.y_max = y_max.clamp(0.0, 1.0);
}

/// Toggle visibility of the render region overlay.
pub fn rrv_set_enabled(view: &mut RenderRegionView, enabled: bool) {
    view.enabled = enabled;
}

/// Set border color as RGBA.
pub fn rrv_set_border_color(view: &mut RenderRegionView, r: f32, g: f32, b: f32, a: f32) {
    view.border_color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Compute region area as a fraction of total viewport area.
pub fn rrv_region_area(view: &RenderRegionView) -> f32 {
    let w = (view.x_max - view.x_min).max(0.0);
    let h = (view.y_max - view.y_min).max(0.0);
    w * h
}

/// Serialize to JSON-like string.
pub fn render_region_view_to_json(view: &RenderRegionView) -> String {
    format!(
        r#"{{"x_min":{:.4},"y_min":{:.4},"x_max":{:.4},"y_max":{:.4},"enabled":{}}}"#,
        view.x_min, view.y_min, view.x_max, view.y_max, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_render_region_view();
        assert_eq!(v.x_min, 0.0);
        assert_eq!(v.x_max, 1.0);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_bounds() {
        let mut v = new_render_region_view();
        rrv_set_bounds(&mut v, 0.1, 0.1, 0.9, 0.9);
        assert!((v.x_min - 0.1).abs() < 1e-6);
        assert!((v.y_max - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_bounds_clamp() {
        let mut v = new_render_region_view();
        rrv_set_bounds(&mut v, -1.0, -1.0, 2.0, 2.0);
        assert_eq!(v.x_min, 0.0);
        assert_eq!(v.x_max, 1.0);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_render_region_view();
        rrv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_region_area_full() {
        let v = new_render_region_view();
        assert!((rrv_region_area(&v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_area_half() {
        let mut v = new_render_region_view();
        rrv_set_bounds(&mut v, 0.0, 0.0, 0.5, 1.0);
        assert!((rrv_region_area(&v) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_border_color_set() {
        let mut v = new_render_region_view();
        rrv_set_border_color(&mut v, 1.0, 0.0, 0.0, 1.0);
        assert!((v.border_color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let v = new_render_region_view();
        let s = render_region_view_to_json(&v);
        assert!(s.contains("x_min"));
    }

    #[test]
    fn test_clone() {
        let v = new_render_region_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }
}
