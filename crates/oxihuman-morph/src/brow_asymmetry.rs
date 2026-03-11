// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Brow asymmetry — per-eyebrow independent asymmetric morph control.

use std::f32::consts::PI;

/// Which eyebrow side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BrowAsymmetryConfig {
    /// Max lift angle in radians.
    pub max_lift_rad: f32,
    /// Max furrow depth in normalised units.
    pub max_furrow: f32,
}

impl Default for BrowAsymmetryConfig {
    fn default() -> Self {
        Self {
            max_lift_rad: PI / 12.0,
            max_furrow: 0.8,
        }
    }
}

/// Per-eyebrow morph state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BrowAsymmetryState {
    /// Lift for each brow: -1 (down) .. 0 (neutral) .. 1 (raised).
    pub lift_left: f32,
    pub lift_right: f32,
    /// Furrow (inner squeeze) for each brow: 0 (neutral) .. 1 (fully furrowed).
    pub furrow_left: f32,
    pub furrow_right: f32,
}

#[allow(dead_code)]
pub fn new_brow_asymmetry_state() -> BrowAsymmetryState {
    BrowAsymmetryState::default()
}

#[allow(dead_code)]
pub fn default_brow_asymmetry_config() -> BrowAsymmetryConfig {
    BrowAsymmetryConfig::default()
}

#[allow(dead_code)]
pub fn ba_set_lift(state: &mut BrowAsymmetryState, side: BrowSide, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    match side {
        BrowSide::Left => state.lift_left = v,
        BrowSide::Right => state.lift_right = v,
    }
}

#[allow(dead_code)]
pub fn ba_set_furrow(state: &mut BrowAsymmetryState, side: BrowSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        BrowSide::Left => state.furrow_left = v,
        BrowSide::Right => state.furrow_right = v,
    }
}

#[allow(dead_code)]
pub fn ba_reset(state: &mut BrowAsymmetryState) {
    *state = BrowAsymmetryState::default();
}

#[allow(dead_code)]
pub fn ba_is_neutral(state: &BrowAsymmetryState) -> bool {
    state.lift_left.abs() < 1e-4
        && state.lift_right.abs() < 1e-4
        && state.furrow_left < 1e-4
        && state.furrow_right < 1e-4
}

/// Asymmetry index: 0 = symmetric, 1 = maximally asymmetric.
#[allow(dead_code)]
pub fn ba_lift_asymmetry(state: &BrowAsymmetryState) -> f32 {
    (state.lift_left - state.lift_right).abs() * 0.5
}

/// Lift angle in radians for a given brow.
#[allow(dead_code)]
pub fn ba_lift_angle_rad(
    state: &BrowAsymmetryState,
    side: BrowSide,
    cfg: &BrowAsymmetryConfig,
) -> f32 {
    let v = match side {
        BrowSide::Left => state.lift_left,
        BrowSide::Right => state.lift_right,
    };
    v * cfg.max_lift_rad
}

/// Furrow depth in normalised units for a given brow.
#[allow(dead_code)]
pub fn ba_furrow_depth(
    state: &BrowAsymmetryState,
    side: BrowSide,
    cfg: &BrowAsymmetryConfig,
) -> f32 {
    let v = match side {
        BrowSide::Left => state.furrow_left,
        BrowSide::Right => state.furrow_right,
    };
    v * cfg.max_furrow
}

/// Returns morph weights \[lift_l+, lift_l-, lift_r+, lift_r-, furrow_l, furrow_r\].
#[allow(dead_code)]
pub fn ba_to_weights(state: &BrowAsymmetryState) -> [f32; 6] {
    [
        state.lift_left.max(0.0),
        (-state.lift_left).max(0.0),
        state.lift_right.max(0.0),
        (-state.lift_right).max(0.0),
        state.furrow_left,
        state.furrow_right,
    ]
}

#[allow(dead_code)]
pub fn ba_blend(a: &BrowAsymmetryState, b: &BrowAsymmetryState, t: f32) -> BrowAsymmetryState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BrowAsymmetryState {
        lift_left: a.lift_left * inv + b.lift_left * t,
        lift_right: a.lift_right * inv + b.lift_right * t,
        furrow_left: a.furrow_left * inv + b.furrow_left * t,
        furrow_right: a.furrow_right * inv + b.furrow_right * t,
    }
}

#[allow(dead_code)]
pub fn ba_to_json(state: &BrowAsymmetryState) -> String {
    format!(
        "{{\"lift_l\":{:.4},\"lift_r\":{:.4},\"furrow_l\":{:.4},\"furrow_r\":{:.4}}}",
        state.lift_left, state.lift_right, state.furrow_left, state.furrow_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn default_neutral() {
        assert!(ba_is_neutral(&new_brow_asymmetry_state()));
    }

    #[test]
    fn lift_clamps_high() {
        let mut s = new_brow_asymmetry_state();
        ba_set_lift(&mut s, BrowSide::Left, 5.0);
        assert!((s.lift_left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lift_clamps_low() {
        let mut s = new_brow_asymmetry_state();
        ba_set_lift(&mut s, BrowSide::Right, -5.0);
        assert!((s.lift_right + 1.0).abs() < 1e-6);
    }

    #[test]
    fn furrow_clamps() {
        let mut s = new_brow_asymmetry_state();
        ba_set_furrow(&mut s, BrowSide::Left, 5.0);
        assert!((s.furrow_left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_brow_asymmetry_state();
        ba_set_lift(&mut s, BrowSide::Left, 0.9);
        ba_reset(&mut s);
        assert!(ba_is_neutral(&s));
    }

    #[test]
    fn lift_angle_proportional() {
        let cfg = default_brow_asymmetry_config();
        let mut s = new_brow_asymmetry_state();
        ba_set_lift(&mut s, BrowSide::Right, 1.0);
        let a = ba_lift_angle_rad(&s, BrowSide::Right, &cfg);
        assert!(a > 0.0 && a <= PI / 12.0 + 1e-5);
    }

    #[test]
    fn asymmetry_symmetric() {
        let mut s = new_brow_asymmetry_state();
        ba_set_lift(&mut s, BrowSide::Left, 0.5);
        ba_set_lift(&mut s, BrowSide::Right, 0.5);
        assert!(ba_lift_asymmetry(&s) < 1e-6);
    }

    #[test]
    fn weights_six_elements() {
        let w = ba_to_weights(&new_brow_asymmetry_state());
        assert_eq!(w.len(), 6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_brow_asymmetry_state();
        ba_set_lift(&mut b, BrowSide::Left, 1.0);
        let r = ba_blend(&new_brow_asymmetry_state(), &b, 0.5);
        assert!((r.lift_left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = ba_to_json(&new_brow_asymmetry_state());
        assert!(j.contains("lift_l") && j.contains("furrow_r"));
    }
}
