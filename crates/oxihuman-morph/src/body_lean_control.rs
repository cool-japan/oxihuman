// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body lean morph — controls forward/backward and lateral trunk lean.

use std::f32::consts::FRAC_PI_2;

/// Configuration for body lean control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyLeanConfig {
    pub max_forward: f32,
    pub max_lateral: f32,
}

/// Runtime state for body lean morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyLeanState {
    pub forward: f32,
    pub backward: f32,
    pub lateral_left: f32,
    pub lateral_right: f32,
}

#[allow(dead_code)]
pub fn default_body_lean_config() -> BodyLeanConfig {
    BodyLeanConfig {
        max_forward: 1.0,
        max_lateral: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_body_lean_state() -> BodyLeanState {
    BodyLeanState {
        forward: 0.0,
        backward: 0.0,
        lateral_left: 0.0,
        lateral_right: 0.0,
    }
}

#[allow(dead_code)]
pub fn bl_set_forward(state: &mut BodyLeanState, cfg: &BodyLeanConfig, v: f32) {
    state.forward = v.clamp(0.0, cfg.max_forward);
    state.backward = 0.0;
}

#[allow(dead_code)]
pub fn bl_set_backward(state: &mut BodyLeanState, cfg: &BodyLeanConfig, v: f32) {
    state.backward = v.clamp(0.0, cfg.max_forward);
    state.forward = 0.0;
}

#[allow(dead_code)]
pub fn bl_set_lateral(state: &mut BodyLeanState, cfg: &BodyLeanConfig, left: f32, right: f32) {
    state.lateral_left = left.clamp(0.0, cfg.max_lateral);
    state.lateral_right = right.clamp(0.0, cfg.max_lateral);
}

#[allow(dead_code)]
pub fn bl_reset(state: &mut BodyLeanState) {
    *state = new_body_lean_state();
}

#[allow(dead_code)]
pub fn bl_sagittal_angle_rad(state: &BodyLeanState) -> f32 {
    (state.forward - state.backward) * FRAC_PI_2 * 0.5
}

#[allow(dead_code)]
pub fn bl_is_neutral(state: &BodyLeanState) -> bool {
    let vals = [
        state.forward,
        state.backward,
        state.lateral_left,
        state.lateral_right,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn bl_blend(a: &BodyLeanState, b: &BodyLeanState, t: f32) -> BodyLeanState {
    let t = t.clamp(0.0, 1.0);
    BodyLeanState {
        forward: a.forward + (b.forward - a.forward) * t,
        backward: a.backward + (b.backward - a.backward) * t,
        lateral_left: a.lateral_left + (b.lateral_left - a.lateral_left) * t,
        lateral_right: a.lateral_right + (b.lateral_right - a.lateral_right) * t,
    }
}

#[allow(dead_code)]
pub fn bl_to_weights(state: &BodyLeanState) -> Vec<(String, f32)> {
    vec![
        ("body_lean_forward".to_string(), state.forward),
        ("body_lean_backward".to_string(), state.backward),
        ("body_lean_left".to_string(), state.lateral_left),
        ("body_lean_right".to_string(), state.lateral_right),
    ]
}

#[allow(dead_code)]
pub fn bl_to_json(state: &BodyLeanState) -> String {
    format!(
        r#"{{"forward":{:.4},"backward":{:.4},"lateral_left":{:.4},"lateral_right":{:.4}}}"#,
        state.forward, state.backward, state.lateral_left, state.lateral_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sane() {
        let cfg = default_body_lean_config();
        assert!((cfg.max_forward - 1.0).abs() < 1e-6);
        assert!((cfg.max_lateral - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_is_neutral() {
        let s = new_body_lean_state();
        assert!(bl_is_neutral(&s));
    }

    #[test]
    fn set_forward_clamps() {
        let cfg = default_body_lean_config();
        let mut s = new_body_lean_state();
        bl_set_forward(&mut s, &cfg, 5.0);
        assert!((s.forward - 1.0).abs() < 1e-6);
        assert_eq!(s.backward, 0.0);
    }

    #[test]
    fn set_backward_clears_forward() {
        let cfg = default_body_lean_config();
        let mut s = new_body_lean_state();
        bl_set_forward(&mut s, &cfg, 0.5);
        bl_set_backward(&mut s, &cfg, 0.3);
        assert_eq!(s.forward, 0.0);
        assert!((s.backward - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_lateral() {
        let cfg = default_body_lean_config();
        let mut s = new_body_lean_state();
        bl_set_lateral(&mut s, &cfg, 0.4, 0.6);
        assert!((s.lateral_left - 0.4).abs() < 1e-6);
        assert!((s.lateral_right - 0.6).abs() < 1e-6);
    }

    #[test]
    fn reset_clears_all() {
        let cfg = default_body_lean_config();
        let mut s = new_body_lean_state();
        bl_set_forward(&mut s, &cfg, 0.5);
        bl_reset(&mut s);
        assert!(bl_is_neutral(&s));
    }

    #[test]
    fn sagittal_angle_forward() {
        let cfg = default_body_lean_config();
        let mut s = new_body_lean_state();
        bl_set_forward(&mut s, &cfg, 1.0);
        assert!(bl_sagittal_angle_rad(&s) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = new_body_lean_state();
        let cfg = default_body_lean_config();
        let mut b = new_body_lean_state();
        bl_set_forward(&mut b, &cfg, 1.0);
        let mid = bl_blend(&a, &b, 0.5);
        assert!((mid.forward - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_body_lean_state();
        assert_eq!(bl_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_contains_fields() {
        let s = new_body_lean_state();
        let j = bl_to_json(&s);
        assert!(j.contains("forward"));
        assert!(j.contains("lateral_right"));
    }
}
