// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Narrow-phase contact debug view — shows GJK/EPA results and penetration depth.

/// Narrow phase view configuration.
#[derive(Debug, Clone)]
pub struct NarrowPhaseView {
    pub enabled: bool,
    pub show_penetration_depth: bool,
    pub show_witness_points: bool,
    pub color: [f32; 4],
    pub depth_scale: f32,
}

impl NarrowPhaseView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_penetration_depth: true,
            show_witness_points: true,
            color: [1.0, 0.0, 0.5, 1.0],
            depth_scale: 10.0,
        }
    }
}

impl Default for NarrowPhaseView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new narrow phase view.
pub fn new_narrow_phase_view() -> NarrowPhaseView {
    NarrowPhaseView::new()
}

/// Enable or disable narrow-phase overlay.
pub fn npv_set_enabled(v: &mut NarrowPhaseView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle penetration depth display.
pub fn npv_set_show_depth(v: &mut NarrowPhaseView, show: bool) {
    v.show_penetration_depth = show;
}

/// Toggle witness point display.
pub fn npv_set_show_witnesses(v: &mut NarrowPhaseView, show: bool) {
    v.show_witness_points = show;
}

/// Set depth visualisation scale.
pub fn npv_set_depth_scale(v: &mut NarrowPhaseView, scale: f32) {
    v.depth_scale = scale.clamp(0.1, 1000.0);
}

/// Compute display arrow length for a given penetration depth.
pub fn npv_depth_display_length(v: &NarrowPhaseView, depth: f32) -> f32 {
    depth * v.depth_scale
}

/// Serialize to JSON-like string.
pub fn narrow_phase_view_to_json(v: &NarrowPhaseView) -> String {
    format!(
        r#"{{"enabled":{},"show_penetration_depth":{},"show_witness_points":{},"depth_scale":{:.4}}}"#,
        v.enabled, v.show_penetration_depth, v.show_witness_points, v.depth_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_narrow_phase_view();
        assert!(!v.enabled);
        assert!(v.show_penetration_depth);
    }

    #[test]
    fn test_enable() {
        let mut v = new_narrow_phase_view();
        npv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_show_depth_toggle() {
        let mut v = new_narrow_phase_view();
        npv_set_show_depth(&mut v, false);
        assert!(!v.show_penetration_depth);
    }

    #[test]
    fn test_show_witnesses_toggle() {
        let mut v = new_narrow_phase_view();
        npv_set_show_witnesses(&mut v, false);
        assert!(!v.show_witness_points);
    }

    #[test]
    fn test_depth_scale_clamp() {
        let mut v = new_narrow_phase_view();
        npv_set_depth_scale(&mut v, 0.0);
        assert_eq!(v.depth_scale, 0.1);
    }

    #[test]
    fn test_depth_display_length() {
        let v = new_narrow_phase_view();
        let l = npv_depth_display_length(&v, 0.01);
        assert!((l - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_depth_display_zero() {
        let v = new_narrow_phase_view();
        assert_eq!(npv_depth_display_length(&v, 0.0), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_narrow_phase_view();
        let s = narrow_phase_view_to_json(&v);
        assert!(s.contains("show_witness_points"));
    }

    #[test]
    fn test_clone() {
        let v = new_narrow_phase_view();
        let v2 = v.clone();
        assert!((v2.depth_scale - v.depth_scale).abs() < 1e-6);
    }
}
