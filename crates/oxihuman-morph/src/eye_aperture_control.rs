// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Eye aperture morph controls for palpebral fissure dimensions.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeApertureControlConfig {
    pub openness: f32,
    pub width: f32,
    pub tilt: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeApertureControlState {
    pub openness: f32,
    pub width: f32,
    pub tilt: f32,
    pub roundness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeApertureControlWeights {
    pub open: f32,
    pub wide: f32,
    pub tilted: f32,
    pub rounded: f32,
    pub narrow: f32,
}

#[allow(dead_code)]
pub fn default_eye_aperture_control_config() -> EyeApertureControlConfig {
    EyeApertureControlConfig { openness: 0.5, width: 0.5, tilt: 0.5 }
}

#[allow(dead_code)]
pub fn new_eye_aperture_control_state() -> EyeApertureControlState {
    EyeApertureControlState { openness: 0.5, width: 0.5, tilt: 0.5, roundness: 0.5 }
}

#[allow(dead_code)]
pub fn set_eye_aperture_control_openness(state: &mut EyeApertureControlState, value: f32) {
    state.openness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_aperture_control_width(state: &mut EyeApertureControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_aperture_control_tilt(state: &mut EyeApertureControlState, value: f32) {
    state.tilt = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_aperture_control_roundness(state: &mut EyeApertureControlState, value: f32) {
    state.roundness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_eye_aperture_control_weights(state: &EyeApertureControlState, cfg: &EyeApertureControlConfig) -> EyeApertureControlWeights {
    let open = (state.openness * cfg.openness * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let tilted = (state.tilt * cfg.tilt).clamp(0.0, 1.0);
    let rounded = state.roundness.clamp(0.0, 1.0);
    let narrow = (1.0 - state.openness).clamp(0.0, 1.0);
    EyeApertureControlWeights { open, wide, tilted, rounded, narrow }
}

#[allow(dead_code)]
pub fn eye_aperture_control_to_json(state: &EyeApertureControlState) -> String {
    format!(
        r#"{{\"openness\":{},\"width\":{},\"tilt\":{},\"roundness\":{}}}"#,
        state.openness, state.width, state.tilt, state.roundness
    )
}

#[allow(dead_code)]
pub fn blend_eye_aperture_controls(a: &EyeApertureControlState, b: &EyeApertureControlState, t: f32) -> EyeApertureControlState {
    let t = t.clamp(0.0, 1.0);
    EyeApertureControlState {
        openness: a.openness + (b.openness - a.openness) * t,
        width: a.width + (b.width - a.width) * t,
        tilt: a.tilt + (b.tilt - a.tilt) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_aperture_control_config();
        assert!((0.0..=1.0).contains(&cfg.openness));
    }

    #[test]
    fn test_new_state() {
        let s = new_eye_aperture_control_state();
        assert!((s.openness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_openness_clamp() {
        let mut s = new_eye_aperture_control_state();
        set_eye_aperture_control_openness(&mut s, 1.5);
        assert!((s.openness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_eye_aperture_control_state();
        set_eye_aperture_control_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_tilt() {
        let mut s = new_eye_aperture_control_state();
        set_eye_aperture_control_tilt(&mut s, 0.7);
        assert!((s.tilt - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness() {
        let mut s = new_eye_aperture_control_state();
        set_eye_aperture_control_roundness(&mut s, 0.6);
        assert!((s.roundness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_eye_aperture_control_state();
        let cfg = default_eye_aperture_control_config();
        let w = compute_eye_aperture_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.open));
        assert!((0.0..=1.0).contains(&w.wide));
    }

    #[test]
    fn test_to_json() {
        let s = new_eye_aperture_control_state();
        let json = eye_aperture_control_to_json(&s);
        assert!(json.contains("openness"));
        assert!(json.contains("roundness"));
    }

    #[test]
    fn test_blend() {
        let a = new_eye_aperture_control_state();
        let mut b = new_eye_aperture_control_state();
        b.openness = 1.0;
        let mid = blend_eye_aperture_controls(&a, &b, 0.5);
        assert!((mid.openness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_eye_aperture_control_state();
        let r = blend_eye_aperture_controls(&a, &a, 0.5);
        assert!((r.openness - a.openness).abs() < 1e-6);
    }
}
