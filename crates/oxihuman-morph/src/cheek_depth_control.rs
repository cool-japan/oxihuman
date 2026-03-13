// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Cheek depth morph control: adjusts how deep or hollow cheeks appear.

/// Configuration for cheek depth morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekDepthConfig {
    pub min_depth: f32,
    pub max_depth: f32,
}

/// Runtime state for cheek depth morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekDepthState {
    pub left_depth: f32,
    pub right_depth: f32,
    pub hollow_factor: f32,
}

#[allow(dead_code)]
pub fn default_cheek_depth_config() -> CheekDepthConfig {
    CheekDepthConfig {
        min_depth: -1.0,
        max_depth: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_cheek_depth_state() -> CheekDepthState {
    CheekDepthState {
        left_depth: 0.0,
        right_depth: 0.0,
        hollow_factor: 0.0,
    }
}

#[allow(dead_code)]
pub fn cd_set_left(state: &mut CheekDepthState, cfg: &CheekDepthConfig, v: f32) {
    state.left_depth = v.clamp(cfg.min_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn cd_set_right(state: &mut CheekDepthState, cfg: &CheekDepthConfig, v: f32) {
    state.right_depth = v.clamp(cfg.min_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn cd_set_both(state: &mut CheekDepthState, cfg: &CheekDepthConfig, v: f32) {
    let clamped = v.clamp(cfg.min_depth, cfg.max_depth);
    state.left_depth = clamped;
    state.right_depth = clamped;
}

#[allow(dead_code)]
pub fn cd_set_hollow(state: &mut CheekDepthState, v: f32) {
    state.hollow_factor = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cd_reset(state: &mut CheekDepthState) {
    *state = new_cheek_depth_state();
}

#[allow(dead_code)]
pub fn cd_to_weights(state: &CheekDepthState) -> Vec<(String, f32)> {
    vec![
        ("cheek_depth_left".to_string(), state.left_depth),
        ("cheek_depth_right".to_string(), state.right_depth),
        ("cheek_depth_hollow".to_string(), state.hollow_factor),
    ]
}

#[allow(dead_code)]
pub fn cd_to_json(state: &CheekDepthState) -> String {
    format!(
        r#"{{"left_depth":{:.4},"right_depth":{:.4},"hollow_factor":{:.4}}}"#,
        state.left_depth, state.right_depth, state.hollow_factor
    )
}

#[allow(dead_code)]
pub fn cd_blend(a: &CheekDepthState, b: &CheekDepthState, t: f32) -> CheekDepthState {
    let t = t.clamp(0.0, 1.0);
    CheekDepthState {
        left_depth: a.left_depth + (b.left_depth - a.left_depth) * t,
        right_depth: a.right_depth + (b.right_depth - a.right_depth) * t,
        hollow_factor: a.hollow_factor + (b.hollow_factor - a.hollow_factor) * t,
    }
}

#[allow(dead_code)]
pub fn cd_effective_depth(state: &CheekDepthState) -> f32 {
    (state.left_depth + state.right_depth) * 0.5 - state.hollow_factor * 0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheek_depth_config();
        assert!((cfg.min_depth + 1.0).abs() < 1e-6);
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_cheek_depth_state();
        assert!(s.left_depth.abs() < 1e-6);
        assert!(s.hollow_factor.abs() < 1e-6);
    }

    #[test]
    fn test_set_left_clamps() {
        let cfg = default_cheek_depth_config();
        let mut s = new_cheek_depth_state();
        cd_set_left(&mut s, &cfg, 5.0);
        assert!((s.left_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_both() {
        let cfg = default_cheek_depth_config();
        let mut s = new_cheek_depth_state();
        cd_set_both(&mut s, &cfg, 0.5);
        assert!((s.left_depth - 0.5).abs() < 1e-6);
        assert!((s.right_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_hollow() {
        let mut s = new_cheek_depth_state();
        cd_set_hollow(&mut s, 0.7);
        assert!((s.hollow_factor - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_cheek_depth_config();
        let mut s = new_cheek_depth_state();
        cd_set_left(&mut s, &cfg, 0.5);
        cd_reset(&mut s);
        assert!(s.left_depth.abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_cheek_depth_state();
        assert_eq!(cd_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_cheek_depth_state();
        let j = cd_to_json(&s);
        assert!(j.contains("left_depth"));
    }

    #[test]
    fn test_blend() {
        let a = new_cheek_depth_state();
        let mut b = new_cheek_depth_state();
        b.left_depth = 1.0;
        let mid = cd_blend(&a, &b, 0.5);
        assert!((mid.left_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_effective_depth() {
        let s = new_cheek_depth_state();
        let d = cd_effective_depth(&s);
        assert!(d.abs() < 1e-6);
    }
}
