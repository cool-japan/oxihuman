// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shoulder slope / height asymmetry control.

use std::f32::consts::FRAC_PI_4;

/// Side.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShoulderSide {
    Left,
    Right,
    Both,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShoulderSlopeState {
    /// Slope in normalised units (-1 = drooping, 0 = flat, 1 = raised).
    pub slope_left: f32,
    pub slope_right: f32,
    /// Shoulder height offset (-1..1).
    pub height_left: f32,
    pub height_right: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShoulderSlopeConfig {
    pub max_slope: f32,
    pub max_height: f32,
}

impl Default for ShoulderSlopeConfig {
    fn default() -> Self {
        Self {
            max_slope: 1.0,
            max_height: 1.0,
        }
    }
}
impl Default for ShoulderSlopeState {
    fn default() -> Self {
        Self {
            slope_left: 0.0,
            slope_right: 0.0,
            height_left: 0.0,
            height_right: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_shoulder_slope_state() -> ShoulderSlopeState {
    ShoulderSlopeState::default()
}

#[allow(dead_code)]
pub fn default_shoulder_slope_config() -> ShoulderSlopeConfig {
    ShoulderSlopeConfig::default()
}

#[allow(dead_code)]
pub fn ss_set_slope(
    state: &mut ShoulderSlopeState,
    cfg: &ShoulderSlopeConfig,
    side: ShoulderSide,
    v: f32,
) {
    let v = v.clamp(-cfg.max_slope, cfg.max_slope);
    match side {
        ShoulderSide::Left => state.slope_left = v,
        ShoulderSide::Right => state.slope_right = v,
        ShoulderSide::Both => {
            state.slope_left = v;
            state.slope_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn ss_set_height(
    state: &mut ShoulderSlopeState,
    cfg: &ShoulderSlopeConfig,
    side: ShoulderSide,
    v: f32,
) {
    let v = v.clamp(-cfg.max_height, cfg.max_height);
    match side {
        ShoulderSide::Left => state.height_left = v,
        ShoulderSide::Right => state.height_right = v,
        ShoulderSide::Both => {
            state.height_left = v;
            state.height_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn ss_reset(state: &mut ShoulderSlopeState) {
    *state = ShoulderSlopeState::default();
}

#[allow(dead_code)]
pub fn ss_is_neutral(state: &ShoulderSlopeState) -> bool {
    state.slope_left.abs() < 1e-4
        && state.slope_right.abs() < 1e-4
        && state.height_left.abs() < 1e-4
        && state.height_right.abs() < 1e-4
}

#[allow(dead_code)]
pub fn ss_blend(a: &ShoulderSlopeState, b: &ShoulderSlopeState, t: f32) -> ShoulderSlopeState {
    let t = t.clamp(0.0, 1.0);
    ShoulderSlopeState {
        slope_left: a.slope_left + (b.slope_left - a.slope_left) * t,
        slope_right: a.slope_right + (b.slope_right - a.slope_right) * t,
        height_left: a.height_left + (b.height_left - a.height_left) * t,
        height_right: a.height_right + (b.height_right - a.height_right) * t,
    }
}

#[allow(dead_code)]
pub fn ss_symmetry(state: &ShoulderSlopeState) -> f32 {
    1.0 - (state.slope_left - state.slope_right).abs().min(1.0)
}

/// Slope in radians (heuristic: slope * PI/4).
#[allow(dead_code)]
pub fn ss_slope_rad(state: &ShoulderSlopeState, side: ShoulderSide) -> f32 {
    let s = match side {
        ShoulderSide::Left => state.slope_left,
        ShoulderSide::Right => state.slope_right,
        ShoulderSide::Both => (state.slope_left + state.slope_right) * 0.5,
    };
    s * FRAC_PI_4
}

#[allow(dead_code)]
pub fn ss_to_weights(state: &ShoulderSlopeState) -> [f32; 4] {
    [
        state.slope_left,
        state.slope_right,
        state.height_left,
        state.height_right,
    ]
}

#[allow(dead_code)]
pub fn ss_to_json(state: &ShoulderSlopeState) -> String {
    format!(
        "{{\"slope_l\":{:.4},\"slope_r\":{:.4},\"h_l\":{:.4},\"h_r\":{:.4}}}",
        state.slope_left, state.slope_right, state.height_left, state.height_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(ss_is_neutral(&new_shoulder_slope_state()));
    }

    #[test]
    fn slope_clamps_max() {
        let mut s = new_shoulder_slope_state();
        let cfg = default_shoulder_slope_config();
        ss_set_slope(&mut s, &cfg, ShoulderSide::Left, 5.0);
        assert!(s.slope_left <= cfg.max_slope);
    }

    #[test]
    fn slope_clamps_min() {
        let mut s = new_shoulder_slope_state();
        let cfg = default_shoulder_slope_config();
        ss_set_slope(&mut s, &cfg, ShoulderSide::Right, -5.0);
        assert!(s.slope_right >= -cfg.max_slope);
    }

    #[test]
    fn both_sides() {
        let mut s = new_shoulder_slope_state();
        let cfg = default_shoulder_slope_config();
        ss_set_slope(&mut s, &cfg, ShoulderSide::Both, 0.5);
        assert!((s.slope_left - s.slope_right).abs() < 1e-5);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_shoulder_slope_state();
        let cfg = default_shoulder_slope_config();
        ss_set_slope(&mut s, &cfg, ShoulderSide::Both, 0.5);
        ss_reset(&mut s);
        assert!(ss_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_shoulder_slope_config();
        let mut a = new_shoulder_slope_state();
        let mut b = new_shoulder_slope_state();
        ss_set_slope(&mut a, &cfg, ShoulderSide::Left, 0.0);
        ss_set_slope(&mut b, &cfg, ShoulderSide::Left, 1.0);
        let m = ss_blend(&a, &b, 0.5);
        assert!((m.slope_left - 0.5).abs() < 1e-4);
    }

    #[test]
    fn symmetry_equal() {
        let s = new_shoulder_slope_state();
        assert!((ss_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn slope_rad_zero() {
        let s = new_shoulder_slope_state();
        assert!((ss_slope_rad(&s, ShoulderSide::Left)).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(ss_to_weights(&new_shoulder_slope_state()).len(), 4);
    }

    #[test]
    fn json_has_slope() {
        assert!(ss_to_json(&new_shoulder_slope_state()).contains("slope"));
    }
}
