// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cloth simulation properties panel view.

/// Cloth properties panel state.
#[derive(Debug, Clone)]
pub struct ClothPropsView {
    pub vertex_mass: f32,
    pub tension_stiffness: f32,
    pub compression_stiffness: f32,
    pub shear_stiffness: f32,
    pub bending_stiffness: f32,
    pub damping: f32,
    pub quality_steps: u32,
}

impl Default for ClothPropsView {
    fn default() -> Self {
        Self {
            vertex_mass: 0.3,
            tension_stiffness: 15.0,
            compression_stiffness: 15.0,
            shear_stiffness: 5.0,
            bending_stiffness: 0.5,
            damping: 0.01,
            quality_steps: 5,
        }
    }
}

/// Create a new ClothPropsView.
pub fn new_cloth_props_view() -> ClothPropsView {
    ClothPropsView::default()
}

/// Set vertex mass.
pub fn cloth_pv_set_mass(view: &mut ClothPropsView, v: f32) {
    view.vertex_mass = v.clamp(0.001, 100.0);
}

/// Set tension stiffness.
pub fn cloth_pv_set_tension(view: &mut ClothPropsView, v: f32) {
    view.tension_stiffness = v.clamp(0.0, 10000.0);
}

/// Set bending stiffness.
pub fn cloth_pv_set_bending(view: &mut ClothPropsView, v: f32) {
    view.bending_stiffness = v.clamp(0.0, 10000.0);
}

/// Set quality (substep) count.
pub fn cloth_pv_set_quality(view: &mut ClothPropsView, n: u32) {
    view.quality_steps = n.clamp(1, 100);
}

/// Set overall damping factor.
pub fn cloth_pv_set_damping(view: &mut ClothPropsView, v: f32) {
    view.damping = v.clamp(0.0, 1.0);
}

/// Serialize to JSON.
pub fn cloth_props_to_json(view: &ClothPropsView) -> String {
    format!(
        r#"{{"mass":{},"tension":{},"bending":{},"damping":{},"quality":{}}}"#,
        view.vertex_mass,
        view.tension_stiffness,
        view.bending_stiffness,
        view.damping,
        view.quality_steps,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_cloth_props_view();
        assert!((v.vertex_mass - 0.3).abs() < 1e-6 /* default mass */);
    }

    #[test]
    fn test_mass_clamp_low() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_mass(&mut v, 0.0);
        assert!(v.vertex_mass > 0.0 /* above zero */);
    }

    #[test]
    fn test_mass_clamp_high() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_mass(&mut v, 10000.0);
        assert!((v.vertex_mass - 100.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_tension() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_tension(&mut v, 100.0);
        assert!((v.tension_stiffness - 100.0).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_bending() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_bending(&mut v, 5.0);
        assert!((v.bending_stiffness - 5.0).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_quality_clamp() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_quality(&mut v, 500);
        assert_eq!(v.quality_steps, 100 /* max */);
    }

    #[test]
    fn test_quality_min() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_quality(&mut v, 0);
        assert_eq!(v.quality_steps, 1 /* min */);
    }

    #[test]
    fn test_damping() {
        let mut v = new_cloth_props_view();
        cloth_pv_set_damping(&mut v, 0.05);
        assert!((v.damping - 0.05).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_cloth_props_view();
        let j = cloth_props_to_json(&v);
        assert!(j.contains("tension") /* key */);
    }
}
