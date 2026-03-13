// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Shoulder roll (internal/external rotation) morph control.

use std::f32::consts::FRAC_PI_4;

/// Shoulder side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ShoulderRollConfig {
    pub max_roll_rad: f32,
}

impl Default for ShoulderRollConfig {
    fn default() -> Self {
        Self {
            max_roll_rad: FRAC_PI_4,
        }
    }
}

/// State — roll per side, −1..=1 (negative = internal, positive = external).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ShoulderRollState {
    pub left: f32,
    pub right: f32,
}

#[allow(dead_code)]
pub fn new_shoulder_roll_state() -> ShoulderRollState {
    ShoulderRollState::default()
}

#[allow(dead_code)]
pub fn default_shoulder_roll_config() -> ShoulderRollConfig {
    ShoulderRollConfig::default()
}

#[allow(dead_code)]
pub fn shr_set(state: &mut ShoulderRollState, side: RollSide, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    match side {
        RollSide::Left => state.left = v,
        RollSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn shr_set_both(state: &mut ShoulderRollState, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn shr_reset(state: &mut ShoulderRollState) {
    *state = ShoulderRollState::default();
}

#[allow(dead_code)]
pub fn shr_is_neutral(state: &ShoulderRollState) -> bool {
    state.left.abs() < 1e-4 && state.right.abs() < 1e-4
}

#[allow(dead_code)]
pub fn shr_asymmetry(state: &ShoulderRollState) -> f32 {
    (state.left - state.right).abs()
}

/// Roll angle in radians for a side.
#[allow(dead_code)]
pub fn shr_angle_rad(state: &ShoulderRollState, side: RollSide, cfg: &ShoulderRollConfig) -> f32 {
    let v = match side {
        RollSide::Left => state.left,
        RollSide::Right => state.right,
    };
    v * cfg.max_roll_rad
}

#[allow(dead_code)]
pub fn shr_to_weights(state: &ShoulderRollState, cfg: &ShoulderRollConfig) -> [f32; 2] {
    [
        state.left * cfg.max_roll_rad,
        state.right * cfg.max_roll_rad,
    ]
}

#[allow(dead_code)]
pub fn shr_blend(a: &ShoulderRollState, b: &ShoulderRollState, t: f32) -> ShoulderRollState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ShoulderRollState {
        left: a.left * inv + b.left * t,
        right: a.right * inv + b.right * t,
    }
}

#[allow(dead_code)]
pub fn shr_to_json(state: &ShoulderRollState) -> String {
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
        assert!(shr_is_neutral(&new_shoulder_roll_state()));
    }

    #[test]
    fn set_clamps_high() {
        let mut s = new_shoulder_roll_state();
        shr_set(&mut s, RollSide::Left, 5.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_low() {
        let mut s = new_shoulder_roll_state();
        shr_set(&mut s, RollSide::Right, -5.0);
        assert!((s.right + 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_shoulder_roll_state();
        shr_set_both(&mut s, 0.7);
        shr_reset(&mut s);
        assert!(shr_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_equal() {
        let mut s = new_shoulder_roll_state();
        shr_set_both(&mut s, 0.5);
        assert!(shr_asymmetry(&s) < 1e-6);
    }

    #[test]
    fn angle_rad_sign() {
        let cfg = default_shoulder_roll_config();
        let mut s = new_shoulder_roll_state();
        shr_set(&mut s, RollSide::Left, 1.0);
        assert!(shr_angle_rad(&s, RollSide::Left, &cfg) > 0.0);
        shr_set(&mut s, RollSide::Left, -1.0);
        assert!(shr_angle_rad(&s, RollSide::Left, &cfg) < 0.0);
    }

    #[test]
    fn weights_correct() {
        let cfg = default_shoulder_roll_config();
        let mut s = new_shoulder_roll_state();
        shr_set_both(&mut s, 1.0);
        let w = shr_to_weights(&s, &cfg);
        assert!((w[0] - cfg.max_roll_rad).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_shoulder_roll_state();
        shr_set_both(&mut b, 1.0);
        let r = shr_blend(&new_shoulder_roll_state(), &b, 0.5);
        assert!((r.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_left_right() {
        let j = shr_to_json(&new_shoulder_roll_state());
        assert!(j.contains("left") && j.contains("right"));
    }

    #[test]
    fn set_both_equal() {
        let mut s = new_shoulder_roll_state();
        shr_set_both(&mut s, -0.5);
        assert!((s.left - s.right).abs() < 1e-6);
    }
}
