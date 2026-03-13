// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lip retraction control — pulling the lips back toward the teeth.

use std::f32::consts::FRAC_PI_6;

/// Which lip.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LipSide {
    Upper,
    Lower,
}

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipRetractConfig {
    /// Maximum retraction angle in radians (informational).
    pub max_angle_rad: f32,
}

impl Default for LipRetractConfig {
    fn default() -> Self {
        LipRetractConfig {
            max_angle_rad: FRAC_PI_6,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipRetractState {
    upper: f32,
    lower: f32,
    /// Corner pull in `[0.0, 1.0]`.
    corners: f32,
    config: LipRetractConfig,
}

/// Default config.
pub fn default_lip_retract_config() -> LipRetractConfig {
    LipRetractConfig::default()
}

/// New neutral state.
pub fn new_lip_retract_state(config: LipRetractConfig) -> LipRetractState {
    LipRetractState {
        upper: 0.0,
        lower: 0.0,
        corners: 0.0,
        config,
    }
}

/// Set retraction for a lip.
pub fn lrc_set_retract(state: &mut LipRetractState, side: LipSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        LipSide::Upper => state.upper = v,
        LipSide::Lower => state.lower = v,
    }
}

/// Set both lips.
pub fn lrc_set_both(state: &mut LipRetractState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.upper = v;
    state.lower = v;
}

/// Set corner pull.
pub fn lrc_set_corners(state: &mut LipRetractState, v: f32) {
    state.corners = v.clamp(0.0, 1.0);
}

/// Reset.
pub fn lrc_reset(state: &mut LipRetractState) {
    state.upper = 0.0;
    state.lower = 0.0;
    state.corners = 0.0;
}

/// True when neutral.
pub fn lrc_is_neutral(state: &LipRetractState) -> bool {
    state.upper < 1e-5 && state.lower < 1e-5 && state.corners < 1e-5
}

/// Average retraction.
pub fn lrc_average(state: &LipRetractState) -> f32 {
    (state.upper + state.lower) * 0.5
}

/// Retraction angle in radians.
pub fn lrc_angle_rad(state: &LipRetractState) -> f32 {
    lrc_average(state) * state.config.max_angle_rad
}

/// Morph weights: `[upper, lower, corners]`.
pub fn lrc_to_weights(state: &LipRetractState) -> [f32; 3] {
    [state.upper, state.lower, state.corners]
}

/// Blend.
pub fn lrc_blend(a: &LipRetractState, b: &LipRetractState, t: f32) -> LipRetractState {
    let t = t.clamp(0.0, 1.0);
    LipRetractState {
        upper: a.upper + (b.upper - a.upper) * t,
        lower: a.lower + (b.lower - a.lower) * t,
        corners: a.corners + (b.corners - a.corners) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn lrc_to_json(state: &LipRetractState) -> String {
    format!(
        r#"{{"upper":{:.4},"lower":{:.4},"corners":{:.4}}}"#,
        state.upper, state.lower, state.corners
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> LipRetractState {
        new_lip_retract_state(default_lip_retract_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(lrc_is_neutral(&make()));
    }

    #[test]
    fn set_upper() {
        let mut s = make();
        lrc_set_retract(&mut s, LipSide::Upper, 0.5);
        assert!((s.upper - 0.5).abs() < 1e-5);
    }

    #[test]
    fn set_both_syncs() {
        let mut s = make();
        lrc_set_both(&mut s, 0.7);
        assert!((s.upper - s.lower).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = make();
        lrc_set_both(&mut s, 1.0);
        lrc_reset(&mut s);
        assert!(lrc_is_neutral(&s));
    }

    #[test]
    fn angle_positive_when_retracted() {
        let mut s = make();
        lrc_set_both(&mut s, 0.5);
        assert!(lrc_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_in_range() {
        let mut s = make();
        lrc_set_both(&mut s, 0.6);
        for v in lrc_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        lrc_set_both(&mut b, 1.0);
        let m = lrc_blend(&make(), &b, 0.5);
        assert!((m.upper - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_upper() {
        assert!(lrc_to_json(&make()).contains("upper"));
    }

    #[test]
    fn corners_clamped_high() {
        let mut s = make();
        lrc_set_corners(&mut s, 10.0);
        assert!((s.corners - 1.0).abs() < 1e-5);
    }
}
