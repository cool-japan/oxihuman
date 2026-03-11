// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Individual toe length and splay control.

use std::f32::consts::FRAC_PI_6;

pub const TOE_COUNT: usize = 5;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToeFootSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToeControlConfig {
    pub max_length: f32,
    pub max_splay: f32,
}

impl Default for ToeControlConfig {
    fn default() -> Self {
        Self {
            max_length: 1.0,
            max_splay: 1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToeControlState {
    pub left_len: [f32; TOE_COUNT],
    pub right_len: [f32; TOE_COUNT],
    pub splay: f32,
    pub config: ToeControlConfig,
}

#[allow(dead_code)]
pub fn default_toe_control_config() -> ToeControlConfig {
    ToeControlConfig::default()
}

#[allow(dead_code)]
pub fn new_toe_control_state(config: ToeControlConfig) -> ToeControlState {
    ToeControlState {
        left_len: [0.0; TOE_COUNT],
        right_len: [0.0; TOE_COUNT],
        splay: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn tc_set_length(state: &mut ToeControlState, side: ToeFootSide, toe: usize, v: f32) {
    if toe < TOE_COUNT {
        let v = v.clamp(0.0, state.config.max_length);
        match side {
            ToeFootSide::Left => state.left_len[toe] = v,
            ToeFootSide::Right => state.right_len[toe] = v,
        }
    }
}

#[allow(dead_code)]
pub fn tc_set_splay(state: &mut ToeControlState, v: f32) {
    state.splay = v.clamp(0.0, state.config.max_splay);
}

#[allow(dead_code)]
pub fn tc_reset(state: &mut ToeControlState) {
    state.left_len = [0.0; TOE_COUNT];
    state.right_len = [0.0; TOE_COUNT];
    state.splay = 0.0;
}

#[allow(dead_code)]
pub fn tc_is_neutral(state: &ToeControlState) -> bool {
    state.left_len.iter().all(|v| v.abs() < 1e-6)
        && state.right_len.iter().all(|v| v.abs() < 1e-6)
        && state.splay.abs() < 1e-6
}

#[allow(dead_code)]
pub fn tc_average_length(state: &ToeControlState) -> f32 {
    let sum: f32 = state.left_len.iter().chain(state.right_len.iter()).sum();
    sum / (TOE_COUNT * 2) as f32
}

#[allow(dead_code)]
pub fn tc_splay_angle_rad(state: &ToeControlState) -> f32 {
    state.splay * FRAC_PI_6
}

#[allow(dead_code)]
pub fn tc_to_weights(state: &ToeControlState) -> [f32; TOE_COUNT] {
    let m = state.config.max_length;
    let mut w = [0.0f32; TOE_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        w[i] = if m > 1e-9 {
            (state.left_len[i] + state.right_len[i]) * 0.5 / m
        } else {
            0.0
        };
    }
    w
}

#[allow(dead_code)]
pub fn tc_to_json(state: &ToeControlState) -> String {
    format!(
        "{{\"splay\":{:.4},\"avg_len\":{:.4}}}",
        state.splay,
        tc_average_length(state)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(tc_is_neutral(&new_toe_control_state(
            default_toe_control_config()
        )));
    }
    #[test]
    fn set_length_clamps() {
        let mut s = new_toe_control_state(default_toe_control_config());
        tc_set_length(&mut s, ToeFootSide::Left, 0, 5.0);
        assert!((0.0..=1.0).contains(&s.left_len[0]));
    }
    #[test]
    fn set_splay_clamps() {
        let mut s = new_toe_control_state(default_toe_control_config());
        tc_set_splay(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.splay));
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_toe_control_state(default_toe_control_config());
        tc_set_splay(&mut s, 0.5);
        tc_reset(&mut s);
        assert!(tc_is_neutral(&s));
    }
    #[test]
    fn average_length_zero_by_default() {
        let s = new_toe_control_state(default_toe_control_config());
        assert!(tc_average_length(&s).abs() < 1e-6);
    }
    #[test]
    fn splay_angle_nonneg() {
        let s = new_toe_control_state(default_toe_control_config());
        assert!(tc_splay_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_zero_by_default() {
        let s = new_toe_control_state(default_toe_control_config());
        assert!(tc_to_weights(&s)[0].abs() < 1e-6);
    }
    #[test]
    fn out_of_range_ignored() {
        let mut s = new_toe_control_state(default_toe_control_config());
        tc_set_length(&mut s, ToeFootSide::Left, 99, 1.0);
        assert!(tc_is_neutral(&s));
    }
    #[test]
    fn to_json_has_splay() {
        assert!(
            tc_to_json(&new_toe_control_state(default_toe_control_config())).contains("\"splay\"")
        );
    }
    #[test]
    fn toe_count_is_five() {
        assert_eq!(TOE_COUNT, 5);
    }
}
