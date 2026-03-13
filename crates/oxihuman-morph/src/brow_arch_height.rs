// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brow arch height — controls the vertical peak height of each brow arch.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowArchHeightConfig {
    pub max_height: f32,
}

impl Default for BrowArchHeightConfig {
    fn default() -> Self {
        Self { max_height: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowArchHeightState {
    pub left: f32,
    pub right: f32,
    pub config: BrowArchHeightConfig,
}

#[allow(dead_code)]
pub fn default_brow_arch_height_config() -> BrowArchHeightConfig {
    BrowArchHeightConfig::default()
}

#[allow(dead_code)]
pub fn new_brow_arch_height_state(config: BrowArchHeightConfig) -> BrowArchHeightState {
    BrowArchHeightState {
        left: 0.0,
        right: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn bah_set(state: &mut BrowArchHeightState, side: BrowSide, v: f32) {
    let v = v.clamp(0.0, state.config.max_height);
    match side {
        BrowSide::Left => state.left = v,
        BrowSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn bah_set_both(state: &mut BrowArchHeightState, v: f32) {
    let v = v.clamp(0.0, state.config.max_height);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn bah_reset(state: &mut BrowArchHeightState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[allow(dead_code)]
pub fn bah_is_neutral(state: &BrowArchHeightState) -> bool {
    state.left.abs() < 1e-6 && state.right.abs() < 1e-6
}

#[allow(dead_code)]
pub fn bah_average(state: &BrowArchHeightState) -> f32 {
    (state.left + state.right) * 0.5
}

#[allow(dead_code)]
pub fn bah_asymmetry(state: &BrowArchHeightState) -> f32 {
    (state.left - state.right).abs()
}

#[allow(dead_code)]
pub fn bah_arch_angle_rad(state: &BrowArchHeightState, side: BrowSide) -> f32 {
    let h = match side {
        BrowSide::Left => state.left,
        BrowSide::Right => state.right,
    };
    (h * PI * 0.25).clamp(0.0, PI * 0.5)
}

#[allow(dead_code)]
pub fn bah_to_weights(state: &BrowArchHeightState) -> [f32; 2] {
    let max = state.config.max_height;
    let norm = |v: f32| if max > 1e-9 { v / max } else { 0.0 };
    [norm(state.left), norm(state.right)]
}

#[allow(dead_code)]
pub fn bah_blend(a: &BrowArchHeightState, b: &BrowArchHeightState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.left * (1.0 - t) + b.left * t,
        a.right * (1.0 - t) + b.right * t,
    ]
}

#[allow(dead_code)]
pub fn bah_to_json(state: &BrowArchHeightState) -> String {
    format!(
        "{{\"left\":{:.4},\"right\":{:.4}}}",
        state.left, state.right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        let s = new_brow_arch_height_state(default_brow_arch_height_config());
        assert!(bah_is_neutral(&s));
    }

    #[test]
    fn set_clamps_to_max() {
        let mut s = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set(&mut s, BrowSide::Left, 5.0);
        assert!((0.0..=1.0).contains(&s.left));
    }

    #[test]
    fn set_both_applies_to_both_sides() {
        let mut s = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set_both(&mut s, 0.7);
        assert!((s.left - 0.7).abs() < 1e-5);
        assert!((s.right - 0.7).abs() < 1e-5);
    }

    #[test]
    fn reset_zeroes_values() {
        let mut s = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set_both(&mut s, 0.5);
        bah_reset(&mut s);
        assert!(bah_is_neutral(&s));
    }

    #[test]
    fn average_midpoint() {
        let mut s = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set(&mut s, BrowSide::Left, 0.4);
        bah_set(&mut s, BrowSide::Right, 0.6);
        assert!((bah_average(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn asymmetry_is_abs_diff() {
        let mut s = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set(&mut s, BrowSide::Left, 0.3);
        bah_set(&mut s, BrowSide::Right, 0.7);
        assert!((bah_asymmetry(&s) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn arch_angle_nonneg() {
        let s = new_brow_arch_height_state(default_brow_arch_height_config());
        assert!(bah_arch_angle_rad(&s, BrowSide::Left) >= 0.0);
    }

    #[test]
    fn to_weights_max_gives_one() {
        let mut s = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set(&mut s, BrowSide::Left, 1.0);
        let w = bah_to_weights(&s);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn blend_at_zero_is_a() {
        let mut a = new_brow_arch_height_state(default_brow_arch_height_config());
        let b = new_brow_arch_height_state(default_brow_arch_height_config());
        bah_set(&mut a, BrowSide::Left, 0.6);
        let w = bah_blend(&a, &b, 0.0);
        assert!((w[0] - 0.6).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_left() {
        let s = new_brow_arch_height_state(default_brow_arch_height_config());
        assert!(bah_to_json(&s).contains("\"left\""));
    }
}
