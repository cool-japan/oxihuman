// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Cheek width morphology controls for facial width at the cheek level.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekWidthConfig {
    pub width: f32,
    pub fullness: f32,
    pub angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekWidthState {
    pub width: f32,
    pub fullness: f32,
    pub angle: f32,
    pub taper: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekWidthWeights {
    pub wide: f32,
    pub narrow: f32,
    pub full: f32,
    pub hollow: f32,
    pub angled: f32,
}

#[allow(dead_code)]
pub fn default_cheek_width_config() -> CheekWidthConfig {
    CheekWidthConfig { width: 0.5, fullness: 0.5, angle: 0.5 }
}

#[allow(dead_code)]
pub fn new_cheek_width_state() -> CheekWidthState {
    CheekWidthState { width: 0.5, fullness: 0.5, angle: 0.5, taper: 0.0 }
}

#[allow(dead_code)]
pub fn set_cheek_width_val(state: &mut CheekWidthState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheek_fullness(state: &mut CheekWidthState, value: f32) {
    state.fullness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheek_angle(state: &mut CheekWidthState, value: f32) {
    state.angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cheek_taper(state: &mut CheekWidthState, value: f32) {
    state.taper = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_cheek_width_weights(state: &CheekWidthState, cfg: &CheekWidthConfig) -> CheekWidthWeights {
    let w = state.width * cfg.width;
    let wide = (w * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let narrow = (1.0 - w).clamp(0.0, 1.0);
    let full = (state.fullness * cfg.fullness).clamp(0.0, 1.0);
    let hollow = (1.0 - full).clamp(0.0, 1.0);
    let angled = (state.angle * cfg.angle).clamp(0.0, 1.0);
    CheekWidthWeights { wide, narrow, full, hollow, angled }
}

#[allow(dead_code)]
pub fn cheek_width_to_json(state: &CheekWidthState) -> String {
    format!(
        r#"{{"width":{},"fullness":{},"angle":{},"taper":{}}}"#,
        state.width, state.fullness, state.angle, state.taper
    )
}

#[allow(dead_code)]
pub fn blend_cheek_widths(a: &CheekWidthState, b: &CheekWidthState, t: f32) -> CheekWidthState {
    let t = t.clamp(0.0, 1.0);
    CheekWidthState {
        width: a.width + (b.width - a.width) * t,
        fullness: a.fullness + (b.fullness - a.fullness) * t,
        angle: a.angle + (b.angle - a.angle) * t,
        taper: a.taper + (b.taper - a.taper) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheek_width_config();
        assert!((0.0..=1.0).contains(&cfg.width));
    }

    #[test]
    fn test_new_state() {
        let s = new_cheek_width_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp() {
        let mut s = new_cheek_width_state();
        set_cheek_width_val(&mut s, 1.5);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_fullness() {
        let mut s = new_cheek_width_state();
        set_cheek_fullness(&mut s, 0.8);
        assert!((s.fullness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle() {
        let mut s = new_cheek_width_state();
        set_cheek_angle(&mut s, 0.7);
        assert!((s.angle - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_cheek_width_state();
        set_cheek_taper(&mut s, 0.6);
        assert!((s.taper - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_cheek_width_state();
        let cfg = default_cheek_width_config();
        let w = compute_cheek_width_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide));
        assert!((0.0..=1.0).contains(&w.full));
    }

    #[test]
    fn test_to_json() {
        let s = new_cheek_width_state();
        let json = cheek_width_to_json(&s);
        assert!(json.contains("width"));
        assert!(json.contains("taper"));
    }

    #[test]
    fn test_blend() {
        let a = new_cheek_width_state();
        let mut b = new_cheek_width_state();
        b.width = 1.0;
        let mid = blend_cheek_widths(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_cheek_width_state();
        let r = blend_cheek_widths(&a, &a, 0.5);
        assert!((r.width - a.width).abs() < 1e-6);
    }
}
