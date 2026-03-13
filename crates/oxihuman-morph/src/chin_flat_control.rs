// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Chin flatness control — flattening of the mental protuberance inferior surface.

use std::f32::consts::FRAC_PI_6;

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChinFlatConfig {
    pub max_flatten_m: f32,
}

impl Default for ChinFlatConfig {
    fn default() -> Self {
        Self {
            max_flatten_m: 0.008,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ChinFlatState {
    /// Flatten amount, 0..=1 (0 = round, 1 = flat).
    pub flatten: f32,
    /// Vertical bias of flattening, −1..=1.
    pub v_bias: f32,
}

#[allow(dead_code)]
pub fn new_chin_flat_state() -> ChinFlatState {
    ChinFlatState::default()
}

#[allow(dead_code)]
pub fn default_chin_flat_config() -> ChinFlatConfig {
    ChinFlatConfig::default()
}

#[allow(dead_code)]
pub fn cf_set_flatten(state: &mut ChinFlatState, v: f32) {
    state.flatten = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cf_set_v_bias(state: &mut ChinFlatState, v: f32) {
    state.v_bias = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn cf_reset(state: &mut ChinFlatState) {
    *state = ChinFlatState::default();
}

#[allow(dead_code)]
pub fn cf_is_neutral(state: &ChinFlatState) -> bool {
    state.flatten < 1e-4 && state.v_bias.abs() < 1e-4
}

/// Inferior surface angle change in radians.
#[allow(dead_code)]
pub fn cf_angle_rad(state: &ChinFlatState) -> f32 {
    state.flatten * FRAC_PI_6
}

/// Morph weight for the chin-flatten target.
#[allow(dead_code)]
pub fn cf_to_weights(state: &ChinFlatState, cfg: &ChinFlatConfig) -> f32 {
    state.flatten * cfg.max_flatten_m
}

/// Blend two states.
#[allow(dead_code)]
pub fn cf_blend(a: &ChinFlatState, b: &ChinFlatState, t: f32) -> ChinFlatState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ChinFlatState {
        flatten: a.flatten * inv + b.flatten * t,
        v_bias: a.v_bias * inv + b.v_bias * t,
    }
}

/// JSON.
#[allow(dead_code)]
pub fn cf_to_json(state: &ChinFlatState) -> String {
    format!(
        "{{\"flatten\":{:.4},\"v_bias\":{:.4}}}",
        state.flatten, state.v_bias
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(cf_is_neutral(&new_chin_flat_state()));
    }

    #[test]
    fn flatten_clamps_high() {
        let mut s = new_chin_flat_state();
        cf_set_flatten(&mut s, 2.0);
        assert!((s.flatten - 1.0).abs() < 1e-6);
    }

    #[test]
    fn flatten_clamps_low() {
        let mut s = new_chin_flat_state();
        cf_set_flatten(&mut s, -1.0);
        assert!(s.flatten < 1e-6);
    }

    #[test]
    fn v_bias_clamps() {
        let mut s = new_chin_flat_state();
        cf_set_v_bias(&mut s, 5.0);
        assert!((s.v_bias - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_chin_flat_state();
        cf_set_flatten(&mut s, 0.9);
        cf_reset(&mut s);
        assert!(cf_is_neutral(&s));
    }

    #[test]
    fn angle_zero_at_neutral() {
        let s = new_chin_flat_state();
        assert!(cf_angle_rad(&s).abs() < 1e-6);
    }

    #[test]
    fn angle_positive_when_flat() {
        let mut s = new_chin_flat_state();
        cf_set_flatten(&mut s, 1.0);
        assert!(cf_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weight_scales_with_config() {
        let cfg = default_chin_flat_config();
        let mut s = new_chin_flat_state();
        cf_set_flatten(&mut s, 1.0);
        assert!((cf_to_weights(&s, &cfg) - cfg.max_flatten_m).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_chin_flat_state();
        let b = ChinFlatState {
            flatten: 1.0,
            v_bias: 0.0,
        };
        cf_set_flatten(&mut a, 0.0);
        let r = cf_blend(&a, &b, 0.5);
        assert!((r.flatten - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_contains_flatten() {
        let j = cf_to_json(&new_chin_flat_state());
        assert!(j.contains("flatten"));
    }
}
