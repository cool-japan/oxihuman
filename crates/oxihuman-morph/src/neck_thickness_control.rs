// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Neck width/thickness morph control.

#![allow(dead_code)]

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckThicknessConfig {
    pub min_radius: f32,
    pub max_radius: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckThicknessState {
    pub width: f32,
    pub depth: f32,
    pub length: f32,
}

#[allow(dead_code)]
pub fn default_neck_thickness_config() -> NeckThicknessConfig {
    NeckThicknessConfig {
        min_radius: 0.1,
        max_radius: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_neck_thickness_state() -> NeckThicknessState {
    NeckThicknessState {
        width: 0.4,
        depth: 0.4,
        length: 0.5,
    }
}

#[allow(dead_code)]
pub fn neck_set_width(state: &mut NeckThicknessState, cfg: &NeckThicknessConfig, value: f32) {
    state.width = value.clamp(cfg.min_radius, cfg.max_radius);
}

#[allow(dead_code)]
pub fn neck_set_depth(state: &mut NeckThicknessState, cfg: &NeckThicknessConfig, value: f32) {
    state.depth = value.clamp(cfg.min_radius, cfg.max_radius);
}

#[allow(dead_code)]
pub fn neck_set_length(state: &mut NeckThicknessState, value: f32) {
    state.length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn neck_reset(state: &mut NeckThicknessState) {
    *state = new_neck_thickness_state();
}

#[allow(dead_code)]
pub fn neck_to_weights(state: &NeckThicknessState) -> Vec<(String, f32)> {
    vec![
        ("neck_width".to_string(), state.width),
        ("neck_depth".to_string(), state.depth),
        ("neck_length".to_string(), state.length),
    ]
}

#[allow(dead_code)]
pub fn neck_to_json(state: &NeckThicknessState) -> String {
    format!(
        r#"{{"width":{:.4},"depth":{:.4},"length":{:.4}}}"#,
        state.width, state.depth, state.length
    )
}

#[allow(dead_code)]
pub fn neck_clamp(state: &mut NeckThicknessState, cfg: &NeckThicknessConfig) {
    state.width = state.width.clamp(cfg.min_radius, cfg.max_radius);
    state.depth = state.depth.clamp(cfg.min_radius, cfg.max_radius);
    state.length = state.length.clamp(0.0, 1.0);
}

/// Approximate volume of neck as an elliptic cylinder (V = π * a * b * h).
#[allow(dead_code)]
pub fn neck_compute_volume(state: &NeckThicknessState, height_m: f32) -> f32 {
    PI * state.width * state.depth * (state.length * height_m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_neck_thickness_config();
        assert!((cfg.min_radius - 0.1).abs() < 1e-6);
        assert_eq!(cfg.max_radius, 1.0);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_neck_thickness_state();
        assert!((s.width - 0.4).abs() < 1e-6);
        assert!((s.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_neck_thickness_config();
        let mut s = new_neck_thickness_state();
        neck_set_width(&mut s, &cfg, 0.0);
        assert!((s.width - cfg.min_radius).abs() < 1e-6);
        neck_set_width(&mut s, &cfg, 5.0);
        assert_eq!(s.width, cfg.max_radius);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_neck_thickness_config();
        let mut s = new_neck_thickness_state();
        neck_set_depth(&mut s, &cfg, 0.6);
        assert!((s.depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamps() {
        let mut s = new_neck_thickness_state();
        neck_set_length(&mut s, 2.0);
        assert_eq!(s.length, 1.0);
        neck_set_length(&mut s, -1.0);
        assert_eq!(s.length, 0.0);
    }

    #[test]
    fn test_reset() {
        let cfg = default_neck_thickness_config();
        let mut s = new_neck_thickness_state();
        neck_set_width(&mut s, &cfg, 0.9);
        neck_reset(&mut s);
        assert!((s.width - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_neck_thickness_state();
        assert_eq!(neck_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json_has_keys() {
        let s = new_neck_thickness_state();
        let j = neck_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("length"));
    }

    #[test]
    fn test_compute_volume_positive() {
        let s = new_neck_thickness_state();
        let v = neck_compute_volume(&s, 0.15);
        assert!(v > 0.0);
    }

    #[test]
    fn test_clamp_enforces_bounds() {
        let cfg = default_neck_thickness_config();
        let mut s = NeckThicknessState {
            width: 0.0,
            depth: 5.0,
            length: -1.0,
        };
        neck_clamp(&mut s, &cfg);
        assert!((s.width - cfg.min_radius).abs() < 1e-6);
        assert_eq!(s.depth, cfg.max_radius);
        assert_eq!(s.length, 0.0);
    }
}
