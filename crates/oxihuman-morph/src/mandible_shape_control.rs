// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Mandible shape morph controls for lower jaw bone shape and proportion.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MandibleShapeConfig {
    pub width: f32,
    pub height: f32,
    pub projection: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MandibleShapeState {
    pub width: f32,
    pub height: f32,
    pub projection: f32,
    pub ramus_angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MandibleShapeWeights {
    pub wide: f32,
    pub tall: f32,
    pub projected: f32,
    pub angled: f32,
    pub narrow: f32,
}

#[allow(dead_code)]
pub fn default_mandible_shape_config() -> MandibleShapeConfig {
    MandibleShapeConfig { width: 0.5, height: 0.5, projection: 0.5 }
}

#[allow(dead_code)]
pub fn new_mandible_shape_state() -> MandibleShapeState {
    MandibleShapeState { width: 0.5, height: 0.5, projection: 0.5, ramus_angle: 0.5 }
}

#[allow(dead_code)]
pub fn set_mandible_shape_width(state: &mut MandibleShapeState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_mandible_shape_height(state: &mut MandibleShapeState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_mandible_shape_projection(state: &mut MandibleShapeState, value: f32) {
    state.projection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_mandible_shape_ramus_angle(state: &mut MandibleShapeState, value: f32) {
    state.ramus_angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_mandible_shape_weights(state: &MandibleShapeState, cfg: &MandibleShapeConfig) -> MandibleShapeWeights {
    let wide = (state.width * cfg.width * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let tall = (state.height * cfg.height).clamp(0.0, 1.0);
    let projected = (state.projection * cfg.projection).clamp(0.0, 1.0);
    let angled = state.ramus_angle.clamp(0.0, 1.0);
    let narrow = (1.0 - state.width).clamp(0.0, 1.0);
    MandibleShapeWeights { wide, tall, projected, angled, narrow }
}

#[allow(dead_code)]
pub fn mandible_shape_to_json(state: &MandibleShapeState) -> String {
    format!(
        r#"{{"width":{},"height":{},"projection":{},"ramus_angle":{}}}"#,
        state.width, state.height, state.projection, state.ramus_angle
    )
}

#[allow(dead_code)]
pub fn blend_mandible_shapes(a: &MandibleShapeState, b: &MandibleShapeState, t: f32) -> MandibleShapeState {
    let t = t.clamp(0.0, 1.0);
    MandibleShapeState {
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        ramus_angle: a.ramus_angle + (b.ramus_angle - a.ramus_angle) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_mandible_shape_config();
        assert!((0.0..=1.0).contains(&cfg.width));
    }

    #[test]
    fn test_new_state() {
        let s = new_mandible_shape_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp() {
        let mut s = new_mandible_shape_state();
        set_mandible_shape_width(&mut s, 1.5);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut s = new_mandible_shape_state();
        set_mandible_shape_height(&mut s, 0.8);
        assert!((s.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection() {
        let mut s = new_mandible_shape_state();
        set_mandible_shape_projection(&mut s, 0.7);
        assert!((s.projection - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_ramus_angle() {
        let mut s = new_mandible_shape_state();
        set_mandible_shape_ramus_angle(&mut s, 0.6);
        assert!((s.ramus_angle - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_mandible_shape_state();
        let cfg = default_mandible_shape_config();
        let w = compute_mandible_shape_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide));
        assert!((0.0..=1.0).contains(&w.tall));
    }

    #[test]
    fn test_to_json() {
        let s = new_mandible_shape_state();
        let json = mandible_shape_to_json(&s);
        assert!(json.contains("width"));
        assert!(json.contains("ramus_angle"));
    }

    #[test]
    fn test_blend() {
        let a = new_mandible_shape_state();
        let mut b = new_mandible_shape_state();
        b.width = 1.0;
        let mid = blend_mandible_shapes(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_mandible_shape_state();
        let r = blend_mandible_shapes(&a, &a, 0.5);
        assert!((r.width - a.width).abs() < 1e-6);
    }
}
