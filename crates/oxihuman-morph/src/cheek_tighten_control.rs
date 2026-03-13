// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cheek tightening control — pulls the cheek skin inward / upward.

/// Configuration for cheek tightening.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekTightenConfig {
    /// Scale factor applied to the tighten output.
    pub scale: f32,
}

impl Default for CheekTightenConfig {
    fn default() -> Self {
        CheekTightenConfig { scale: 1.0 }
    }
}

/// Runtime state for cheek tightening.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekTightenState {
    left: f32,
    right: f32,
    /// Vertical bias in `[-1.0, 1.0]` (positive = pull upward).
    vertical_bias: f32,
    config: CheekTightenConfig,
}

/// Return a default config.
pub fn default_cheek_tighten_config() -> CheekTightenConfig {
    CheekTightenConfig::default()
}

/// Create a new neutral state.
pub fn new_cheek_tighten_state(config: CheekTightenConfig) -> CheekTightenState {
    CheekTightenState {
        left: 0.0,
        right: 0.0,
        vertical_bias: 0.0,
        config,
    }
}

/// Set the left cheek tighten amount.
pub fn ct_set_left(state: &mut CheekTightenState, v: f32) {
    state.left = v.clamp(0.0, 1.0);
}

/// Set the right cheek tighten amount.
pub fn ct_set_right(state: &mut CheekTightenState, v: f32) {
    state.right = v.clamp(0.0, 1.0);
}

/// Set both sides to the same value.
pub fn ct_set_both(state: &mut CheekTightenState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

/// Set the vertical bias in `[-1.0, 1.0]`.
pub fn ct_set_vertical_bias(state: &mut CheekTightenState, v: f32) {
    state.vertical_bias = v.clamp(-1.0, 1.0);
}

/// Reset all parameters to zero.
pub fn ct_reset(state: &mut CheekTightenState) {
    state.left = 0.0;
    state.right = 0.0;
    state.vertical_bias = 0.0;
}

/// Return true when all parameters are effectively zero.
pub fn ct_is_neutral(state: &CheekTightenState) -> bool {
    state.left < 1e-5 && state.right < 1e-5 && state.vertical_bias.abs() < 1e-5
}

/// Asymmetry between left and right.
pub fn ct_asymmetry(state: &CheekTightenState) -> f32 {
    (state.left - state.right).abs()
}

/// Average tighten value across both sides.
pub fn ct_average(state: &CheekTightenState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Evaluate morph weights: `[left, right, vertical_component]`.
pub fn ct_to_weights(state: &CheekTightenState) -> [f32; 3] {
    let s = state.config.scale;
    [
        (state.left * s).clamp(0.0, 1.0),
        (state.right * s).clamp(0.0, 1.0),
        ((state.vertical_bias * 0.5 + 0.5) * s).clamp(0.0, 1.0),
    ]
}

/// Blend between two states.
pub fn ct_blend(a: &CheekTightenState, b: &CheekTightenState, t: f32) -> CheekTightenState {
    let t = t.clamp(0.0, 1.0);
    CheekTightenState {
        left: a.left + (b.left - a.left) * t,
        right: a.right + (b.right - a.right) * t,
        vertical_bias: a.vertical_bias + (b.vertical_bias - a.vertical_bias) * t,
        config: a.config.clone(),
    }
}

/// Serialise to JSON-like string.
pub fn ct_to_json(state: &CheekTightenState) -> String {
    format!(
        r#"{{"left":{:.4},"right":{:.4},"vertical_bias":{:.4}}}"#,
        state.left, state.right, state.vertical_bias
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> CheekTightenState {
        new_cheek_tighten_state(default_cheek_tighten_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(ct_is_neutral(&make()));
    }

    #[test]
    fn set_left_clamps() {
        let mut s = make();
        ct_set_left(&mut s, 5.0);
        assert!((s.left - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_both_equal() {
        let mut s = make();
        ct_set_both(&mut s, 0.6);
        assert!((s.left - s.right).abs() < 1e-5);
    }

    #[test]
    fn reset_clears_all() {
        let mut s = make();
        ct_set_both(&mut s, 0.9);
        ct_reset(&mut s);
        assert!(ct_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_when_equal() {
        let mut s = make();
        ct_set_both(&mut s, 0.4);
        assert!(ct_asymmetry(&s) < 1e-5);
    }

    #[test]
    fn weights_in_unit_range() {
        let mut s = make();
        ct_set_both(&mut s, 0.7);
        for v in ct_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_at_one_is_b() {
        let mut b = make();
        ct_set_both(&mut b, 0.8);
        let r = ct_blend(&make(), &b, 1.0);
        assert!((r.left - 0.8).abs() < 1e-5);
    }

    #[test]
    fn json_contains_left_key() {
        assert!(ct_to_json(&make()).contains("left"));
    }

    #[test]
    fn vertical_bias_clamped() {
        let mut s = make();
        ct_set_vertical_bias(&mut s, 10.0);
        assert!((s.vertical_bias - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_is_mean_of_sides() {
        let mut s = make();
        ct_set_left(&mut s, 0.2);
        ct_set_right(&mut s, 0.8);
        assert!((ct_average(&s) - 0.5).abs() < 1e-5);
    }
}
