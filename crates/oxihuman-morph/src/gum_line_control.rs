// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Gum line morph controls for gingival display and tooth show.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GumLineControlConfig {
    pub exposure: f32,
    pub curvature: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GumLineControlState {
    pub exposure: f32,
    pub curvature: f32,
    pub width: f32,
    pub recession: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GumLineControlWeights {
    pub exposed: f32,
    pub curved: f32,
    pub wide: f32,
    pub receded: f32,
    pub minimal: f32,
}

#[allow(dead_code)]
pub fn default_gum_line_control_config() -> GumLineControlConfig {
    GumLineControlConfig {
        exposure: 0.5,
        curvature: 0.5,
        width: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_gum_line_control_state() -> GumLineControlState {
    GumLineControlState {
        exposure: 0.5,
        curvature: 0.5,
        width: 0.5,
        recession: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_gum_line_control_exposure(state: &mut GumLineControlState, value: f32) {
    state.exposure = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gum_line_control_curvature(state: &mut GumLineControlState, value: f32) {
    state.curvature = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gum_line_control_width(state: &mut GumLineControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gum_line_control_recession(state: &mut GumLineControlState, value: f32) {
    state.recession = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_gum_line_control_weights(
    state: &GumLineControlState,
    cfg: &GumLineControlConfig,
) -> GumLineControlWeights {
    let exposed = (state.exposure * cfg.exposure * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let curved = (state.curvature * cfg.curvature).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let receded = state.recession.clamp(0.0, 1.0);
    let minimal = (1.0 - state.exposure).clamp(0.0, 1.0);
    GumLineControlWeights {
        exposed,
        curved,
        wide,
        receded,
        minimal,
    }
}

#[allow(dead_code)]
pub fn gum_line_control_to_json(state: &GumLineControlState) -> String {
    format!(
        r#"{{\"exposure\":{},\"curvature\":{},\"width\":{},\"recession\":{}}}"#,
        state.exposure, state.curvature, state.width, state.recession
    )
}

#[allow(dead_code)]
pub fn blend_gum_line_controls(
    a: &GumLineControlState,
    b: &GumLineControlState,
    t: f32,
) -> GumLineControlState {
    let t = t.clamp(0.0, 1.0);
    GumLineControlState {
        exposure: a.exposure + (b.exposure - a.exposure) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        width: a.width + (b.width - a.width) * t,
        recession: a.recession + (b.recession - a.recession) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_gum_line_control_config();
        assert!((0.0..=1.0).contains(&cfg.exposure));
    }

    #[test]
    fn test_new_state() {
        let s = new_gum_line_control_state();
        assert!((s.exposure - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_exposure_clamp() {
        let mut s = new_gum_line_control_state();
        set_gum_line_control_exposure(&mut s, 1.5);
        assert!((s.exposure - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_curvature() {
        let mut s = new_gum_line_control_state();
        set_gum_line_control_curvature(&mut s, 0.8);
        assert!((s.curvature - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_gum_line_control_state();
        set_gum_line_control_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_recession() {
        let mut s = new_gum_line_control_state();
        set_gum_line_control_recession(&mut s, 0.6);
        assert!((s.recession - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_gum_line_control_state();
        let cfg = default_gum_line_control_config();
        let w = compute_gum_line_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.exposed));
        assert!((0.0..=1.0).contains(&w.curved));
    }

    #[test]
    fn test_to_json() {
        let s = new_gum_line_control_state();
        let json = gum_line_control_to_json(&s);
        assert!(json.contains("exposure"));
        assert!(json.contains("recession"));
    }

    #[test]
    fn test_blend() {
        let a = new_gum_line_control_state();
        let mut b = new_gum_line_control_state();
        b.exposure = 1.0;
        let mid = blend_gum_line_controls(&a, &b, 0.5);
        assert!((mid.exposure - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_gum_line_control_state();
        let r = blend_gum_line_controls(&a, &a, 0.5);
        assert!((r.exposure - a.exposure).abs() < 1e-6);
    }
}
