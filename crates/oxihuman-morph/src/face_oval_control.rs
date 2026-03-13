// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Face oval morph controls for overall face shape contour.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceOvalControlConfig {
    pub length: f32,
    pub width: f32,
    pub taper: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceOvalControlState {
    pub length: f32,
    pub width: f32,
    pub taper: f32,
    pub roundness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceOvalControlWeights {
    pub long: f32,
    pub wide: f32,
    pub tapered: f32,
    pub rounded: f32,
    pub angular: f32,
}

#[allow(dead_code)]
pub fn default_face_oval_control_config() -> FaceOvalControlConfig {
    FaceOvalControlConfig { length: 0.5, width: 0.5, taper: 0.5 }
}

#[allow(dead_code)]
pub fn new_face_oval_control_state() -> FaceOvalControlState {
    FaceOvalControlState { length: 0.5, width: 0.5, taper: 0.5, roundness: 0.5 }
}

#[allow(dead_code)]
pub fn set_face_oval_control_length(state: &mut FaceOvalControlState, value: f32) {
    state.length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_face_oval_control_width(state: &mut FaceOvalControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_face_oval_control_taper(state: &mut FaceOvalControlState, value: f32) {
    state.taper = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_face_oval_control_roundness(state: &mut FaceOvalControlState, value: f32) {
    state.roundness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_face_oval_control_weights(state: &FaceOvalControlState, cfg: &FaceOvalControlConfig) -> FaceOvalControlWeights {
    let long = (state.length * cfg.length * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let tapered = (state.taper * cfg.taper).clamp(0.0, 1.0);
    let rounded = state.roundness.clamp(0.0, 1.0);
    let angular = (1.0 - state.length).clamp(0.0, 1.0);
    FaceOvalControlWeights { long, wide, tapered, rounded, angular }
}

#[allow(dead_code)]
pub fn face_oval_control_to_json(state: &FaceOvalControlState) -> String {
    format!(
        r#"{{\"length\":{},\"width\":{},\"taper\":{},\"roundness\":{}}}"#,
        state.length, state.width, state.taper, state.roundness
    )
}

#[allow(dead_code)]
pub fn blend_face_oval_controls(a: &FaceOvalControlState, b: &FaceOvalControlState, t: f32) -> FaceOvalControlState {
    let t = t.clamp(0.0, 1.0);
    FaceOvalControlState {
        length: a.length + (b.length - a.length) * t,
        width: a.width + (b.width - a.width) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_face_oval_control_config();
        assert!((0.0..=1.0).contains(&cfg.length));
    }

    #[test]
    fn test_new_state() {
        let s = new_face_oval_control_state();
        assert!((s.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamp() {
        let mut s = new_face_oval_control_state();
        set_face_oval_control_length(&mut s, 1.5);
        assert!((s.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_face_oval_control_state();
        set_face_oval_control_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_face_oval_control_state();
        set_face_oval_control_taper(&mut s, 0.7);
        assert!((s.taper - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness() {
        let mut s = new_face_oval_control_state();
        set_face_oval_control_roundness(&mut s, 0.6);
        assert!((s.roundness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_face_oval_control_state();
        let cfg = default_face_oval_control_config();
        let w = compute_face_oval_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.long));
        assert!((0.0..=1.0).contains(&w.wide));
    }

    #[test]
    fn test_to_json() {
        let s = new_face_oval_control_state();
        let json = face_oval_control_to_json(&s);
        assert!(json.contains("length"));
        assert!(json.contains("roundness"));
    }

    #[test]
    fn test_blend() {
        let a = new_face_oval_control_state();
        let mut b = new_face_oval_control_state();
        b.length = 1.0;
        let mid = blend_face_oval_controls(&a, &b, 0.5);
        assert!((mid.length - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_face_oval_control_state();
        let r = blend_face_oval_controls(&a, &a, 0.5);
        assert!((r.length - a.length).abs() < 1e-6);
    }
}
