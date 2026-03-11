// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hand metacarpal bone length/width control.

use std::f32::consts::FRAC_PI_4;

/// Number of metacarpal bones per hand.
pub const MC_COUNT: usize = 5;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSide {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandMetacarpalConfig {
    pub max_length: f32,
}

impl Default for HandMetacarpalConfig {
    fn default() -> Self {
        Self { max_length: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandMetacarpalState {
    pub left: [f32; MC_COUNT],
    pub right: [f32; MC_COUNT],
    pub config: HandMetacarpalConfig,
}

#[allow(dead_code)]
pub fn default_hand_metacarpal_config() -> HandMetacarpalConfig {
    HandMetacarpalConfig::default()
}

#[allow(dead_code)]
pub fn new_hand_metacarpal_state(config: HandMetacarpalConfig) -> HandMetacarpalState {
    HandMetacarpalState {
        left: [0.0; MC_COUNT],
        right: [0.0; MC_COUNT],
        config,
    }
}

#[allow(dead_code)]
pub fn hmc_set(state: &mut HandMetacarpalState, side: HandSide, bone: usize, v: f32) {
    if bone < MC_COUNT {
        let v = v.clamp(0.0, state.config.max_length);
        match side {
            HandSide::Left => state.left[bone] = v,
            HandSide::Right => state.right[bone] = v,
        }
    }
}

#[allow(dead_code)]
pub fn hmc_set_all(state: &mut HandMetacarpalState, v: f32) {
    let v = v.clamp(0.0, state.config.max_length);
    #[allow(clippy::needless_range_loop)]
    for i in 0..MC_COUNT {
        state.left[i] = v;
        state.right[i] = v;
    }
}

#[allow(dead_code)]
pub fn hmc_reset(state: &mut HandMetacarpalState) {
    state.left = [0.0; MC_COUNT];
    state.right = [0.0; MC_COUNT];
}

#[allow(dead_code)]
pub fn hmc_is_neutral(state: &HandMetacarpalState) -> bool {
    state.left.iter().all(|v| v.abs() < 1e-6) && state.right.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn hmc_average(state: &HandMetacarpalState, side: HandSide) -> f32 {
    let arr = match side {
        HandSide::Left => &state.left,
        HandSide::Right => &state.right,
    };
    arr.iter().sum::<f32>() / MC_COUNT as f32
}

#[allow(dead_code)]
pub fn hmc_span_angle_rad(state: &HandMetacarpalState) -> f32 {
    let avg = (hmc_average(state, HandSide::Left) + hmc_average(state, HandSide::Right)) * 0.5;
    avg * FRAC_PI_4
}

#[allow(dead_code)]
pub fn hmc_to_weights(state: &HandMetacarpalState) -> [f32; MC_COUNT] {
    let m = state.config.max_length;
    let mut w = [0.0f32; MC_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..MC_COUNT {
        w[i] = if m > 1e-9 {
            (state.left[i] + state.right[i]) * 0.5 / m
        } else {
            0.0
        };
    }
    w
}

#[allow(dead_code)]
pub fn hmc_to_json(state: &HandMetacarpalState) -> String {
    format!(
        "{{\"left_avg\":{:.4},\"right_avg\":{:.4}}}",
        hmc_average(state, HandSide::Left),
        hmc_average(state, HandSide::Right)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(hmc_is_neutral(&new_hand_metacarpal_state(
            default_hand_metacarpal_config()
        )));
    }
    #[test]
    fn set_clamps() {
        let mut s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        hmc_set(&mut s, HandSide::Left, 0, 5.0);
        assert!((0.0..=1.0).contains(&s.left[0]));
    }
    #[test]
    fn set_all_applies() {
        let mut s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        hmc_set_all(&mut s, 0.5);
        assert!((s.left[0] - 0.5).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        hmc_set_all(&mut s, 0.7);
        hmc_reset(&mut s);
        assert!(hmc_is_neutral(&s));
    }
    #[test]
    fn average_is_zero_by_default() {
        let s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        assert!(hmc_average(&s, HandSide::Left).abs() < 1e-6);
    }
    #[test]
    fn span_angle_nonneg() {
        let s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        assert!(hmc_span_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        hmc_set_all(&mut s, 1.0);
        let w = hmc_to_weights(&s);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn out_of_range_ignored() {
        let mut s = new_hand_metacarpal_state(default_hand_metacarpal_config());
        hmc_set(&mut s, HandSide::Left, 99, 1.0);
        assert!(hmc_is_neutral(&s));
    }
    #[test]
    fn to_json_has_left_avg() {
        assert!(
            hmc_to_json(&new_hand_metacarpal_state(default_hand_metacarpal_config()))
                .contains("left_avg")
        );
    }
    #[test]
    fn mc_count_correct() {
        assert_eq!(MC_COUNT, 5);
    }
}
