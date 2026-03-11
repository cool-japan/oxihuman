// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Neck tilt control — lateral and sagittal tilt of the neck column.

use std::f32::consts::FRAC_PI_4;

/// Configuration for neck tilt.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckTiltConfig {
    pub max_tilt_rad: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckTiltState {
    pub lateral_tilt_rad: f32,
    pub sagittal_tilt_rad: f32,
}

#[allow(dead_code)]
pub fn default_neck_tilt_config() -> NeckTiltConfig {
    NeckTiltConfig {
        max_tilt_rad: FRAC_PI_4,
    }
}

#[allow(dead_code)]
pub fn new_neck_tilt_state() -> NeckTiltState {
    NeckTiltState {
        lateral_tilt_rad: 0.0,
        sagittal_tilt_rad: 0.0,
    }
}

#[allow(dead_code)]
pub fn ntilt_set_lateral(state: &mut NeckTiltState, cfg: &NeckTiltConfig, v: f32) {
    state.lateral_tilt_rad = v.clamp(-cfg.max_tilt_rad, cfg.max_tilt_rad);
}

#[allow(dead_code)]
pub fn ntilt_set_sagittal(state: &mut NeckTiltState, cfg: &NeckTiltConfig, v: f32) {
    state.sagittal_tilt_rad = v.clamp(-cfg.max_tilt_rad, cfg.max_tilt_rad);
}

#[allow(dead_code)]
pub fn ntilt_reset(state: &mut NeckTiltState) {
    *state = new_neck_tilt_state();
}

#[allow(dead_code)]
pub fn ntilt_is_neutral(state: &NeckTiltState) -> bool {
    state.lateral_tilt_rad.abs() < 1e-6 && state.sagittal_tilt_rad.abs() < 1e-6
}

#[allow(dead_code)]
pub fn ntilt_total_angle_rad(state: &NeckTiltState) -> f32 {
    (state.lateral_tilt_rad * state.lateral_tilt_rad
        + state.sagittal_tilt_rad * state.sagittal_tilt_rad)
        .sqrt()
}

#[allow(dead_code)]
pub fn ntilt_blend(a: &NeckTiltState, b: &NeckTiltState, t: f32) -> NeckTiltState {
    let t = t.clamp(0.0, 1.0);
    NeckTiltState {
        lateral_tilt_rad: a.lateral_tilt_rad + (b.lateral_tilt_rad - a.lateral_tilt_rad) * t,
        sagittal_tilt_rad: a.sagittal_tilt_rad + (b.sagittal_tilt_rad - a.sagittal_tilt_rad) * t,
    }
}

#[allow(dead_code)]
pub fn ntilt_to_weights(state: &NeckTiltState) -> Vec<(String, f32)> {
    let norm = 1.0 / FRAC_PI_4;
    vec![
        (
            "neck_tilt_lateral".to_string(),
            state.lateral_tilt_rad * norm,
        ),
        (
            "neck_tilt_sagittal".to_string(),
            state.sagittal_tilt_rad * norm,
        ),
    ]
}

#[allow(dead_code)]
pub fn ntilt_to_json(state: &NeckTiltState) -> String {
    format!(
        r#"{{"lateral_tilt_rad":{:.4},"sagittal_tilt_rad":{:.4}}}"#,
        state.lateral_tilt_rad, state.sagittal_tilt_rad
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_neck_tilt_config();
        assert!((cfg.max_tilt_rad - FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_neck_tilt_state();
        assert!(ntilt_is_neutral(&s));
    }

    #[test]
    fn set_lateral_clamps() {
        let cfg = default_neck_tilt_config();
        let mut s = new_neck_tilt_state();
        ntilt_set_lateral(&mut s, &cfg, 10.0);
        assert!((s.lateral_tilt_rad - FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn set_sagittal() {
        let cfg = default_neck_tilt_config();
        let mut s = new_neck_tilt_state();
        ntilt_set_sagittal(&mut s, &cfg, 0.3);
        assert!((s.sagittal_tilt_rad - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_lateral_negative() {
        let cfg = default_neck_tilt_config();
        let mut s = new_neck_tilt_state();
        ntilt_set_lateral(&mut s, &cfg, -FRAC_PI_4);
        assert!((s.lateral_tilt_rad + FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn total_angle_pythagoras() {
        let cfg = default_neck_tilt_config();
        let mut s = new_neck_tilt_state();
        ntilt_set_lateral(&mut s, &cfg, 0.3);
        ntilt_set_sagittal(&mut s, &cfg, 0.4);
        assert!((ntilt_total_angle_rad(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_neck_tilt_config();
        let mut s = new_neck_tilt_state();
        ntilt_set_lateral(&mut s, &cfg, 0.5);
        ntilt_reset(&mut s);
        assert!(ntilt_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_neck_tilt_state();
        let cfg = default_neck_tilt_config();
        let mut b = new_neck_tilt_state();
        ntilt_set_lateral(&mut b, &cfg, 0.6);
        let m = ntilt_blend(&a, &b, 0.5);
        assert!((m.lateral_tilt_rad - 0.3).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_neck_tilt_state();
        assert_eq!(ntilt_to_weights(&s).len(), 2);
    }

    #[test]
    fn to_json_fields() {
        let s = new_neck_tilt_state();
        let j = ntilt_to_json(&s);
        assert!(j.contains("lateral_tilt_rad"));
    }
}
