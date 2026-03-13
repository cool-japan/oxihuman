// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brow lateral-shift control — independent left/right horizontal brow translation.

use std::f32::consts::PI;

/// Configuration for brow lateral control.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BrowLateralConfig {
    /// Maximum lateral offset in normalised units.
    pub max_offset: f32,
    /// Allow asymmetric shifts.
    pub allow_asymmetry: bool,
}

impl Default for BrowLateralConfig {
    fn default() -> Self {
        Self {
            max_offset: 0.04,
            allow_asymmetry: true,
        }
    }
}

/// Lateral brow shift for one side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BrowLateralState {
    /// Left brow horizontal shift, −1..=1.
    pub left: f32,
    /// Right brow horizontal shift, −1..=1.
    pub right: f32,
}

/// Create new state.
#[allow(dead_code)]
pub fn new_brow_lateral_state() -> BrowLateralState {
    BrowLateralState::default()
}

/// Default config.
#[allow(dead_code)]
pub fn default_brow_lateral_config() -> BrowLateralConfig {
    BrowLateralConfig::default()
}

/// Set shift for one side.
#[allow(dead_code)]
pub fn blat_set(state: &mut BrowLateralState, side: BrowSide, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    match side {
        BrowSide::Left => state.left = v,
        BrowSide::Right => state.right = v,
    }
}

/// Set both sides to the same value.
#[allow(dead_code)]
pub fn blat_set_both(state: &mut BrowLateralState, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    state.left = v;
    state.right = v;
}

/// Reset to neutral.
#[allow(dead_code)]
pub fn blat_reset(state: &mut BrowLateralState) {
    *state = BrowLateralState::default();
}

/// Whether effectively neutral.
#[allow(dead_code)]
pub fn blat_is_neutral(state: &BrowLateralState) -> bool {
    state.left.abs() < 1e-4 && state.right.abs() < 1e-4
}

/// Asymmetry magnitude.
#[allow(dead_code)]
pub fn blat_asymmetry(state: &BrowLateralState) -> f32 {
    (state.left - state.right).abs()
}

/// Average lateral value.
#[allow(dead_code)]
pub fn blat_average(state: &BrowLateralState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Convert to world-space offsets.
#[allow(dead_code)]
pub fn blat_to_weights(state: &BrowLateralState, cfg: &BrowLateralConfig) -> [f32; 2] {
    [state.left * cfg.max_offset, state.right * cfg.max_offset]
}

/// Effective rotation angle in radians (approximation).
#[allow(dead_code)]
pub fn blat_rotation_rad(state: &BrowLateralState) -> f32 {
    blat_average(state) * (PI / 12.0)
}

/// Blend two states.
#[allow(dead_code)]
pub fn blat_blend(a: &BrowLateralState, b: &BrowLateralState, t: f32) -> BrowLateralState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BrowLateralState {
        left: a.left * inv + b.left * t,
        right: a.right * inv + b.right * t,
    }
}

/// Serialise to JSON-like string.
#[allow(dead_code)]
pub fn blat_to_json(state: &BrowLateralState) -> String {
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
        assert!(blat_is_neutral(&new_brow_lateral_state()));
    }

    #[test]
    fn set_clamps_high() {
        let mut s = new_brow_lateral_state();
        blat_set(&mut s, BrowSide::Left, 10.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_low() {
        let mut s = new_brow_lateral_state();
        blat_set(&mut s, BrowSide::Right, -10.0);
        assert!((s.right + 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_equal() {
        let mut s = new_brow_lateral_state();
        blat_set_both(&mut s, 0.5);
        assert!((s.left - s.right).abs() < 1e-6);
    }

    #[test]
    fn reset_clears_values() {
        let mut s = new_brow_lateral_state();
        blat_set_both(&mut s, 0.7);
        blat_reset(&mut s);
        assert!(blat_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_when_equal() {
        let mut s = new_brow_lateral_state();
        blat_set_both(&mut s, 0.4);
        assert!(blat_asymmetry(&s) < 1e-6);
    }

    #[test]
    fn asymmetry_positive_when_different() {
        let mut s = new_brow_lateral_state();
        blat_set(&mut s, BrowSide::Left, 0.3);
        blat_set(&mut s, BrowSide::Right, -0.3);
        assert!(blat_asymmetry(&s) > 0.5);
    }

    #[test]
    fn weights_scale_by_max_offset() {
        let cfg = BrowLateralConfig {
            max_offset: 0.1,
            allow_asymmetry: true,
        };
        let mut s = new_brow_lateral_state();
        blat_set_both(&mut s, 1.0);
        let w = blat_to_weights(&s, &cfg);
        assert!((w[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_brow_lateral_state();
        let mut b = new_brow_lateral_state();
        blat_set_both(&mut a, 0.0);
        blat_set_both(&mut b, 1.0);
        let r = blat_blend(&a, &b, 0.5);
        assert!((r.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let s = new_brow_lateral_state();
        let j = blat_to_json(&s);
        assert!(j.contains("left") && j.contains("right"));
    }
}
