// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nasal ala crease (alar groove) depth control.

use std::f32::consts::FRAC_PI_6;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NasalSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalAlaCreaseConfig {
    pub max_depth: f32,
}

impl Default for NasalAlaCreaseConfig {
    fn default() -> Self {
        Self { max_depth: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalAlaCreaseState {
    pub left: f32,
    pub right: f32,
    pub config: NasalAlaCreaseConfig,
}

#[allow(dead_code)]
pub fn default_nasal_ala_crease_config() -> NasalAlaCreaseConfig {
    NasalAlaCreaseConfig::default()
}

#[allow(dead_code)]
pub fn new_nasal_ala_crease_state(config: NasalAlaCreaseConfig) -> NasalAlaCreaseState {
    NasalAlaCreaseState {
        left: 0.0,
        right: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn nac_set(state: &mut NasalAlaCreaseState, side: NasalSide, v: f32) {
    let v = v.clamp(0.0, state.config.max_depth);
    match side {
        NasalSide::Left => state.left = v,
        NasalSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn nac_set_both(state: &mut NasalAlaCreaseState, v: f32) {
    let v = v.clamp(0.0, state.config.max_depth);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn nac_reset(state: &mut NasalAlaCreaseState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[allow(dead_code)]
pub fn nac_is_neutral(state: &NasalAlaCreaseState) -> bool {
    state.left.abs() < 1e-6 && state.right.abs() < 1e-6
}

#[allow(dead_code)]
pub fn nac_average(state: &NasalAlaCreaseState) -> f32 {
    (state.left + state.right) * 0.5
}

#[allow(dead_code)]
pub fn nac_symmetry(state: &NasalAlaCreaseState) -> f32 {
    (state.left - state.right).abs()
}

#[allow(dead_code)]
pub fn nac_crease_angle_rad(state: &NasalAlaCreaseState) -> f32 {
    nac_average(state) * FRAC_PI_6
}

#[allow(dead_code)]
pub fn nac_to_weights(state: &NasalAlaCreaseState) -> [f32; 2] {
    let m = state.config.max_depth;
    let n = |v: f32| if m > 1e-9 { v / m } else { 0.0 };
    [n(state.left), n(state.right)]
}

#[allow(dead_code)]
pub fn nac_blend(a: &NasalAlaCreaseState, b: &NasalAlaCreaseState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.left * (1.0 - t) + b.left * t,
        a.right * (1.0 - t) + b.right * t,
    ]
}

#[allow(dead_code)]
pub fn nac_to_json(state: &NasalAlaCreaseState) -> String {
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
        assert!(nac_is_neutral(&new_nasal_ala_crease_state(
            default_nasal_ala_crease_config()
        )));
    }
    #[test]
    fn set_clamps() {
        let mut s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set(&mut s, NasalSide::Left, 5.0);
        assert!((0.0..=1.0).contains(&s.left));
    }
    #[test]
    fn set_both_applies() {
        let mut s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set_both(&mut s, 0.6);
        assert!((s.left - 0.6).abs() < 1e-5 && (s.right - 0.6).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set_both(&mut s, 0.5);
        nac_reset(&mut s);
        assert!(nac_is_neutral(&s));
    }
    #[test]
    fn average_mid() {
        let mut s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set(&mut s, NasalSide::Left, 0.4);
        nac_set(&mut s, NasalSide::Right, 0.6);
        assert!((nac_average(&s) - 0.5).abs() < 1e-5);
    }
    #[test]
    fn symmetry_abs_diff() {
        let mut s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set(&mut s, NasalSide::Left, 0.1);
        nac_set(&mut s, NasalSide::Right, 0.7);
        assert!((nac_symmetry(&s) - 0.6).abs() < 1e-5);
    }
    #[test]
    fn crease_angle_nonneg() {
        let s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        assert!(nac_crease_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set(&mut s, NasalSide::Left, 1.0);
        assert!((nac_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_half() {
        let mut a = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        let b = new_nasal_ala_crease_state(default_nasal_ala_crease_config());
        nac_set(&mut a, NasalSide::Right, 0.6);
        let w = nac_blend(&a, &b, 0.5);
        assert!((w[1] - 0.3).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_right() {
        assert!(nac_to_json(&new_nasal_ala_crease_state(
            default_nasal_ala_crease_config()
        ))
        .contains("\"right\""));
    }
}
