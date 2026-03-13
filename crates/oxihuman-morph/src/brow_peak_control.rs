// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brow peak control — lateral peak / arch height of the eyebrow.

use std::f32::consts::FRAC_PI_4;

/// Which side of the face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowPeakSide {
    Left,
    Right,
    Both,
}

/// Configuration for the brow-peak control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowPeakConfig {
    /// Maximum raise amount (default 1.0).
    pub max_raise: f32,
    /// Reference angle in radians (used for approximate arch shape).
    pub ref_angle: f32,
}

impl Default for BrowPeakConfig {
    fn default() -> Self {
        BrowPeakConfig {
            max_raise: 1.0,
            ref_angle: FRAC_PI_4,
        }
    }
}

/// Runtime state for brow-peak control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowPeakState {
    left: f32,
    right: f32,
    /// Horizontal position of peak in `[0.0, 1.0]` (0 = inner, 1 = outer).
    peak_position: f32,
    config: BrowPeakConfig,
}

/// Return a default [`BrowPeakConfig`].
pub fn default_brow_peak_config() -> BrowPeakConfig {
    BrowPeakConfig::default()
}

/// Create a new neutral [`BrowPeakState`].
pub fn new_brow_peak_state(config: BrowPeakConfig) -> BrowPeakState {
    BrowPeakState {
        left: 0.0,
        right: 0.0,
        peak_position: 0.5,
        config,
    }
}

/// Set the peak raise on a specific side.
pub fn bp_set_peak(state: &mut BrowPeakState, side: BrowPeakSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        BrowPeakSide::Left => state.left = v,
        BrowPeakSide::Right => state.right = v,
        BrowPeakSide::Both => {
            state.left = v;
            state.right = v;
        }
    }
}

/// Set the horizontal peak position in `[0.0, 1.0]`.
pub fn bp_set_position(state: &mut BrowPeakState, pos: f32) {
    state.peak_position = pos.clamp(0.0, 1.0);
}

/// Reset to neutral.
pub fn bp_reset(state: &mut BrowPeakState) {
    state.left = 0.0;
    state.right = 0.0;
    state.peak_position = 0.5;
}

/// Return true if both sides are at zero.
pub fn bp_is_neutral(state: &BrowPeakState) -> bool {
    state.left < 1e-5 && state.right < 1e-5
}

/// Average peak raise across both sides.
pub fn bp_average(state: &BrowPeakState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Asymmetry factor: absolute difference between left and right.
pub fn bp_asymmetry(state: &BrowPeakState) -> f32 {
    (state.left - state.right).abs()
}

/// Compute the arch curve value at normalised horizontal position `x ∈ [0,1]`.
///
/// Returns 0 at edges, `peak_height` at `peak_position`.
pub fn bp_arch_at(state: &BrowPeakState, side: BrowPeakSide, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let height = match side {
        BrowPeakSide::Left => state.left,
        BrowPeakSide::Right => state.right,
        BrowPeakSide::Both => (state.left + state.right) * 0.5,
    };
    // Gaussian-like bell centred at peak_position
    let sigma = 0.3_f32;
    let diff = x - state.peak_position;
    height * (-0.5 * (diff / sigma) * (diff / sigma)).exp()
}

/// Blend between two states by `t ∈ [0.0, 1.0]`.
pub fn bp_blend(a: &BrowPeakState, b: &BrowPeakState, t: f32) -> BrowPeakState {
    let t = t.clamp(0.0, 1.0);
    BrowPeakState {
        left: a.left + (b.left - a.left) * t,
        right: a.right + (b.right - a.right) * t,
        peak_position: a.peak_position + (b.peak_position - a.peak_position) * t,
        config: a.config.clone(),
    }
}

/// Serialise to JSON-like string.
pub fn bp_to_json(state: &BrowPeakState) -> String {
    format!(
        r#"{{"left":{:.4},"right":{:.4},"peak_position":{:.4}}}"#,
        state.left, state.right, state.peak_position
    )
}

/// Compute morph weights as `[left_peak, right_peak, arch_curve_mid]`.
pub fn bp_to_weights(state: &BrowPeakState) -> [f32; 3] {
    [
        state.left * state.config.max_raise,
        state.right * state.config.max_raise,
        bp_arch_at(state, BrowPeakSide::Both, state.peak_position),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> BrowPeakState {
        new_brow_peak_state(default_brow_peak_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(bp_is_neutral(&make()));
    }

    #[test]
    fn set_left_peak() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Left, 0.7);
        assert!((s.left - 0.7).abs() < 1e-5);
    }

    #[test]
    fn set_both_sets_both() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Both, 0.5);
        assert!((s.left - 0.5).abs() < 1e-5);
        assert!((s.right - 0.5).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Both, 1.0);
        bp_reset(&mut s);
        assert!(bp_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_when_equal() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Both, 0.5);
        assert!(bp_asymmetry(&s) < 1e-5);
    }

    #[test]
    fn arch_at_peak_position_is_max() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Left, 1.0);
        let at_peak = bp_arch_at(&s, BrowPeakSide::Left, s.peak_position);
        let at_edge = bp_arch_at(&s, BrowPeakSide::Left, 0.0);
        assert!(at_peak > at_edge);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = make();
        let mut b = make();
        bp_set_peak(&mut a, BrowPeakSide::Left, 0.0);
        bp_set_peak(&mut b, BrowPeakSide::Left, 1.0);
        let m = bp_blend(&a, &b, 0.5);
        assert!((m.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn weights_in_range() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Both, 0.6);
        let w = bp_to_weights(&s);
        for v in w {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn json_contains_left_key() {
        assert!(bp_to_json(&make()).contains("left"));
    }

    #[test]
    fn clamp_high() {
        let mut s = make();
        bp_set_peak(&mut s, BrowPeakSide::Right, 99.0);
        assert!((s.right - 1.0).abs() < 1e-5);
    }
}
