// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Nape morph controls for posterior neck contour and musculature.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NapeControlConfig {
    pub thickness: f32,
    pub slope: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NapeControlState {
    pub thickness: f32,
    pub slope: f32,
    pub width: f32,
    pub muscularity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NapeControlWeights {
    pub thick: f32,
    pub sloped: f32,
    pub wide: f32,
    pub muscular: f32,
    pub slim: f32,
}

#[allow(dead_code)]
pub fn default_nape_control_config() -> NapeControlConfig {
    NapeControlConfig { thickness: 0.5, slope: 0.5, width: 0.5 }
}

#[allow(dead_code)]
pub fn new_nape_control_state() -> NapeControlState {
    NapeControlState { thickness: 0.5, slope: 0.5, width: 0.5, muscularity: 0.5 }
}

#[allow(dead_code)]
pub fn set_nape_control_thickness(state: &mut NapeControlState, value: f32) {
    state.thickness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_nape_control_slope(state: &mut NapeControlState, value: f32) {
    state.slope = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_nape_control_width(state: &mut NapeControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_nape_control_muscularity(state: &mut NapeControlState, value: f32) {
    state.muscularity = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_nape_control_weights(state: &NapeControlState, cfg: &NapeControlConfig) -> NapeControlWeights {
    let thick = (state.thickness * cfg.thickness * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let sloped = (state.slope * cfg.slope).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let muscular = state.muscularity.clamp(0.0, 1.0);
    let slim = (1.0 - state.thickness).clamp(0.0, 1.0);
    NapeControlWeights { thick, sloped, wide, muscular, slim }
}

#[allow(dead_code)]
pub fn nape_control_to_json(state: &NapeControlState) -> String {
    format!(
        r#"{{\"thickness\":{},\"slope\":{},\"width\":{},\"muscularity\":{}}}"#,
        state.thickness, state.slope, state.width, state.muscularity
    )
}

#[allow(dead_code)]
pub fn blend_nape_controls(a: &NapeControlState, b: &NapeControlState, t: f32) -> NapeControlState {
    let t = t.clamp(0.0, 1.0);
    NapeControlState {
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        slope: a.slope + (b.slope - a.slope) * t,
        width: a.width + (b.width - a.width) * t,
        muscularity: a.muscularity + (b.muscularity - a.muscularity) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nape_control_config();
        assert!((0.0..=1.0).contains(&cfg.thickness));
    }

    #[test]
    fn test_new_state() {
        let s = new_nape_control_state();
        assert!((s.thickness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness_clamp() {
        let mut s = new_nape_control_state();
        set_nape_control_thickness(&mut s, 1.5);
        assert!((s.thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_slope() {
        let mut s = new_nape_control_state();
        set_nape_control_slope(&mut s, 0.8);
        assert!((s.slope - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_nape_control_state();
        set_nape_control_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_muscularity() {
        let mut s = new_nape_control_state();
        set_nape_control_muscularity(&mut s, 0.6);
        assert!((s.muscularity - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_nape_control_state();
        let cfg = default_nape_control_config();
        let w = compute_nape_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.thick));
        assert!((0.0..=1.0).contains(&w.sloped));
    }

    #[test]
    fn test_to_json() {
        let s = new_nape_control_state();
        let json = nape_control_to_json(&s);
        assert!(json.contains("thickness"));
        assert!(json.contains("muscularity"));
    }

    #[test]
    fn test_blend() {
        let a = new_nape_control_state();
        let mut b = new_nape_control_state();
        b.thickness = 1.0;
        let mid = blend_nape_controls(&a, &b, 0.5);
        assert!((mid.thickness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_nape_control_state();
        let r = blend_nape_controls(&a, &a, 0.5);
        assert!((r.thickness - a.thickness).abs() < 1e-6);
    }
}
