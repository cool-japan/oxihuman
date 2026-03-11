// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Eye depth (deep-set vs prominent eye) morph.

#![allow(dead_code)]

/// Configuration for eye depth morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeDepthConfig {
    pub max_depth: f32,
}

/// Runtime state for eye depth morph.
/// Positive = deep-set, negative = prominent.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeDepthState {
    pub depth_l: f32,
    pub depth_r: f32,
}

#[allow(dead_code)]
pub fn default_eye_depth_config() -> EyeDepthConfig {
    EyeDepthConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_eye_depth_state() -> EyeDepthState {
    EyeDepthState {
        depth_l: 0.0,
        depth_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn ed_set_depth_left(state: &mut EyeDepthState, cfg: &EyeDepthConfig, v: f32) {
    state.depth_l = v.clamp(-cfg.max_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn ed_set_depth_right(state: &mut EyeDepthState, cfg: &EyeDepthConfig, v: f32) {
    state.depth_r = v.clamp(-cfg.max_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn ed_set_depth_both(state: &mut EyeDepthState, cfg: &EyeDepthConfig, v: f32) {
    let clamped = v.clamp(-cfg.max_depth, cfg.max_depth);
    state.depth_l = clamped;
    state.depth_r = clamped;
}

#[allow(dead_code)]
pub fn ed_mirror(state: &mut EyeDepthState) {
    let avg = (state.depth_l + state.depth_r) * 0.5;
    state.depth_l = avg;
    state.depth_r = avg;
}

#[allow(dead_code)]
pub fn ed_reset(state: &mut EyeDepthState) {
    *state = new_eye_depth_state();
}

#[allow(dead_code)]
pub fn ed_to_weights(state: &EyeDepthState) -> Vec<(String, f32)> {
    vec![
        ("eye_depth_l".to_string(), state.depth_l),
        ("eye_depth_r".to_string(), state.depth_r),
    ]
}

#[allow(dead_code)]
pub fn ed_to_json(state: &EyeDepthState) -> String {
    format!(
        r#"{{"depth_l":{:.4},"depth_r":{:.4}}}"#,
        state.depth_l, state.depth_r
    )
}

#[allow(dead_code)]
pub fn ed_clamp(state: &mut EyeDepthState, cfg: &EyeDepthConfig) {
    state.depth_l = state.depth_l.clamp(-cfg.max_depth, cfg.max_depth);
    state.depth_r = state.depth_r.clamp(-cfg.max_depth, cfg.max_depth);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_depth_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_eye_depth_state();
        assert_eq!(s.depth_l, 0.0);
        assert_eq!(s.depth_r, 0.0);
    }

    #[test]
    fn test_set_depth_left_clamps() {
        let cfg = default_eye_depth_config();
        let mut s = new_eye_depth_state();
        ed_set_depth_left(&mut s, &cfg, 3.0);
        assert!((s.depth_l - 1.0).abs() < 1e-6);
        ed_set_depth_left(&mut s, &cfg, -3.0);
        assert!((s.depth_l + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_right() {
        let cfg = default_eye_depth_config();
        let mut s = new_eye_depth_state();
        ed_set_depth_right(&mut s, &cfg, 0.5);
        assert!((s.depth_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_both() {
        let cfg = default_eye_depth_config();
        let mut s = new_eye_depth_state();
        ed_set_depth_both(&mut s, &cfg, -0.7);
        assert!((s.depth_l + 0.7).abs() < 1e-6);
        assert!((s.depth_r + 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_mirror() {
        let cfg = default_eye_depth_config();
        let mut s = new_eye_depth_state();
        ed_set_depth_left(&mut s, &cfg, 0.4);
        ed_set_depth_right(&mut s, &cfg, 0.8);
        ed_mirror(&mut s);
        assert!((s.depth_l - 0.6).abs() < 1e-6);
        assert!((s.depth_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_eye_depth_config();
        let mut s = new_eye_depth_state();
        ed_set_depth_both(&mut s, &cfg, 0.5);
        ed_reset(&mut s);
        assert_eq!(s.depth_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_eye_depth_state();
        assert_eq!(ed_to_weights(&s).len(), 2);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_eye_depth_state();
        let j = ed_to_json(&s);
        assert!(j.contains("depth_l"));
        assert!(j.contains("depth_r"));
    }
}
