// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Neck sternocleidomastoid (SCM) prominence control.

use std::f32::consts::FRAC_PI_4;

/// SCM side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NeckSternoConfig {
    pub max_prominence_m: f32,
}

impl Default for NeckSternoConfig {
    fn default() -> Self {
        Self {
            max_prominence_m: 0.009,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct NeckSternoState {
    pub left: f32,
    pub right: f32,
    /// Head rotation that drives SCM activation, −1..=1.
    pub head_rotation: f32,
}

#[allow(dead_code)]
pub fn new_neck_sterno_state() -> NeckSternoState {
    NeckSternoState::default()
}

#[allow(dead_code)]
pub fn default_neck_sterno_config() -> NeckSternoConfig {
    NeckSternoConfig::default()
}

#[allow(dead_code)]
pub fn nst_set(state: &mut NeckSternoState, side: ScmSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        ScmSide::Left => state.left = v,
        ScmSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn nst_set_both(state: &mut NeckSternoState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn nst_set_head_rotation(state: &mut NeckSternoState, v: f32) {
    state.head_rotation = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nst_reset(state: &mut NeckSternoState) {
    *state = NeckSternoState::default();
}

#[allow(dead_code)]
pub fn nst_is_neutral(state: &NeckSternoState) -> bool {
    state.left < 1e-4 && state.right < 1e-4 && state.head_rotation.abs() < 1e-4
}

#[allow(dead_code)]
pub fn nst_asymmetry(state: &NeckSternoState) -> f32 {
    (state.left - state.right).abs()
}

/// Angle of SCM pull in radians.
#[allow(dead_code)]
pub fn nst_pull_angle_rad(state: &NeckSternoState) -> f32 {
    ((state.left + state.right) * 0.5) * FRAC_PI_4
}

#[allow(dead_code)]
pub fn nst_to_weights(state: &NeckSternoState, cfg: &NeckSternoConfig) -> [f32; 2] {
    [
        state.left * cfg.max_prominence_m,
        state.right * cfg.max_prominence_m,
    ]
}

#[allow(dead_code)]
pub fn nst_blend(a: &NeckSternoState, b: &NeckSternoState, t: f32) -> NeckSternoState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    NeckSternoState {
        left: a.left * inv + b.left * t,
        right: a.right * inv + b.right * t,
        head_rotation: a.head_rotation * inv + b.head_rotation * t,
    }
}

#[allow(dead_code)]
pub fn nst_to_json(state: &NeckSternoState) -> String {
    format!(
        "{{\"left\":{:.4},\"right\":{:.4},\"head_rotation\":{:.4}}}",
        state.left, state.right, state.head_rotation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(nst_is_neutral(&new_neck_sterno_state()));
    }

    #[test]
    fn set_clamps_high() {
        let mut s = new_neck_sterno_state();
        nst_set(&mut s, ScmSide::Left, 5.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_low() {
        let mut s = new_neck_sterno_state();
        nst_set(&mut s, ScmSide::Right, -1.0);
        assert!(s.right < 1e-6);
    }

    #[test]
    fn head_rotation_clamps() {
        let mut s = new_neck_sterno_state();
        nst_set_head_rotation(&mut s, 2.0);
        assert!((s.head_rotation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_neck_sterno_state();
        nst_set_both(&mut s, 0.8);
        nst_reset(&mut s);
        assert!(nst_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_when_equal() {
        let mut s = new_neck_sterno_state();
        nst_set_both(&mut s, 0.5);
        assert!(nst_asymmetry(&s) < 1e-6);
    }

    #[test]
    fn pull_angle_positive() {
        let mut s = new_neck_sterno_state();
        nst_set_both(&mut s, 1.0);
        assert!(nst_pull_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_correct() {
        let cfg = default_neck_sterno_config();
        let mut s = new_neck_sterno_state();
        nst_set_both(&mut s, 1.0);
        let w = nst_to_weights(&s, &cfg);
        assert!((w[0] - cfg.max_prominence_m).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_neck_sterno_state();
        nst_set_both(&mut b, 1.0);
        let r = nst_blend(&new_neck_sterno_state(), &b, 0.5);
        assert!((r.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = nst_to_json(&new_neck_sterno_state());
        assert!(j.contains("left") && j.contains("head_rotation"));
    }
}
