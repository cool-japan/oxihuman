// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hip/pelvis shape morph control.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HipConfig {
    pub max_width: f32,
    pub max_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HipState {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub tilt: f32,
}

#[allow(dead_code)]
pub fn default_hip_config() -> HipConfig {
    HipConfig { max_width: 1.0, max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_hip_state() -> HipState {
    HipState { width: 0.5, depth: 0.5, height: 0.5, tilt: 0.0 }
}

#[allow(dead_code)]
pub fn hip_set_width(state: &mut HipState, cfg: &HipConfig, value: f32) {
    state.width = value.clamp(0.0, cfg.max_width);
}

#[allow(dead_code)]
pub fn hip_set_depth(state: &mut HipState, cfg: &HipConfig, value: f32) {
    state.depth = value.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn hip_set_height(state: &mut HipState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hip_set_tilt(state: &mut HipState, value: f32) {
    state.tilt = value.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn hip_reset(state: &mut HipState) {
    *state = new_hip_state();
}

#[allow(dead_code)]
pub fn hip_to_weights(state: &HipState) -> Vec<(String, f32)> {
    vec![
        ("hip_width".to_string(), state.width),
        ("hip_depth".to_string(), state.depth),
        ("hip_height".to_string(), state.height),
        ("hip_tilt".to_string(), state.tilt),
    ]
}

#[allow(dead_code)]
pub fn hip_to_json(state: &HipState) -> String {
    format!(
        r#"{{"width":{:.4},"depth":{:.4},"height":{:.4},"tilt":{:.4}}}"#,
        state.width, state.depth, state.height, state.tilt
    )
}

#[allow(dead_code)]
pub fn hip_clamp(state: &mut HipState, cfg: &HipConfig) {
    state.width = state.width.clamp(0.0, cfg.max_width);
    state.depth = state.depth.clamp(0.0, cfg.max_depth);
    state.height = state.height.clamp(0.0, 1.0);
    state.tilt = state.tilt.clamp(-1.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_hip_config();
        assert_eq!(cfg.max_width, 1.0);
        assert_eq!(cfg.max_depth, 1.0);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_hip_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert_eq!(s.tilt, 0.0);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_hip_config();
        let mut s = new_hip_state();
        hip_set_width(&mut s, &cfg, 2.0);
        assert_eq!(s.width, 1.0);
        hip_set_width(&mut s, &cfg, -1.0);
        assert_eq!(s.width, 0.0);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_hip_config();
        let mut s = new_hip_state();
        hip_set_depth(&mut s, &cfg, 0.7);
        assert!((s.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamps() {
        let mut s = new_hip_state();
        hip_set_height(&mut s, 1.5);
        assert_eq!(s.height, 1.0);
        hip_set_height(&mut s, -0.5);
        assert_eq!(s.height, 0.0);
    }

    #[test]
    fn test_set_tilt_clamps() {
        let mut s = new_hip_state();
        hip_set_tilt(&mut s, 2.0);
        assert_eq!(s.tilt, 1.0);
        hip_set_tilt(&mut s, -2.0);
        assert_eq!(s.tilt, -1.0);
    }

    #[test]
    fn test_reset() {
        let cfg = default_hip_config();
        let mut s = new_hip_state();
        hip_set_width(&mut s, &cfg, 0.9);
        hip_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_hip_state();
        assert_eq!(hip_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_to_json_contains_keys() {
        let s = new_hip_state();
        let j = hip_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("tilt"));
    }

    #[test]
    fn test_clamp_enforces_bounds() {
        let cfg = default_hip_config();
        let mut s = HipState { width: 3.0, depth: -1.0, height: 5.0, tilt: -5.0 };
        hip_clamp(&mut s, &cfg);
        assert_eq!(s.width, 1.0);
        assert_eq!(s.depth, 0.0);
        assert_eq!(s.height, 1.0);
        assert_eq!(s.tilt, -1.0);
    }
}
