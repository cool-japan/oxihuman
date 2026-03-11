// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaw ramus height and flare control.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawRamusConfig {
    pub max_height: f32,
    pub max_flare: f32,
}

impl Default for JawRamusConfig {
    fn default() -> Self {
        Self {
            max_height: 1.0,
            max_flare: 1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawRamusState {
    pub height: f32,
    pub flare: f32,
    pub config: JawRamusConfig,
}

#[allow(dead_code)]
pub fn default_jaw_ramus_config() -> JawRamusConfig {
    JawRamusConfig::default()
}

#[allow(dead_code)]
pub fn new_jaw_ramus_state(config: JawRamusConfig) -> JawRamusState {
    JawRamusState {
        height: 0.0,
        flare: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn jram_set_height(state: &mut JawRamusState, v: f32) {
    state.height = v.clamp(0.0, state.config.max_height);
}

#[allow(dead_code)]
pub fn jram_set_flare(state: &mut JawRamusState, v: f32) {
    state.flare = v.clamp(0.0, state.config.max_flare);
}

#[allow(dead_code)]
pub fn jram_reset(state: &mut JawRamusState) {
    state.height = 0.0;
    state.flare = 0.0;
}

#[allow(dead_code)]
pub fn jram_is_neutral(state: &JawRamusState) -> bool {
    state.height.abs() < 1e-6 && state.flare.abs() < 1e-6
}

#[allow(dead_code)]
pub fn jram_ramus_area(state: &JawRamusState) -> f32 {
    state.height * (1.0 + state.flare * 0.5)
}

#[allow(dead_code)]
pub fn jram_flare_angle_rad(state: &JawRamusState) -> f32 {
    state.flare * FRAC_PI_4
}

#[allow(dead_code)]
pub fn jram_to_weights(state: &JawRamusState) -> [f32; 2] {
    let nh = if state.config.max_height > 1e-9 {
        state.height / state.config.max_height
    } else {
        0.0
    };
    let nf = if state.config.max_flare > 1e-9 {
        state.flare / state.config.max_flare
    } else {
        0.0
    };
    [nh, nf]
}

#[allow(dead_code)]
pub fn jram_blend(a: &JawRamusState, b: &JawRamusState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.height * (1.0 - t) + b.height * t,
        a.flare * (1.0 - t) + b.flare * t,
    ]
}

#[allow(dead_code)]
pub fn jram_to_json(state: &JawRamusState) -> String {
    format!(
        "{{\"height\":{:.4},\"flare\":{:.4}}}",
        state.height, state.flare
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(jram_is_neutral(&new_jaw_ramus_state(
            default_jaw_ramus_config()
        )));
    }
    #[test]
    fn set_height_clamps() {
        let mut s = new_jaw_ramus_state(default_jaw_ramus_config());
        jram_set_height(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.height));
    }
    #[test]
    fn set_flare_clamps() {
        let mut s = new_jaw_ramus_state(default_jaw_ramus_config());
        jram_set_flare(&mut s, -1.0);
        assert!(s.flare.abs() < 1e-6);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_jaw_ramus_state(default_jaw_ramus_config());
        jram_set_height(&mut s, 0.5);
        jram_reset(&mut s);
        assert!(jram_is_neutral(&s));
    }
    #[test]
    fn ramus_area_positive() {
        let mut s = new_jaw_ramus_state(default_jaw_ramus_config());
        jram_set_height(&mut s, 0.5);
        assert!(jram_ramus_area(&s) > 0.0);
    }
    #[test]
    fn flare_angle_nonneg() {
        let s = new_jaw_ramus_state(default_jaw_ramus_config());
        assert!(jram_flare_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_max_height() {
        let mut s = new_jaw_ramus_state(default_jaw_ramus_config());
        jram_set_height(&mut s, 1.0);
        assert!((jram_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_at_one() {
        let a = new_jaw_ramus_state(default_jaw_ramus_config());
        let mut b = new_jaw_ramus_state(default_jaw_ramus_config());
        jram_set_height(&mut b, 0.8);
        let w = jram_blend(&a, &b, 1.0);
        assert!((w[0] - 0.8).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_height() {
        assert!(
            jram_to_json(&new_jaw_ramus_state(default_jaw_ramus_config())).contains("\"height\"")
        );
    }
    #[test]
    fn to_json_has_flare() {
        assert!(
            jram_to_json(&new_jaw_ramus_state(default_jaw_ramus_config())).contains("\"flare\"")
        );
    }
}
