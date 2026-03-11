// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Nasal saddle (sellion depression) control — depth and width of the nose root concavity.

use std::f32::consts::FRAC_PI_6;

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NasalSaddleConfig {
    pub max_depth_m: f32,
    pub max_width_m: f32,
}

impl Default for NasalSaddleConfig {
    fn default() -> Self {
        Self {
            max_depth_m: 0.005,
            max_width_m: 0.008,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct NasalSaddleState {
    /// Depression depth, 0..=1.
    pub depth: f32,
    /// Saddle width, 0..=1.
    pub width: f32,
    /// Vertical position shift, −1..=1.
    pub v_shift: f32,
}

#[allow(dead_code)]
pub fn new_nasal_saddle_state() -> NasalSaddleState {
    NasalSaddleState::default()
}

#[allow(dead_code)]
pub fn default_nasal_saddle_config() -> NasalSaddleConfig {
    NasalSaddleConfig::default()
}

#[allow(dead_code)]
pub fn nsd_set_depth(state: &mut NasalSaddleState, v: f32) {
    state.depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nsd_set_width(state: &mut NasalSaddleState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nsd_set_v_shift(state: &mut NasalSaddleState, v: f32) {
    state.v_shift = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nsd_reset(state: &mut NasalSaddleState) {
    *state = NasalSaddleState::default();
}

#[allow(dead_code)]
pub fn nsd_is_neutral(state: &NasalSaddleState) -> bool {
    state.depth < 1e-4 && state.width < 1e-4 && state.v_shift.abs() < 1e-4
}

/// Nasal root angle change in radians.
#[allow(dead_code)]
pub fn nsd_root_angle_rad(state: &NasalSaddleState) -> f32 {
    state.depth * FRAC_PI_6
}

#[allow(dead_code)]
pub fn nsd_to_weights(state: &NasalSaddleState, cfg: &NasalSaddleConfig) -> [f32; 2] {
    [state.depth * cfg.max_depth_m, state.width * cfg.max_width_m]
}

#[allow(dead_code)]
pub fn nsd_blend(a: &NasalSaddleState, b: &NasalSaddleState, t: f32) -> NasalSaddleState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    NasalSaddleState {
        depth: a.depth * inv + b.depth * t,
        width: a.width * inv + b.width * t,
        v_shift: a.v_shift * inv + b.v_shift * t,
    }
}

#[allow(dead_code)]
pub fn nsd_to_json(state: &NasalSaddleState) -> String {
    format!(
        "{{\"depth\":{:.4},\"width\":{:.4},\"v_shift\":{:.4}}}",
        state.depth, state.width, state.v_shift
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(nsd_is_neutral(&new_nasal_saddle_state()));
    }

    #[test]
    fn depth_clamps() {
        let mut s = new_nasal_saddle_state();
        nsd_set_depth(&mut s, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
        nsd_set_depth(&mut s, -1.0);
        assert!(s.depth < 1e-6);
    }

    #[test]
    fn width_clamps() {
        let mut s = new_nasal_saddle_state();
        nsd_set_width(&mut s, -3.0);
        assert!(s.width < 1e-6);
    }

    #[test]
    fn v_shift_clamps() {
        let mut s = new_nasal_saddle_state();
        nsd_set_v_shift(&mut s, 5.0);
        assert!((s.v_shift - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_nasal_saddle_state();
        nsd_set_depth(&mut s, 0.8);
        nsd_reset(&mut s);
        assert!(nsd_is_neutral(&s));
    }

    #[test]
    fn root_angle_zero_at_neutral() {
        assert!(nsd_root_angle_rad(&new_nasal_saddle_state()).abs() < 1e-6);
    }

    #[test]
    fn root_angle_positive_at_max() {
        let mut s = new_nasal_saddle_state();
        nsd_set_depth(&mut s, 1.0);
        assert!(nsd_root_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_two_values() {
        let w = nsd_to_weights(&new_nasal_saddle_state(), &default_nasal_saddle_config());
        assert_eq!(w.len(), 2);
    }

    #[test]
    fn blend_midpoint() {
        let b = NasalSaddleState {
            depth: 1.0,
            width: 0.0,
            v_shift: 0.0,
        };
        let r = nsd_blend(&new_nasal_saddle_state(), &b, 0.5);
        assert!((r.depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = nsd_to_json(&new_nasal_saddle_state());
        assert!(j.contains("depth") && j.contains("v_shift"));
    }
}
