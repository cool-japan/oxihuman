// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eye droop control — ptosis-style downward drift of upper eyelid margin.

use std::f32::consts::FRAC_PI_8;

/// Which eye.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EyeDroopConfig {
    pub max_droop_m: f32,
}

impl Default for EyeDroopConfig {
    fn default() -> Self {
        Self { max_droop_m: 0.006 }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EyeDroopState {
    pub left: f32,
    pub right: f32,
}

#[allow(dead_code)]
pub fn new_eye_droop_state() -> EyeDroopState {
    EyeDroopState::default()
}

#[allow(dead_code)]
pub fn default_eye_droop_config() -> EyeDroopConfig {
    EyeDroopConfig::default()
}

#[allow(dead_code)]
pub fn edr_set(state: &mut EyeDroopState, side: EyeSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        EyeSide::Left => state.left = v,
        EyeSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn edr_set_both(state: &mut EyeDroopState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn edr_reset(state: &mut EyeDroopState) {
    *state = EyeDroopState::default();
}

#[allow(dead_code)]
pub fn edr_is_neutral(state: &EyeDroopState) -> bool {
    state.left < 1e-4 && state.right < 1e-4
}

#[allow(dead_code)]
pub fn edr_asymmetry(state: &EyeDroopState) -> f32 {
    (state.left - state.right).abs()
}

/// Lid angle delta in radians.
#[allow(dead_code)]
pub fn edr_lid_angle_rad(state: &EyeDroopState, side: EyeSide) -> f32 {
    let v = match side {
        EyeSide::Left => state.left,
        EyeSide::Right => state.right,
    };
    v * FRAC_PI_8
}

#[allow(dead_code)]
pub fn edr_to_weights(state: &EyeDroopState, cfg: &EyeDroopConfig) -> [f32; 2] {
    [state.left * cfg.max_droop_m, state.right * cfg.max_droop_m]
}

#[allow(dead_code)]
pub fn edr_blend(a: &EyeDroopState, b: &EyeDroopState, t: f32) -> EyeDroopState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    EyeDroopState {
        left: a.left * inv + b.left * t,
        right: a.right * inv + b.right * t,
    }
}

#[allow(dead_code)]
pub fn edr_to_json(state: &EyeDroopState) -> String {
    format!(
        "{{\"left\":{:.4},\"right\":{:.4}}}",
        state.left, state.right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(edr_is_neutral(&new_eye_droop_state()));
    }

    #[test]
    fn set_clamps_high() {
        let mut s = new_eye_droop_state();
        edr_set(&mut s, EyeSide::Left, 10.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_low() {
        let mut s = new_eye_droop_state();
        edr_set(&mut s, EyeSide::Right, -1.0);
        assert!(s.right < 1e-6);
    }

    #[test]
    fn reset_works() {
        let mut s = new_eye_droop_state();
        edr_set_both(&mut s, 0.5);
        edr_reset(&mut s);
        assert!(edr_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_when_equal() {
        let mut s = new_eye_droop_state();
        edr_set_both(&mut s, 0.5);
        assert!(edr_asymmetry(&s) < 1e-6);
    }

    #[test]
    fn lid_angle_positive() {
        let mut s = new_eye_droop_state();
        edr_set(&mut s, EyeSide::Left, 1.0);
        assert!(edr_lid_angle_rad(&s, EyeSide::Left) > 0.0);
    }

    #[test]
    fn weights_correct() {
        let cfg = default_eye_droop_config();
        let mut s = new_eye_droop_state();
        edr_set_both(&mut s, 1.0);
        let w = edr_to_weights(&s, &cfg);
        assert!((w[0] - cfg.max_droop_m).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_eye_droop_state();
        edr_set_both(&mut b, 1.0);
        let r = edr_blend(&new_eye_droop_state(), &b, 0.5);
        assert!((r.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_left_right() {
        let j = edr_to_json(&new_eye_droop_state());
        assert!(j.contains("left") && j.contains("right"));
    }

    #[test]
    fn set_both_equal() {
        let mut s = new_eye_droop_state();
        edr_set_both(&mut s, 0.7);
        assert!((s.left - s.right).abs() < 1e-6);
    }
}
