// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Finger length morphology controls for individual finger proportions.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerLengthConfig {
    pub overall_length: f32,
    pub thickness: f32,
    pub taper: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerLengthState {
    pub overall_length: f32,
    pub thickness: f32,
    pub taper: f32,
    pub knuckle_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerLengthWeights {
    pub long: f32,
    pub short: f32,
    pub thick: f32,
    pub thin: f32,
    pub tapered: f32,
    pub knobby: f32,
}

#[allow(dead_code)]
pub fn default_finger_length_config() -> FingerLengthConfig {
    FingerLengthConfig {
        overall_length: 0.5,
        thickness: 0.5,
        taper: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_finger_length_state() -> FingerLengthState {
    FingerLengthState {
        overall_length: 0.5,
        thickness: 0.5,
        taper: 0.5,
        knuckle_size: 0.3,
    }
}

#[allow(dead_code)]
pub fn set_finger_overall_length(state: &mut FingerLengthState, value: f32) {
    state.overall_length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_finger_thickness(state: &mut FingerLengthState, value: f32) {
    state.thickness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_finger_taper(state: &mut FingerLengthState, value: f32) {
    state.taper = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_finger_knuckle_size(state: &mut FingerLengthState, value: f32) {
    state.knuckle_size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_finger_length_weights(
    state: &FingerLengthState,
    cfg: &FingerLengthConfig,
) -> FingerLengthWeights {
    let l = state.overall_length * cfg.overall_length;
    let long = (l * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let short = (1.0 - l).clamp(0.0, 1.0);
    let thick = (state.thickness * cfg.thickness).clamp(0.0, 1.0);
    let thin = (1.0 - thick).clamp(0.0, 1.0);
    let tapered = (state.taper * cfg.taper).clamp(0.0, 1.0);
    let knobby = state.knuckle_size.clamp(0.0, 1.0);
    FingerLengthWeights {
        long,
        short,
        thick,
        thin,
        tapered,
        knobby,
    }
}

#[allow(dead_code)]
pub fn finger_length_to_json(state: &FingerLengthState) -> String {
    format!(
        r#"{{"overall_length":{},"thickness":{},"taper":{},"knuckle_size":{}}}"#,
        state.overall_length, state.thickness, state.taper, state.knuckle_size
    )
}

#[allow(dead_code)]
pub fn blend_finger_lengths(
    a: &FingerLengthState,
    b: &FingerLengthState,
    t: f32,
) -> FingerLengthState {
    let t = t.clamp(0.0, 1.0);
    FingerLengthState {
        overall_length: a.overall_length + (b.overall_length - a.overall_length) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        knuckle_size: a.knuckle_size + (b.knuckle_size - a.knuckle_size) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_finger_length_config();
        assert!((0.0..=1.0).contains(&cfg.overall_length));
    }

    #[test]
    fn test_new_state() {
        let s = new_finger_length_state();
        assert!((s.overall_length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamp() {
        let mut s = new_finger_length_state();
        set_finger_overall_length(&mut s, 1.5);
        assert!((s.overall_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut s = new_finger_length_state();
        set_finger_thickness(&mut s, 0.8);
        assert!((s.thickness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_finger_length_state();
        set_finger_taper(&mut s, 0.7);
        assert!((s.taper - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_knuckle_size() {
        let mut s = new_finger_length_state();
        set_finger_knuckle_size(&mut s, 0.6);
        assert!((s.knuckle_size - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_finger_length_state();
        let cfg = default_finger_length_config();
        let w = compute_finger_length_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.long));
        assert!((0.0..=1.0).contains(&w.thick));
    }

    #[test]
    fn test_to_json() {
        let s = new_finger_length_state();
        let json = finger_length_to_json(&s);
        assert!(json.contains("overall_length"));
        assert!(json.contains("knuckle_size"));
    }

    #[test]
    fn test_blend() {
        let a = new_finger_length_state();
        let mut b = new_finger_length_state();
        b.overall_length = 1.0;
        let mid = blend_finger_lengths(&a, &b, 0.5);
        assert!((mid.overall_length - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_finger_length_state();
        let r = blend_finger_lengths(&a, &a, 0.5);
        assert!((r.overall_length - a.overall_length).abs() < 1e-6);
    }
}
