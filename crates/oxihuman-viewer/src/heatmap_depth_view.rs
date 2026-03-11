// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct HeatmapDepthView {
    pub near: f32,
    pub far: f32,
    pub enabled: bool,
    pub invert: bool,
}

pub fn new_heatmap_depth_view() -> HeatmapDepthView {
    HeatmapDepthView {
        near: 0.1,
        far: 100.0,
        enabled: false,
        invert: false,
    }
}

pub fn hdv_set_range(v: &mut HeatmapDepthView, near: f32, far: f32) {
    v.near = near.max(0.001);
    v.far = far.max(v.near + 0.001);
}

pub fn hdv_enable(v: &mut HeatmapDepthView) {
    v.enabled = true;
}

pub fn hdv_depth_to_color(v: &HeatmapDepthView, depth: f32) -> [f32; 3] {
    let range = (v.far - v.near).max(1e-6);
    let mut t = ((depth - v.near) / range).clamp(0.0, 1.0);
    if v.invert {
        t = 1.0 - t;
    }
    /* heatmap: blue->cyan->green->yellow->red */
    let r = (t * 2.0 - 1.0).clamp(0.0, 1.0);
    let g = (1.0 - (t * 2.0 - 1.0).abs()).clamp(0.0, 1.0);
    let b = (1.0 - t * 2.0).clamp(0.0, 1.0);
    [r, g, b]
}

pub fn hdv_is_enabled(v: &HeatmapDepthView) -> bool {
    v.enabled
}

pub fn hdv_to_json(v: &HeatmapDepthView) -> String {
    format!(
        r#"{{"near":{:.4},"far":{:.4},"enabled":{},"invert":{}}}"#,
        v.near, v.far, v.enabled, v.invert
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default near/far sensible */
        let v = new_heatmap_depth_view();
        assert!(v.near < v.far);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_range() {
        /* range stored correctly */
        let mut v = new_heatmap_depth_view();
        hdv_set_range(&mut v, 1.0, 50.0);
        assert!((v.near - 1.0).abs() < 1e-6);
        assert!((v.far - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_heatmap_depth_view();
        hdv_enable(&mut v);
        assert!(hdv_is_enabled(&v));
    }

    #[test]
    fn test_depth_to_color_near() {
        /* depth at near => cool color */
        let v = new_heatmap_depth_view();
        let c = hdv_depth_to_color(&v, v.near);
        /* near end should have blue component */
        assert!(c[2] >= c[0]);
    }

    #[test]
    fn test_depth_to_color_far() {
        /* depth at far => warm color */
        let v = new_heatmap_depth_view();
        let c = hdv_depth_to_color(&v, v.far);
        assert!(c[0] >= c[2]);
    }

    #[test]
    fn test_depth_to_color_out_of_range() {
        /* depth beyond far clamped */
        let v = new_heatmap_depth_view();
        let c = hdv_depth_to_color(&v, v.far * 2.0);
        assert!(c[0] <= 1.0);
        assert!(c[0] >= 0.0);
    }

    #[test]
    fn test_to_json_has_near() {
        /* JSON contains near field */
        let v = new_heatmap_depth_view();
        assert!(hdv_to_json(&v).contains("near"));
    }

    #[test]
    fn test_to_json_has_invert() {
        /* JSON contains invert field */
        let v = new_heatmap_depth_view();
        assert!(hdv_to_json(&v).contains("invert"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_heatmap_depth_view();
        let v2 = v.clone();
        assert_eq!(v.enabled, v2.enabled);
    }
}
