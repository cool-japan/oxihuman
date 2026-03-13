// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eye outer corner control — lateral canthus position and angle.

/// Configuration for eye outer corner.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeOuterCornerConfig {
    pub max_depth: f32,
    pub max_tilt_rad: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeOuterSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeOuterCornerState {
    pub left_depth: f32,
    pub right_depth: f32,
    pub left_tilt_rad: f32,
    pub right_tilt_rad: f32,
}

#[allow(dead_code)]
pub fn default_eye_outer_corner_config() -> EyeOuterCornerConfig {
    use std::f32::consts::FRAC_PI_6;
    EyeOuterCornerConfig {
        max_depth: 1.0,
        max_tilt_rad: FRAC_PI_6,
    }
}

#[allow(dead_code)]
pub fn new_eye_outer_corner_state() -> EyeOuterCornerState {
    EyeOuterCornerState {
        left_depth: 0.0,
        right_depth: 0.0,
        left_tilt_rad: 0.0,
        right_tilt_rad: 0.0,
    }
}

#[allow(dead_code)]
pub fn eoc_set_depth(
    state: &mut EyeOuterCornerState,
    cfg: &EyeOuterCornerConfig,
    side: EyeOuterSide,
    v: f32,
) {
    let clamped = v.clamp(-cfg.max_depth, cfg.max_depth);
    match side {
        EyeOuterSide::Left => state.left_depth = clamped,
        EyeOuterSide::Right => state.right_depth = clamped,
    }
}

#[allow(dead_code)]
pub fn eoc_set_tilt(
    state: &mut EyeOuterCornerState,
    cfg: &EyeOuterCornerConfig,
    side: EyeOuterSide,
    v: f32,
) {
    let clamped = v.clamp(-cfg.max_tilt_rad, cfg.max_tilt_rad);
    match side {
        EyeOuterSide::Left => state.left_tilt_rad = clamped,
        EyeOuterSide::Right => state.right_tilt_rad = clamped,
    }
}

#[allow(dead_code)]
pub fn eoc_set_both_depth(state: &mut EyeOuterCornerState, cfg: &EyeOuterCornerConfig, v: f32) {
    let clamped = v.clamp(-cfg.max_depth, cfg.max_depth);
    state.left_depth = clamped;
    state.right_depth = clamped;
}

#[allow(dead_code)]
pub fn eoc_reset(state: &mut EyeOuterCornerState) {
    *state = new_eye_outer_corner_state();
}

#[allow(dead_code)]
pub fn eoc_is_neutral(state: &EyeOuterCornerState) -> bool {
    let vals = [
        state.left_depth,
        state.right_depth,
        state.left_tilt_rad,
        state.right_tilt_rad,
    ];
    vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn eoc_average_depth(state: &EyeOuterCornerState) -> f32 {
    (state.left_depth + state.right_depth) * 0.5
}

#[allow(dead_code)]
pub fn eoc_symmetry(state: &EyeOuterCornerState) -> f32 {
    (state.left_depth - state.right_depth).abs()
}

#[allow(dead_code)]
pub fn eoc_blend(a: &EyeOuterCornerState, b: &EyeOuterCornerState, t: f32) -> EyeOuterCornerState {
    let t = t.clamp(0.0, 1.0);
    EyeOuterCornerState {
        left_depth: a.left_depth + (b.left_depth - a.left_depth) * t,
        right_depth: a.right_depth + (b.right_depth - a.right_depth) * t,
        left_tilt_rad: a.left_tilt_rad + (b.left_tilt_rad - a.left_tilt_rad) * t,
        right_tilt_rad: a.right_tilt_rad + (b.right_tilt_rad - a.right_tilt_rad) * t,
    }
}

#[allow(dead_code)]
pub fn eoc_to_weights(state: &EyeOuterCornerState) -> Vec<(String, f32)> {
    use std::f32::consts::FRAC_PI_6;
    let norm = 1.0 / FRAC_PI_6;
    vec![
        ("eye_outer_depth_l".to_string(), state.left_depth),
        ("eye_outer_depth_r".to_string(), state.right_depth),
        ("eye_outer_tilt_l".to_string(), state.left_tilt_rad * norm),
        ("eye_outer_tilt_r".to_string(), state.right_tilt_rad * norm),
    ]
}

#[allow(dead_code)]
pub fn eoc_to_json(state: &EyeOuterCornerState) -> String {
    format!(
        r#"{{"left_depth":{:.4},"right_depth":{:.4},"left_tilt_rad":{:.4},"right_tilt_rad":{:.4}}}"#,
        state.left_depth, state.right_depth, state.left_tilt_rad, state.right_tilt_rad
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_eye_outer_corner_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_eye_outer_corner_state();
        assert!(eoc_is_neutral(&s));
    }

    #[test]
    fn set_depth_left() {
        let cfg = default_eye_outer_corner_config();
        let mut s = new_eye_outer_corner_state();
        eoc_set_depth(&mut s, &cfg, EyeOuterSide::Left, 0.6);
        assert!((s.left_depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_depth_clamps() {
        let cfg = default_eye_outer_corner_config();
        let mut s = new_eye_outer_corner_state();
        eoc_set_depth(&mut s, &cfg, EyeOuterSide::Right, 5.0);
        assert!((s.right_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_depth() {
        let cfg = default_eye_outer_corner_config();
        let mut s = new_eye_outer_corner_state();
        eoc_set_both_depth(&mut s, &cfg, 0.4);
        assert!(eoc_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_tilt_clamps() {
        let cfg = default_eye_outer_corner_config();
        let mut s = new_eye_outer_corner_state();
        eoc_set_tilt(&mut s, &cfg, EyeOuterSide::Left, 10.0);
        assert!((s.left_tilt_rad - cfg.max_tilt_rad).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_eye_outer_corner_config();
        let mut s = new_eye_outer_corner_state();
        eoc_set_both_depth(&mut s, &cfg, 0.8);
        eoc_reset(&mut s);
        assert!(eoc_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_eye_outer_corner_state();
        let cfg = default_eye_outer_corner_config();
        let mut b = new_eye_outer_corner_state();
        eoc_set_both_depth(&mut b, &cfg, 1.0);
        let m = eoc_blend(&a, &b, 0.5);
        assert!((m.left_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_eye_outer_corner_state();
        assert_eq!(eoc_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_eye_outer_corner_state();
        let j = eoc_to_json(&s);
        assert!(j.contains("left_depth"));
    }
}
