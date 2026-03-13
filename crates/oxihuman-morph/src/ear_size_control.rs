// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Ear size morphology controls for overall ear dimensions.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarSizeConfig {
    pub overall_size: f32,
    pub lobe_size: f32,
    pub helix_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarSizeState {
    pub overall_size: f32,
    pub lobe_size: f32,
    pub helix_width: f32,
    pub protrusion: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarSizeWeights {
    pub large: f32,
    pub small: f32,
    pub big_lobe: f32,
    pub wide_helix: f32,
    pub protruding: f32,
}

#[allow(dead_code)]
pub fn default_ear_size_config() -> EarSizeConfig {
    EarSizeConfig { overall_size: 0.5, lobe_size: 0.5, helix_width: 0.5 }
}

#[allow(dead_code)]
pub fn new_ear_size_state() -> EarSizeState {
    EarSizeState { overall_size: 0.5, lobe_size: 0.5, helix_width: 0.5, protrusion: 0.3 }
}

#[allow(dead_code)]
pub fn set_ear_overall_size(state: &mut EarSizeState, value: f32) {
    state.overall_size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ear_lobe_size(state: &mut EarSizeState, value: f32) {
    state.lobe_size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ear_helix_width(state: &mut EarSizeState, value: f32) {
    state.helix_width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ear_protrusion(state: &mut EarSizeState, value: f32) {
    state.protrusion = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_ear_size_weights(state: &EarSizeState, cfg: &EarSizeConfig) -> EarSizeWeights {
    let s = state.overall_size * cfg.overall_size;
    let large = (s * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let small = (1.0 - s).clamp(0.0, 1.0);
    let big_lobe = (state.lobe_size * cfg.lobe_size).clamp(0.0, 1.0);
    let wide_helix = (state.helix_width * cfg.helix_width).clamp(0.0, 1.0);
    let protruding = state.protrusion.clamp(0.0, 1.0);
    EarSizeWeights { large, small, big_lobe, wide_helix, protruding }
}

#[allow(dead_code)]
pub fn ear_size_to_json(state: &EarSizeState) -> String {
    format!(
        r#"{{"overall_size":{},"lobe_size":{},"helix_width":{},"protrusion":{}}}"#,
        state.overall_size, state.lobe_size, state.helix_width, state.protrusion
    )
}

#[allow(dead_code)]
pub fn blend_ear_sizes(a: &EarSizeState, b: &EarSizeState, t: f32) -> EarSizeState {
    let t = t.clamp(0.0, 1.0);
    EarSizeState {
        overall_size: a.overall_size + (b.overall_size - a.overall_size) * t,
        lobe_size: a.lobe_size + (b.lobe_size - a.lobe_size) * t,
        helix_width: a.helix_width + (b.helix_width - a.helix_width) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ear_size_config();
        assert!((0.0..=1.0).contains(&cfg.overall_size));
    }

    #[test]
    fn test_new_state() {
        let s = new_ear_size_state();
        assert!((s.overall_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_overall_size_clamp() {
        let mut s = new_ear_size_state();
        set_ear_overall_size(&mut s, 1.5);
        assert!((s.overall_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_lobe_size() {
        let mut s = new_ear_size_state();
        set_ear_lobe_size(&mut s, 0.8);
        assert!((s.lobe_size - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_helix_width() {
        let mut s = new_ear_size_state();
        set_ear_helix_width(&mut s, 0.7);
        assert!((s.helix_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion() {
        let mut s = new_ear_size_state();
        set_ear_protrusion(&mut s, 0.6);
        assert!((s.protrusion - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_ear_size_state();
        let cfg = default_ear_size_config();
        let w = compute_ear_size_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.large));
        assert!((0.0..=1.0).contains(&w.big_lobe));
    }

    #[test]
    fn test_to_json() {
        let s = new_ear_size_state();
        let json = ear_size_to_json(&s);
        assert!(json.contains("overall_size"));
        assert!(json.contains("protrusion"));
    }

    #[test]
    fn test_blend() {
        let a = new_ear_size_state();
        let mut b = new_ear_size_state();
        b.overall_size = 1.0;
        let mid = blend_ear_sizes(&a, &b, 0.5);
        assert!((mid.overall_size - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_ear_size_state();
        let r = blend_ear_sizes(&a, &a, 0.5);
        assert!((r.overall_size - a.overall_size).abs() < 1e-6);
    }
}
