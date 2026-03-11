// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hand finger splay control — abduction spread of digits.

use std::f32::consts::FRAC_PI_6;

/// Number of fingers (index..pinky = 4, plus thumb).
pub const FINGER_COUNT: usize = 5;

/// Which hand.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FingerSplayConfig {
    pub max_angle_rad: f32,
}

impl Default for FingerSplayConfig {
    fn default() -> Self {
        Self {
            max_angle_rad: FRAC_PI_6,
        }
    }
}

/// Per-hand state: splay per finger, 0..=1.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerSplayState {
    pub left: [f32; FINGER_COUNT],
    pub right: [f32; FINGER_COUNT],
}

impl Default for FingerSplayState {
    fn default() -> Self {
        Self {
            left: [0.0; FINGER_COUNT],
            right: [0.0; FINGER_COUNT],
        }
    }
}

#[allow(dead_code)]
pub fn new_finger_splay_state() -> FingerSplayState {
    FingerSplayState::default()
}

#[allow(dead_code)]
pub fn default_finger_splay_config() -> FingerSplayConfig {
    FingerSplayConfig::default()
}

#[allow(dead_code)]
pub fn hfs_set_finger(state: &mut FingerSplayState, side: HandSide, finger: usize, v: f32) {
    if finger >= FINGER_COUNT {
        return;
    }
    let v = v.clamp(0.0, 1.0);
    match side {
        HandSide::Left => state.left[finger] = v,
        HandSide::Right => state.right[finger] = v,
    }
}

#[allow(dead_code)]
pub fn hfs_set_all(state: &mut FingerSplayState, side: HandSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    let arr = match side {
        HandSide::Left => &mut state.left,
        HandSide::Right => &mut state.right,
    };
    #[allow(clippy::needless_range_loop)]
    for i in 0..FINGER_COUNT {
        arr[i] = v;
    }
}

#[allow(dead_code)]
pub fn hfs_reset(state: &mut FingerSplayState) {
    *state = FingerSplayState::default();
}

#[allow(dead_code)]
pub fn hfs_is_neutral(state: &FingerSplayState) -> bool {
    state.left.iter().all(|&v| v < 1e-4) && state.right.iter().all(|&v| v < 1e-4)
}

/// Average splay angle in radians for one side.
#[allow(dead_code)]
pub fn hfs_average_angle_rad(
    state: &FingerSplayState,
    side: HandSide,
    cfg: &FingerSplayConfig,
) -> f32 {
    let arr = match side {
        HandSide::Left => &state.left,
        HandSide::Right => &state.right,
    };
    let sum: f32 = arr.iter().sum();
    (sum / FINGER_COUNT as f32) * cfg.max_angle_rad
}

/// Per-finger angle in radians.
#[allow(dead_code)]
pub fn hfs_finger_angle_rad(
    state: &FingerSplayState,
    side: HandSide,
    finger: usize,
    cfg: &FingerSplayConfig,
) -> f32 {
    if finger >= FINGER_COUNT {
        return 0.0;
    }
    let v = match side {
        HandSide::Left => state.left[finger],
        HandSide::Right => state.right[finger],
    };
    v * cfg.max_angle_rad
}

#[allow(dead_code)]
pub fn hfs_blend(a: &FingerSplayState, b: &FingerSplayState, t: f32) -> FingerSplayState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let mut left = [0.0f32; FINGER_COUNT];
    let mut right = [0.0f32; FINGER_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..FINGER_COUNT {
        left[i] = a.left[i] * inv + b.left[i] * t;
        right[i] = a.right[i] * inv + b.right[i] * t;
    }
    FingerSplayState { left, right }
}

#[allow(dead_code)]
pub fn hfs_to_json(state: &FingerSplayState) -> String {
    format!("{{\"left\":{:?},\"right\":{:?}}}", state.left, state.right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(hfs_is_neutral(&new_finger_splay_state()));
    }

    #[test]
    fn set_finger_clamps() {
        let mut s = new_finger_splay_state();
        hfs_set_finger(&mut s, HandSide::Left, 0, 5.0);
        assert!((s.left[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_all_fills_array() {
        let mut s = new_finger_splay_state();
        hfs_set_all(&mut s, HandSide::Right, 0.7);
        assert!(s.right.iter().all(|&v| (v - 0.7).abs() < 1e-6));
    }

    #[test]
    fn out_of_range_finger_ignored() {
        let mut s = new_finger_splay_state();
        hfs_set_finger(&mut s, HandSide::Left, 99, 1.0);
        assert!(hfs_is_neutral(&s));
    }

    #[test]
    fn reset_clears() {
        let mut s = new_finger_splay_state();
        hfs_set_all(&mut s, HandSide::Left, 0.9);
        hfs_reset(&mut s);
        assert!(hfs_is_neutral(&s));
    }

    #[test]
    fn average_angle_zero_at_neutral() {
        let cfg = default_finger_splay_config();
        let s = new_finger_splay_state();
        assert!(hfs_average_angle_rad(&s, HandSide::Left, &cfg) < 1e-6);
    }

    #[test]
    fn finger_angle_positive() {
        let cfg = default_finger_splay_config();
        let mut s = new_finger_splay_state();
        hfs_set_finger(&mut s, HandSide::Left, 2, 1.0);
        assert!(hfs_finger_angle_rad(&s, HandSide::Left, 2, &cfg) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_finger_splay_state();
        hfs_set_all(&mut b, HandSide::Left, 1.0);
        let r = hfs_blend(&new_finger_splay_state(), &b, 0.5);
        assert!((r.left[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_left_right() {
        let j = hfs_to_json(&new_finger_splay_state());
        assert!(j.contains("left") && j.contains("right"));
    }

    #[test]
    fn finger_count_constant() {
        assert_eq!(FINGER_COUNT, 5);
    }
}
