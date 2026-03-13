// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Body symmetry control for left-right balance adjustments.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodySymmetryConfig {
    pub mirror_strength: f32,
    pub tolerance: f32,
    pub blend_falloff: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodySymmetryState {
    pub mirror_strength: f32,
    pub asymmetry_amount: f32,
    pub left_bias: f32,
    pub right_bias: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodySymmetryWeights {
    pub symmetric: f32,
    pub left_dominant: f32,
    pub right_dominant: f32,
    pub balanced: f32,
}

#[allow(dead_code)]
pub fn default_body_symmetry_config() -> BodySymmetryConfig {
    BodySymmetryConfig { mirror_strength: 1.0, tolerance: 0.01, blend_falloff: 0.5 }
}

#[allow(dead_code)]
pub fn new_body_symmetry_state() -> BodySymmetryState {
    BodySymmetryState { mirror_strength: 1.0, asymmetry_amount: 0.0, left_bias: 0.0, right_bias: 0.0 }
}

#[allow(dead_code)]
pub fn set_body_mirror_strength(state: &mut BodySymmetryState, value: f32) {
    state.mirror_strength = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_body_asymmetry_amount(state: &mut BodySymmetryState, value: f32) {
    state.asymmetry_amount = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_body_left_bias(state: &mut BodySymmetryState, value: f32) {
    state.left_bias = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_body_right_bias(state: &mut BodySymmetryState, value: f32) {
    state.right_bias = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_body_symmetry_weights(state: &BodySymmetryState, cfg: &BodySymmetryConfig) -> BodySymmetryWeights {
    let s = state.mirror_strength * cfg.mirror_strength;
    let symmetric = (s * (PI * 0.25).cos()).clamp(0.0, 1.0);
    let asym = state.asymmetry_amount * (1.0 - s);
    let left_dominant = (state.left_bias * asym).clamp(0.0, 1.0);
    let right_dominant = (state.right_bias * asym).clamp(0.0, 1.0);
    let balanced = (1.0 - (left_dominant - right_dominant).abs()).clamp(0.0, 1.0);
    BodySymmetryWeights { symmetric, left_dominant, right_dominant, balanced }
}

#[allow(dead_code)]
pub fn body_symmetry_to_json(state: &BodySymmetryState) -> String {
    format!(
        r#"{{"mirror_strength":{},"asymmetry_amount":{},"left_bias":{},"right_bias":{}}}"#,
        state.mirror_strength, state.asymmetry_amount, state.left_bias, state.right_bias
    )
}

#[allow(dead_code)]
pub fn blend_body_symmetry(a: &BodySymmetryState, b: &BodySymmetryState, t: f32) -> BodySymmetryState {
    let t = t.clamp(0.0, 1.0);
    BodySymmetryState {
        mirror_strength: a.mirror_strength + (b.mirror_strength - a.mirror_strength) * t,
        asymmetry_amount: a.asymmetry_amount + (b.asymmetry_amount - a.asymmetry_amount) * t,
        left_bias: a.left_bias + (b.left_bias - a.left_bias) * t,
        right_bias: a.right_bias + (b.right_bias - a.right_bias) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_body_symmetry_config();
        assert!((cfg.mirror_strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_body_symmetry_state();
        assert!((s.mirror_strength - 1.0).abs() < 1e-6);
        assert!(s.asymmetry_amount.abs() < 1e-6);
    }

    #[test]
    fn test_set_mirror_strength_clamp() {
        let mut s = new_body_symmetry_state();
        set_body_mirror_strength(&mut s, 1.5);
        assert!((s.mirror_strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_asymmetry() {
        let mut s = new_body_symmetry_state();
        set_body_asymmetry_amount(&mut s, 0.7);
        assert!((s.asymmetry_amount - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_bias() {
        let mut s = new_body_symmetry_state();
        set_body_left_bias(&mut s, 0.6);
        assert!((s.left_bias - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_right_bias() {
        let mut s = new_body_symmetry_state();
        set_body_right_bias(&mut s, 0.4);
        assert!((s.right_bias - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_body_symmetry_state();
        let cfg = default_body_symmetry_config();
        let w = compute_body_symmetry_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.symmetric));
        assert!((0.0..=1.0).contains(&w.balanced));
    }

    #[test]
    fn test_to_json() {
        let s = new_body_symmetry_state();
        let json = body_symmetry_to_json(&s);
        assert!(json.contains("mirror_strength"));
    }

    #[test]
    fn test_blend() {
        let a = new_body_symmetry_state();
        let mut b = new_body_symmetry_state();
        b.asymmetry_amount = 1.0;
        let mid = blend_body_symmetry(&a, &b, 0.5);
        assert!((mid.asymmetry_amount - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_body_symmetry_state();
        let r = blend_body_symmetry(&a, &a, 0.5);
        assert!((r.mirror_strength - a.mirror_strength).abs() < 1e-6);
    }
}
