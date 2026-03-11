// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shoulder acromion process prominence control.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShoulderSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderAcromionConfig {
    pub max_prominence: f32,
}

impl Default for ShoulderAcromionConfig {
    fn default() -> Self {
        Self {
            max_prominence: 1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderAcromionState {
    pub left: f32,
    pub right: f32,
    pub config: ShoulderAcromionConfig,
}

#[allow(dead_code)]
pub fn default_shoulder_acromion_config() -> ShoulderAcromionConfig {
    ShoulderAcromionConfig::default()
}

#[allow(dead_code)]
pub fn new_shoulder_acromion_state(config: ShoulderAcromionConfig) -> ShoulderAcromionState {
    ShoulderAcromionState {
        left: 0.0,
        right: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn sac_set(state: &mut ShoulderAcromionState, side: ShoulderSide, v: f32) {
    let v = v.clamp(0.0, state.config.max_prominence);
    match side {
        ShoulderSide::Left => state.left = v,
        ShoulderSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn sac_set_both(state: &mut ShoulderAcromionState, v: f32) {
    let v = v.clamp(0.0, state.config.max_prominence);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn sac_reset(state: &mut ShoulderAcromionState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[allow(dead_code)]
pub fn sac_is_neutral(state: &ShoulderAcromionState) -> bool {
    state.left.abs() < 1e-6 && state.right.abs() < 1e-6
}

#[allow(dead_code)]
pub fn sac_average(state: &ShoulderAcromionState) -> f32 {
    (state.left + state.right) * 0.5
}

#[allow(dead_code)]
pub fn sac_asymmetry(state: &ShoulderAcromionState) -> f32 {
    (state.left - state.right).abs()
}

#[allow(dead_code)]
pub fn sac_prominence_angle_rad(state: &ShoulderAcromionState) -> f32 {
    sac_average(state) * FRAC_PI_4
}

#[allow(dead_code)]
pub fn sac_to_weights(state: &ShoulderAcromionState) -> [f32; 2] {
    let m = state.config.max_prominence;
    let n = |v: f32| if m > 1e-9 { v / m } else { 0.0 };
    [n(state.left), n(state.right)]
}

#[allow(dead_code)]
pub fn sac_blend(a: &ShoulderAcromionState, b: &ShoulderAcromionState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.left * (1.0 - t) + b.left * t,
        a.right * (1.0 - t) + b.right * t,
    ]
}

#[allow(dead_code)]
pub fn sac_to_json(state: &ShoulderAcromionState) -> String {
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
        assert!(sac_is_neutral(&new_shoulder_acromion_state(
            default_shoulder_acromion_config()
        )));
    }
    #[test]
    fn set_clamps() {
        let mut s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set(&mut s, ShoulderSide::Left, 5.0);
        assert!((0.0..=1.0).contains(&s.left));
    }
    #[test]
    fn set_both_applies() {
        let mut s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set_both(&mut s, 0.7);
        assert!((s.right - 0.7).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set_both(&mut s, 0.5);
        sac_reset(&mut s);
        assert!(sac_is_neutral(&s));
    }
    #[test]
    fn average_mid() {
        let mut s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set(&mut s, ShoulderSide::Left, 0.4);
        sac_set(&mut s, ShoulderSide::Right, 0.8);
        assert!((sac_average(&s) - 0.6).abs() < 1e-5);
    }
    #[test]
    fn asymmetry_abs_diff() {
        let mut s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set(&mut s, ShoulderSide::Left, 0.2);
        sac_set(&mut s, ShoulderSide::Right, 0.8);
        assert!((sac_asymmetry(&s) - 0.6).abs() < 1e-5);
    }
    #[test]
    fn angle_nonneg() {
        let s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        assert!(sac_prominence_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set(&mut s, ShoulderSide::Left, 1.0);
        assert!((sac_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_at_half() {
        let mut a = new_shoulder_acromion_state(default_shoulder_acromion_config());
        let b = new_shoulder_acromion_state(default_shoulder_acromion_config());
        sac_set(&mut a, ShoulderSide::Left, 0.6);
        let w = sac_blend(&a, &b, 0.5);
        assert!((w[0] - 0.3).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_left() {
        assert!(sac_to_json(&new_shoulder_acromion_state(
            default_shoulder_acromion_config()
        ))
        .contains("\"left\""));
    }
}
