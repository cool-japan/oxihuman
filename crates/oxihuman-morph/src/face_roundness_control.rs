// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Face roundness control: adjusts overall face roundness from angular to round.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceRoundnessConfig {
    pub min_roundness: f32,
    pub max_roundness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceRoundnessState {
    pub roundness: f32,
    pub jaw_softness: f32,
    pub forehead_curve: f32,
}

#[allow(dead_code)]
pub fn default_face_roundness_config() -> FaceRoundnessConfig {
    FaceRoundnessConfig {
        min_roundness: 0.0,
        max_roundness: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_face_roundness_state() -> FaceRoundnessState {
    FaceRoundnessState {
        roundness: 0.5,
        jaw_softness: 0.5,
        forehead_curve: 0.5,
    }
}

#[allow(dead_code)]
pub fn fr_set_roundness(state: &mut FaceRoundnessState, cfg: &FaceRoundnessConfig, v: f32) {
    state.roundness = v.clamp(cfg.min_roundness, cfg.max_roundness);
}

#[allow(dead_code)]
pub fn fr_set_jaw_softness(state: &mut FaceRoundnessState, v: f32) {
    state.jaw_softness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fr_set_forehead_curve(state: &mut FaceRoundnessState, v: f32) {
    state.forehead_curve = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fr_reset(state: &mut FaceRoundnessState) {
    *state = new_face_roundness_state();
}

#[allow(dead_code)]
pub fn fr_overall_softness(state: &FaceRoundnessState) -> f32 {
    (state.roundness + state.jaw_softness + state.forehead_curve) / 3.0
}

/// Approximate face perimeter as ellipse.
#[allow(dead_code)]
pub fn fr_perimeter_estimate(state: &FaceRoundnessState) -> f32 {
    let a = 0.5 + state.roundness * 0.3;
    let b = 0.5 + state.jaw_softness * 0.2;
    PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
}

#[allow(dead_code)]
pub fn fr_to_weights(state: &FaceRoundnessState) -> Vec<(String, f32)> {
    vec![
        ("face_roundness".to_string(), state.roundness),
        ("face_jaw_softness".to_string(), state.jaw_softness),
        ("face_forehead_curve".to_string(), state.forehead_curve),
    ]
}

#[allow(dead_code)]
pub fn fr_to_json(state: &FaceRoundnessState) -> String {
    format!(
        r#"{{"roundness":{:.4},"jaw_softness":{:.4},"forehead_curve":{:.4}}}"#,
        state.roundness, state.jaw_softness, state.forehead_curve
    )
}

#[allow(dead_code)]
pub fn fr_blend(a: &FaceRoundnessState, b: &FaceRoundnessState, t: f32) -> FaceRoundnessState {
    let t = t.clamp(0.0, 1.0);
    FaceRoundnessState {
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        jaw_softness: a.jaw_softness + (b.jaw_softness - a.jaw_softness) * t,
        forehead_curve: a.forehead_curve + (b.forehead_curve - a.forehead_curve) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_face_roundness_config();
        assert!(cfg.min_roundness.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_face_roundness_state();
        assert!((s.roundness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness_clamps() {
        let cfg = default_face_roundness_config();
        let mut s = new_face_roundness_state();
        fr_set_roundness(&mut s, &cfg, 5.0);
        assert!((s.roundness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_jaw_softness() {
        let mut s = new_face_roundness_state();
        fr_set_jaw_softness(&mut s, 0.8);
        assert!((s.jaw_softness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_forehead_curve() {
        let mut s = new_face_roundness_state();
        fr_set_forehead_curve(&mut s, 0.3);
        assert!((s.forehead_curve - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_face_roundness_config();
        let mut s = new_face_roundness_state();
        fr_set_roundness(&mut s, &cfg, 0.9);
        fr_reset(&mut s);
        assert!((s.roundness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_overall_softness() {
        let s = new_face_roundness_state();
        let o = fr_overall_softness(&s);
        assert!((o - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_perimeter_positive() {
        let s = new_face_roundness_state();
        assert!(fr_perimeter_estimate(&s) > 0.0);
    }

    #[test]
    fn test_blend() {
        let a = new_face_roundness_state();
        let mut b = new_face_roundness_state();
        b.roundness = 1.0;
        let mid = fr_blend(&a, &b, 0.5);
        assert!((mid.roundness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_face_roundness_state();
        assert_eq!(fr_to_weights(&s).len(), 3);
    }
}
