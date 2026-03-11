// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Arch height morphology controls for foot arch shape.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArchHeightConfig {
    pub height: f32,
    pub curvature: f32,
    pub flexibility: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArchHeightState {
    pub height: f32,
    pub curvature: f32,
    pub flexibility: f32,
    pub collapse: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArchHeightWeights {
    pub high_arch: f32,
    pub flat_arch: f32,
    pub curved: f32,
    pub flexible: f32,
}

#[allow(dead_code)]
pub fn default_arch_height_config() -> ArchHeightConfig {
    ArchHeightConfig { height: 0.5, curvature: 0.5, flexibility: 0.5 }
}

#[allow(dead_code)]
pub fn new_arch_height_state() -> ArchHeightState {
    ArchHeightState { height: 0.5, curvature: 0.5, flexibility: 0.5, collapse: 0.0 }
}

#[allow(dead_code)]
pub fn set_arch_height(state: &mut ArchHeightState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_arch_curvature(state: &mut ArchHeightState, value: f32) {
    state.curvature = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_arch_flexibility(state: &mut ArchHeightState, value: f32) {
    state.flexibility = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_arch_collapse(state: &mut ArchHeightState, value: f32) {
    state.collapse = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_arch_weights(state: &ArchHeightState, cfg: &ArchHeightConfig) -> ArchHeightWeights {
    let h = state.height * cfg.height;
    let high_arch = (h * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let flat_arch = ((1.0 - h) * (1.0 - state.collapse)).clamp(0.0, 1.0);
    let curved = (state.curvature * cfg.curvature).clamp(0.0, 1.0);
    let flexible = (state.flexibility * cfg.flexibility).clamp(0.0, 1.0);
    ArchHeightWeights { high_arch, flat_arch, curved, flexible }
}

#[allow(dead_code)]
pub fn arch_height_to_json(state: &ArchHeightState) -> String {
    format!(
        r#"{{"height":{},"curvature":{},"flexibility":{},"collapse":{}}}"#,
        state.height, state.curvature, state.flexibility, state.collapse
    )
}

#[allow(dead_code)]
pub fn blend_arch_heights(a: &ArchHeightState, b: &ArchHeightState, t: f32) -> ArchHeightState {
    let t = t.clamp(0.0, 1.0);
    ArchHeightState {
        height: a.height + (b.height - a.height) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        flexibility: a.flexibility + (b.flexibility - a.flexibility) * t,
        collapse: a.collapse + (b.collapse - a.collapse) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_arch_height_config();
        assert!((0.0..=1.0).contains(&cfg.height));
    }

    #[test]
    fn test_new_state() {
        let s = new_arch_height_state();
        assert!((s.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        let mut s = new_arch_height_state();
        set_arch_height(&mut s, 1.5);
        assert!((s.height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_curvature() {
        let mut s = new_arch_height_state();
        set_arch_curvature(&mut s, 0.8);
        assert!((s.curvature - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_flexibility() {
        let mut s = new_arch_height_state();
        set_arch_flexibility(&mut s, 0.7);
        assert!((s.flexibility - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_collapse() {
        let mut s = new_arch_height_state();
        set_arch_collapse(&mut s, 0.6);
        assert!((s.collapse - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_in_range() {
        let s = new_arch_height_state();
        let cfg = default_arch_height_config();
        let w = compute_arch_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.high_arch));
        assert!((0.0..=1.0).contains(&w.flat_arch));
    }

    #[test]
    fn test_to_json() {
        let s = new_arch_height_state();
        let json = arch_height_to_json(&s);
        assert!(json.contains("height"));
        assert!(json.contains("collapse"));
    }

    #[test]
    fn test_blend() {
        let a = new_arch_height_state();
        let mut b = new_arch_height_state();
        b.height = 1.0;
        let mid = blend_arch_heights(&a, &b, 0.5);
        assert!((mid.height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_arch_height_state();
        let r = blend_arch_heights(&a, &a, 0.5);
        assert!((r.height - a.height).abs() < 1e-6);
    }
}
