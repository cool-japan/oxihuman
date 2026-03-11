// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cheek puff depth morph — controls how deeply cheeks puff outward.

/// Configuration for cheek puff depth.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekPuffDepthConfig {
    pub max_depth: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekPuffDepthState {
    pub left_depth: f32,
    pub right_depth: f32,
    pub upper_bias: f32,
    pub lower_bias: f32,
}

#[allow(dead_code)]
pub fn default_cheek_puff_depth_config() -> CheekPuffDepthConfig {
    CheekPuffDepthConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_cheek_puff_depth_state() -> CheekPuffDepthState {
    CheekPuffDepthState {
        left_depth: 0.0,
        right_depth: 0.0,
        upper_bias: 0.0,
        lower_bias: 0.0,
    }
}

#[allow(dead_code)]
pub fn cpd_set_left(state: &mut CheekPuffDepthState, cfg: &CheekPuffDepthConfig, v: f32) {
    state.left_depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn cpd_set_right(state: &mut CheekPuffDepthState, cfg: &CheekPuffDepthConfig, v: f32) {
    state.right_depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn cpd_set_both(state: &mut CheekPuffDepthState, cfg: &CheekPuffDepthConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_depth);
    state.left_depth = clamped;
    state.right_depth = clamped;
}

#[allow(dead_code)]
pub fn cpd_set_bias(state: &mut CheekPuffDepthState, upper: f32, lower: f32) {
    state.upper_bias = upper.clamp(0.0, 1.0);
    state.lower_bias = lower.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cpd_reset(state: &mut CheekPuffDepthState) {
    *state = new_cheek_puff_depth_state();
}

#[allow(dead_code)]
pub fn cpd_is_neutral(state: &CheekPuffDepthState) -> bool {
    let vals = [
        state.left_depth,
        state.right_depth,
        state.upper_bias,
        state.lower_bias,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn cpd_average_depth(state: &CheekPuffDepthState) -> f32 {
    (state.left_depth + state.right_depth) * 0.5
}

#[allow(dead_code)]
pub fn cpd_symmetry(state: &CheekPuffDepthState) -> f32 {
    (state.left_depth - state.right_depth).abs()
}

#[allow(dead_code)]
pub fn cpd_blend(a: &CheekPuffDepthState, b: &CheekPuffDepthState, t: f32) -> CheekPuffDepthState {
    let t = t.clamp(0.0, 1.0);
    CheekPuffDepthState {
        left_depth: a.left_depth + (b.left_depth - a.left_depth) * t,
        right_depth: a.right_depth + (b.right_depth - a.right_depth) * t,
        upper_bias: a.upper_bias + (b.upper_bias - a.upper_bias) * t,
        lower_bias: a.lower_bias + (b.lower_bias - a.lower_bias) * t,
    }
}

#[allow(dead_code)]
pub fn cpd_to_weights(state: &CheekPuffDepthState) -> Vec<(String, f32)> {
    vec![
        ("cheek_puff_depth_l".to_string(), state.left_depth),
        ("cheek_puff_depth_r".to_string(), state.right_depth),
        ("cheek_puff_upper".to_string(), state.upper_bias),
        ("cheek_puff_lower".to_string(), state.lower_bias),
    ]
}

#[allow(dead_code)]
pub fn cpd_to_json(state: &CheekPuffDepthState) -> String {
    format!(
        r#"{{"left_depth":{:.4},"right_depth":{:.4},"upper_bias":{:.4},"lower_bias":{:.4}}}"#,
        state.left_depth, state.right_depth, state.upper_bias, state.lower_bias
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_cheek_puff_depth_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_cheek_puff_depth_state();
        assert!(cpd_is_neutral(&s));
    }

    #[test]
    fn set_left_clamps() {
        let cfg = default_cheek_puff_depth_config();
        let mut s = new_cheek_puff_depth_state();
        cpd_set_left(&mut s, &cfg, 3.0);
        assert!((s.left_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_equal() {
        let cfg = default_cheek_puff_depth_config();
        let mut s = new_cheek_puff_depth_state();
        cpd_set_both(&mut s, &cfg, 0.6);
        assert!((s.left_depth - 0.6).abs() < 1e-6);
        assert!((s.right_depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_bias_clamped() {
        let mut s = new_cheek_puff_depth_state();
        cpd_set_bias(&mut s, 2.0, -1.0);
        assert!((s.upper_bias - 1.0).abs() < 1e-6);
        assert_eq!(s.lower_bias, 0.0);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_cheek_puff_depth_config();
        let mut s = new_cheek_puff_depth_state();
        cpd_set_left(&mut s, &cfg, 0.5);
        cpd_reset(&mut s);
        assert!(cpd_is_neutral(&s));
    }

    #[test]
    fn average_depth() {
        let cfg = default_cheek_puff_depth_config();
        let mut s = new_cheek_puff_depth_state();
        cpd_set_left(&mut s, &cfg, 0.4);
        cpd_set_right(&mut s, &cfg, 0.6);
        assert!((cpd_average_depth(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn symmetry_difference() {
        let cfg = default_cheek_puff_depth_config();
        let mut s = new_cheek_puff_depth_state();
        cpd_set_left(&mut s, &cfg, 0.3);
        cpd_set_right(&mut s, &cfg, 0.7);
        assert!((cpd_symmetry(&s) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let a = new_cheek_puff_depth_state();
        let cfg = default_cheek_puff_depth_config();
        let mut b = new_cheek_puff_depth_state();
        cpd_set_both(&mut b, &cfg, 1.0);
        let mid = cpd_blend(&a, &b, 0.5);
        assert!((mid.left_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_cheek_puff_depth_state();
        assert_eq!(cpd_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_cheek_puff_depth_state();
        let j = cpd_to_json(&s);
        assert!(j.contains("left_depth"));
        assert!(j.contains("lower_bias"));
    }
}
