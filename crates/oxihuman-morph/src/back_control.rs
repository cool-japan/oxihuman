// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Back musculature and curvature morph controls.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BackConfig {
    pub upper_back_width: f32,
    pub lower_back_curve: f32,
    pub lat_spread: f32,
    pub spine_curvature: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BackState {
    pub upper_width: f32,
    pub lower_curve: f32,
    pub lats: f32,
    pub spine: f32,
    pub trapezius: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BackMorphWeights {
    pub wide_upper: f32,
    pub narrow_upper: f32,
    pub lordosis: f32,
    pub kyphosis: f32,
    pub lat_flare: f32,
    pub trap_bulk: f32,
}

#[allow(dead_code)]
pub fn default_back_config() -> BackConfig {
    BackConfig {
        upper_back_width: 0.5,
        lower_back_curve: 0.5,
        lat_spread: 0.5,
        spine_curvature: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_back_state() -> BackState {
    BackState {
        upper_width: 0.5,
        lower_curve: 0.5,
        lats: 0.5,
        spine: 0.5,
        trapezius: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_upper_back_width(state: &mut BackState, value: f32) {
    state.upper_width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lower_back_curve(state: &mut BackState, value: f32) {
    state.lower_curve = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lat_spread(state: &mut BackState, value: f32) {
    state.lats = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_trapezius(state: &mut BackState, value: f32) {
    state.trapezius = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_back_weights(state: &BackState, cfg: &BackConfig) -> BackMorphWeights {
    let uw = state.upper_width * cfg.upper_back_width;
    let wide_upper = uw.clamp(0.0, 1.0);
    let narrow_upper = (1.0 - uw).clamp(0.0, 1.0);
    let lc = state.lower_curve * cfg.lower_back_curve;
    let lordosis = (lc * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let kyphosis = ((1.0 - lc) * 0.5).clamp(0.0, 1.0);
    let lat_flare = (state.lats * cfg.lat_spread).clamp(0.0, 1.0);
    let trap_bulk = (state.trapezius * cfg.spine_curvature).clamp(0.0, 1.0);
    BackMorphWeights {
        wide_upper,
        narrow_upper,
        lordosis,
        kyphosis,
        lat_flare,
        trap_bulk,
    }
}

#[allow(dead_code)]
pub fn back_to_json(state: &BackState) -> String {
    format!(
        r#"{{"upper_width":{},"lower_curve":{},"lats":{},"spine":{},"trapezius":{}}}"#,
        state.upper_width, state.lower_curve, state.lats, state.spine, state.trapezius
    )
}

#[allow(dead_code)]
pub fn blend_back_states(a: &BackState, b: &BackState, t: f32) -> BackState {
    let t = t.clamp(0.0, 1.0);
    BackState {
        upper_width: a.upper_width + (b.upper_width - a.upper_width) * t,
        lower_curve: a.lower_curve + (b.lower_curve - a.lower_curve) * t,
        lats: a.lats + (b.lats - a.lats) * t,
        spine: a.spine + (b.spine - a.spine) * t,
        trapezius: a.trapezius + (b.trapezius - a.trapezius) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_back_config();
        assert!((0.0..=1.0).contains(&cfg.upper_back_width));
    }

    #[test]
    fn test_new_state() {
        let s = new_back_state();
        assert!((s.upper_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper_width_clamp() {
        let mut s = new_back_state();
        set_upper_back_width(&mut s, 2.0);
        assert!((s.upper_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_lower_curve() {
        let mut s = new_back_state();
        set_lower_back_curve(&mut s, 0.3);
        assert!((s.lower_curve - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_lat_spread() {
        let mut s = new_back_state();
        set_lat_spread(&mut s, 0.9);
        assert!((s.lats - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_range() {
        let s = new_back_state();
        let cfg = default_back_config();
        let w = compute_back_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide_upper));
        assert!((0.0..=1.0).contains(&w.lordosis));
    }

    #[test]
    fn test_back_to_json() {
        let s = new_back_state();
        let j = back_to_json(&s);
        assert!(j.contains("upper_width"));
    }

    #[test]
    fn test_blend_back() {
        let a = new_back_state();
        let mut b = new_back_state();
        b.lats = 1.0;
        let mid = blend_back_states(&a, &b, 0.5);
        assert!((mid.lats - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_trapezius() {
        let mut s = new_back_state();
        set_trapezius(&mut s, 0.4);
        assert!((s.trapezius - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_back_state();
        let r = blend_back_states(&a, &a, 0.5);
        assert!((r.lats - a.lats).abs() < 1e-6);
    }
}
