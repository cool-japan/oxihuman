// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Nose bridge morphology controls for bridge width, height, and curvature.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeNoseConfig {
    pub width: f32,
    pub height: f32,
    pub curvature: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeNoseState {
    pub width: f32,
    pub height: f32,
    pub curvature: f32,
    pub bump: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeNoseWeights {
    pub wide: f32,
    pub narrow: f32,
    pub high: f32,
    pub low: f32,
    pub curved: f32,
    pub bumpy: f32,
}

#[allow(dead_code)]
pub fn default_bridge_nose_config() -> BridgeNoseConfig {
    BridgeNoseConfig { width: 0.5, height: 0.5, curvature: 0.5 }
}

#[allow(dead_code)]
pub fn new_bridge_nose_state() -> BridgeNoseState {
    BridgeNoseState { width: 0.5, height: 0.5, curvature: 0.5, bump: 0.0 }
}

#[allow(dead_code)]
pub fn set_bridge_width(state: &mut BridgeNoseState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_bridge_height(state: &mut BridgeNoseState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_bridge_curvature(state: &mut BridgeNoseState, value: f32) {
    state.curvature = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_bridge_bump(state: &mut BridgeNoseState, value: f32) {
    state.bump = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_bridge_nose_weights(state: &BridgeNoseState, cfg: &BridgeNoseConfig) -> BridgeNoseWeights {
    let w = state.width * cfg.width;
    let wide = w.clamp(0.0, 1.0);
    let narrow = (1.0 - w).clamp(0.0, 1.0);
    let h = state.height * cfg.height;
    let high = (h * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let low = (1.0 - h).clamp(0.0, 1.0);
    let curved = (state.curvature * cfg.curvature).clamp(0.0, 1.0);
    let bumpy = state.bump.clamp(0.0, 1.0);
    BridgeNoseWeights { wide, narrow, high, low, curved, bumpy }
}

#[allow(dead_code)]
pub fn bridge_nose_to_json(state: &BridgeNoseState) -> String {
    format!(
        r#"{{"width":{},"height":{},"curvature":{},"bump":{}}}"#,
        state.width, state.height, state.curvature, state.bump
    )
}

#[allow(dead_code)]
pub fn blend_bridge_nose(a: &BridgeNoseState, b: &BridgeNoseState, t: f32) -> BridgeNoseState {
    let t = t.clamp(0.0, 1.0);
    BridgeNoseState {
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        bump: a.bump + (b.bump - a.bump) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_bridge_nose_config();
        assert!((0.0..=1.0).contains(&cfg.width));
    }

    #[test]
    fn test_new_state() {
        let s = new_bridge_nose_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp() {
        let mut s = new_bridge_nose_state();
        set_bridge_width(&mut s, 1.5);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut s = new_bridge_nose_state();
        set_bridge_height(&mut s, 0.8);
        assert!((s.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_curvature() {
        let mut s = new_bridge_nose_state();
        set_bridge_curvature(&mut s, 0.7);
        assert!((s.curvature - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_bump() {
        let mut s = new_bridge_nose_state();
        set_bridge_bump(&mut s, 0.3);
        assert!((s.bump - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_bridge_nose_state();
        let cfg = default_bridge_nose_config();
        let w = compute_bridge_nose_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide));
        assert!((0.0..=1.0).contains(&w.high));
    }

    #[test]
    fn test_to_json() {
        let s = new_bridge_nose_state();
        let json = bridge_nose_to_json(&s);
        assert!(json.contains("width"));
        assert!(json.contains("bump"));
    }

    #[test]
    fn test_blend() {
        let a = new_bridge_nose_state();
        let mut b = new_bridge_nose_state();
        b.width = 1.0;
        let mid = blend_bridge_nose(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_bridge_nose_state();
        let r = blend_bridge_nose(&a, &a, 0.5);
        assert!((r.width - a.width).abs() < 1e-6);
    }
}
