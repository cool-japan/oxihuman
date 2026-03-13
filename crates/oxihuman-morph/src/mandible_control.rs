// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Mandible (lower jaw) morphology controls.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MandibleConfig {
    pub angle_range: f32,
    pub width_range: f32,
    pub chin_depth_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MandibleState {
    pub angle: f32,
    pub width: f32,
    pub chin_depth: f32,
}

#[allow(dead_code)]
pub fn default_mandible_config() -> MandibleConfig {
    MandibleConfig { angle_range: 1.0, width_range: 1.0, chin_depth_range: 1.0 }
}

#[allow(dead_code)]
pub fn new_mandible_state() -> MandibleState {
    MandibleState { angle: 0.5, width: 0.5, chin_depth: 0.5 }
}

#[allow(dead_code)]
pub fn mandible_set_angle(state: &mut MandibleState, value: f32) {
    state.angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn mandible_set_width(state: &mut MandibleState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn mandible_set_chin_depth(state: &mut MandibleState, value: f32) {
    state.chin_depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn mandible_reset(state: &mut MandibleState) {
    *state = new_mandible_state();
}

#[allow(dead_code)]
pub fn mandible_to_weights(state: &MandibleState, cfg: &MandibleConfig) -> [f32; 3] {
    let a = (state.angle * cfg.angle_range * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let w = (state.width * cfg.width_range).clamp(0.0, 1.0);
    let c = (state.chin_depth * cfg.chin_depth_range).clamp(0.0, 1.0);
    [a, w, c]
}

#[allow(dead_code)]
pub fn mandible_to_json(state: &MandibleState) -> String {
    format!(
        r#"{{"angle":{},"width":{},"chin_depth":{}}}"#,
        state.angle, state.width, state.chin_depth
    )
}

#[allow(dead_code)]
pub fn mandible_clamp(state: &mut MandibleState) {
    state.angle = state.angle.clamp(0.0, 1.0);
    state.width = state.width.clamp(0.0, 1.0);
    state.chin_depth = state.chin_depth.clamp(0.0, 1.0);
}

// ── MandibleControl (simple blend API) ────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MandibleControl {
    pub angle: f32,
    pub width: f32,
    pub chin_depth: f32,
}

#[allow(dead_code)]
pub fn default_mandible_control() -> MandibleControl {
    MandibleControl { angle: 0.5, width: 0.5, chin_depth: 0.5 }
}

#[allow(dead_code)]
pub fn apply_mandible_control(weights: &mut [f32], mc: &MandibleControl) {
    if weights.len() >= 3 {
        weights[0] = mc.angle.clamp(0.0, 1.0);
        weights[1] = mc.width.clamp(0.0, 1.0);
        weights[2] = mc.chin_depth.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn mandible_blend(a: &MandibleControl, b: &MandibleControl, t: f32) -> MandibleControl {
    let t = t.clamp(0.0, 1.0);
    MandibleControl {
        angle: a.angle + (b.angle - a.angle) * t,
        width: a.width + (b.width - a.width) * t,
        chin_depth: a.chin_depth + (b.chin_depth - a.chin_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_mandible_config();
        assert!((cfg.angle_range - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_mandible_state();
        assert!((s.angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_clamp() {
        let mut s = new_mandible_state();
        mandible_set_angle(&mut s, 1.5);
        assert!((s.angle - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_mandible_state();
        mandible_set_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_chin_depth() {
        let mut s = new_mandible_state();
        mandible_set_chin_depth(&mut s, 0.7);
        assert!((s.chin_depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut s = new_mandible_state();
        mandible_set_angle(&mut s, 1.0);
        mandible_reset(&mut s);
        assert!((s.angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_mandible_state();
        let cfg = default_mandible_config();
        let w = mandible_to_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w[0]));
        assert!((0.0..=1.0).contains(&w[1]));
    }

    #[test]
    fn test_to_json() {
        let s = new_mandible_state();
        let json = mandible_to_json(&s);
        assert!(json.contains("angle"));
        assert!(json.contains("chin_depth"));
    }

    #[test]
    fn test_clamp() {
        let mut s = MandibleState { angle: 1.5, width: -0.1, chin_depth: 0.5 };
        mandible_clamp(&mut s);
        assert!((s.angle - 1.0).abs() < 1e-6);
        assert!(s.width.abs() < 1e-6);
    }

    #[test]
    fn test_negative_clamp() {
        let mut s = new_mandible_state();
        mandible_set_angle(&mut s, -0.5);
        assert!(s.angle.abs() < 1e-6);
    }
}
