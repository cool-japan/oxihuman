// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eye shape morph controls: roundness, width, depth, and lid crease.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeShapeConfig {
    pub roundness_range: f32,
    pub width_range: f32,
    pub depth_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeShapeState {
    pub roundness: f32,
    pub width: f32,
    pub depth: f32,
    pub lid_crease: f32,
    pub prominence: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeShapeMorphWeights {
    pub round: f32,
    pub almond: f32,
    pub wide: f32,
    pub narrow: f32,
    pub deep_set: f32,
    pub protruding: f32,
}

#[allow(dead_code)]
pub fn default_eye_shape_config() -> EyeShapeConfig {
    EyeShapeConfig {
        roundness_range: 0.8,
        width_range: 0.6,
        depth_range: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_eye_shape_state() -> EyeShapeState {
    EyeShapeState {
        roundness: 0.5,
        width: 0.5,
        depth: 0.5,
        lid_crease: 0.5,
        prominence: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_eye_roundness(state: &mut EyeShapeState, value: f32) {
    state.roundness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_width(state: &mut EyeShapeState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_depth(state: &mut EyeShapeState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lid_crease(state: &mut EyeShapeState, value: f32) {
    state.lid_crease = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_eye_shape_weights(state: &EyeShapeState, cfg: &EyeShapeConfig) -> EyeShapeMorphWeights {
    let r = state.roundness * cfg.roundness_range;
    let round = r.clamp(0.0, 1.0);
    let almond = (1.0 - r).clamp(0.0, 1.0);
    let w = (state.width - 0.5) * 2.0 * cfg.width_range;
    let wide = w.max(0.0).clamp(0.0, 1.0);
    let narrow = (-w).max(0.0).clamp(0.0, 1.0);
    let d = (state.depth - 0.5) * 2.0 * cfg.depth_range;
    let deep_set = d.max(0.0).clamp(0.0, 1.0);
    let protruding = (-d).max(0.0).clamp(0.0, 1.0);
    EyeShapeMorphWeights {
        round,
        almond,
        wide,
        narrow,
        deep_set,
        protruding,
    }
}

#[allow(dead_code)]
pub fn eye_shape_to_json(state: &EyeShapeState) -> String {
    format!(
        r#"{{"roundness":{},"width":{},"depth":{},"lid_crease":{},"prominence":{}}}"#,
        state.roundness, state.width, state.depth, state.lid_crease, state.prominence
    )
}

#[allow(dead_code)]
pub fn blend_eye_shape_states(a: &EyeShapeState, b: &EyeShapeState, t: f32) -> EyeShapeState {
    let t = t.clamp(0.0, 1.0);
    EyeShapeState {
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        width: a.width + (b.width - a.width) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        lid_crease: a.lid_crease + (b.lid_crease - a.lid_crease) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_eye_shape_config();
        assert!((0.0..=1.0).contains(&c.roundness_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_eye_shape_state();
        assert!((s.roundness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness() {
        let mut s = new_eye_shape_state();
        set_eye_roundness(&mut s, 0.8);
        assert!((s.roundness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp() {
        let mut s = new_eye_shape_state();
        set_eye_width(&mut s, -0.5);
        assert!(s.width.abs() < 1e-6);
    }

    #[test]
    fn test_set_depth() {
        let mut s = new_eye_shape_state();
        set_eye_depth(&mut s, 0.9);
        assert!((s.depth - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_weights_neutral() {
        let s = new_eye_shape_state();
        let cfg = default_eye_shape_config();
        let w = compute_eye_shape_weights(&s, &cfg);
        assert!(w.wide.abs() < 1e-6);
        assert!(w.narrow.abs() < 1e-6);
    }

    #[test]
    fn test_weights_range() {
        let mut s = new_eye_shape_state();
        s.roundness = 1.0;
        s.width = 1.0;
        let cfg = default_eye_shape_config();
        let w = compute_eye_shape_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.round));
        assert!((0.0..=1.0).contains(&w.wide));
    }

    #[test]
    fn test_to_json() {
        let s = new_eye_shape_state();
        let j = eye_shape_to_json(&s);
        assert!(j.contains("roundness"));
    }

    #[test]
    fn test_blend() {
        let a = new_eye_shape_state();
        let mut b = new_eye_shape_state();
        b.roundness = 1.0;
        let mid = blend_eye_shape_states(&a, &b, 0.5);
        assert!((mid.roundness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_lid_crease() {
        let mut s = new_eye_shape_state();
        set_lid_crease(&mut s, 0.3);
        assert!((s.lid_crease - 0.3).abs() < 1e-6);
    }
}
