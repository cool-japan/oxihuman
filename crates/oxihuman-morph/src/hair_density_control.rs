// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Hair density morph controls for scalp hair coverage and thickness.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairDensityControlConfig {
    pub density: f32,
    pub thickness: f32,
    pub recession: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairDensityControlState {
    pub density: f32,
    pub thickness: f32,
    pub recession: f32,
    pub coverage: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairDensityControlWeights {
    pub dense: f32,
    pub thick: f32,
    pub receding: f32,
    pub full_coverage: f32,
    pub thin: f32,
}

#[allow(dead_code)]
pub fn default_hair_density_control_config() -> HairDensityControlConfig {
    HairDensityControlConfig { density: 0.5, thickness: 0.5, recession: 0.5 }
}

#[allow(dead_code)]
pub fn new_hair_density_control_state() -> HairDensityControlState {
    HairDensityControlState { density: 0.5, thickness: 0.5, recession: 0.5, coverage: 0.5 }
}

#[allow(dead_code)]
pub fn set_hair_density_control_density(state: &mut HairDensityControlState, value: f32) {
    state.density = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_hair_density_control_thickness(state: &mut HairDensityControlState, value: f32) {
    state.thickness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_hair_density_control_recession(state: &mut HairDensityControlState, value: f32) {
    state.recession = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_hair_density_control_coverage(state: &mut HairDensityControlState, value: f32) {
    state.coverage = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_hair_density_control_weights(state: &HairDensityControlState, cfg: &HairDensityControlConfig) -> HairDensityControlWeights {
    let dense = (state.density * cfg.density * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let thick = (state.thickness * cfg.thickness).clamp(0.0, 1.0);
    let receding = (state.recession * cfg.recession).clamp(0.0, 1.0);
    let full_coverage = state.coverage.clamp(0.0, 1.0);
    let thin = (1.0 - state.density).clamp(0.0, 1.0);
    HairDensityControlWeights { dense, thick, receding, full_coverage, thin }
}

#[allow(dead_code)]
pub fn hair_density_control_to_json(state: &HairDensityControlState) -> String {
    format!(
        r#"{{\"density\":{},\"thickness\":{},\"recession\":{},\"coverage\":{}}}"#,
        state.density, state.thickness, state.recession, state.coverage
    )
}

#[allow(dead_code)]
pub fn blend_hair_density_controls(a: &HairDensityControlState, b: &HairDensityControlState, t: f32) -> HairDensityControlState {
    let t = t.clamp(0.0, 1.0);
    HairDensityControlState {
        density: a.density + (b.density - a.density) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        recession: a.recession + (b.recession - a.recession) * t,
        coverage: a.coverage + (b.coverage - a.coverage) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_hair_density_control_config();
        assert!((0.0..=1.0).contains(&cfg.density));
    }

    #[test]
    fn test_new_state() {
        let s = new_hair_density_control_state();
        assert!((s.density - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_density_clamp() {
        let mut s = new_hair_density_control_state();
        set_hair_density_control_density(&mut s, 1.5);
        assert!((s.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut s = new_hair_density_control_state();
        set_hair_density_control_thickness(&mut s, 0.8);
        assert!((s.thickness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_recession() {
        let mut s = new_hair_density_control_state();
        set_hair_density_control_recession(&mut s, 0.7);
        assert!((s.recession - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_coverage() {
        let mut s = new_hair_density_control_state();
        set_hair_density_control_coverage(&mut s, 0.6);
        assert!((s.coverage - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_hair_density_control_state();
        let cfg = default_hair_density_control_config();
        let w = compute_hair_density_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.dense));
        assert!((0.0..=1.0).contains(&w.thick));
    }

    #[test]
    fn test_to_json() {
        let s = new_hair_density_control_state();
        let json = hair_density_control_to_json(&s);
        assert!(json.contains("density"));
        assert!(json.contains("coverage"));
    }

    #[test]
    fn test_blend() {
        let a = new_hair_density_control_state();
        let mut b = new_hair_density_control_state();
        b.density = 1.0;
        let mid = blend_hair_density_controls(&a, &b, 0.5);
        assert!((mid.density - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_hair_density_control_state();
        let r = blend_hair_density_controls(&a, &a, 0.5);
        assert!((r.density - a.density).abs() < 1e-6);
    }
}
