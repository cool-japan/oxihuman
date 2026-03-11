// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Chin height morphology controls for vertical chin position and projection.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinHeightConfig {
    pub height: f32,
    pub projection: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinHeightState {
    pub height: f32,
    pub projection: f32,
    pub width: f32,
    pub pointedness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinHeightWeights {
    pub tall: f32,
    pub short: f32,
    pub projected: f32,
    pub receded: f32,
    pub pointed: f32,
}

#[allow(dead_code)]
pub fn default_chin_height_config() -> ChinHeightConfig {
    ChinHeightConfig { height: 0.5, projection: 0.5, width: 0.5 }
}

#[allow(dead_code)]
pub fn new_chin_height_state() -> ChinHeightState {
    ChinHeightState { height: 0.5, projection: 0.5, width: 0.5, pointedness: 0.0 }
}

#[allow(dead_code)]
pub fn set_chin_height_val(state: &mut ChinHeightState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_projection(state: &mut ChinHeightState, value: f32) {
    state.projection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_width_val(state: &mut ChinHeightState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_pointedness(state: &mut ChinHeightState, value: f32) {
    state.pointedness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_chin_height_weights(state: &ChinHeightState, cfg: &ChinHeightConfig) -> ChinHeightWeights {
    let h = state.height * cfg.height;
    let tall = (h * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let short = (1.0 - h).clamp(0.0, 1.0);
    let projected = (state.projection * cfg.projection).clamp(0.0, 1.0);
    let receded = (1.0 - projected).clamp(0.0, 1.0);
    let pointed = state.pointedness.clamp(0.0, 1.0);
    ChinHeightWeights { tall, short, projected, receded, pointed }
}

#[allow(dead_code)]
pub fn chin_height_to_json(state: &ChinHeightState) -> String {
    format!(
        r#"{{"height":{},"projection":{},"width":{},"pointedness":{}}}"#,
        state.height, state.projection, state.width, state.pointedness
    )
}

#[allow(dead_code)]
pub fn blend_chin_heights(a: &ChinHeightState, b: &ChinHeightState, t: f32) -> ChinHeightState {
    let t = t.clamp(0.0, 1.0);
    ChinHeightState {
        height: a.height + (b.height - a.height) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        width: a.width + (b.width - a.width) * t,
        pointedness: a.pointedness + (b.pointedness - a.pointedness) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_chin_height_config();
        assert!((0.0..=1.0).contains(&cfg.height));
    }

    #[test]
    fn test_new_state() {
        let s = new_chin_height_state();
        assert!((s.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        let mut s = new_chin_height_state();
        set_chin_height_val(&mut s, 1.5);
        assert!((s.height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection() {
        let mut s = new_chin_height_state();
        set_chin_projection(&mut s, 0.8);
        assert!((s.projection - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_chin_height_state();
        set_chin_width_val(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_pointedness() {
        let mut s = new_chin_height_state();
        set_chin_pointedness(&mut s, 0.6);
        assert!((s.pointedness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_chin_height_state();
        let cfg = default_chin_height_config();
        let w = compute_chin_height_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.tall));
        assert!((0.0..=1.0).contains(&w.projected));
    }

    #[test]
    fn test_to_json() {
        let s = new_chin_height_state();
        let json = chin_height_to_json(&s);
        assert!(json.contains("height"));
        assert!(json.contains("pointedness"));
    }

    #[test]
    fn test_blend() {
        let a = new_chin_height_state();
        let mut b = new_chin_height_state();
        b.height = 1.0;
        let mid = blend_chin_heights(&a, &b, 0.5);
        assert!((mid.height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_chin_height_state();
        let r = blend_chin_heights(&a, &a, 0.5);
        assert!((r.height - a.height).abs() < 1e-6);
    }
}
