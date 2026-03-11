// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Eye socket depth/shape morph control (depth, width, brow height per side).

#![allow(dead_code)]

/// Configuration for eye socket morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSocketConfig {
    pub max_depth: f32,
    pub max_width: f32,
}

/// Runtime state for eye socket morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSocketState {
    pub depth_l: f32,
    pub depth_r: f32,
    pub width_l: f32,
    pub width_r: f32,
    pub brow_height_l: f32,
    pub brow_height_r: f32,
}

#[allow(dead_code)]
pub fn default_eye_socket_config() -> EyeSocketConfig {
    EyeSocketConfig { max_depth: 1.0, max_width: 1.0 }
}

#[allow(dead_code)]
pub fn new_eye_socket_state() -> EyeSocketState {
    EyeSocketState {
        depth_l: 0.0,
        depth_r: 0.0,
        width_l: 0.0,
        width_r: 0.0,
        brow_height_l: 0.0,
        brow_height_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn eye_socket_set_depth(state: &mut EyeSocketState, cfg: &EyeSocketConfig, left: f32, right: f32) {
    state.depth_l = left.clamp(0.0, cfg.max_depth);
    state.depth_r = right.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn eye_socket_set_width(state: &mut EyeSocketState, cfg: &EyeSocketConfig, left: f32, right: f32) {
    state.width_l = left.clamp(0.0, cfg.max_width);
    state.width_r = right.clamp(0.0, cfg.max_width);
}

#[allow(dead_code)]
pub fn eye_socket_set_brow(state: &mut EyeSocketState, left: f32, right: f32) {
    state.brow_height_l = left.clamp(-1.0, 1.0);
    state.brow_height_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn eye_socket_mirror(state: &mut EyeSocketState) {
    let avg_d = (state.depth_l + state.depth_r) * 0.5;
    let avg_w = (state.width_l + state.width_r) * 0.5;
    let avg_b = (state.brow_height_l + state.brow_height_r) * 0.5;
    state.depth_l = avg_d;
    state.depth_r = avg_d;
    state.width_l = avg_w;
    state.width_r = avg_w;
    state.brow_height_l = avg_b;
    state.brow_height_r = avg_b;
}

#[allow(dead_code)]
pub fn eye_socket_reset(state: &mut EyeSocketState) {
    *state = new_eye_socket_state();
}

#[allow(dead_code)]
pub fn eye_socket_to_weights(state: &EyeSocketState) -> Vec<(String, f32)> {
    vec![
        ("eye_socket_depth_l".to_string(), state.depth_l),
        ("eye_socket_depth_r".to_string(), state.depth_r),
        ("eye_socket_width_l".to_string(), state.width_l),
        ("eye_socket_width_r".to_string(), state.width_r),
        ("eye_socket_brow_l".to_string(), state.brow_height_l),
        ("eye_socket_brow_r".to_string(), state.brow_height_r),
    ]
}

#[allow(dead_code)]
pub fn eye_socket_to_json(state: &EyeSocketState) -> String {
    format!(
        r#"{{"depth_l":{:.4},"depth_r":{:.4},"width_l":{:.4},"width_r":{:.4},"brow_height_l":{:.4},"brow_height_r":{:.4}}}"#,
        state.depth_l, state.depth_r, state.width_l, state.width_r,
        state.brow_height_l, state.brow_height_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_socket_config();
        assert_eq!(cfg.max_depth, 1.0);
        assert_eq!(cfg.max_width, 1.0);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_eye_socket_state();
        assert_eq!(s.depth_l, 0.0);
        assert_eq!(s.depth_r, 0.0);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_eye_socket_config();
        let mut s = new_eye_socket_state();
        eye_socket_set_depth(&mut s, &cfg, 2.0, -1.0);
        assert_eq!(s.depth_l, 1.0);
        assert_eq!(s.depth_r, 0.0);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_eye_socket_config();
        let mut s = new_eye_socket_state();
        eye_socket_set_width(&mut s, &cfg, 0.5, 0.7);
        assert!((s.width_l - 0.5).abs() < 1e-6);
        assert!((s.width_r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_brow_clamps() {
        let mut s = new_eye_socket_state();
        eye_socket_set_brow(&mut s, -2.0, 2.0);
        assert_eq!(s.brow_height_l, -1.0);
        assert_eq!(s.brow_height_r, 1.0);
    }

    #[test]
    fn test_mirror() {
        let mut s = new_eye_socket_state();
        s.depth_l = 0.4;
        s.depth_r = 0.8;
        eye_socket_mirror(&mut s);
        assert!((s.depth_l - 0.6).abs() < 1e-6);
        assert!((s.depth_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_eye_socket_config();
        let mut s = new_eye_socket_state();
        eye_socket_set_depth(&mut s, &cfg, 0.5, 0.5);
        eye_socket_reset(&mut s);
        assert_eq!(s.depth_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_eye_socket_state();
        let w = eye_socket_to_weights(&s);
        assert_eq!(w.len(), 6);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_eye_socket_state();
        let j = eye_socket_to_json(&s);
        assert!(j.contains("depth_l"));
        assert!(j.contains("brow_height_r"));
    }

    #[test]
    fn test_set_depth_valid() {
        let cfg = default_eye_socket_config();
        let mut s = new_eye_socket_state();
        eye_socket_set_depth(&mut s, &cfg, 0.3, 0.6);
        assert!((s.depth_l - 0.3).abs() < 1e-6);
        assert!((s.depth_r - 0.6).abs() < 1e-6);
    }
}
