// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cheek nasolabial fold depth control.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekNasalFoldConfig {
    pub max_depth: f32,
}

impl Default for CheekNasalFoldConfig {
    fn default() -> Self {
        Self { max_depth: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekNasalFoldState {
    pub left: f32,
    pub right: f32,
    pub config: CheekNasalFoldConfig,
}

#[allow(dead_code)]
pub fn default_cheek_nasal_fold_config() -> CheekNasalFoldConfig {
    CheekNasalFoldConfig::default()
}

#[allow(dead_code)]
pub fn new_cheek_nasal_fold_state(config: CheekNasalFoldConfig) -> CheekNasalFoldState {
    CheekNasalFoldState {
        left: 0.0,
        right: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn cnf_set(state: &mut CheekNasalFoldState, side: FoldSide, v: f32) {
    let v = v.clamp(0.0, state.config.max_depth);
    match side {
        FoldSide::Left => state.left = v,
        FoldSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn cnf_set_both(state: &mut CheekNasalFoldState, v: f32) {
    let v = v.clamp(0.0, state.config.max_depth);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn cnf_reset(state: &mut CheekNasalFoldState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[allow(dead_code)]
pub fn cnf_is_neutral(state: &CheekNasalFoldState) -> bool {
    state.left.abs() < 1e-6 && state.right.abs() < 1e-6
}

#[allow(dead_code)]
pub fn cnf_average(state: &CheekNasalFoldState) -> f32 {
    (state.left + state.right) * 0.5
}

#[allow(dead_code)]
pub fn cnf_asymmetry(state: &CheekNasalFoldState) -> f32 {
    (state.left - state.right).abs()
}

#[allow(dead_code)]
pub fn cnf_fold_angle_rad(state: &CheekNasalFoldState) -> f32 {
    cnf_average(state) * PI * 0.3
}

#[allow(dead_code)]
pub fn cnf_to_weights(state: &CheekNasalFoldState) -> [f32; 2] {
    let m = state.config.max_depth;
    let n = |v: f32| if m > 1e-9 { v / m } else { 0.0 };
    [n(state.left), n(state.right)]
}

#[allow(dead_code)]
pub fn cnf_blend(a: &CheekNasalFoldState, b: &CheekNasalFoldState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.left * (1.0 - t) + b.left * t,
        a.right * (1.0 - t) + b.right * t,
    ]
}

#[allow(dead_code)]
pub fn cnf_to_json(state: &CheekNasalFoldState) -> String {
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
        assert!(cnf_is_neutral(&new_cheek_nasal_fold_state(
            default_cheek_nasal_fold_config()
        )));
    }
    #[test]
    fn set_clamps() {
        let mut s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set(&mut s, FoldSide::Left, 9.0);
        assert!((0.0..=1.0).contains(&s.left));
    }
    #[test]
    fn set_both_applies() {
        let mut s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set_both(&mut s, 0.5);
        assert!((s.left - 0.5).abs() < 1e-5 && (s.right - 0.5).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set_both(&mut s, 0.4);
        cnf_reset(&mut s);
        assert!(cnf_is_neutral(&s));
    }
    #[test]
    fn average_mid() {
        let mut s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set(&mut s, FoldSide::Left, 0.2);
        cnf_set(&mut s, FoldSide::Right, 0.6);
        assert!((cnf_average(&s) - 0.4).abs() < 1e-5);
    }
    #[test]
    fn asymmetry_abs_diff() {
        let mut s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set(&mut s, FoldSide::Left, 0.1);
        cnf_set(&mut s, FoldSide::Right, 0.5);
        assert!((cnf_asymmetry(&s) - 0.4).abs() < 1e-5);
    }
    #[test]
    fn fold_angle_nonneg() {
        let s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        assert!(cnf_fold_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_one_at_max() {
        let mut s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set(&mut s, FoldSide::Right, 1.0);
        assert!((cnf_to_weights(&s)[1] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_midpoint() {
        let mut a = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        let b = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        cnf_set(&mut a, FoldSide::Left, 0.8);
        let w = cnf_blend(&a, &b, 0.5);
        assert!((w[0] - 0.4).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_right() {
        let s = new_cheek_nasal_fold_state(default_cheek_nasal_fold_config());
        assert!(cnf_to_json(&s).contains("\"right\""));
    }
}
