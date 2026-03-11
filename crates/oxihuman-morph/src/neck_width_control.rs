// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Neck width morph control: adjusts the lateral width and circumference of the neck.

use std::f32::consts::PI;

/// Configuration for neck width morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckWidthConfig {
    pub min_width: f32,
    pub max_width: f32,
}

/// Runtime state for neck width morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckWidthState {
    pub width: f32,
    pub front_depth: f32,
    pub trapezius_bulk: f32,
}

#[allow(dead_code)]
pub fn default_neck_width_config() -> NeckWidthConfig {
    NeckWidthConfig {
        min_width: 0.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_neck_width_state() -> NeckWidthState {
    NeckWidthState {
        width: 0.5,
        front_depth: 0.5,
        trapezius_bulk: 0.3,
    }
}

#[allow(dead_code)]
pub fn nw_set_width(state: &mut NeckWidthState, cfg: &NeckWidthConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn nw_set_front_depth(state: &mut NeckWidthState, v: f32) {
    state.front_depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nw_set_trapezius(state: &mut NeckWidthState, v: f32) {
    state.trapezius_bulk = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nw_reset(state: &mut NeckWidthState) {
    *state = new_neck_width_state();
}

#[allow(dead_code)]
pub fn nw_to_weights(state: &NeckWidthState) -> Vec<(String, f32)> {
    vec![
        ("neck_width".to_string(), state.width),
        ("neck_front_depth".to_string(), state.front_depth),
        ("neck_trapezius_bulk".to_string(), state.trapezius_bulk),
    ]
}

#[allow(dead_code)]
pub fn nw_to_json(state: &NeckWidthState) -> String {
    format!(
        r#"{{"width":{:.4},"front_depth":{:.4},"trapezius_bulk":{:.4}}}"#,
        state.width, state.front_depth, state.trapezius_bulk
    )
}

#[allow(dead_code)]
pub fn nw_blend(a: &NeckWidthState, b: &NeckWidthState, t: f32) -> NeckWidthState {
    let t = t.clamp(0.0, 1.0);
    NeckWidthState {
        width: a.width + (b.width - a.width) * t,
        front_depth: a.front_depth + (b.front_depth - a.front_depth) * t,
        trapezius_bulk: a.trapezius_bulk + (b.trapezius_bulk - a.trapezius_bulk) * t,
    }
}

/// Approximate circumference assuming ellipse cross-section.
#[allow(dead_code)]
pub fn nw_circumference(state: &NeckWidthState) -> f32 {
    let a = state.width * 0.5;
    let b = state.front_depth * 0.5;
    PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_neck_width_config();
        assert!(cfg.min_width.abs() < 1e-6);
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_neck_width_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.trapezius_bulk - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_neck_width_config();
        let mut s = new_neck_width_state();
        nw_set_width(&mut s, &cfg, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_front_depth() {
        let mut s = new_neck_width_state();
        nw_set_front_depth(&mut s, 0.8);
        assert!((s.front_depth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_trapezius() {
        let mut s = new_neck_width_state();
        nw_set_trapezius(&mut s, 0.7);
        assert!((s.trapezius_bulk - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_neck_width_config();
        let mut s = new_neck_width_state();
        nw_set_width(&mut s, &cfg, 0.9);
        nw_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_neck_width_state();
        assert_eq!(nw_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_neck_width_state();
        let j = nw_to_json(&s);
        assert!(j.contains("width"));
    }

    #[test]
    fn test_blend() {
        let a = new_neck_width_state();
        let mut b = new_neck_width_state();
        b.width = 1.0;
        let mid = nw_blend(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_circumference() {
        let s = new_neck_width_state();
        let c = nw_circumference(&s);
        assert!(c > 0.0);
    }
}
