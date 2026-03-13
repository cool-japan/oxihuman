// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Calf shape morphology controls for muscle definition and taper.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalfShapeConfig {
    pub muscle_size: f32,
    pub taper: f32,
    pub definition: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalfShapeState {
    pub muscle_size: f32,
    pub taper: f32,
    pub definition: f32,
    pub inner_bulge: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalfShapeWeights {
    pub muscular: f32,
    pub slim: f32,
    pub tapered: f32,
    pub defined: f32,
    pub bulging: f32,
}

#[allow(dead_code)]
pub fn default_calf_shape_config() -> CalfShapeConfig {
    CalfShapeConfig { muscle_size: 0.5, taper: 0.5, definition: 0.5 }
}

#[allow(dead_code)]
pub fn new_calf_shape_state() -> CalfShapeState {
    CalfShapeState { muscle_size: 0.5, taper: 0.5, definition: 0.5, inner_bulge: 0.0 }
}

#[allow(dead_code)]
pub fn set_calf_muscle_size(state: &mut CalfShapeState, value: f32) {
    state.muscle_size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_calf_taper(state: &mut CalfShapeState, value: f32) {
    state.taper = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_calf_definition(state: &mut CalfShapeState, value: f32) {
    state.definition = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_calf_inner_bulge(state: &mut CalfShapeState, value: f32) {
    state.inner_bulge = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_calf_shape_weights(state: &CalfShapeState, cfg: &CalfShapeConfig) -> CalfShapeWeights {
    let m = state.muscle_size * cfg.muscle_size;
    let muscular = (m * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let slim = (1.0 - m).clamp(0.0, 1.0);
    let tapered = (state.taper * cfg.taper).clamp(0.0, 1.0);
    let defined = (state.definition * cfg.definition).clamp(0.0, 1.0);
    let bulging = state.inner_bulge.clamp(0.0, 1.0);
    CalfShapeWeights { muscular, slim, tapered, defined, bulging }
}

#[allow(dead_code)]
pub fn calf_shape_to_json(state: &CalfShapeState) -> String {
    format!(
        r#"{{"muscle_size":{},"taper":{},"definition":{},"inner_bulge":{}}}"#,
        state.muscle_size, state.taper, state.definition, state.inner_bulge
    )
}

#[allow(dead_code)]
pub fn blend_calf_shapes(a: &CalfShapeState, b: &CalfShapeState, t: f32) -> CalfShapeState {
    let t = t.clamp(0.0, 1.0);
    CalfShapeState {
        muscle_size: a.muscle_size + (b.muscle_size - a.muscle_size) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        definition: a.definition + (b.definition - a.definition) * t,
        inner_bulge: a.inner_bulge + (b.inner_bulge - a.inner_bulge) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_calf_shape_config();
        assert!((0.0..=1.0).contains(&cfg.muscle_size));
    }

    #[test]
    fn test_new_state() {
        let s = new_calf_shape_state();
        assert!((s.muscle_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_muscle_size_clamp() {
        let mut s = new_calf_shape_state();
        set_calf_muscle_size(&mut s, 1.5);
        assert!((s.muscle_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_calf_shape_state();
        set_calf_taper(&mut s, 0.8);
        assert!((s.taper - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_definition() {
        let mut s = new_calf_shape_state();
        set_calf_definition(&mut s, 0.7);
        assert!((s.definition - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_inner_bulge() {
        let mut s = new_calf_shape_state();
        set_calf_inner_bulge(&mut s, 0.6);
        assert!((s.inner_bulge - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_calf_shape_state();
        let cfg = default_calf_shape_config();
        let w = compute_calf_shape_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.muscular));
        assert!((0.0..=1.0).contains(&w.tapered));
    }

    #[test]
    fn test_to_json() {
        let s = new_calf_shape_state();
        let json = calf_shape_to_json(&s);
        assert!(json.contains("muscle_size"));
        assert!(json.contains("inner_bulge"));
    }

    #[test]
    fn test_blend() {
        let a = new_calf_shape_state();
        let mut b = new_calf_shape_state();
        b.muscle_size = 1.0;
        let mid = blend_calf_shapes(&a, &b, 0.5);
        assert!((mid.muscle_size - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_calf_shape_state();
        let r = blend_calf_shapes(&a, &a, 0.5);
        assert!((r.muscle_size - a.muscle_size).abs() < 1e-6);
    }
}
