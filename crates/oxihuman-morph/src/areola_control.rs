// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Areola morphology controls for size, shape, and color intensity.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AreolaConfig {
    pub size: f32,
    pub puffiness: f32,
    pub color_intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AreolaState {
    pub size: f32,
    pub puffiness: f32,
    pub color_intensity: f32,
    pub asymmetry: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AreolaWeights {
    pub large: f32,
    pub small: f32,
    pub puffy: f32,
    pub flat: f32,
    pub dark: f32,
}

#[allow(dead_code)]
pub fn default_areola_config() -> AreolaConfig {
    AreolaConfig { size: 0.5, puffiness: 0.3, color_intensity: 0.5 }
}

#[allow(dead_code)]
pub fn new_areola_state() -> AreolaState {
    AreolaState { size: 0.5, puffiness: 0.3, color_intensity: 0.5, asymmetry: 0.0 }
}

#[allow(dead_code)]
pub fn set_areola_size(state: &mut AreolaState, value: f32) {
    state.size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_areola_puffiness(state: &mut AreolaState, value: f32) {
    state.puffiness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_areola_color_intensity(state: &mut AreolaState, value: f32) {
    state.color_intensity = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_areola_asymmetry(state: &mut AreolaState, value: f32) {
    state.asymmetry = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_areola_weights(state: &AreolaState, cfg: &AreolaConfig) -> AreolaWeights {
    let s = state.size * cfg.size;
    let large = (s * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let small = (1.0 - s).clamp(0.0, 1.0);
    let puffy = (state.puffiness * cfg.puffiness).clamp(0.0, 1.0);
    let flat = (1.0 - puffy).clamp(0.0, 1.0);
    let dark = (state.color_intensity * cfg.color_intensity).clamp(0.0, 1.0);
    AreolaWeights { large, small, puffy, flat, dark }
}

#[allow(dead_code)]
pub fn areola_to_json(state: &AreolaState) -> String {
    format!(
        r#"{{"size":{},"puffiness":{},"color_intensity":{},"asymmetry":{}}}"#,
        state.size, state.puffiness, state.color_intensity, state.asymmetry
    )
}

#[allow(dead_code)]
pub fn blend_areola_states(a: &AreolaState, b: &AreolaState, t: f32) -> AreolaState {
    let t = t.clamp(0.0, 1.0);
    AreolaState {
        size: a.size + (b.size - a.size) * t,
        puffiness: a.puffiness + (b.puffiness - a.puffiness) * t,
        color_intensity: a.color_intensity + (b.color_intensity - a.color_intensity) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_areola_config();
        assert!((0.0..=1.0).contains(&cfg.size));
    }

    #[test]
    fn test_new_state() {
        let s = new_areola_state();
        assert!((s.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_size_clamp() {
        let mut s = new_areola_state();
        set_areola_size(&mut s, 1.5);
        assert!((s.size - 1.0).abs() < 1e-6);
        set_areola_size(&mut s, -0.5);
        assert!(s.size.abs() < 1e-6);
    }

    #[test]
    fn test_set_puffiness() {
        let mut s = new_areola_state();
        set_areola_puffiness(&mut s, 0.8);
        assert!((s.puffiness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_intensity() {
        let mut s = new_areola_state();
        set_areola_color_intensity(&mut s, 0.9);
        assert!((s.color_intensity - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_asymmetry() {
        let mut s = new_areola_state();
        set_areola_asymmetry(&mut s, 0.4);
        assert!((s.asymmetry - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_areola_state();
        let cfg = default_areola_config();
        let w = compute_areola_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.large));
        assert!((0.0..=1.0).contains(&w.puffy));
    }

    #[test]
    fn test_to_json() {
        let s = new_areola_state();
        let json = areola_to_json(&s);
        assert!(json.contains("size"));
        assert!(json.contains("asymmetry"));
    }

    #[test]
    fn test_blend() {
        let a = new_areola_state();
        let mut b = new_areola_state();
        b.size = 1.0;
        let mid = blend_areola_states(&a, &b, 0.5);
        assert!((mid.size - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_areola_state();
        let r = blend_areola_states(&a, &a, 0.5);
        assert!((r.size - a.size).abs() < 1e-6);
    }
}
