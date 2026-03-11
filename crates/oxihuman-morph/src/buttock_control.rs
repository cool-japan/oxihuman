// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Buttock morphology controls for size, roundness, and projection.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ButtockConfig {
    pub size: f32,
    pub roundness: f32,
    pub projection: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ButtockState {
    pub size: f32,
    pub roundness: f32,
    pub projection: f32,
    pub firmness: f32,
    pub cleft_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ButtockWeights {
    pub large: f32,
    pub small: f32,
    pub round: f32,
    pub flat: f32,
    pub projected: f32,
    pub firm: f32,
}

#[allow(dead_code)]
pub fn default_buttock_config() -> ButtockConfig {
    ButtockConfig { size: 0.5, roundness: 0.5, projection: 0.3 }
}

#[allow(dead_code)]
pub fn new_buttock_state() -> ButtockState {
    ButtockState { size: 0.5, roundness: 0.5, projection: 0.3, firmness: 0.5, cleft_depth: 0.3 }
}

#[allow(dead_code)]
pub fn set_buttock_size(state: &mut ButtockState, value: f32) {
    state.size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_buttock_roundness(state: &mut ButtockState, value: f32) {
    state.roundness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_buttock_projection(state: &mut ButtockState, value: f32) {
    state.projection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_buttock_firmness(state: &mut ButtockState, value: f32) {
    state.firmness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_buttock_weights(state: &ButtockState, cfg: &ButtockConfig) -> ButtockWeights {
    let s = state.size * cfg.size;
    let large = (s * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let small = (1.0 - s).clamp(0.0, 1.0);
    let round = (state.roundness * cfg.roundness).clamp(0.0, 1.0);
    let flat = (1.0 - round).clamp(0.0, 1.0);
    let projected = (state.projection * cfg.projection).clamp(0.0, 1.0);
    let firm = state.firmness.clamp(0.0, 1.0);
    ButtockWeights { large, small, round, flat, projected, firm }
}

#[allow(dead_code)]
pub fn buttock_to_json(state: &ButtockState) -> String {
    format!(
        r#"{{"size":{},"roundness":{},"projection":{},"firmness":{},"cleft_depth":{}}}"#,
        state.size, state.roundness, state.projection, state.firmness, state.cleft_depth
    )
}

#[allow(dead_code)]
pub fn blend_buttock_states(a: &ButtockState, b: &ButtockState, t: f32) -> ButtockState {
    let t = t.clamp(0.0, 1.0);
    ButtockState {
        size: a.size + (b.size - a.size) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        firmness: a.firmness + (b.firmness - a.firmness) * t,
        cleft_depth: a.cleft_depth + (b.cleft_depth - a.cleft_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_buttock_config();
        assert!((0.0..=1.0).contains(&cfg.size));
    }

    #[test]
    fn test_new_state() {
        let s = new_buttock_state();
        assert!((s.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_size_clamp() {
        let mut s = new_buttock_state();
        set_buttock_size(&mut s, 1.5);
        assert!((s.size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness() {
        let mut s = new_buttock_state();
        set_buttock_roundness(&mut s, 0.8);
        assert!((s.roundness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection() {
        let mut s = new_buttock_state();
        set_buttock_projection(&mut s, 0.7);
        assert!((s.projection - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_firmness() {
        let mut s = new_buttock_state();
        set_buttock_firmness(&mut s, 0.9);
        assert!((s.firmness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_buttock_state();
        let cfg = default_buttock_config();
        let w = compute_buttock_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.large));
        assert!((0.0..=1.0).contains(&w.round));
    }

    #[test]
    fn test_to_json() {
        let s = new_buttock_state();
        let json = buttock_to_json(&s);
        assert!(json.contains("size"));
        assert!(json.contains("cleft_depth"));
    }

    #[test]
    fn test_blend() {
        let a = new_buttock_state();
        let mut b = new_buttock_state();
        b.size = 1.0;
        let mid = blend_buttock_states(&a, &b, 0.5);
        assert!((mid.size - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_buttock_state();
        let r = blend_buttock_states(&a, &a, 0.5);
        assert!((r.size - a.size).abs() < 1e-6);
    }
}
