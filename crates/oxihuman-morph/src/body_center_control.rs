// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body centre-of-mass shift control — anterior/posterior lean and lateral sway.

use std::f32::consts::FRAC_PI_4;

/// Configuration for the body centre control.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BodyCenterConfig {
    /// Max anterior shift in metres.
    pub max_anterior: f32,
    /// Max posterior shift in metres.
    pub max_posterior: f32,
    /// Max lateral sway in metres.
    pub max_lateral: f32,
}

impl Default for BodyCenterConfig {
    fn default() -> Self {
        Self {
            max_anterior: 0.08,
            max_posterior: 0.06,
            max_lateral: 0.05,
        }
    }
}

/// Runtime state for the body centre morph.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BodyCenterState {
    /// Anterior (+) / posterior (−) shift, −1..=1.
    pub ap_shift: f32,
    /// Lateral shift left (−) / right (+), −1..=1.
    pub lateral_shift: f32,
}

/// Create a new body centre state.
#[allow(dead_code)]
pub fn new_body_center_state() -> BodyCenterState {
    BodyCenterState::default()
}

/// Default config.
#[allow(dead_code)]
pub fn default_body_center_config() -> BodyCenterConfig {
    BodyCenterConfig::default()
}

/// Set anterior/posterior shift (clamped to −1..=1).
#[allow(dead_code)]
pub fn bcc_set_ap(state: &mut BodyCenterState, v: f32) {
    state.ap_shift = v.clamp(-1.0, 1.0);
}

/// Set lateral shift (clamped to −1..=1).
#[allow(dead_code)]
pub fn bcc_set_lateral(state: &mut BodyCenterState, v: f32) {
    state.lateral_shift = v.clamp(-1.0, 1.0);
}

/// Reset to neutral.
#[allow(dead_code)]
pub fn bcc_reset(state: &mut BodyCenterState) {
    *state = BodyCenterState::default();
}

/// Whether the state is effectively neutral.
#[allow(dead_code)]
pub fn bcc_is_neutral(state: &BodyCenterState) -> bool {
    state.ap_shift.abs() < 1e-4 && state.lateral_shift.abs() < 1e-4
}

/// Total displacement magnitude (0..=1 normalised).
#[allow(dead_code)]
pub fn bcc_displacement(state: &BodyCenterState) -> f32 {
    (state.ap_shift.powi(2) + state.lateral_shift.powi(2))
        .sqrt()
        .min(1.0)
}

/// Convert to morph weight map (two weights: anterior-posterior, lateral).
#[allow(dead_code)]
pub fn bcc_to_weights(state: &BodyCenterState, cfg: &BodyCenterConfig) -> [f32; 2] {
    let ap = if state.ap_shift >= 0.0 {
        state.ap_shift * cfg.max_anterior
    } else {
        state.ap_shift * cfg.max_posterior
    };
    let lat = state.lateral_shift * cfg.max_lateral;
    [ap, lat]
}

/// Lean angle in radians (sagittal plane).
#[allow(dead_code)]
pub fn bcc_lean_angle_rad(state: &BodyCenterState) -> f32 {
    state.ap_shift * FRAC_PI_4 * 0.25
}

/// Blend two states.
#[allow(dead_code)]
pub fn bcc_blend(a: &BodyCenterState, b: &BodyCenterState, t: f32) -> BodyCenterState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BodyCenterState {
        ap_shift: a.ap_shift * inv + b.ap_shift * t,
        lateral_shift: a.lateral_shift * inv + b.lateral_shift * t,
    }
}

/// Serialise to a JSON-like string.
#[allow(dead_code)]
pub fn bcc_to_json(state: &BodyCenterState) -> String {
    format!(
        "{{\"ap_shift\":{:.4},\"lateral_shift\":{:.4}}}",
        state.ap_shift, state.lateral_shift
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        let s = new_body_center_state();
        assert!(bcc_is_neutral(&s));
    }

    #[test]
    fn set_ap_clamps() {
        let mut s = new_body_center_state();
        bcc_set_ap(&mut s, 5.0);
        assert!((s.ap_shift - 1.0).abs() < 1e-6);
        bcc_set_ap(&mut s, -5.0);
        assert!((s.ap_shift + 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_lateral_clamps() {
        let mut s = new_body_center_state();
        bcc_set_lateral(&mut s, -3.0);
        assert!((s.lateral_shift + 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_body_center_state();
        bcc_set_ap(&mut s, 0.5);
        bcc_set_lateral(&mut s, 0.3);
        bcc_reset(&mut s);
        assert!(bcc_is_neutral(&s));
    }

    #[test]
    fn displacement_zero_at_neutral() {
        let s = new_body_center_state();
        assert!(bcc_displacement(&s) < 1e-6);
    }

    #[test]
    fn displacement_positive_when_shifted() {
        let mut s = new_body_center_state();
        bcc_set_ap(&mut s, 0.6);
        bcc_set_lateral(&mut s, 0.4);
        assert!(bcc_displacement(&s) > 0.5);
    }

    #[test]
    fn weights_sign_matches_direction() {
        let cfg = default_body_center_config();
        let mut s = new_body_center_state();
        bcc_set_ap(&mut s, 1.0);
        let w = bcc_to_weights(&s, &cfg);
        assert!(w[0] > 0.0);
    }

    #[test]
    fn lean_angle_sign() {
        let mut s = new_body_center_state();
        bcc_set_ap(&mut s, 1.0);
        assert!(bcc_lean_angle_rad(&s) > 0.0);
        bcc_set_ap(&mut s, -1.0);
        assert!(bcc_lean_angle_rad(&s) < 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_body_center_state();
        let mut b = new_body_center_state();
        bcc_set_ap(&mut a, 0.0);
        bcc_set_ap(&mut b, 1.0);
        let r = bcc_blend(&a, &b, 0.5);
        assert!((r.ap_shift - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_contains_ap_key() {
        let s = new_body_center_state();
        let j = bcc_to_json(&s);
        assert!(j.contains("ap_shift"));
    }
}
