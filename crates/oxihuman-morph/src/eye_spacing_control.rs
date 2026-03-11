// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Eye spacing morphology controls for inter-ocular distance.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSpacingConfig {
    pub distance: f32,
    pub depth: f32,
    pub vertical_offset: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSpacingState {
    pub distance: f32,
    pub depth: f32,
    pub vertical_offset: f32,
    pub convergence: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSpacingWeights {
    pub wide_set: f32,
    pub close_set: f32,
    pub deep_set: f32,
    pub shallow: f32,
    pub converged: f32,
}

#[allow(dead_code)]
pub fn default_eye_spacing_config() -> EyeSpacingConfig {
    EyeSpacingConfig {
        distance: 0.5,
        depth: 0.5,
        vertical_offset: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_eye_spacing_state() -> EyeSpacingState {
    EyeSpacingState {
        distance: 0.5,
        depth: 0.5,
        vertical_offset: 0.5,
        convergence: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_eye_distance(state: &mut EyeSpacingState, value: f32) {
    state.distance = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_depth(state: &mut EyeSpacingState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_vertical_offset(state: &mut EyeSpacingState, value: f32) {
    state.vertical_offset = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_convergence(state: &mut EyeSpacingState, value: f32) {
    state.convergence = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_eye_spacing_weights(
    state: &EyeSpacingState,
    cfg: &EyeSpacingConfig,
) -> EyeSpacingWeights {
    let d = state.distance * cfg.distance;
    let wide_set = (d * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let close_set = (1.0 - d).clamp(0.0, 1.0);
    let dp = state.depth * cfg.depth;
    let deep_set = dp.clamp(0.0, 1.0);
    let shallow = (1.0 - dp).clamp(0.0, 1.0);
    let converged = state.convergence.clamp(0.0, 1.0);
    EyeSpacingWeights {
        wide_set,
        close_set,
        deep_set,
        shallow,
        converged,
    }
}

#[allow(dead_code)]
pub fn eye_spacing_to_json(state: &EyeSpacingState) -> String {
    format!(
        r#"{{"distance":{},"depth":{},"vertical_offset":{},"convergence":{}}}"#,
        state.distance, state.depth, state.vertical_offset, state.convergence
    )
}

#[allow(dead_code)]
pub fn blend_eye_spacings(a: &EyeSpacingState, b: &EyeSpacingState, t: f32) -> EyeSpacingState {
    let t = t.clamp(0.0, 1.0);
    EyeSpacingState {
        distance: a.distance + (b.distance - a.distance) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        vertical_offset: a.vertical_offset + (b.vertical_offset - a.vertical_offset) * t,
        convergence: a.convergence + (b.convergence - a.convergence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_spacing_config();
        assert!((0.0..=1.0).contains(&cfg.distance));
    }

    #[test]
    fn test_new_state() {
        let s = new_eye_spacing_state();
        assert!((s.distance - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_distance_clamp() {
        let mut s = new_eye_spacing_state();
        set_eye_distance(&mut s, 1.5);
        assert!((s.distance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth() {
        let mut s = new_eye_spacing_state();
        set_eye_depth(&mut s, 0.8);
        assert!((s.depth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical_offset() {
        let mut s = new_eye_spacing_state();
        set_eye_vertical_offset(&mut s, 0.7);
        assert!((s.vertical_offset - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_convergence() {
        let mut s = new_eye_spacing_state();
        set_eye_convergence(&mut s, 0.6);
        assert!((s.convergence - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_eye_spacing_state();
        let cfg = default_eye_spacing_config();
        let w = compute_eye_spacing_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide_set));
        assert!((0.0..=1.0).contains(&w.deep_set));
    }

    #[test]
    fn test_to_json() {
        let s = new_eye_spacing_state();
        let json = eye_spacing_to_json(&s);
        assert!(json.contains("distance"));
        assert!(json.contains("convergence"));
    }

    #[test]
    fn test_blend() {
        let a = new_eye_spacing_state();
        let mut b = new_eye_spacing_state();
        b.distance = 1.0;
        let mid = blend_eye_spacings(&a, &b, 0.5);
        assert!((mid.distance - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_eye_spacing_state();
        let r = blend_eye_spacings(&a, &a, 0.5);
        assert!((r.distance - a.distance).abs() < 1e-6);
    }
}
