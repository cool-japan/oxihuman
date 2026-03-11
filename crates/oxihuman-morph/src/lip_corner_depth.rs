// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Lip corner depth morph controls for commissure positioning.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCornerDepthConfig {
    pub depth: f32,
    pub droop: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCornerDepthState {
    pub depth: f32,
    pub droop: f32,
    pub width: f32,
    pub crease: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCornerDepthWeights {
    pub deep: f32,
    pub drooped: f32,
    pub wide: f32,
    pub creased: f32,
    pub neutral: f32,
}

#[allow(dead_code)]
pub fn default_lip_corner_depth_config() -> LipCornerDepthConfig {
    LipCornerDepthConfig { depth: 0.5, droop: 0.5, width: 0.5 }
}

#[allow(dead_code)]
pub fn new_lip_corner_depth_state() -> LipCornerDepthState {
    LipCornerDepthState { depth: 0.5, droop: 0.5, width: 0.5, crease: 0.5 }
}

#[allow(dead_code)]
pub fn set_lip_corner_depth_depth(state: &mut LipCornerDepthState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lip_corner_depth_droop(state: &mut LipCornerDepthState, value: f32) {
    state.droop = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lip_corner_depth_width(state: &mut LipCornerDepthState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lip_corner_depth_crease(state: &mut LipCornerDepthState, value: f32) {
    state.crease = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_lip_corner_depth_weights(state: &LipCornerDepthState, cfg: &LipCornerDepthConfig) -> LipCornerDepthWeights {
    let deep = (state.depth * cfg.depth * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let drooped = (state.droop * cfg.droop).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let creased = state.crease.clamp(0.0, 1.0);
    let neutral = (1.0 - state.depth).clamp(0.0, 1.0);
    LipCornerDepthWeights { deep, drooped, wide, creased, neutral }
}

#[allow(dead_code)]
pub fn lip_corner_depth_to_json(state: &LipCornerDepthState) -> String {
    format!(
        r#"{{\"depth\":{},\"droop\":{},\"width\":{},\"crease\":{}}}"#,
        state.depth, state.droop, state.width, state.crease
    )
}

#[allow(dead_code)]
pub fn blend_lip_corner_depths(a: &LipCornerDepthState, b: &LipCornerDepthState, t: f32) -> LipCornerDepthState {
    let t = t.clamp(0.0, 1.0);
    LipCornerDepthState {
        depth: a.depth + (b.depth - a.depth) * t,
        droop: a.droop + (b.droop - a.droop) * t,
        width: a.width + (b.width - a.width) * t,
        crease: a.crease + (b.crease - a.crease) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_corner_depth_config();
        assert!((0.0..=1.0).contains(&cfg.depth));
    }

    #[test]
    fn test_new_state() {
        let s = new_lip_corner_depth_state();
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamp() {
        let mut s = new_lip_corner_depth_state();
        set_lip_corner_depth_depth(&mut s, 1.5);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_droop() {
        let mut s = new_lip_corner_depth_state();
        set_lip_corner_depth_droop(&mut s, 0.8);
        assert!((s.droop - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_lip_corner_depth_state();
        set_lip_corner_depth_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_crease() {
        let mut s = new_lip_corner_depth_state();
        set_lip_corner_depth_crease(&mut s, 0.6);
        assert!((s.crease - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_lip_corner_depth_state();
        let cfg = default_lip_corner_depth_config();
        let w = compute_lip_corner_depth_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.deep));
        assert!((0.0..=1.0).contains(&w.drooped));
    }

    #[test]
    fn test_to_json() {
        let s = new_lip_corner_depth_state();
        let json = lip_corner_depth_to_json(&s);
        assert!(json.contains("depth"));
        assert!(json.contains("crease"));
    }

    #[test]
    fn test_blend() {
        let a = new_lip_corner_depth_state();
        let mut b = new_lip_corner_depth_state();
        b.depth = 1.0;
        let mid = blend_lip_corner_depths(&a, &b, 0.5);
        assert!((mid.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_lip_corner_depth_state();
        let r = blend_lip_corner_depths(&a, &a, 0.5);
        assert!((r.depth - a.depth).abs() < 1e-6);
    }
}
