// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eye inner corner morph — controls the shape and depth of the inner eye canthus.

/// Configuration for eye inner corner control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeInnerCornerConfig {
    pub max_depth: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeInnerCornerState {
    pub left_depth: f32,
    pub right_depth: f32,
    pub left_tilt: f32,
    pub right_tilt: f32,
}

#[allow(dead_code)]
pub fn default_eye_inner_corner_config() -> EyeInnerCornerConfig {
    EyeInnerCornerConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_eye_inner_corner_state() -> EyeInnerCornerState {
    EyeInnerCornerState {
        left_depth: 0.0,
        right_depth: 0.0,
        left_tilt: 0.0,
        right_tilt: 0.0,
    }
}

#[allow(dead_code)]
pub fn eic_set_depth_left(state: &mut EyeInnerCornerState, cfg: &EyeInnerCornerConfig, v: f32) {
    state.left_depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn eic_set_depth_right(state: &mut EyeInnerCornerState, cfg: &EyeInnerCornerConfig, v: f32) {
    state.right_depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn eic_set_both_depth(state: &mut EyeInnerCornerState, cfg: &EyeInnerCornerConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_depth);
    state.left_depth = clamped;
    state.right_depth = clamped;
}

#[allow(dead_code)]
pub fn eic_set_tilt(state: &mut EyeInnerCornerState, left: f32, right: f32) {
    state.left_tilt = left.clamp(-1.0, 1.0);
    state.right_tilt = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn eic_reset(state: &mut EyeInnerCornerState) {
    *state = new_eye_inner_corner_state();
}

#[allow(dead_code)]
pub fn eic_is_neutral(state: &EyeInnerCornerState) -> bool {
    let vals = [
        state.left_depth,
        state.right_depth,
        state.left_tilt,
        state.right_tilt,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn eic_average_depth(state: &EyeInnerCornerState) -> f32 {
    (state.left_depth + state.right_depth) * 0.5
}

#[allow(dead_code)]
pub fn eic_symmetry(state: &EyeInnerCornerState) -> f32 {
    (state.left_depth - state.right_depth).abs()
}

#[allow(dead_code)]
pub fn eic_blend(a: &EyeInnerCornerState, b: &EyeInnerCornerState, t: f32) -> EyeInnerCornerState {
    let t = t.clamp(0.0, 1.0);
    EyeInnerCornerState {
        left_depth: a.left_depth + (b.left_depth - a.left_depth) * t,
        right_depth: a.right_depth + (b.right_depth - a.right_depth) * t,
        left_tilt: a.left_tilt + (b.left_tilt - a.left_tilt) * t,
        right_tilt: a.right_tilt + (b.right_tilt - a.right_tilt) * t,
    }
}

#[allow(dead_code)]
pub fn eic_to_weights(state: &EyeInnerCornerState) -> Vec<(String, f32)> {
    vec![
        ("eye_inner_depth_l".to_string(), state.left_depth),
        ("eye_inner_depth_r".to_string(), state.right_depth),
        ("eye_inner_tilt_l".to_string(), state.left_tilt),
        ("eye_inner_tilt_r".to_string(), state.right_tilt),
    ]
}

#[allow(dead_code)]
pub fn eic_to_json(state: &EyeInnerCornerState) -> String {
    format!(
        r#"{{"left_depth":{:.4},"right_depth":{:.4},"left_tilt":{:.4},"right_tilt":{:.4}}}"#,
        state.left_depth, state.right_depth, state.left_tilt, state.right_tilt
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_eye_inner_corner_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_eye_inner_corner_state();
        assert!(eic_is_neutral(&s));
    }

    #[test]
    fn set_depth_left_clamps() {
        let cfg = default_eye_inner_corner_config();
        let mut s = new_eye_inner_corner_state();
        eic_set_depth_left(&mut s, &cfg, 5.0);
        assert!((s.left_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_depth_equal() {
        let cfg = default_eye_inner_corner_config();
        let mut s = new_eye_inner_corner_state();
        eic_set_both_depth(&mut s, &cfg, 0.7);
        assert!((s.left_depth - 0.7).abs() < 1e-6);
        assert!((s.right_depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_tilt_clamped() {
        let mut s = new_eye_inner_corner_state();
        eic_set_tilt(&mut s, 2.0, -3.0);
        assert!((s.left_tilt - 1.0).abs() < 1e-6);
        assert!((s.right_tilt + 1.0).abs() < 1e-6);
    }

    #[test]
    fn average_depth() {
        let cfg = default_eye_inner_corner_config();
        let mut s = new_eye_inner_corner_state();
        eic_set_depth_left(&mut s, &cfg, 0.4);
        eic_set_depth_right(&mut s, &cfg, 0.6);
        assert!((eic_average_depth(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn symmetry_zero_when_equal() {
        let cfg = default_eye_inner_corner_config();
        let mut s = new_eye_inner_corner_state();
        eic_set_both_depth(&mut s, &cfg, 0.5);
        assert!(eic_symmetry(&s) < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_eye_inner_corner_config();
        let mut s = new_eye_inner_corner_state();
        eic_set_both_depth(&mut s, &cfg, 0.8);
        eic_reset(&mut s);
        assert!(eic_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_eye_inner_corner_state();
        let cfg = default_eye_inner_corner_config();
        let mut b = new_eye_inner_corner_state();
        eic_set_both_depth(&mut b, &cfg, 1.0);
        let mid = eic_blend(&a, &b, 0.5);
        assert!((mid.left_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_eye_inner_corner_state();
        assert_eq!(eic_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_eye_inner_corner_state();
        let j = eic_to_json(&s);
        assert!(j.contains("left_depth"));
        assert!(j.contains("right_tilt"));
    }
}
