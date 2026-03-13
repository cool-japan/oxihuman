// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Chin projection morph controls for mentum depth and shape.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinProjectionControlConfig {
    pub projection: f32,
    pub width: f32,
    pub cleft_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinProjectionControlState {
    pub projection: f32,
    pub width: f32,
    pub cleft_depth: f32,
    pub vertical: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinProjectionControlWeights {
    pub projected: f32,
    pub recessed: f32,
    pub wide: f32,
    pub cleft: f32,
    pub vertical: f32,
}

#[allow(dead_code)]
pub fn default_chin_projection_control_config() -> ChinProjectionControlConfig {
    ChinProjectionControlConfig { projection: 0.5, width: 0.5, cleft_depth: 0.5 }
}

#[allow(dead_code)]
pub fn new_chin_projection_control_state() -> ChinProjectionControlState {
    ChinProjectionControlState { projection: 0.5, width: 0.5, cleft_depth: 0.5, vertical: 0.5 }
}

#[allow(dead_code)]
pub fn set_chin_projection_control_projection(state: &mut ChinProjectionControlState, value: f32) {
    state.projection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_projection_control_width(state: &mut ChinProjectionControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_projection_control_cleft_depth(state: &mut ChinProjectionControlState, value: f32) {
    state.cleft_depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_projection_control_vertical(state: &mut ChinProjectionControlState, value: f32) {
    state.vertical = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_chin_projection_control_weights(state: &ChinProjectionControlState, cfg: &ChinProjectionControlConfig) -> ChinProjectionControlWeights {
    let projected = (state.projection * cfg.projection * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let recessed = (state.width * cfg.width).clamp(0.0, 1.0);
    let wide = (state.cleft_depth * cfg.cleft_depth).clamp(0.0, 1.0);
    let cleft = state.vertical.clamp(0.0, 1.0);
    let vertical = (1.0 - state.projection).clamp(0.0, 1.0);
    ChinProjectionControlWeights { projected, recessed, wide, cleft, vertical }
}

#[allow(dead_code)]
pub fn chin_projection_control_to_json(state: &ChinProjectionControlState) -> String {
    format!(
        r#"{{\"projection\":{},\"width\":{},\"cleft_depth\":{},\"vertical\":{}}}"#,
        state.projection, state.width, state.cleft_depth, state.vertical
    )
}

#[allow(dead_code)]
pub fn blend_chin_projection_controls(a: &ChinProjectionControlState, b: &ChinProjectionControlState, t: f32) -> ChinProjectionControlState {
    let t = t.clamp(0.0, 1.0);
    ChinProjectionControlState {
        projection: a.projection + (b.projection - a.projection) * t,
        width: a.width + (b.width - a.width) * t,
        cleft_depth: a.cleft_depth + (b.cleft_depth - a.cleft_depth) * t,
        vertical: a.vertical + (b.vertical - a.vertical) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_chin_projection_control_config();
        assert!((0.0..=1.0).contains(&cfg.projection));
    }

    #[test]
    fn test_new_state() {
        let s = new_chin_projection_control_state();
        assert!((s.projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection_clamp() {
        let mut s = new_chin_projection_control_state();
        set_chin_projection_control_projection(&mut s, 1.5);
        assert!((s.projection - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_chin_projection_control_state();
        set_chin_projection_control_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_cleft_depth() {
        let mut s = new_chin_projection_control_state();
        set_chin_projection_control_cleft_depth(&mut s, 0.7);
        assert!((s.cleft_depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical() {
        let mut s = new_chin_projection_control_state();
        set_chin_projection_control_vertical(&mut s, 0.6);
        assert!((s.vertical - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_chin_projection_control_state();
        let cfg = default_chin_projection_control_config();
        let w = compute_chin_projection_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.projected));
        assert!((0.0..=1.0).contains(&w.recessed));
    }

    #[test]
    fn test_to_json() {
        let s = new_chin_projection_control_state();
        let json = chin_projection_control_to_json(&s);
        assert!(json.contains("projection"));
        assert!(json.contains("vertical"));
    }

    #[test]
    fn test_blend() {
        let a = new_chin_projection_control_state();
        let mut b = new_chin_projection_control_state();
        b.projection = 1.0;
        let mid = blend_chin_projection_controls(&a, &b, 0.5);
        assert!((mid.projection - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_chin_projection_control_state();
        let r = blend_chin_projection_controls(&a, &a, 0.5);
        assert!((r.projection - a.projection).abs() < 1e-6);
    }
}
