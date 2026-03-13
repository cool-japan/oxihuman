// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Gluteal fold morphology controls for crease depth and shape.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlutealFoldConfig {
    pub depth: f32,
    pub length: f32,
    pub curvature: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlutealFoldState {
    pub depth: f32,
    pub length: f32,
    pub curvature: f32,
    pub asymmetry: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlutealFoldWeights {
    pub deep: f32,
    pub shallow: f32,
    pub long: f32,
    pub curved: f32,
    pub asymmetric: f32,
}

#[allow(dead_code)]
pub fn default_gluteal_fold_config() -> GlutealFoldConfig {
    GlutealFoldConfig { depth: 0.5, length: 0.5, curvature: 0.3 }
}

#[allow(dead_code)]
pub fn new_gluteal_fold_state() -> GlutealFoldState {
    GlutealFoldState { depth: 0.5, length: 0.5, curvature: 0.3, asymmetry: 0.0 }
}

#[allow(dead_code)]
pub fn set_gluteal_depth(state: &mut GlutealFoldState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gluteal_length(state: &mut GlutealFoldState, value: f32) {
    state.length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gluteal_curvature(state: &mut GlutealFoldState, value: f32) {
    state.curvature = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gluteal_asymmetry(state: &mut GlutealFoldState, value: f32) {
    state.asymmetry = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_gluteal_fold_weights(state: &GlutealFoldState, cfg: &GlutealFoldConfig) -> GlutealFoldWeights {
    let d = state.depth * cfg.depth;
    let deep = (d * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let shallow = (1.0 - d).clamp(0.0, 1.0);
    let long = (state.length * cfg.length).clamp(0.0, 1.0);
    let curved = (state.curvature * cfg.curvature).clamp(0.0, 1.0);
    let asymmetric = state.asymmetry.clamp(0.0, 1.0);
    GlutealFoldWeights { deep, shallow, long, curved, asymmetric }
}

#[allow(dead_code)]
pub fn gluteal_fold_to_json(state: &GlutealFoldState) -> String {
    format!(
        r#"{{"depth":{},"length":{},"curvature":{},"asymmetry":{}}}"#,
        state.depth, state.length, state.curvature, state.asymmetry
    )
}

#[allow(dead_code)]
pub fn blend_gluteal_folds(a: &GlutealFoldState, b: &GlutealFoldState, t: f32) -> GlutealFoldState {
    let t = t.clamp(0.0, 1.0);
    GlutealFoldState {
        depth: a.depth + (b.depth - a.depth) * t,
        length: a.length + (b.length - a.length) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_gluteal_fold_config();
        assert!((0.0..=1.0).contains(&cfg.depth));
    }

    #[test]
    fn test_new_state() {
        let s = new_gluteal_fold_state();
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamp() {
        let mut s = new_gluteal_fold_state();
        set_gluteal_depth(&mut s, 1.5);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_length() {
        let mut s = new_gluteal_fold_state();
        set_gluteal_length(&mut s, 0.8);
        assert!((s.length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_curvature() {
        let mut s = new_gluteal_fold_state();
        set_gluteal_curvature(&mut s, 0.7);
        assert!((s.curvature - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_asymmetry() {
        let mut s = new_gluteal_fold_state();
        set_gluteal_asymmetry(&mut s, 0.4);
        assert!((s.asymmetry - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_gluteal_fold_state();
        let cfg = default_gluteal_fold_config();
        let w = compute_gluteal_fold_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.deep));
        assert!((0.0..=1.0).contains(&w.long));
    }

    #[test]
    fn test_to_json() {
        let s = new_gluteal_fold_state();
        let json = gluteal_fold_to_json(&s);
        assert!(json.contains("depth"));
        assert!(json.contains("asymmetry"));
    }

    #[test]
    fn test_blend() {
        let a = new_gluteal_fold_state();
        let mut b = new_gluteal_fold_state();
        b.depth = 1.0;
        let mid = blend_gluteal_folds(&a, &b, 0.5);
        assert!((mid.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_gluteal_fold_state();
        let r = blend_gluteal_folds(&a, &a, 0.5);
        assert!((r.depth - a.depth).abs() < 1e-6);
    }
}
