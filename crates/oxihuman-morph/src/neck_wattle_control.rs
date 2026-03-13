// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Neck wattle (submental skin laxity) control.

use std::f32::consts::FRAC_PI_6;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckWattleConfig {
    pub max_sag: f32,
}

impl Default for NeckWattleConfig {
    fn default() -> Self {
        Self { max_sag: 1.0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckWattleState {
    pub sag: f32,
    pub spread: f32,
    pub config: NeckWattleConfig,
}

#[allow(dead_code)]
pub fn default_neck_wattle_config() -> NeckWattleConfig {
    NeckWattleConfig::default()
}

#[allow(dead_code)]
pub fn new_neck_wattle_state(config: NeckWattleConfig) -> NeckWattleState {
    NeckWattleState {
        sag: 0.0,
        spread: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn nwat_set_sag(state: &mut NeckWattleState, v: f32) {
    state.sag = v.clamp(0.0, state.config.max_sag);
}

#[allow(dead_code)]
pub fn nwat_set_spread(state: &mut NeckWattleState, v: f32) {
    state.spread = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nwat_reset(state: &mut NeckWattleState) {
    state.sag = 0.0;
    state.spread = 0.0;
}

#[allow(dead_code)]
pub fn nwat_is_neutral(state: &NeckWattleState) -> bool {
    state.sag.abs() < 1e-6 && state.spread.abs() < 1e-6
}

#[allow(dead_code)]
pub fn nwat_volume_estimate(state: &NeckWattleState) -> f32 {
    state.sag * state.spread
}

#[allow(dead_code)]
pub fn nwat_sag_angle_rad(state: &NeckWattleState) -> f32 {
    state.sag * FRAC_PI_6
}

#[allow(dead_code)]
pub fn nwat_to_weights(state: &NeckWattleState) -> [f32; 2] {
    let m = state.config.max_sag;
    [if m > 1e-9 { state.sag / m } else { 0.0 }, state.spread]
}

#[allow(dead_code)]
pub fn nwat_blend(a: &NeckWattleState, b: &NeckWattleState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.sag * (1.0 - t) + b.sag * t,
        a.spread * (1.0 - t) + b.spread * t,
    ]
}

#[allow(dead_code)]
pub fn nwat_to_json(state: &NeckWattleState) -> String {
    format!(
        "{{\"sag\":{:.4},\"spread\":{:.4}}}",
        state.sag, state.spread
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(nwat_is_neutral(&new_neck_wattle_state(
            default_neck_wattle_config()
        )));
    }
    #[test]
    fn set_sag_clamps() {
        let mut s = new_neck_wattle_state(default_neck_wattle_config());
        nwat_set_sag(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.sag));
    }
    #[test]
    fn set_spread_clamps() {
        let mut s = new_neck_wattle_state(default_neck_wattle_config());
        nwat_set_spread(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.spread));
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_neck_wattle_state(default_neck_wattle_config());
        nwat_set_sag(&mut s, 0.5);
        nwat_reset(&mut s);
        assert!(nwat_is_neutral(&s));
    }
    #[test]
    fn volume_product() {
        let mut s = new_neck_wattle_state(default_neck_wattle_config());
        nwat_set_sag(&mut s, 0.5);
        nwat_set_spread(&mut s, 0.4);
        assert!((nwat_volume_estimate(&s) - 0.2).abs() < 1e-5);
    }
    #[test]
    fn sag_angle_nonneg() {
        let s = new_neck_wattle_state(default_neck_wattle_config());
        assert!(nwat_sag_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max() {
        let mut s = new_neck_wattle_state(default_neck_wattle_config());
        nwat_set_sag(&mut s, 1.0);
        assert!((nwat_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_at_zero_is_a() {
        let mut a = new_neck_wattle_state(default_neck_wattle_config());
        let b = new_neck_wattle_state(default_neck_wattle_config());
        nwat_set_sag(&mut a, 0.7);
        let w = nwat_blend(&a, &b, 0.0);
        assert!((w[0] - 0.7).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_sag() {
        assert!(
            nwat_to_json(&new_neck_wattle_state(default_neck_wattle_config())).contains("\"sag\"")
        );
    }
    #[test]
    fn volume_zero_when_neutral() {
        let s = new_neck_wattle_state(default_neck_wattle_config());
        assert!(nwat_volume_estimate(&s).abs() < 1e-6);
    }
}
