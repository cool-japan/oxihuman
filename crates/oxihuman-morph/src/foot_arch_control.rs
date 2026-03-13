// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Foot arch morphology controls for plantar arch shape.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootArchConfig {
    pub arch_height: f32,
    pub arch_length: f32,
    pub stiffness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootArchState {
    pub arch_height: f32,
    pub arch_length: f32,
    pub stiffness: f32,
    pub pronation: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootArchWeights {
    pub high_arch: f32,
    pub flat_foot: f32,
    pub long_arch: f32,
    pub stiff: f32,
    pub pronated: f32,
}

#[allow(dead_code)]
pub fn default_foot_arch_config() -> FootArchConfig {
    FootArchConfig {
        arch_height: 0.5,
        arch_length: 0.5,
        stiffness: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_foot_arch_state() -> FootArchState {
    FootArchState {
        arch_height: 0.5,
        arch_length: 0.5,
        stiffness: 0.5,
        pronation: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_foot_arch_height(state: &mut FootArchState, value: f32) {
    state.arch_height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_foot_arch_length(state: &mut FootArchState, value: f32) {
    state.arch_length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_foot_arch_stiffness(state: &mut FootArchState, value: f32) {
    state.stiffness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_foot_pronation(state: &mut FootArchState, value: f32) {
    state.pronation = value.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_foot_arch_weights(state: &FootArchState, cfg: &FootArchConfig) -> FootArchWeights {
    let h = state.arch_height * cfg.arch_height;
    let high_arch = (h * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let flat_foot = (1.0 - h).clamp(0.0, 1.0);
    let long_arch = (state.arch_length * cfg.arch_length).clamp(0.0, 1.0);
    let stiff = (state.stiffness * cfg.stiffness).clamp(0.0, 1.0);
    let pronated = (state.pronation.abs() * 0.5).clamp(0.0, 1.0);
    FootArchWeights {
        high_arch,
        flat_foot,
        long_arch,
        stiff,
        pronated,
    }
}

#[allow(dead_code)]
pub fn foot_arch_to_json(state: &FootArchState) -> String {
    format!(
        r#"{{"arch_height":{},"arch_length":{},"stiffness":{},"pronation":{}}}"#,
        state.arch_height, state.arch_length, state.stiffness, state.pronation
    )
}

#[allow(dead_code)]
pub fn blend_foot_arches(a: &FootArchState, b: &FootArchState, t: f32) -> FootArchState {
    let t = t.clamp(0.0, 1.0);
    FootArchState {
        arch_height: a.arch_height + (b.arch_height - a.arch_height) * t,
        arch_length: a.arch_length + (b.arch_length - a.arch_length) * t,
        stiffness: a.stiffness + (b.stiffness - a.stiffness) * t,
        pronation: a.pronation + (b.pronation - a.pronation) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_foot_arch_config();
        assert!((0.0..=1.0).contains(&cfg.arch_height));
    }

    #[test]
    fn test_new_state() {
        let s = new_foot_arch_state();
        assert!((s.arch_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch_height_clamp() {
        let mut s = new_foot_arch_state();
        set_foot_arch_height(&mut s, 1.5);
        assert!((s.arch_height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch_length() {
        let mut s = new_foot_arch_state();
        set_foot_arch_length(&mut s, 0.8);
        assert!((s.arch_length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_stiffness() {
        let mut s = new_foot_arch_state();
        set_foot_arch_stiffness(&mut s, 0.7);
        assert!((s.stiffness - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_pronation() {
        let mut s = new_foot_arch_state();
        set_foot_pronation(&mut s, -0.5);
        assert!((s.pronation - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_foot_arch_state();
        let cfg = default_foot_arch_config();
        let w = compute_foot_arch_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.high_arch));
        assert!((0.0..=1.0).contains(&w.flat_foot));
    }

    #[test]
    fn test_to_json() {
        let s = new_foot_arch_state();
        let json = foot_arch_to_json(&s);
        assert!(json.contains("arch_height"));
        assert!(json.contains("pronation"));
    }

    #[test]
    fn test_blend() {
        let a = new_foot_arch_state();
        let mut b = new_foot_arch_state();
        b.arch_height = 1.0;
        let mid = blend_foot_arches(&a, &b, 0.5);
        assert!((mid.arch_height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_foot_arch_state();
        let r = blend_foot_arches(&a, &a, 0.5);
        assert!((r.arch_height - a.arch_height).abs() < 1e-6);
    }
}
