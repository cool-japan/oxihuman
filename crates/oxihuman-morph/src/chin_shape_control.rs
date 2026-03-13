// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Chin shape control: adjusts chin projection, width and vertical length.

use std::f32::consts::FRAC_PI_2;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinShapeConfig {
    pub min_val: f32,
    pub max_val: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinShapeState {
    pub projection: f32,
    pub width: f32,
    pub vertical: f32,
    pub cleft_depth: f32,
}

#[allow(dead_code)]
pub fn default_chin_shape_config() -> ChinShapeConfig {
    ChinShapeConfig {
        min_val: 0.0,
        max_val: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_chin_shape_state() -> ChinShapeState {
    ChinShapeState {
        projection: 0.5,
        width: 0.5,
        vertical: 0.5,
        cleft_depth: 0.0,
    }
}

#[allow(dead_code)]
pub fn cs_set_projection(state: &mut ChinShapeState, cfg: &ChinShapeConfig, v: f32) {
    state.projection = v.clamp(cfg.min_val, cfg.max_val);
}

#[allow(dead_code)]
pub fn cs_set_width(state: &mut ChinShapeState, cfg: &ChinShapeConfig, v: f32) {
    state.width = v.clamp(cfg.min_val, cfg.max_val);
}

#[allow(dead_code)]
pub fn cs_set_vertical(state: &mut ChinShapeState, cfg: &ChinShapeConfig, v: f32) {
    state.vertical = v.clamp(cfg.min_val, cfg.max_val);
}

#[allow(dead_code)]
pub fn cs_set_cleft(state: &mut ChinShapeState, v: f32) {
    state.cleft_depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cs_reset(state: &mut ChinShapeState) {
    *state = new_chin_shape_state();
}

#[allow(dead_code)]
pub fn cs_profile_angle(state: &ChinShapeState) -> f32 {
    state.projection * FRAC_PI_2
}

#[allow(dead_code)]
pub fn cs_to_weights(state: &ChinShapeState) -> Vec<(String, f32)> {
    vec![
        ("chin_projection".to_string(), state.projection),
        ("chin_width".to_string(), state.width),
        ("chin_vertical".to_string(), state.vertical),
        ("chin_cleft_depth".to_string(), state.cleft_depth),
    ]
}

#[allow(dead_code)]
pub fn cs_to_json(state: &ChinShapeState) -> String {
    format!(
        r#"{{"projection":{:.4},"width":{:.4},"vertical":{:.4},"cleft_depth":{:.4}}}"#,
        state.projection, state.width, state.vertical, state.cleft_depth
    )
}

#[allow(dead_code)]
pub fn cs_blend(a: &ChinShapeState, b: &ChinShapeState, t: f32) -> ChinShapeState {
    let t = t.clamp(0.0, 1.0);
    ChinShapeState {
        projection: a.projection + (b.projection - a.projection) * t,
        width: a.width + (b.width - a.width) * t,
        vertical: a.vertical + (b.vertical - a.vertical) * t,
        cleft_depth: a.cleft_depth + (b.cleft_depth - a.cleft_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_chin_shape_config();
        assert!(cfg.min_val.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_chin_shape_state();
        assert!((s.projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection_clamps() {
        let cfg = default_chin_shape_config();
        let mut s = new_chin_shape_state();
        cs_set_projection(&mut s, &cfg, 5.0);
        assert!((s.projection - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let cfg = default_chin_shape_config();
        let mut s = new_chin_shape_state();
        cs_set_width(&mut s, &cfg, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical() {
        let cfg = default_chin_shape_config();
        let mut s = new_chin_shape_state();
        cs_set_vertical(&mut s, &cfg, 0.3);
        assert!((s.vertical - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_cleft() {
        let mut s = new_chin_shape_state();
        cs_set_cleft(&mut s, 0.6);
        assert!((s.cleft_depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_chin_shape_config();
        let mut s = new_chin_shape_state();
        cs_set_projection(&mut s, &cfg, 0.9);
        cs_reset(&mut s);
        assert!((s.projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_profile_angle() {
        let s = new_chin_shape_state();
        let a = cs_profile_angle(&s);
        assert!((a - 0.5 * FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_chin_shape_state();
        assert_eq!(cs_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_blend() {
        let a = new_chin_shape_state();
        let mut b = new_chin_shape_state();
        b.projection = 1.0;
        let mid = cs_blend(&a, &b, 0.5);
        assert!((mid.projection - 0.75).abs() < 1e-6);
    }
}
