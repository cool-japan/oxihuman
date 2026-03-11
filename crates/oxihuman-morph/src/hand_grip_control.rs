// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hand grip control — finger curl and palm compression for grip poses.

/// Configuration for hand grip.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandGripConfig {
    pub max_curl: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandGripSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandGripState {
    pub left_curl: f32,
    pub right_curl: f32,
    pub palm_compression: f32,
}

#[allow(dead_code)]
pub fn default_hand_grip_config() -> HandGripConfig {
    HandGripConfig { max_curl: 1.0 }
}

#[allow(dead_code)]
pub fn new_hand_grip_state() -> HandGripState {
    HandGripState {
        left_curl: 0.0,
        right_curl: 0.0,
        palm_compression: 0.0,
    }
}

#[allow(dead_code)]
pub fn hg_set_curl(state: &mut HandGripState, cfg: &HandGripConfig, side: HandGripSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_curl);
    match side {
        HandGripSide::Left => state.left_curl = clamped,
        HandGripSide::Right => state.right_curl = clamped,
    }
}

#[allow(dead_code)]
pub fn hg_set_both(state: &mut HandGripState, cfg: &HandGripConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_curl);
    state.left_curl = clamped;
    state.right_curl = clamped;
}

#[allow(dead_code)]
pub fn hg_set_palm_compression(state: &mut HandGripState, v: f32) {
    state.palm_compression = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hg_reset(state: &mut HandGripState) {
    *state = new_hand_grip_state();
}

#[allow(dead_code)]
pub fn hg_is_neutral(state: &HandGripState) -> bool {
    state.left_curl.abs() < 1e-6
        && state.right_curl.abs() < 1e-6
        && state.palm_compression.abs() < 1e-6
}

#[allow(dead_code)]
pub fn hg_average_curl(state: &HandGripState) -> f32 {
    (state.left_curl + state.right_curl) * 0.5
}

#[allow(dead_code)]
pub fn hg_symmetry(state: &HandGripState) -> f32 {
    (state.left_curl - state.right_curl).abs()
}

#[allow(dead_code)]
pub fn hg_blend(a: &HandGripState, b: &HandGripState, t: f32) -> HandGripState {
    let t = t.clamp(0.0, 1.0);
    HandGripState {
        left_curl: a.left_curl + (b.left_curl - a.left_curl) * t,
        right_curl: a.right_curl + (b.right_curl - a.right_curl) * t,
        palm_compression: a.palm_compression + (b.palm_compression - a.palm_compression) * t,
    }
}

#[allow(dead_code)]
pub fn hg_to_weights(state: &HandGripState) -> Vec<(String, f32)> {
    vec![
        ("hand_grip_curl_l".to_string(), state.left_curl),
        ("hand_grip_curl_r".to_string(), state.right_curl),
        ("palm_compression".to_string(), state.palm_compression),
    ]
}

#[allow(dead_code)]
pub fn hg_to_json(state: &HandGripState) -> String {
    format!(
        r#"{{"left_curl":{:.4},"right_curl":{:.4},"palm_compression":{:.4}}}"#,
        state.left_curl, state.right_curl, state.palm_compression
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_hand_grip_config();
        assert!((cfg.max_curl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_hand_grip_state();
        assert!(hg_is_neutral(&s));
    }

    #[test]
    fn set_curl_left() {
        let cfg = default_hand_grip_config();
        let mut s = new_hand_grip_state();
        hg_set_curl(&mut s, &cfg, HandGripSide::Left, 0.7);
        assert!((s.left_curl - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_curl_clamps() {
        let cfg = default_hand_grip_config();
        let mut s = new_hand_grip_state();
        hg_set_curl(&mut s, &cfg, HandGripSide::Right, 5.0);
        assert!((s.right_curl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_symmetric() {
        let cfg = default_hand_grip_config();
        let mut s = new_hand_grip_state();
        hg_set_both(&mut s, &cfg, 0.5);
        assert!(hg_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_palm_compression() {
        let mut s = new_hand_grip_state();
        hg_set_palm_compression(&mut s, 0.6);
        assert!((s.palm_compression - 0.6).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_hand_grip_config();
        let mut s = new_hand_grip_state();
        hg_set_both(&mut s, &cfg, 0.8);
        hg_reset(&mut s);
        assert!(hg_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_hand_grip_state();
        let cfg = default_hand_grip_config();
        let mut b = new_hand_grip_state();
        hg_set_both(&mut b, &cfg, 1.0);
        let m = hg_blend(&a, &b, 0.5);
        assert!((m.left_curl - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_hand_grip_state();
        assert_eq!(hg_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_hand_grip_state();
        let j = hg_to_json(&s);
        assert!(j.contains("left_curl"));
    }
}
