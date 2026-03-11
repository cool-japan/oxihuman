// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lip Cupid's bow peak shape control.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCupidConfig {
    pub max_peak: f32,
    pub max_depth: f32,
}

impl Default for LipCupidConfig {
    fn default() -> Self {
        Self {
            max_peak: 1.0,
            max_depth: 1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCupidState {
    pub peak: f32,
    pub depth: f32,
    pub config: LipCupidConfig,
}

#[allow(dead_code)]
pub fn default_lip_cupid_config() -> LipCupidConfig {
    LipCupidConfig::default()
}

#[allow(dead_code)]
pub fn new_lip_cupid_state(config: LipCupidConfig) -> LipCupidState {
    LipCupidState {
        peak: 0.0,
        depth: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn lc_set_peak(state: &mut LipCupidState, v: f32) {
    state.peak = v.clamp(0.0, state.config.max_peak);
}

#[allow(dead_code)]
pub fn lc_set_depth(state: &mut LipCupidState, v: f32) {
    state.depth = v.clamp(0.0, state.config.max_depth);
}

#[allow(dead_code)]
pub fn lc_reset(state: &mut LipCupidState) {
    state.peak = 0.0;
    state.depth = 0.0;
}

#[allow(dead_code)]
pub fn lc_is_neutral(state: &LipCupidState) -> bool {
    state.peak.abs() < 1e-6 && state.depth.abs() < 1e-6
}

#[allow(dead_code)]
pub fn lc_bow_acuity(state: &LipCupidState) -> f32 {
    if state.depth > 1e-9 {
        state.peak / state.depth
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn lc_peak_angle_rad(state: &LipCupidState) -> f32 {
    state.peak * FRAC_PI_4
}

#[allow(dead_code)]
pub fn lc_to_weights(state: &LipCupidState) -> [f32; 2] {
    let np = if state.config.max_peak > 1e-9 {
        state.peak / state.config.max_peak
    } else {
        0.0
    };
    let nd = if state.config.max_depth > 1e-9 {
        state.depth / state.config.max_depth
    } else {
        0.0
    };
    [np, nd]
}

#[allow(dead_code)]
pub fn lc_blend(a: &LipCupidState, b: &LipCupidState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.peak * (1.0 - t) + b.peak * t,
        a.depth * (1.0 - t) + b.depth * t,
    ]
}

#[allow(dead_code)]
pub fn lc_to_json(state: &LipCupidState) -> String {
    format!(
        "{{\"peak\":{:.4},\"depth\":{:.4}}}",
        state.peak, state.depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(lc_is_neutral(&new_lip_cupid_state(
            default_lip_cupid_config()
        )));
    }
    #[test]
    fn set_peak_clamps() {
        let mut s = new_lip_cupid_state(default_lip_cupid_config());
        lc_set_peak(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.peak));
    }
    #[test]
    fn set_depth_clamps() {
        let mut s = new_lip_cupid_state(default_lip_cupid_config());
        lc_set_depth(&mut s, -0.5);
        assert!(s.depth.abs() < 1e-6);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_lip_cupid_state(default_lip_cupid_config());
        lc_set_peak(&mut s, 0.5);
        lc_reset(&mut s);
        assert!(lc_is_neutral(&s));
    }
    #[test]
    fn bow_acuity_zero_when_depth_zero() {
        let s = new_lip_cupid_state(default_lip_cupid_config());
        assert!(lc_bow_acuity(&s).abs() < 1e-6);
    }
    #[test]
    fn peak_angle_nonneg() {
        let s = new_lip_cupid_state(default_lip_cupid_config());
        assert!(lc_peak_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_lip_cupid_state(default_lip_cupid_config());
        lc_set_peak(&mut s, 1.0);
        assert!((lc_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_mid() {
        let mut a = new_lip_cupid_state(default_lip_cupid_config());
        let b = new_lip_cupid_state(default_lip_cupid_config());
        lc_set_peak(&mut a, 0.8);
        let w = lc_blend(&a, &b, 0.5);
        assert!((w[0] - 0.4).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_peak() {
        assert!(lc_to_json(&new_lip_cupid_state(default_lip_cupid_config())).contains("\"peak\""));
    }
    #[test]
    fn bow_acuity_ratio() {
        let mut s = new_lip_cupid_state(default_lip_cupid_config());
        lc_set_peak(&mut s, 0.6);
        lc_set_depth(&mut s, 0.3);
        assert!((lc_bow_acuity(&s) - 2.0).abs() < 1e-4);
    }
}
