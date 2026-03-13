// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Forehead slope morph controls for frontal bone inclination.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadSlopeControlConfig {
    pub slope: f32,
    pub height: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadSlopeControlState {
    pub slope: f32,
    pub height: f32,
    pub width: f32,
    pub bulge: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadSlopeControlWeights {
    pub sloped: f32,
    pub tall: f32,
    pub wide: f32,
    pub bulging: f32,
    pub flat: f32,
}

#[allow(dead_code)]
pub fn default_forehead_slope_control_config() -> ForeheadSlopeControlConfig {
    ForeheadSlopeControlConfig { slope: 0.5, height: 0.5, width: 0.5 }
}

#[allow(dead_code)]
pub fn new_forehead_slope_control_state() -> ForeheadSlopeControlState {
    ForeheadSlopeControlState { slope: 0.5, height: 0.5, width: 0.5, bulge: 0.5 }
}

#[allow(dead_code)]
pub fn set_forehead_slope_control_slope(state: &mut ForeheadSlopeControlState, value: f32) {
    state.slope = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_forehead_slope_control_height(state: &mut ForeheadSlopeControlState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_forehead_slope_control_width(state: &mut ForeheadSlopeControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_forehead_slope_control_bulge(state: &mut ForeheadSlopeControlState, value: f32) {
    state.bulge = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_forehead_slope_control_weights(state: &ForeheadSlopeControlState, cfg: &ForeheadSlopeControlConfig) -> ForeheadSlopeControlWeights {
    let sloped = (state.slope * cfg.slope * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let tall = (state.height * cfg.height).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let bulging = state.bulge.clamp(0.0, 1.0);
    let flat = (1.0 - state.slope).clamp(0.0, 1.0);
    ForeheadSlopeControlWeights { sloped, tall, wide, bulging, flat }
}

#[allow(dead_code)]
pub fn forehead_slope_control_to_json(state: &ForeheadSlopeControlState) -> String {
    format!(
        r#"{{\"slope\":{},\"height\":{},\"width\":{},\"bulge\":{}}}"#,
        state.slope, state.height, state.width, state.bulge
    )
}

#[allow(dead_code)]
pub fn blend_forehead_slope_controls(a: &ForeheadSlopeControlState, b: &ForeheadSlopeControlState, t: f32) -> ForeheadSlopeControlState {
    let t = t.clamp(0.0, 1.0);
    ForeheadSlopeControlState {
        slope: a.slope + (b.slope - a.slope) * t,
        height: a.height + (b.height - a.height) * t,
        width: a.width + (b.width - a.width) * t,
        bulge: a.bulge + (b.bulge - a.bulge) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_forehead_slope_control_config();
        assert!((0.0..=1.0).contains(&cfg.slope));
    }

    #[test]
    fn test_new_state() {
        let s = new_forehead_slope_control_state();
        assert!((s.slope - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_slope_clamp() {
        let mut s = new_forehead_slope_control_state();
        set_forehead_slope_control_slope(&mut s, 1.5);
        assert!((s.slope - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut s = new_forehead_slope_control_state();
        set_forehead_slope_control_height(&mut s, 0.8);
        assert!((s.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_forehead_slope_control_state();
        set_forehead_slope_control_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_bulge() {
        let mut s = new_forehead_slope_control_state();
        set_forehead_slope_control_bulge(&mut s, 0.6);
        assert!((s.bulge - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_forehead_slope_control_state();
        let cfg = default_forehead_slope_control_config();
        let w = compute_forehead_slope_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.sloped));
        assert!((0.0..=1.0).contains(&w.tall));
    }

    #[test]
    fn test_to_json() {
        let s = new_forehead_slope_control_state();
        let json = forehead_slope_control_to_json(&s);
        assert!(json.contains("slope"));
        assert!(json.contains("bulge"));
    }

    #[test]
    fn test_blend() {
        let a = new_forehead_slope_control_state();
        let mut b = new_forehead_slope_control_state();
        b.slope = 1.0;
        let mid = blend_forehead_slope_controls(&a, &b, 0.5);
        assert!((mid.slope - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_forehead_slope_control_state();
        let r = blend_forehead_slope_controls(&a, &a, 0.5);
        assert!((r.slope - a.slope).abs() < 1e-6);
    }
}
