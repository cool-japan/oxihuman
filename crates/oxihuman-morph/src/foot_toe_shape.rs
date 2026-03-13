// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Foot toe shape — controls toe length taper and curvature.

use std::f32::consts::FRAC_PI_6;

/// Number of toes per foot.
pub const TOE_COUNT: usize = 5;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootToeShapeConfig {
    pub max_length: f32,
}

impl Default for FootToeShapeConfig {
    fn default() -> Self {
        Self { max_length: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootToeShapeState {
    pub left_lengths: [f32; TOE_COUNT],
    pub right_lengths: [f32; TOE_COUNT],
    pub curl: f32,
    pub config: FootToeShapeConfig,
}

#[allow(dead_code)]
pub fn default_foot_toe_shape_config() -> FootToeShapeConfig {
    FootToeShapeConfig::default()
}

#[allow(dead_code)]
pub fn new_foot_toe_shape_state(config: FootToeShapeConfig) -> FootToeShapeState {
    FootToeShapeState {
        left_lengths: [0.0; TOE_COUNT],
        right_lengths: [0.0; TOE_COUNT],
        curl: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn fts_set_toe(state: &mut FootToeShapeState, side: FootSide, toe: usize, v: f32) {
    if toe < TOE_COUNT {
        let v = v.clamp(0.0, state.config.max_length);
        match side {
            FootSide::Left => state.left_lengths[toe] = v,
            FootSide::Right => state.right_lengths[toe] = v,
        }
    }
}

#[allow(dead_code)]
pub fn fts_set_all(state: &mut FootToeShapeState, v: f32) {
    let v = v.clamp(0.0, state.config.max_length);
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        state.left_lengths[i] = v;
        state.right_lengths[i] = v;
    }
}

#[allow(dead_code)]
pub fn fts_set_curl(state: &mut FootToeShapeState, v: f32) {
    state.curl = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fts_reset(state: &mut FootToeShapeState) {
    state.left_lengths = [0.0; TOE_COUNT];
    state.right_lengths = [0.0; TOE_COUNT];
    state.curl = 0.0;
}

#[allow(dead_code)]
pub fn fts_is_neutral(state: &FootToeShapeState) -> bool {
    state.left_lengths.iter().all(|v| v.abs() < 1e-6)
        && state.right_lengths.iter().all(|v| v.abs() < 1e-6)
        && state.curl.abs() < 1e-6
}

#[allow(dead_code)]
pub fn fts_average_length(state: &FootToeShapeState, side: FootSide) -> f32 {
    let arr = match side {
        FootSide::Left => &state.left_lengths,
        FootSide::Right => &state.right_lengths,
    };
    arr.iter().sum::<f32>() / TOE_COUNT as f32
}

#[allow(dead_code)]
pub fn fts_curl_angle_rad(state: &FootToeShapeState) -> f32 {
    state.curl * FRAC_PI_6
}

#[allow(dead_code)]
pub fn fts_to_weights(state: &FootToeShapeState) -> [f32; TOE_COUNT] {
    let m = state.config.max_length;
    let mut w = [0.0f32; TOE_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        w[i] = if m > 1e-9 {
            (state.left_lengths[i] + state.right_lengths[i]) * 0.5 / m
        } else {
            0.0
        };
    }
    w
}

#[allow(dead_code)]
pub fn fts_to_json(state: &FootToeShapeState) -> String {
    format!("{{\"curl\":{:.4}}}", state.curl)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(fts_is_neutral(&new_foot_toe_shape_state(
            default_foot_toe_shape_config()
        )));
    }
    #[test]
    fn set_toe_clamps() {
        let mut s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        fts_set_toe(&mut s, FootSide::Left, 0, 5.0);
        assert!((0.0..=1.0).contains(&s.left_lengths[0]));
    }
    #[test]
    fn set_all_applies() {
        let mut s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        fts_set_all(&mut s, 0.4);
        assert!(!s.left_lengths.is_empty() && (s.left_lengths[0] - 0.4).abs() < 1e-5);
    }
    #[test]
    fn curl_clamps() {
        let mut s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        fts_set_curl(&mut s, 2.0);
        assert!((0.0..=1.0).contains(&s.curl));
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        fts_set_all(&mut s, 0.5);
        fts_reset(&mut s);
        assert!(fts_is_neutral(&s));
    }
    #[test]
    fn average_length_zero_by_default() {
        let s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        assert!(fts_average_length(&s, FootSide::Left).abs() < 1e-6);
    }
    #[test]
    fn curl_angle_nonneg() {
        let s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        assert!(fts_curl_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        fts_set_all(&mut s, 1.0);
        let w = fts_to_weights(&s);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn out_of_range_toe_ignored() {
        let mut s = new_foot_toe_shape_state(default_foot_toe_shape_config());
        fts_set_toe(&mut s, FootSide::Left, 99, 1.0);
        assert!(fts_is_neutral(&s));
    }
    #[test]
    fn to_json_has_curl() {
        assert!(
            fts_to_json(&new_foot_toe_shape_state(default_foot_toe_shape_config()))
                .contains("\"curl\"")
        );
    }
}
