// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Overall face width morph.

#![allow(dead_code)]

/// Configuration for face width morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceWidthConfig {
    pub min_scale: f32,
    pub max_scale: f32,
}

/// Runtime state for face width morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceWidthState {
    pub scale: f32,
    pub cheek_contribution: f32,
    pub jaw_contribution: f32,
}

#[allow(dead_code)]
pub fn default_face_width_config() -> FaceWidthConfig {
    FaceWidthConfig {
        min_scale: 0.5,
        max_scale: 2.0,
    }
}

#[allow(dead_code)]
pub fn new_face_width_state() -> FaceWidthState {
    FaceWidthState {
        scale: 1.0,
        cheek_contribution: 0.5,
        jaw_contribution: 0.5,
    }
}

#[allow(dead_code)]
pub fn facew_set_scale(state: &mut FaceWidthState, cfg: &FaceWidthConfig, v: f32) {
    state.scale = v.clamp(cfg.min_scale, cfg.max_scale);
}

#[allow(dead_code)]
pub fn facew_set_cheek_contrib(state: &mut FaceWidthState, v: f32) {
    state.cheek_contribution = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn facew_set_jaw_contrib(state: &mut FaceWidthState, v: f32) {
    state.jaw_contribution = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn facew_reset(state: &mut FaceWidthState) {
    *state = new_face_width_state();
}

#[allow(dead_code)]
pub fn facew_to_weights(state: &FaceWidthState) -> Vec<(String, f32)> {
    vec![
        ("face_width_scale".to_string(), state.scale),
        (
            "face_width_cheek_contrib".to_string(),
            state.cheek_contribution,
        ),
        ("face_width_jaw_contrib".to_string(), state.jaw_contribution),
    ]
}

#[allow(dead_code)]
pub fn facew_to_json(state: &FaceWidthState) -> String {
    format!(
        r#"{{"scale":{:.4},"cheek_contribution":{:.4},"jaw_contribution":{:.4}}}"#,
        state.scale, state.cheek_contribution, state.jaw_contribution
    )
}

#[allow(dead_code)]
pub fn facew_clamp(state: &mut FaceWidthState, cfg: &FaceWidthConfig) {
    state.scale = state.scale.clamp(cfg.min_scale, cfg.max_scale);
    state.cheek_contribution = state.cheek_contribution.clamp(0.0, 1.0);
    state.jaw_contribution = state.jaw_contribution.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn facew_effective_scale(state: &FaceWidthState) -> f32 {
    state.scale
        * (1.0
            + (state.cheek_contribution + state.jaw_contribution) * 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_face_width_config();
        assert!((cfg.min_scale - 0.5).abs() < 1e-6);
        assert!((cfg.max_scale - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_face_width_state();
        assert!((s.scale - 1.0).abs() < 1e-6);
        assert!((s.cheek_contribution - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_scale_clamps() {
        let cfg = default_face_width_config();
        let mut s = new_face_width_state();
        facew_set_scale(&mut s, &cfg, 5.0);
        assert!((s.scale - 2.0).abs() < 1e-6);
        facew_set_scale(&mut s, &cfg, 0.1);
        assert!((s.scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_cheek_contrib() {
        let mut s = new_face_width_state();
        facew_set_cheek_contrib(&mut s, 0.7);
        assert!((s.cheek_contribution - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_jaw_contrib() {
        let mut s = new_face_width_state();
        facew_set_jaw_contrib(&mut s, 0.3);
        assert!((s.jaw_contribution - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_face_width_config();
        let mut s = new_face_width_state();
        facew_set_scale(&mut s, &cfg, 1.5);
        facew_reset(&mut s);
        assert!((s.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_face_width_state();
        assert_eq!(facew_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_face_width_state();
        let j = facew_to_json(&s);
        assert!(j.contains("scale"));
        assert!(j.contains("cheek_contribution"));
    }

    #[test]
    fn test_effective_scale() {
        let s = new_face_width_state();
        let eff = facew_effective_scale(&s);
        assert!(eff >= 1.0);
    }
}
