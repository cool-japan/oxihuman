// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Face vertical length morph.

#![allow(dead_code)]

/// Configuration for face length morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceLengthConfig {
    pub min_scale: f32,
    pub max_scale: f32,
}

/// Runtime state for face length morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceLengthState {
    pub scale: f32,
    pub upper_contribution: f32,
    pub lower_contribution: f32,
}

#[allow(dead_code)]
pub fn default_face_length_config() -> FaceLengthConfig {
    FaceLengthConfig {
        min_scale: 0.5,
        max_scale: 2.0,
    }
}

#[allow(dead_code)]
pub fn new_face_length_state() -> FaceLengthState {
    FaceLengthState {
        scale: 1.0,
        upper_contribution: 0.5,
        lower_contribution: 0.5,
    }
}

#[allow(dead_code)]
pub fn facel_set_scale(state: &mut FaceLengthState, cfg: &FaceLengthConfig, v: f32) {
    state.scale = v.clamp(cfg.min_scale, cfg.max_scale);
}

#[allow(dead_code)]
pub fn facel_set_upper_contrib(state: &mut FaceLengthState, v: f32) {
    state.upper_contribution = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn facel_set_lower_contrib(state: &mut FaceLengthState, v: f32) {
    state.lower_contribution = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn facel_reset(state: &mut FaceLengthState) {
    *state = new_face_length_state();
}

#[allow(dead_code)]
pub fn facel_to_weights(state: &FaceLengthState) -> Vec<(String, f32)> {
    vec![
        ("face_length_scale".to_string(), state.scale),
        (
            "face_length_upper_contrib".to_string(),
            state.upper_contribution,
        ),
        (
            "face_length_lower_contrib".to_string(),
            state.lower_contribution,
        ),
    ]
}

#[allow(dead_code)]
pub fn facel_to_json(state: &FaceLengthState) -> String {
    format!(
        r#"{{"scale":{:.4},"upper_contribution":{:.4},"lower_contribution":{:.4}}}"#,
        state.scale, state.upper_contribution, state.lower_contribution
    )
}

#[allow(dead_code)]
pub fn facel_clamp(state: &mut FaceLengthState, cfg: &FaceLengthConfig) {
    state.scale = state.scale.clamp(cfg.min_scale, cfg.max_scale);
    state.upper_contribution = state.upper_contribution.clamp(0.0, 1.0);
    state.lower_contribution = state.lower_contribution.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn facel_effective_scale(state: &FaceLengthState) -> f32 {
    state.scale * (1.0 + (state.upper_contribution + state.lower_contribution) * 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_face_length_config();
        assert!((cfg.min_scale - 0.5).abs() < 1e-6);
        assert!((cfg.max_scale - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_face_length_state();
        assert!((s.scale - 1.0).abs() < 1e-6);
        assert!((s.upper_contribution - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_scale_clamps() {
        let cfg = default_face_length_config();
        let mut s = new_face_length_state();
        facel_set_scale(&mut s, &cfg, 10.0);
        assert!((s.scale - 2.0).abs() < 1e-6);
        facel_set_scale(&mut s, &cfg, 0.1);
        assert!((s.scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper_contrib() {
        let mut s = new_face_length_state();
        facel_set_upper_contrib(&mut s, 0.8);
        assert!((s.upper_contribution - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_lower_contrib() {
        let mut s = new_face_length_state();
        facel_set_lower_contrib(&mut s, 0.2);
        assert!((s.lower_contribution - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_face_length_config();
        let mut s = new_face_length_state();
        facel_set_scale(&mut s, &cfg, 1.5);
        facel_reset(&mut s);
        assert!((s.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_face_length_state();
        assert_eq!(facel_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_face_length_state();
        let j = facel_to_json(&s);
        assert!(j.contains("scale"));
        assert!(j.contains("upper_contribution"));
    }

    #[test]
    fn test_effective_scale() {
        let s = new_face_length_state();
        let eff = facel_effective_scale(&s);
        assert!(eff >= 1.0);
    }
}
