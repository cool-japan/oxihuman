// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Smoke/volume debug view — shows voxel density and temperature slices.

/// Smoke debug view configuration.
#[derive(Debug, Clone)]
pub struct SmokeDebugView {
    pub enabled: bool,
    pub slice_axis: u8,
    pub slice_position: f32,
    pub density_threshold: f32,
    pub color_scale: f32,
}

impl SmokeDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            slice_axis: 1, /* Y axis default */
            slice_position: 0.5,
            density_threshold: 0.01,
            color_scale: 1.0,
        }
    }
}

impl Default for SmokeDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new smoke debug view.
pub fn new_smoke_debug_view() -> SmokeDebugView {
    SmokeDebugView::new()
}

/// Enable or disable the smoke debug view.
pub fn sdv_set_enabled(v: &mut SmokeDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Set slice axis (0=X, 1=Y, 2=Z).
pub fn sdv_set_slice_axis(v: &mut SmokeDebugView, axis: u8) {
    v.slice_axis = axis.min(2);
}

/// Set normalized slice position along the chosen axis.
pub fn sdv_set_slice_position(v: &mut SmokeDebugView, pos: f32) {
    v.slice_position = pos.clamp(0.0, 1.0);
}

/// Set density display threshold (cells below this are hidden).
pub fn sdv_set_density_threshold(v: &mut SmokeDebugView, thresh: f32) {
    v.density_threshold = thresh.clamp(0.0, 1.0);
}

/// Set color intensity scale.
pub fn sdv_set_color_scale(v: &mut SmokeDebugView, scale: f32) {
    v.color_scale = scale.clamp(0.1, 10.0);
}

/// Serialize to JSON-like string.
pub fn smoke_debug_view_to_json(v: &SmokeDebugView) -> String {
    format!(
        r#"{{"enabled":{},"slice_axis":{},"slice_position":{:.4},"density_threshold":{:.4},"color_scale":{:.4}}}"#,
        v.enabled, v.slice_axis, v.slice_position, v.density_threshold, v.color_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_smoke_debug_view();
        assert!(!v.enabled);
        assert_eq!(v.slice_axis, 1);
    }

    #[test]
    fn test_enable() {
        let mut v = new_smoke_debug_view();
        sdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_slice_axis_clamp() {
        let mut v = new_smoke_debug_view();
        sdv_set_slice_axis(&mut v, 5);
        assert_eq!(v.slice_axis, 2);
    }

    #[test]
    fn test_slice_position_clamp() {
        let mut v = new_smoke_debug_view();
        sdv_set_slice_position(&mut v, 2.0);
        assert_eq!(v.slice_position, 1.0);
    }

    #[test]
    fn test_density_threshold_set() {
        let mut v = new_smoke_debug_view();
        sdv_set_density_threshold(&mut v, 0.05);
        assert!((v.density_threshold - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_color_scale_clamp_low() {
        let mut v = new_smoke_debug_view();
        sdv_set_color_scale(&mut v, 0.0);
        assert_eq!(v.color_scale, 0.1);
    }

    #[test]
    fn test_color_scale_set() {
        let mut v = new_smoke_debug_view();
        sdv_set_color_scale(&mut v, 2.5);
        assert!((v.color_scale - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_smoke_debug_view();
        let s = smoke_debug_view_to_json(&v);
        assert!(s.contains("density_threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_smoke_debug_view();
        let v2 = v.clone();
        assert_eq!(v2.slice_axis, v.slice_axis);
    }
}
