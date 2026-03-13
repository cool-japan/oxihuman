// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Hand palm control: adjusts width, thickness and arch of the palm.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandPalmConfig {
    pub min_width: f32,
    pub max_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandPalmState {
    pub width: f32,
    pub thickness: f32,
    pub arch: f32,
}

#[allow(dead_code)]
pub fn default_hand_palm_config() -> HandPalmConfig {
    HandPalmConfig {
        min_width: 0.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_hand_palm_state() -> HandPalmState {
    HandPalmState {
        width: 0.5,
        thickness: 0.4,
        arch: 0.3,
    }
}

#[allow(dead_code)]
pub fn hp_set_width(state: &mut HandPalmState, cfg: &HandPalmConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn hp_set_thickness(state: &mut HandPalmState, v: f32) {
    state.thickness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hp_set_arch(state: &mut HandPalmState, v: f32) {
    state.arch = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hp_reset(state: &mut HandPalmState) {
    *state = new_hand_palm_state();
}

/// Approximate cross-section area of the palm.
#[allow(dead_code)]
pub fn hp_cross_section(state: &HandPalmState) -> f32 {
    PI * state.width * state.thickness * 0.25
}

#[allow(dead_code)]
pub fn hp_to_weights(state: &HandPalmState) -> Vec<(String, f32)> {
    vec![
        ("hand_palm_width".to_string(), state.width),
        ("hand_palm_thickness".to_string(), state.thickness),
        ("hand_palm_arch".to_string(), state.arch),
    ]
}

#[allow(dead_code)]
pub fn hp_to_json(state: &HandPalmState) -> String {
    format!(
        r#"{{"width":{:.4},"thickness":{:.4},"arch":{:.4}}}"#,
        state.width, state.thickness, state.arch
    )
}

#[allow(dead_code)]
pub fn hp_blend(a: &HandPalmState, b: &HandPalmState, t: f32) -> HandPalmState {
    let t = t.clamp(0.0, 1.0);
    HandPalmState {
        width: a.width + (b.width - a.width) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        arch: a.arch + (b.arch - a.arch) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_hand_palm_config();
        assert!(cfg.min_width.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_hand_palm_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_hand_palm_config();
        let mut s = new_hand_palm_state();
        hp_set_width(&mut s, &cfg, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut s = new_hand_palm_state();
        hp_set_thickness(&mut s, 0.7);
        assert!((s.thickness - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch() {
        let mut s = new_hand_palm_state();
        hp_set_arch(&mut s, 0.6);
        assert!((s.arch - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_hand_palm_config();
        let mut s = new_hand_palm_state();
        hp_set_width(&mut s, &cfg, 0.9);
        hp_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cross_section() {
        let s = new_hand_palm_state();
        assert!(hp_cross_section(&s) > 0.0);
    }

    #[test]
    fn test_to_weights() {
        let s = new_hand_palm_state();
        assert_eq!(hp_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_hand_palm_state();
        let mut b = new_hand_palm_state();
        b.width = 1.0;
        let mid = hp_blend(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_hand_palm_state();
        let j = hp_to_json(&s);
        assert!(j.contains("width"));
    }
}
