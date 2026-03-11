// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Orbital morph controls for eye socket depth and bony contour.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitalControlConfig {
    pub depth: f32,
    pub width: f32,
    pub height: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitalControlState {
    pub depth: f32,
    pub width: f32,
    pub height: f32,
    pub rim_prominence: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitalControlWeights {
    pub deep_set: f32,
    pub wide: f32,
    pub tall: f32,
    pub prominent_rim: f32,
    pub shallow: f32,
}

#[allow(dead_code)]
pub fn default_orbital_control_config() -> OrbitalControlConfig {
    OrbitalControlConfig { depth: 0.5, width: 0.5, height: 0.5 }
}

#[allow(dead_code)]
pub fn new_orbital_control_state() -> OrbitalControlState {
    OrbitalControlState { depth: 0.5, width: 0.5, height: 0.5, rim_prominence: 0.5 }
}

#[allow(dead_code)]
pub fn set_orbital_control_depth(state: &mut OrbitalControlState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_orbital_control_width(state: &mut OrbitalControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_orbital_control_height(state: &mut OrbitalControlState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_orbital_control_rim_prominence(state: &mut OrbitalControlState, value: f32) {
    state.rim_prominence = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_orbital_control_weights(state: &OrbitalControlState, cfg: &OrbitalControlConfig) -> OrbitalControlWeights {
    let deep_set = (state.depth * cfg.depth * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let tall = (state.height * cfg.height).clamp(0.0, 1.0);
    let prominent_rim = state.rim_prominence.clamp(0.0, 1.0);
    let shallow = (1.0 - state.depth).clamp(0.0, 1.0);
    OrbitalControlWeights { deep_set, wide, tall, prominent_rim, shallow }
}

#[allow(dead_code)]
pub fn orbital_control_to_json(state: &OrbitalControlState) -> String {
    format!(
        r#"{{\"depth\":{},\"width\":{},\"height\":{},\"rim_prominence\":{}}}"#,
        state.depth, state.width, state.height, state.rim_prominence
    )
}

#[allow(dead_code)]
pub fn blend_orbital_controls(a: &OrbitalControlState, b: &OrbitalControlState, t: f32) -> OrbitalControlState {
    let t = t.clamp(0.0, 1.0);
    OrbitalControlState {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        rim_prominence: a.rim_prominence + (b.rim_prominence - a.rim_prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_orbital_control_config();
        assert!((0.0..=1.0).contains(&cfg.depth));
    }

    #[test]
    fn test_new_state() {
        let s = new_orbital_control_state();
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamp() {
        let mut s = new_orbital_control_state();
        set_orbital_control_depth(&mut s, 1.5);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_orbital_control_state();
        set_orbital_control_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut s = new_orbital_control_state();
        set_orbital_control_height(&mut s, 0.7);
        assert!((s.height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_rim_prominence() {
        let mut s = new_orbital_control_state();
        set_orbital_control_rim_prominence(&mut s, 0.6);
        assert!((s.rim_prominence - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_orbital_control_state();
        let cfg = default_orbital_control_config();
        let w = compute_orbital_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.deep_set));
        assert!((0.0..=1.0).contains(&w.wide));
    }

    #[test]
    fn test_to_json() {
        let s = new_orbital_control_state();
        let json = orbital_control_to_json(&s);
        assert!(json.contains("depth"));
        assert!(json.contains("rim_prominence"));
    }

    #[test]
    fn test_blend() {
        let a = new_orbital_control_state();
        let mut b = new_orbital_control_state();
        b.depth = 1.0;
        let mid = blend_orbital_controls(&a, &b, 0.5);
        assert!((mid.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_orbital_control_state();
        let r = blend_orbital_controls(&a, &a, 0.5);
        assert!((r.depth - a.depth).abs() < 1e-6);
    }
}
