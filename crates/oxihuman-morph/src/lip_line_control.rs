// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lip line (vermillion border) definition and curvature control.

use std::f32::consts::FRAC_PI_8;

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LipLineConfig {
    pub max_curvature_m: f32,
}

impl Default for LipLineConfig {
    fn default() -> Self {
        Self {
            max_curvature_m: 0.006,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LipLineState {
    /// Upper bow curvature, −1..=1 (positive = peaked, negative = flat).
    pub upper_bow: f32,
    /// Lower lip definition, 0..=1.
    pub lower_def: f32,
    /// Lip line width scale, −1..=1.
    pub width_scale: f32,
}

#[allow(dead_code)]
pub fn new_lip_line_state() -> LipLineState {
    LipLineState::default()
}

#[allow(dead_code)]
pub fn default_lip_line_config() -> LipLineConfig {
    LipLineConfig::default()
}

#[allow(dead_code)]
pub fn ll_set_upper_bow(state: &mut LipLineState, v: f32) {
    state.upper_bow = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ll_set_lower_def(state: &mut LipLineState, v: f32) {
    state.lower_def = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ll_set_width_scale(state: &mut LipLineState, v: f32) {
    state.width_scale = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ll_reset(state: &mut LipLineState) {
    *state = LipLineState::default();
}

#[allow(dead_code)]
pub fn ll_is_neutral(state: &LipLineState) -> bool {
    state.upper_bow.abs() < 1e-4 && state.lower_def < 1e-4 && state.width_scale.abs() < 1e-4
}

/// Bow angle in radians.
#[allow(dead_code)]
pub fn ll_bow_angle_rad(state: &LipLineState) -> f32 {
    state.upper_bow * FRAC_PI_8
}

#[allow(dead_code)]
pub fn ll_to_weights(state: &LipLineState, cfg: &LipLineConfig) -> [f32; 3] {
    [
        state.upper_bow * cfg.max_curvature_m,
        state.lower_def * cfg.max_curvature_m,
        state.width_scale * cfg.max_curvature_m * 0.5,
    ]
}

#[allow(dead_code)]
pub fn ll_blend(a: &LipLineState, b: &LipLineState, t: f32) -> LipLineState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    LipLineState {
        upper_bow: a.upper_bow * inv + b.upper_bow * t,
        lower_def: a.lower_def * inv + b.lower_def * t,
        width_scale: a.width_scale * inv + b.width_scale * t,
    }
}

#[allow(dead_code)]
pub fn ll_to_json(state: &LipLineState) -> String {
    format!(
        "{{\"upper_bow\":{:.4},\"lower_def\":{:.4},\"width_scale\":{:.4}}}",
        state.upper_bow, state.lower_def, state.width_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(ll_is_neutral(&new_lip_line_state()));
    }

    #[test]
    fn upper_bow_clamps() {
        let mut s = new_lip_line_state();
        ll_set_upper_bow(&mut s, 5.0);
        assert!((s.upper_bow - 1.0).abs() < 1e-6);
        ll_set_upper_bow(&mut s, -5.0);
        assert!((s.upper_bow + 1.0).abs() < 1e-6);
    }

    #[test]
    fn lower_def_clamps() {
        let mut s = new_lip_line_state();
        ll_set_lower_def(&mut s, -1.0);
        assert!(s.lower_def < 1e-6);
    }

    #[test]
    fn width_scale_clamps() {
        let mut s = new_lip_line_state();
        ll_set_width_scale(&mut s, 3.0);
        assert!((s.width_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_lip_line_state();
        ll_set_upper_bow(&mut s, 0.7);
        ll_reset(&mut s);
        assert!(ll_is_neutral(&s));
    }

    #[test]
    fn bow_angle_positive_peaked() {
        let mut s = new_lip_line_state();
        ll_set_upper_bow(&mut s, 1.0);
        assert!(ll_bow_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_three_values() {
        let w = ll_to_weights(&new_lip_line_state(), &default_lip_line_config());
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn blend_midpoint() {
        let b = LipLineState {
            upper_bow: 1.0,
            lower_def: 0.0,
            width_scale: 0.0,
        };
        let r = ll_blend(&new_lip_line_state(), &b, 0.5);
        assert!((r.upper_bow - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = ll_to_json(&new_lip_line_state());
        assert!(j.contains("upper_bow") && j.contains("lower_def"));
    }

    #[test]
    fn bow_angle_negative_when_flat() {
        let mut s = new_lip_line_state();
        ll_set_upper_bow(&mut s, -1.0);
        assert!(ll_bow_angle_rad(&s) < 0.0);
    }
}
