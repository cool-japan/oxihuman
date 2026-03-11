// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Jaw angle morph controls for gonial angle and mandibular shape.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawAngleControlConfig {
    pub angle: f32,
    pub width: f32,
    pub definition: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawAngleControlState {
    pub angle: f32,
    pub width: f32,
    pub definition: f32,
    pub flare: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawAngleControlWeights {
    pub angled: f32,
    pub wide: f32,
    pub defined: f32,
    pub flared: f32,
    pub smooth: f32,
}

#[allow(dead_code)]
pub fn default_jaw_angle_control_config() -> JawAngleControlConfig {
    JawAngleControlConfig { angle: 0.5, width: 0.5, definition: 0.5 }
}

#[allow(dead_code)]
pub fn new_jaw_angle_control_state() -> JawAngleControlState {
    JawAngleControlState { angle: 0.5, width: 0.5, definition: 0.5, flare: 0.5 }
}

#[allow(dead_code)]
pub fn set_jaw_angle_control_angle(state: &mut JawAngleControlState, value: f32) {
    state.angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_jaw_angle_control_width(state: &mut JawAngleControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_jaw_angle_control_definition(state: &mut JawAngleControlState, value: f32) {
    state.definition = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_jaw_angle_control_flare(state: &mut JawAngleControlState, value: f32) {
    state.flare = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_jaw_angle_control_weights(state: &JawAngleControlState, cfg: &JawAngleControlConfig) -> JawAngleControlWeights {
    let angled = (state.angle * cfg.angle * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let defined = (state.definition * cfg.definition).clamp(0.0, 1.0);
    let flared = state.flare.clamp(0.0, 1.0);
    let smooth = (1.0 - state.angle).clamp(0.0, 1.0);
    JawAngleControlWeights { angled, wide, defined, flared, smooth }
}

#[allow(dead_code)]
pub fn jaw_angle_control_to_json(state: &JawAngleControlState) -> String {
    format!(
        r#"{{\"angle\":{},\"width\":{},\"definition\":{},\"flare\":{}}}"#,
        state.angle, state.width, state.definition, state.flare
    )
}

#[allow(dead_code)]
pub fn blend_jaw_angle_controls(a: &JawAngleControlState, b: &JawAngleControlState, t: f32) -> JawAngleControlState {
    let t = t.clamp(0.0, 1.0);
    JawAngleControlState {
        angle: a.angle + (b.angle - a.angle) * t,
        width: a.width + (b.width - a.width) * t,
        definition: a.definition + (b.definition - a.definition) * t,
        flare: a.flare + (b.flare - a.flare) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jaw_angle_control_config();
        assert!((0.0..=1.0).contains(&cfg.angle));
    }

    #[test]
    fn test_new_state() {
        let s = new_jaw_angle_control_state();
        assert!((s.angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_clamp() {
        let mut s = new_jaw_angle_control_state();
        set_jaw_angle_control_angle(&mut s, 1.5);
        assert!((s.angle - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_jaw_angle_control_state();
        set_jaw_angle_control_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_definition() {
        let mut s = new_jaw_angle_control_state();
        set_jaw_angle_control_definition(&mut s, 0.7);
        assert!((s.definition - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_flare() {
        let mut s = new_jaw_angle_control_state();
        set_jaw_angle_control_flare(&mut s, 0.6);
        assert!((s.flare - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_jaw_angle_control_state();
        let cfg = default_jaw_angle_control_config();
        let w = compute_jaw_angle_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.angled));
        assert!((0.0..=1.0).contains(&w.wide));
    }

    #[test]
    fn test_to_json() {
        let s = new_jaw_angle_control_state();
        let json = jaw_angle_control_to_json(&s);
        assert!(json.contains("angle"));
        assert!(json.contains("flare"));
    }

    #[test]
    fn test_blend() {
        let a = new_jaw_angle_control_state();
        let mut b = new_jaw_angle_control_state();
        b.angle = 1.0;
        let mid = blend_jaw_angle_controls(&a, &b, 0.5);
        assert!((mid.angle - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_jaw_angle_control_state();
        let r = blend_jaw_angle_controls(&a, &a, 0.5);
        assert!((r.angle - a.angle).abs() < 1e-6);
    }
}
