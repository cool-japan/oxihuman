// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Brow position morph controls for vertical and lateral brow placement.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowPositionConfig {
    pub height: f32,
    pub arch: f32,
    pub spacing: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowPositionState {
    pub height: f32,
    pub arch: f32,
    pub spacing: f32,
    pub thickness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowPositionWeights {
    pub raised: f32,
    pub arched: f32,
    pub wide: f32,
    pub thick: f32,
    pub flat: f32,
}

#[allow(dead_code)]
pub fn default_brow_position_config() -> BrowPositionConfig {
    BrowPositionConfig { height: 0.5, arch: 0.5, spacing: 0.5 }
}

#[allow(dead_code)]
pub fn new_brow_position_state() -> BrowPositionState {
    BrowPositionState { height: 0.5, arch: 0.5, spacing: 0.5, thickness: 0.5 }
}

#[allow(dead_code)]
pub fn set_brow_height(state: &mut BrowPositionState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_brow_arch(state: &mut BrowPositionState, value: f32) {
    state.arch = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_brow_spacing(state: &mut BrowPositionState, value: f32) {
    state.spacing = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_brow_thickness(state: &mut BrowPositionState, value: f32) {
    state.thickness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_brow_position_weights(state: &BrowPositionState, cfg: &BrowPositionConfig) -> BrowPositionWeights {
    let raised = (state.height * cfg.height * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let arched = (state.arch * cfg.arch).clamp(0.0, 1.0);
    let wide = (state.spacing * cfg.spacing).clamp(0.0, 1.0);
    let thick = state.thickness.clamp(0.0, 1.0);
    let flat = (1.0 - state.arch).clamp(0.0, 1.0);
    BrowPositionWeights { raised, arched, wide, thick, flat }
}

#[allow(dead_code)]
pub fn brow_position_to_json(state: &BrowPositionState) -> String {
    format!(
        r#"{{"height":{},"arch":{},"spacing":{},"thickness":{}}}"#,
        state.height, state.arch, state.spacing, state.thickness
    )
}

#[allow(dead_code)]
pub fn blend_brow_positions(a: &BrowPositionState, b: &BrowPositionState, t: f32) -> BrowPositionState {
    let t = t.clamp(0.0, 1.0);
    BrowPositionState {
        height: a.height + (b.height - a.height) * t,
        arch: a.arch + (b.arch - a.arch) * t,
        spacing: a.spacing + (b.spacing - a.spacing) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_position_config();
        assert!((0.0..=1.0).contains(&cfg.height));
    }

    #[test]
    fn test_new_state() {
        let s = new_brow_position_state();
        assert!((s.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        let mut s = new_brow_position_state();
        set_brow_height(&mut s, 1.5);
        assert!((s.height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch() {
        let mut s = new_brow_position_state();
        set_brow_arch(&mut s, 0.8);
        assert!((s.arch - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_spacing() {
        let mut s = new_brow_position_state();
        set_brow_spacing(&mut s, 0.7);
        assert!((s.spacing - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut s = new_brow_position_state();
        set_brow_thickness(&mut s, 0.6);
        assert!((s.thickness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_brow_position_state();
        let cfg = default_brow_position_config();
        let w = compute_brow_position_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.raised));
        assert!((0.0..=1.0).contains(&w.arched));
    }

    #[test]
    fn test_to_json() {
        let s = new_brow_position_state();
        let json = brow_position_to_json(&s);
        assert!(json.contains("height"));
        assert!(json.contains("thickness"));
    }

    #[test]
    fn test_blend() {
        let a = new_brow_position_state();
        let mut b = new_brow_position_state();
        b.height = 1.0;
        let mid = blend_brow_positions(&a, &b, 0.5);
        assert!((mid.height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_brow_position_state();
        let r = blend_brow_positions(&a, &a, 0.5);
        assert!((r.height - a.height).abs() < 1e-6);
    }
}
