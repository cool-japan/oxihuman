// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Forehead globe (frontal bossing) curvature control.

use std::f32::consts::FRAC_PI_3;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadGlobeConfig {
    pub max_curvature: f32,
}

impl Default for ForeheadGlobeConfig {
    fn default() -> Self {
        Self { max_curvature: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadGlobeState {
    pub central: f32,
    pub lateral: f32,
    pub config: ForeheadGlobeConfig,
}

#[allow(dead_code)]
pub fn default_forehead_globe_config() -> ForeheadGlobeConfig {
    ForeheadGlobeConfig::default()
}

#[allow(dead_code)]
pub fn new_forehead_globe_state(config: ForeheadGlobeConfig) -> ForeheadGlobeState {
    ForeheadGlobeState {
        central: 0.0,
        lateral: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn fgl_set_central(state: &mut ForeheadGlobeState, v: f32) {
    state.central = v.clamp(0.0, state.config.max_curvature);
}

#[allow(dead_code)]
pub fn fgl_set_lateral(state: &mut ForeheadGlobeState, v: f32) {
    state.lateral = v.clamp(0.0, state.config.max_curvature);
}

#[allow(dead_code)]
pub fn fgl_set_both(state: &mut ForeheadGlobeState, v: f32) {
    let v = v.clamp(0.0, state.config.max_curvature);
    state.central = v;
    state.lateral = v;
}

#[allow(dead_code)]
pub fn fgl_reset(state: &mut ForeheadGlobeState) {
    state.central = 0.0;
    state.lateral = 0.0;
}

#[allow(dead_code)]
pub fn fgl_is_neutral(state: &ForeheadGlobeState) -> bool {
    state.central.abs() < 1e-6 && state.lateral.abs() < 1e-6
}

#[allow(dead_code)]
pub fn fgl_average(state: &ForeheadGlobeState) -> f32 {
    (state.central + state.lateral) * 0.5
}

#[allow(dead_code)]
pub fn fgl_curvature_angle_rad(state: &ForeheadGlobeState) -> f32 {
    fgl_average(state) * FRAC_PI_3
}

#[allow(dead_code)]
pub fn fgl_to_weights(state: &ForeheadGlobeState) -> [f32; 2] {
    let m = state.config.max_curvature;
    let n = |v: f32| if m > 1e-9 { v / m } else { 0.0 };
    [n(state.central), n(state.lateral)]
}

#[allow(dead_code)]
pub fn fgl_blend(a: &ForeheadGlobeState, b: &ForeheadGlobeState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.central * (1.0 - t) + b.central * t,
        a.lateral * (1.0 - t) + b.lateral * t,
    ]
}

#[allow(dead_code)]
pub fn fgl_to_json(state: &ForeheadGlobeState) -> String {
    format!(
        "{{\"central\":{:.4},\"lateral\":{:.4}}}",
        state.central, state.lateral
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(fgl_is_neutral(&new_forehead_globe_state(
            default_forehead_globe_config()
        )));
    }
    #[test]
    fn set_central_clamps() {
        let mut s = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_central(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.central));
    }
    #[test]
    fn set_lateral_clamps() {
        let mut s = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_lateral(&mut s, -1.0);
        assert!(s.lateral.abs() < 1e-6);
    }
    #[test]
    fn set_both_applies() {
        let mut s = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_both(&mut s, 0.6);
        assert!((s.central - 0.6).abs() < 1e-5 && (s.lateral - 0.6).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_both(&mut s, 0.7);
        fgl_reset(&mut s);
        assert!(fgl_is_neutral(&s));
    }
    #[test]
    fn average_mid() {
        let mut s = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_central(&mut s, 0.2);
        fgl_set_lateral(&mut s, 0.8);
        assert!((fgl_average(&s) - 0.5).abs() < 1e-5);
    }
    #[test]
    fn curvature_angle_nonneg() {
        let s = new_forehead_globe_state(default_forehead_globe_config());
        assert!(fgl_curvature_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_at_max() {
        let mut s = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_central(&mut s, 1.0);
        assert!((fgl_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_at_half() {
        let mut a = new_forehead_globe_state(default_forehead_globe_config());
        let b = new_forehead_globe_state(default_forehead_globe_config());
        fgl_set_central(&mut a, 0.6);
        let w = fgl_blend(&a, &b, 0.5);
        assert!((w[0] - 0.3).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_lateral() {
        assert!(
            fgl_to_json(&new_forehead_globe_state(default_forehead_globe_config()))
                .contains("\"lateral\"")
        );
    }
}
