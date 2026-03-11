// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Cheek bone height morph controls for malar prominence.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekBoneHeightConfig {
    pub height: f32,
    pub prominence: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekBoneHeightState {
    pub height: f32,
    pub prominence: f32,
    pub width: f32,
    pub angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekBoneHeightWeights {
    pub elevated: f32,
    pub prominent: f32,
    pub wide: f32,
    pub angular: f32,
    pub subtle: f32,
}

#[allow(dead_code)]
pub fn default_cheek_bone_height_config() -> CheekBoneHeightConfig {
    CheekBoneHeightConfig { height: 0.5, prominence: 0.5, width: 0.5 }
}

#[allow(dead_code)]
pub fn new_cheek_bone_height_state() -> CheekBoneHeightState {
    CheekBoneHeightState { height: 0.5, prominence: 0.5, width: 0.5, angle: 0.5 }
}

#[allow(dead_code)]
pub fn set_cheek_bone_height_height(state: &mut CheekBoneHeightState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheek_bone_height_prominence(state: &mut CheekBoneHeightState, value: f32) {
    state.prominence = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheek_bone_height_width(state: &mut CheekBoneHeightState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheek_bone_height_angle(state: &mut CheekBoneHeightState, value: f32) {
    state.angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_cheek_bone_height_weights(state: &CheekBoneHeightState, cfg: &CheekBoneHeightConfig) -> CheekBoneHeightWeights {
    let elevated = (state.height * cfg.height * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let prominent = (state.prominence * cfg.prominence).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let angular = state.angle.clamp(0.0, 1.0);
    let subtle = (1.0 - state.height).clamp(0.0, 1.0);
    CheekBoneHeightWeights { elevated, prominent, wide, angular, subtle }
}

#[allow(dead_code)]
pub fn cheek_bone_height_to_json(state: &CheekBoneHeightState) -> String {
    format!(
        r#"{{\"height\":{},\"prominence\":{},\"width\":{},\"angle\":{}}}"#,
        state.height, state.prominence, state.width, state.angle
    )
}

#[allow(dead_code)]
pub fn blend_cheek_bone_heights(a: &CheekBoneHeightState, b: &CheekBoneHeightState, t: f32) -> CheekBoneHeightState {
    let t = t.clamp(0.0, 1.0);
    CheekBoneHeightState {
        height: a.height + (b.height - a.height) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width: a.width + (b.width - a.width) * t,
        angle: a.angle + (b.angle - a.angle) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheek_bone_height_config();
        assert!((0.0..=1.0).contains(&cfg.height));
    }

    #[test]
    fn test_new_state() {
        let s = new_cheek_bone_height_state();
        assert!((s.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        let mut s = new_cheek_bone_height_state();
        set_cheek_bone_height_height(&mut s, 1.5);
        assert!((s.height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence() {
        let mut s = new_cheek_bone_height_state();
        set_cheek_bone_height_prominence(&mut s, 0.8);
        assert!((s.prominence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_cheek_bone_height_state();
        set_cheek_bone_height_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle() {
        let mut s = new_cheek_bone_height_state();
        set_cheek_bone_height_angle(&mut s, 0.6);
        assert!((s.angle - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_cheek_bone_height_state();
        let cfg = default_cheek_bone_height_config();
        let w = compute_cheek_bone_height_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.elevated));
        assert!((0.0..=1.0).contains(&w.prominent));
    }

    #[test]
    fn test_to_json() {
        let s = new_cheek_bone_height_state();
        let json = cheek_bone_height_to_json(&s);
        assert!(json.contains("height"));
        assert!(json.contains("angle"));
    }

    #[test]
    fn test_blend() {
        let a = new_cheek_bone_height_state();
        let mut b = new_cheek_bone_height_state();
        b.height = 1.0;
        let mid = blend_cheek_bone_heights(&a, &b, 0.5);
        assert!((mid.height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_cheek_bone_height_state();
        let r = blend_cheek_bone_heights(&a, &a, 0.5);
        assert!((r.height - a.height).abs() < 1e-6);
    }
}
