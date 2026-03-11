// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Friction cone visualization — renders the Coulomb friction cone at contact points.

/// Friction cone view configuration.
#[derive(Debug, Clone)]
pub struct FrictionConeView {
    pub enabled: bool,
    pub friction_coefficient: f32,
    pub cone_height: f32,
    pub color: [f32; 4],
    pub segments: u32,
}

impl FrictionConeView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            friction_coefficient: 0.5,
            cone_height: 0.05,
            color: [0.8, 0.4, 0.0, 0.6],
            segments: 16,
        }
    }
}

impl Default for FrictionConeView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new friction cone view.
pub fn new_friction_cone_view() -> FrictionConeView {
    FrictionConeView::new()
}

/// Enable or disable friction cone display.
pub fn fcv_set_enabled(v: &mut FrictionConeView, enabled: bool) {
    v.enabled = enabled;
}

/// Set friction coefficient (μ).
pub fn fcv_set_friction_coefficient(v: &mut FrictionConeView, mu: f32) {
    v.friction_coefficient = mu.clamp(0.0, 10.0);
}

/// Set cone height in scene units.
pub fn fcv_set_cone_height(v: &mut FrictionConeView, h: f32) {
    v.cone_height = h.clamp(0.001, 1.0);
}

/// Set cone colour.
pub fn fcv_set_color(v: &mut FrictionConeView, color: [f32; 4]) {
    v.color = color;
}

/// Compute cone base radius for the current friction coefficient and height.
pub fn fcv_cone_radius(v: &FrictionConeView) -> f32 {
    v.friction_coefficient * v.cone_height
}

/// Serialize to JSON-like string.
pub fn friction_cone_view_to_json(v: &FrictionConeView) -> String {
    format!(
        r#"{{"enabled":{},"friction_coefficient":{:.4},"cone_height":{:.4},"segments":{}}}"#,
        v.enabled, v.friction_coefficient, v.cone_height, v.segments
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_friction_cone_view();
        assert!(!v.enabled);
        assert!((v.friction_coefficient - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        let mut v = new_friction_cone_view();
        fcv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_friction_coefficient_clamp() {
        let mut v = new_friction_cone_view();
        fcv_set_friction_coefficient(&mut v, 20.0);
        assert_eq!(v.friction_coefficient, 10.0);
    }

    #[test]
    fn test_cone_height_clamp() {
        let mut v = new_friction_cone_view();
        fcv_set_cone_height(&mut v, 0.0);
        assert_eq!(v.cone_height, 0.001);
    }

    #[test]
    fn test_cone_radius() {
        let v = new_friction_cone_view();
        let r = fcv_cone_radius(&v);
        assert!(r > 0.0);
    }

    #[test]
    fn test_cone_radius_formula() {
        let mut v = new_friction_cone_view();
        fcv_set_friction_coefficient(&mut v, 1.0);
        fcv_set_cone_height(&mut v, 0.1);
        assert!((fcv_cone_radius(&v) - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_color_set() {
        let mut v = new_friction_cone_view();
        fcv_set_color(&mut v, [0.0, 1.0, 0.0, 1.0]);
        assert!((v.color[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_friction_cone_view();
        let s = friction_cone_view_to_json(&v);
        assert!(s.contains("friction_coefficient"));
    }

    #[test]
    fn test_clone() {
        let v = new_friction_cone_view();
        let v2 = v.clone();
        assert!((v2.friction_coefficient - v.friction_coefficient).abs() < 1e-6);
    }
}
