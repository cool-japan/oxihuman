// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eye fissure (palpebral aperture) height control per eye.

use std::f32::consts::FRAC_PI_6;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeFissureConfig {
    pub max_opening: f32,
}

impl Default for EyeFissureConfig {
    fn default() -> Self {
        Self { max_opening: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeFissureState {
    pub left: f32,
    pub right: f32,
    pub config: EyeFissureConfig,
}

#[allow(dead_code)]
pub fn default_eye_fissure_config() -> EyeFissureConfig {
    EyeFissureConfig::default()
}

#[allow(dead_code)]
pub fn new_eye_fissure_state(config: EyeFissureConfig) -> EyeFissureState {
    EyeFissureState {
        left: 0.0,
        right: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn ef_set(state: &mut EyeFissureState, side: EyeSide, v: f32) {
    let v = v.clamp(0.0, state.config.max_opening);
    match side {
        EyeSide::Left => state.left = v,
        EyeSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn ef_set_both(state: &mut EyeFissureState, v: f32) {
    let v = v.clamp(0.0, state.config.max_opening);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn ef_reset(state: &mut EyeFissureState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[allow(dead_code)]
pub fn ef_is_neutral(state: &EyeFissureState) -> bool {
    state.left.abs() < 1e-6 && state.right.abs() < 1e-6
}

#[allow(dead_code)]
pub fn ef_average(state: &EyeFissureState) -> f32 {
    (state.left + state.right) * 0.5
}

#[allow(dead_code)]
pub fn ef_asymmetry(state: &EyeFissureState) -> f32 {
    (state.left - state.right).abs()
}

#[allow(dead_code)]
pub fn ef_opening_angle_rad(state: &EyeFissureState) -> f32 {
    ef_average(state) * FRAC_PI_6
}

#[allow(dead_code)]
pub fn ef_to_weights(state: &EyeFissureState) -> [f32; 2] {
    let m = state.config.max_opening;
    let n = |v: f32| if m > 1e-9 { v / m } else { 0.0 };
    [n(state.left), n(state.right)]
}

#[allow(dead_code)]
pub fn ef_blend(a: &EyeFissureState, b: &EyeFissureState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.left * (1.0 - t) + b.left * t,
        a.right * (1.0 - t) + b.right * t,
    ]
}

#[allow(dead_code)]
pub fn ef_to_json(state: &EyeFissureState) -> String {
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
        assert!(ef_is_neutral(&new_eye_fissure_state(
            default_eye_fissure_config()
        )));
    }
    #[test]
    fn set_clamps() {
        let mut s = new_eye_fissure_state(default_eye_fissure_config());
        ef_set(&mut s, EyeSide::Left, 10.0);
        assert!((0.0..=1.0).contains(&s.left));
    }
    #[test]
    fn set_both_mirrors() {
        let mut s = new_eye_fissure_state(default_eye_fissure_config());
        ef_set_both(&mut s, 0.5);
        assert!((s.right - 0.5).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_eye_fissure_state(default_eye_fissure_config());
        ef_set_both(&mut s, 0.9);
        ef_reset(&mut s);
        assert!(ef_is_neutral(&s));
    }
    #[test]
    fn average_mid() {
        let mut s = new_eye_fissure_state(default_eye_fissure_config());
        ef_set(&mut s, EyeSide::Left, 0.2);
        ef_set(&mut s, EyeSide::Right, 0.8);
        assert!((ef_average(&s) - 0.5).abs() < 1e-5);
    }
    #[test]
    fn asymmetry_abs() {
        let mut s = new_eye_fissure_state(default_eye_fissure_config());
        ef_set(&mut s, EyeSide::Left, 0.3);
        ef_set(&mut s, EyeSide::Right, 0.7);
        assert!((ef_asymmetry(&s) - 0.4).abs() < 1e-5);
    }
    #[test]
    fn angle_nonneg() {
        let s = new_eye_fissure_state(default_eye_fissure_config());
        assert!(ef_opening_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_at_max() {
        let mut s = new_eye_fissure_state(default_eye_fissure_config());
        ef_set(&mut s, EyeSide::Left, 1.0);
        assert!((ef_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_at_zero_is_a() {
        let mut a = new_eye_fissure_state(default_eye_fissure_config());
        let b = new_eye_fissure_state(default_eye_fissure_config());
        ef_set(&mut a, EyeSide::Left, 0.7);
        let w = ef_blend(&a, &b, 0.0);
        assert!((w[0] - 0.7).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_left() {
        assert!(
            ef_to_json(&new_eye_fissure_state(default_eye_fissure_config())).contains("\"left\"")
        );
    }
}
