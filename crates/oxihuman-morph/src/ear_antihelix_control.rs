// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear antihelix ridge prominence control.

use std::f32::consts::FRAC_PI_3;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarAntihelixConfig {
    pub max_prominence: f32,
}

impl Default for EarAntihelixConfig {
    fn default() -> Self {
        Self {
            max_prominence: 1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarAntihelixState {
    pub left: f32,
    pub right: f32,
    pub config: EarAntihelixConfig,
}

#[allow(dead_code)]
pub fn default_ear_antihelix_config() -> EarAntihelixConfig {
    EarAntihelixConfig::default()
}

#[allow(dead_code)]
pub fn new_ear_antihelix_state(config: EarAntihelixConfig) -> EarAntihelixState {
    EarAntihelixState {
        left: 0.0,
        right: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn eah_set(state: &mut EarAntihelixState, side: EarSide, v: f32) {
    let v = v.clamp(0.0, state.config.max_prominence);
    match side {
        EarSide::Left => state.left = v,
        EarSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn eah_set_both(state: &mut EarAntihelixState, v: f32) {
    let v = v.clamp(0.0, state.config.max_prominence);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn eah_reset(state: &mut EarAntihelixState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[allow(dead_code)]
pub fn eah_is_neutral(state: &EarAntihelixState) -> bool {
    state.left.abs() < 1e-6 && state.right.abs() < 1e-6
}

#[allow(dead_code)]
pub fn eah_average(state: &EarAntihelixState) -> f32 {
    (state.left + state.right) * 0.5
}

#[allow(dead_code)]
pub fn eah_symmetry(state: &EarAntihelixState) -> f32 {
    (state.left - state.right).abs()
}

#[allow(dead_code)]
pub fn eah_ridge_angle_rad(state: &EarAntihelixState) -> f32 {
    eah_average(state) * FRAC_PI_3
}

#[allow(dead_code)]
pub fn eah_to_weights(state: &EarAntihelixState) -> [f32; 2] {
    let m = state.config.max_prominence;
    let n = |v: f32| if m > 1e-9 { v / m } else { 0.0 };
    [n(state.left), n(state.right)]
}

#[allow(dead_code)]
pub fn eah_blend(a: &EarAntihelixState, b: &EarAntihelixState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.left * (1.0 - t) + b.left * t,
        a.right * (1.0 - t) + b.right * t,
    ]
}

#[allow(dead_code)]
pub fn eah_to_json(state: &EarAntihelixState) -> String {
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
        assert!(eah_is_neutral(&new_ear_antihelix_state(
            default_ear_antihelix_config()
        )));
    }
    #[test]
    fn set_clamps_above_max() {
        let mut s = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set(&mut s, EarSide::Left, 5.0);
        assert!((0.0..=1.0).contains(&s.left));
    }
    #[test]
    fn set_both_mirrors() {
        let mut s = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set_both(&mut s, 0.6);
        assert!((s.left - 0.6).abs() < 1e-5 && (s.right - 0.6).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set_both(&mut s, 0.8);
        eah_reset(&mut s);
        assert!(eah_is_neutral(&s));
    }
    #[test]
    fn average_is_mid() {
        let mut s = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set(&mut s, EarSide::Left, 0.4);
        eah_set(&mut s, EarSide::Right, 0.6);
        assert!((eah_average(&s) - 0.5).abs() < 1e-5);
    }
    #[test]
    fn symmetry_abs_diff() {
        let mut s = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set(&mut s, EarSide::Left, 0.1);
        eah_set(&mut s, EarSide::Right, 0.9);
        assert!((eah_symmetry(&s) - 0.8).abs() < 1e-5);
    }
    #[test]
    fn ridge_angle_nonneg() {
        let s = new_ear_antihelix_state(default_ear_antihelix_config());
        assert!(eah_ridge_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set(&mut s, EarSide::Right, 1.0);
        assert!((eah_to_weights(&s)[1] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_half() {
        let mut a = new_ear_antihelix_state(default_ear_antihelix_config());
        let b = new_ear_antihelix_state(default_ear_antihelix_config());
        eah_set(&mut a, EarSide::Left, 0.8);
        let w = eah_blend(&a, &b, 0.5);
        assert!((w[0] - 0.4).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_right() {
        assert!(
            eah_to_json(&new_ear_antihelix_state(default_ear_antihelix_config()))
                .contains("\"right\"")
        );
    }
}
