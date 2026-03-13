// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Neck flexion morph — controls forward/backward neck flexion and lateral tilt.

use std::f32::consts::FRAC_PI_2;

/// Configuration for neck flexion control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckFlexionConfig {
    pub max_flexion: f32,
    pub max_lateral: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckFlexionState {
    pub forward_flexion: f32,
    pub backward_extension: f32,
    pub lateral_left: f32,
    pub lateral_right: f32,
}

#[allow(dead_code)]
pub fn default_neck_flexion_config() -> NeckFlexionConfig {
    NeckFlexionConfig {
        max_flexion: 1.0,
        max_lateral: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_neck_flexion_state() -> NeckFlexionState {
    NeckFlexionState {
        forward_flexion: 0.0,
        backward_extension: 0.0,
        lateral_left: 0.0,
        lateral_right: 0.0,
    }
}

#[allow(dead_code)]
pub fn nf_set_forward(state: &mut NeckFlexionState, cfg: &NeckFlexionConfig, v: f32) {
    state.forward_flexion = v.clamp(0.0, cfg.max_flexion);
    state.backward_extension = 0.0;
}

#[allow(dead_code)]
pub fn nf_set_backward(state: &mut NeckFlexionState, cfg: &NeckFlexionConfig, v: f32) {
    state.backward_extension = v.clamp(0.0, cfg.max_flexion);
    state.forward_flexion = 0.0;
}

#[allow(dead_code)]
pub fn nf_set_lateral(
    state: &mut NeckFlexionState,
    cfg: &NeckFlexionConfig,
    left: f32,
    right: f32,
) {
    state.lateral_left = left.clamp(0.0, cfg.max_lateral);
    state.lateral_right = right.clamp(0.0, cfg.max_lateral);
}

#[allow(dead_code)]
pub fn nf_reset(state: &mut NeckFlexionState) {
    *state = new_neck_flexion_state();
}

#[allow(dead_code)]
pub fn nf_is_neutral(state: &NeckFlexionState) -> bool {
    let vals = [
        state.forward_flexion,
        state.backward_extension,
        state.lateral_left,
        state.lateral_right,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn nf_sagittal_angle_rad(state: &NeckFlexionState) -> f32 {
    (state.forward_flexion - state.backward_extension) * FRAC_PI_2 * 0.5
}

#[allow(dead_code)]
pub fn nf_lateral_angle_rad(state: &NeckFlexionState) -> f32 {
    (state.lateral_left - state.lateral_right) * FRAC_PI_2 * 0.3
}

#[allow(dead_code)]
pub fn nf_blend(a: &NeckFlexionState, b: &NeckFlexionState, t: f32) -> NeckFlexionState {
    let t = t.clamp(0.0, 1.0);
    NeckFlexionState {
        forward_flexion: a.forward_flexion + (b.forward_flexion - a.forward_flexion) * t,
        backward_extension: a.backward_extension
            + (b.backward_extension - a.backward_extension) * t,
        lateral_left: a.lateral_left + (b.lateral_left - a.lateral_left) * t,
        lateral_right: a.lateral_right + (b.lateral_right - a.lateral_right) * t,
    }
}

#[allow(dead_code)]
pub fn nf_to_weights(state: &NeckFlexionState) -> Vec<(String, f32)> {
    vec![
        ("neck_flexion_fwd".to_string(), state.forward_flexion),
        ("neck_extension_bwd".to_string(), state.backward_extension),
        ("neck_lateral_l".to_string(), state.lateral_left),
        ("neck_lateral_r".to_string(), state.lateral_right),
    ]
}

#[allow(dead_code)]
pub fn nf_to_json(state: &NeckFlexionState) -> String {
    format!(
        r#"{{"forward_flexion":{:.4},"backward_extension":{:.4},"lateral_left":{:.4},"lateral_right":{:.4}}}"#,
        state.forward_flexion, state.backward_extension, state.lateral_left, state.lateral_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_neck_flexion_config();
        assert!((cfg.max_flexion - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_neck_flexion_state();
        assert!(nf_is_neutral(&s));
    }

    #[test]
    fn set_forward_clears_backward() {
        let cfg = default_neck_flexion_config();
        let mut s = new_neck_flexion_state();
        nf_set_backward(&mut s, &cfg, 0.5);
        nf_set_forward(&mut s, &cfg, 0.3);
        assert_eq!(s.backward_extension, 0.0);
        assert!((s.forward_flexion - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_backward_clears_forward() {
        let cfg = default_neck_flexion_config();
        let mut s = new_neck_flexion_state();
        nf_set_forward(&mut s, &cfg, 0.5);
        nf_set_backward(&mut s, &cfg, 0.4);
        assert_eq!(s.forward_flexion, 0.0);
        assert!((s.backward_extension - 0.4).abs() < 1e-6);
    }

    #[test]
    fn set_lateral() {
        let cfg = default_neck_flexion_config();
        let mut s = new_neck_flexion_state();
        nf_set_lateral(&mut s, &cfg, 0.4, 0.6);
        assert!((s.lateral_left - 0.4).abs() < 1e-6);
        assert!((s.lateral_right - 0.6).abs() < 1e-6);
    }

    #[test]
    fn sagittal_angle_forward_positive() {
        let cfg = default_neck_flexion_config();
        let mut s = new_neck_flexion_state();
        nf_set_forward(&mut s, &cfg, 1.0);
        assert!(nf_sagittal_angle_rad(&s) > 0.0);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_neck_flexion_config();
        let mut s = new_neck_flexion_state();
        nf_set_forward(&mut s, &cfg, 0.5);
        nf_reset(&mut s);
        assert!(nf_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_neck_flexion_state();
        let cfg = default_neck_flexion_config();
        let mut b = new_neck_flexion_state();
        nf_set_forward(&mut b, &cfg, 1.0);
        let mid = nf_blend(&a, &b, 0.5);
        assert!((mid.forward_flexion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_neck_flexion_state();
        assert_eq!(nf_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_neck_flexion_state();
        let j = nf_to_json(&s);
        assert!(j.contains("forward_flexion"));
        assert!(j.contains("lateral_right"));
    }
}
