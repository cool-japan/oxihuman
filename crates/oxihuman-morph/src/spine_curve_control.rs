// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Spine curve control — lordosis and kyphosis morph for the spinal column.

use std::f32::consts::PI;

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SpineCurveConfig {
    /// Maximum lordosis angle in radians (lumbar inward curve).
    pub max_lordosis_rad: f32,
    /// Maximum kyphosis angle in radians (thoracic outward curve).
    pub max_kyphosis_rad: f32,
    /// Maximum lateral scoliosis angle in radians.
    pub max_scoliosis_rad: f32,
}

impl Default for SpineCurveConfig {
    fn default() -> Self {
        Self {
            max_lordosis_rad: PI / 8.0,
            max_kyphosis_rad: PI / 10.0,
            max_scoliosis_rad: PI / 16.0,
        }
    }
}

/// Spinal curvature state, all values -1..=1.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SpineCurveState {
    /// Lumbar lordosis: positive = increased inward curve.
    pub lordosis: f32,
    /// Thoracic kyphosis: positive = increased outward hunch.
    pub kyphosis: f32,
    /// Lateral scoliosis: positive = rightward lean.
    pub scoliosis: f32,
}

#[allow(dead_code)]
pub fn new_spine_curve_state() -> SpineCurveState {
    SpineCurveState::default()
}

#[allow(dead_code)]
pub fn default_spine_curve_config() -> SpineCurveConfig {
    SpineCurveConfig::default()
}

#[allow(dead_code)]
pub fn scc_set_lordosis(state: &mut SpineCurveState, v: f32) {
    state.lordosis = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn scc_set_kyphosis(state: &mut SpineCurveState, v: f32) {
    state.kyphosis = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn scc_set_scoliosis(state: &mut SpineCurveState, v: f32) {
    state.scoliosis = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn scc_reset(state: &mut SpineCurveState) {
    *state = SpineCurveState::default();
}

#[allow(dead_code)]
pub fn scc_is_neutral(state: &SpineCurveState) -> bool {
    state.lordosis.abs() < 1e-4 && state.kyphosis.abs() < 1e-4 && state.scoliosis.abs() < 1e-4
}

/// Lordosis angle in radians.
#[allow(dead_code)]
pub fn scc_lordosis_angle_rad(state: &SpineCurveState, cfg: &SpineCurveConfig) -> f32 {
    state.lordosis * cfg.max_lordosis_rad
}

/// Kyphosis angle in radians.
#[allow(dead_code)]
pub fn scc_kyphosis_angle_rad(state: &SpineCurveState, cfg: &SpineCurveConfig) -> f32 {
    state.kyphosis * cfg.max_kyphosis_rad
}

/// Scoliosis angle in radians.
#[allow(dead_code)]
pub fn scc_scoliosis_angle_rad(state: &SpineCurveState, cfg: &SpineCurveConfig) -> f32 {
    state.scoliosis * cfg.max_scoliosis_rad
}

/// Returns total curvature magnitude (0..=3).
#[allow(dead_code)]
pub fn scc_total_curvature(state: &SpineCurveState) -> f32 {
    state.lordosis.abs() + state.kyphosis.abs() + state.scoliosis.abs()
}

/// Returns morph weights \[lordosis+, lordosis-, kyphosis+, kyphosis-, scoliosis+, scoliosis-\].
#[allow(dead_code)]
pub fn scc_to_weights(state: &SpineCurveState) -> [f32; 6] {
    [
        state.lordosis.max(0.0),
        (-state.lordosis).max(0.0),
        state.kyphosis.max(0.0),
        (-state.kyphosis).max(0.0),
        state.scoliosis.max(0.0),
        (-state.scoliosis).max(0.0),
    ]
}

#[allow(dead_code)]
pub fn scc_blend(a: &SpineCurveState, b: &SpineCurveState, t: f32) -> SpineCurveState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    SpineCurveState {
        lordosis: a.lordosis * inv + b.lordosis * t,
        kyphosis: a.kyphosis * inv + b.kyphosis * t,
        scoliosis: a.scoliosis * inv + b.scoliosis * t,
    }
}

#[allow(dead_code)]
pub fn scc_to_json(state: &SpineCurveState) -> String {
    format!(
        "{{\"lordosis\":{:.4},\"kyphosis\":{:.4},\"scoliosis\":{:.4}}}",
        state.lordosis, state.kyphosis, state.scoliosis
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn default_is_neutral() {
        assert!(scc_is_neutral(&new_spine_curve_state()));
    }

    #[test]
    fn lordosis_clamps() {
        let mut s = new_spine_curve_state();
        scc_set_lordosis(&mut s, 5.0);
        assert!((s.lordosis - 1.0).abs() < 1e-6);
    }

    #[test]
    fn kyphosis_clamps_negative() {
        let mut s = new_spine_curve_state();
        scc_set_kyphosis(&mut s, -5.0);
        assert!((s.kyphosis + 1.0).abs() < 1e-6);
    }

    #[test]
    fn scoliosis_clamps() {
        let mut s = new_spine_curve_state();
        scc_set_scoliosis(&mut s, 2.0);
        assert!((s.scoliosis - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_spine_curve_state();
        scc_set_lordosis(&mut s, 0.8);
        scc_reset(&mut s);
        assert!(scc_is_neutral(&s));
    }

    #[test]
    fn lordosis_angle_positive() {
        let cfg = default_spine_curve_config();
        let mut s = new_spine_curve_state();
        scc_set_lordosis(&mut s, 1.0);
        let a = scc_lordosis_angle_rad(&s, &cfg);
        assert!(a > 0.0);
        assert!(a <= PI / 8.0 + 1e-5);
    }

    #[test]
    fn total_curvature_sums() {
        let mut s = new_spine_curve_state();
        scc_set_lordosis(&mut s, 0.5);
        scc_set_kyphosis(&mut s, 0.5);
        assert!((scc_total_curvature(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_six_elements() {
        let w = scc_to_weights(&new_spine_curve_state());
        assert_eq!(w.len(), 6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_spine_curve_state();
        scc_set_lordosis(&mut b, 1.0);
        let r = scc_blend(&new_spine_curve_state(), &b, 0.5);
        assert!((r.lordosis - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = scc_to_json(&new_spine_curve_state());
        assert!(j.contains("lordosis") && j.contains("kyphosis"));
    }
}
