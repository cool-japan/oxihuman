// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face vertical control — vertical proportions of the face regions.

/// Configuration for face vertical control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceVerticalConfig {
    pub max_scale: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceVerticalState {
    pub upper_third: f32,
    pub middle_third: f32,
    pub lower_third: f32,
}

#[allow(dead_code)]
pub fn default_face_vertical_config() -> FaceVerticalConfig {
    FaceVerticalConfig { max_scale: 1.5 }
}

#[allow(dead_code)]
pub fn new_face_vertical_state() -> FaceVerticalState {
    FaceVerticalState {
        upper_third: 0.0,
        middle_third: 0.0,
        lower_third: 0.0,
    }
}

#[allow(dead_code)]
pub fn fv_set_upper(state: &mut FaceVerticalState, cfg: &FaceVerticalConfig, v: f32) {
    state.upper_third = v.clamp(-cfg.max_scale, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fv_set_middle(state: &mut FaceVerticalState, cfg: &FaceVerticalConfig, v: f32) {
    state.middle_third = v.clamp(-cfg.max_scale, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fv_set_lower(state: &mut FaceVerticalState, cfg: &FaceVerticalConfig, v: f32) {
    state.lower_third = v.clamp(-cfg.max_scale, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fv_set_all(state: &mut FaceVerticalState, cfg: &FaceVerticalConfig, v: f32) {
    let clamped = v.clamp(-cfg.max_scale, cfg.max_scale);
    state.upper_third = clamped;
    state.middle_third = clamped;
    state.lower_third = clamped;
}

#[allow(dead_code)]
pub fn fv_reset(state: &mut FaceVerticalState) {
    *state = new_face_vertical_state();
}

#[allow(dead_code)]
pub fn fv_is_neutral(state: &FaceVerticalState) -> bool {
    state.upper_third.abs() < 1e-6
        && state.middle_third.abs() < 1e-6
        && state.lower_third.abs() < 1e-6
}

#[allow(dead_code)]
pub fn fv_total_scale(state: &FaceVerticalState) -> f32 {
    state.upper_third + state.middle_third + state.lower_third
}

#[allow(dead_code)]
pub fn fv_blend(a: &FaceVerticalState, b: &FaceVerticalState, t: f32) -> FaceVerticalState {
    let t = t.clamp(0.0, 1.0);
    FaceVerticalState {
        upper_third: a.upper_third + (b.upper_third - a.upper_third) * t,
        middle_third: a.middle_third + (b.middle_third - a.middle_third) * t,
        lower_third: a.lower_third + (b.lower_third - a.lower_third) * t,
    }
}

#[allow(dead_code)]
pub fn fv_to_weights(state: &FaceVerticalState) -> Vec<(String, f32)> {
    vec![
        ("face_upper_third".to_string(), state.upper_third),
        ("face_middle_third".to_string(), state.middle_third),
        ("face_lower_third".to_string(), state.lower_third),
    ]
}

#[allow(dead_code)]
pub fn fv_to_json(state: &FaceVerticalState) -> String {
    format!(
        r#"{{"upper_third":{:.4},"middle_third":{:.4},"lower_third":{:.4}}}"#,
        state.upper_third, state.middle_third, state.lower_third
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_face_vertical_config();
        assert!((cfg.max_scale - 1.5).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_face_vertical_state();
        assert!(fv_is_neutral(&s));
    }

    #[test]
    fn set_upper_clamps() {
        let cfg = default_face_vertical_config();
        let mut s = new_face_vertical_state();
        fv_set_upper(&mut s, &cfg, 5.0);
        assert!((s.upper_third - 1.5).abs() < 1e-6);
    }

    #[test]
    fn set_middle() {
        let cfg = default_face_vertical_config();
        let mut s = new_face_vertical_state();
        fv_set_middle(&mut s, &cfg, 0.3);
        assert!((s.middle_third - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_all() {
        let cfg = default_face_vertical_config();
        let mut s = new_face_vertical_state();
        fv_set_all(&mut s, &cfg, 0.5);
        assert!((fv_total_scale(&s) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_face_vertical_config();
        let mut s = new_face_vertical_state();
        fv_set_lower(&mut s, &cfg, 0.9);
        fv_reset(&mut s);
        assert!(fv_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_face_vertical_state();
        let cfg = default_face_vertical_config();
        let mut b = new_face_vertical_state();
        fv_set_upper(&mut b, &cfg, 1.0);
        let m = fv_blend(&a, &b, 0.5);
        assert!((m.upper_third - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_face_vertical_state();
        assert_eq!(fv_to_weights(&s).len(), 3);
    }

    #[test]
    fn total_scale_sum() {
        let cfg = default_face_vertical_config();
        let mut s = new_face_vertical_state();
        fv_set_upper(&mut s, &cfg, 0.1);
        fv_set_middle(&mut s, &cfg, 0.2);
        fv_set_lower(&mut s, &cfg, 0.3);
        assert!((fv_total_scale(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let s = new_face_vertical_state();
        let j = fv_to_json(&s);
        assert!(j.contains("upper_third"));
    }
}
