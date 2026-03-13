// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Ankle shape morphology controls for ankle width, taper, and bone prominence.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnkleShapeConfig {
    pub width: f32,
    pub taper: f32,
    pub bone_prominence: f32,
    pub circumference: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnkleShapeState {
    pub width: f32,
    pub taper: f32,
    pub prominence: f32,
    pub circumference: f32,
    pub rotation: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnkleShapeWeights {
    pub wide: f32,
    pub narrow: f32,
    pub tapered: f32,
    pub bony: f32,
    pub round: f32,
    pub rotated: f32,
}

#[allow(dead_code)]
pub fn default_ankle_shape_config() -> AnkleShapeConfig {
    AnkleShapeConfig {
        width: 0.5,
        taper: 0.5,
        bone_prominence: 0.3,
        circumference: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_ankle_shape_state() -> AnkleShapeState {
    AnkleShapeState {
        width: 0.5,
        taper: 0.5,
        prominence: 0.3,
        circumference: 0.5,
        rotation: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_ankle_width(state: &mut AnkleShapeState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ankle_taper(state: &mut AnkleShapeState, value: f32) {
    state.taper = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ankle_prominence(state: &mut AnkleShapeState, value: f32) {
    state.prominence = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ankle_rotation(state: &mut AnkleShapeState, value: f32) {
    state.rotation = value.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_ankle_weights(state: &AnkleShapeState, cfg: &AnkleShapeConfig) -> AnkleShapeWeights {
    let w = state.width * cfg.width;
    let wide = w.clamp(0.0, 1.0);
    let narrow = (1.0 - w).clamp(0.0, 1.0);
    let tapered = (state.taper * cfg.taper * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let bony = (state.prominence * cfg.bone_prominence).clamp(0.0, 1.0);
    let round = (state.circumference * cfg.circumference).clamp(0.0, 1.0);
    let rotated = (state.rotation.abs() * 0.5).clamp(0.0, 1.0);
    AnkleShapeWeights { wide, narrow, tapered, bony, round, rotated }
}

#[allow(dead_code)]
pub fn ankle_shape_to_json(state: &AnkleShapeState) -> String {
    format!(
        r#"{{"width":{},"taper":{},"prominence":{},"circumference":{},"rotation":{}}}"#,
        state.width, state.taper, state.prominence, state.circumference, state.rotation
    )
}

#[allow(dead_code)]
pub fn blend_ankle_shapes(a: &AnkleShapeState, b: &AnkleShapeState, t: f32) -> AnkleShapeState {
    let t = t.clamp(0.0, 1.0);
    AnkleShapeState {
        width: a.width + (b.width - a.width) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        circumference: a.circumference + (b.circumference - a.circumference) * t,
        rotation: a.rotation + (b.rotation - a.rotation) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ankle_shape_config();
        assert!((0.0..=1.0).contains(&cfg.width));
        assert!((0.0..=1.0).contains(&cfg.taper));
    }

    #[test]
    fn test_new_state() {
        let s = new_ankle_shape_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!(s.rotation.abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp() {
        let mut s = new_ankle_shape_state();
        set_ankle_width(&mut s, 1.5);
        assert!((s.width - 1.0).abs() < 1e-6);
        set_ankle_width(&mut s, -0.5);
        assert!(s.width.abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_ankle_shape_state();
        set_ankle_taper(&mut s, 0.8);
        assert!((s.taper - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence() {
        let mut s = new_ankle_shape_state();
        set_ankle_prominence(&mut s, 0.9);
        assert!((s.prominence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_rotation() {
        let mut s = new_ankle_shape_state();
        set_ankle_rotation(&mut s, -0.5);
        assert!((s.rotation - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_in_range() {
        let s = new_ankle_shape_state();
        let cfg = default_ankle_shape_config();
        let w = compute_ankle_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide));
        assert!((0.0..=1.0).contains(&w.tapered));
        assert!((0.0..=1.0).contains(&w.bony));
    }

    #[test]
    fn test_to_json() {
        let s = new_ankle_shape_state();
        let json = ankle_shape_to_json(&s);
        assert!(json.contains("width"));
        assert!(json.contains("rotation"));
    }

    #[test]
    fn test_blend_shapes() {
        let a = new_ankle_shape_state();
        let mut b = new_ankle_shape_state();
        b.width = 1.0;
        let mid = blend_ankle_shapes(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_ankle_shape_state();
        let result = blend_ankle_shapes(&a, &a, 0.5);
        assert!((result.width - a.width).abs() < 1e-6);
    }
}
