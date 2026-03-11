// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face anterior-posterior depth (protrusion) control.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceDepthConfig {
    pub max_depth: f32,
    pub min_depth: f32,
}

impl Default for FaceDepthConfig {
    fn default() -> Self {
        Self {
            max_depth: 1.0,
            min_depth: -1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceDepthState {
    pub upper: f32,
    pub middle: f32,
    pub lower: f32,
    pub config: FaceDepthConfig,
}

#[allow(dead_code)]
pub fn default_face_depth_config() -> FaceDepthConfig {
    FaceDepthConfig::default()
}

#[allow(dead_code)]
pub fn new_face_depth_state(config: FaceDepthConfig) -> FaceDepthState {
    FaceDepthState {
        upper: 0.0,
        middle: 0.0,
        lower: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn fdc_set_upper(state: &mut FaceDepthState, v: f32) {
    state.upper = v.clamp(state.config.min_depth, state.config.max_depth);
}

#[allow(dead_code)]
pub fn fdc_set_middle(state: &mut FaceDepthState, v: f32) {
    state.middle = v.clamp(state.config.min_depth, state.config.max_depth);
}

#[allow(dead_code)]
pub fn fdc_set_lower(state: &mut FaceDepthState, v: f32) {
    state.lower = v.clamp(state.config.min_depth, state.config.max_depth);
}

#[allow(dead_code)]
pub fn fdc_set_all(state: &mut FaceDepthState, v: f32) {
    let v = v.clamp(state.config.min_depth, state.config.max_depth);
    state.upper = v;
    state.middle = v;
    state.lower = v;
}

#[allow(dead_code)]
pub fn fdc_reset(state: &mut FaceDepthState) {
    state.upper = 0.0;
    state.middle = 0.0;
    state.lower = 0.0;
}

#[allow(dead_code)]
pub fn fdc_is_neutral(state: &FaceDepthState) -> bool {
    state.upper.abs() < 1e-6 && state.middle.abs() < 1e-6 && state.lower.abs() < 1e-6
}

#[allow(dead_code)]
pub fn fdc_average(state: &FaceDepthState) -> f32 {
    (state.upper + state.middle + state.lower) / 3.0
}

#[allow(dead_code)]
pub fn fdc_range(state: &FaceDepthState) -> f32 {
    let max = state.upper.max(state.middle).max(state.lower);
    let min = state.upper.min(state.middle).min(state.lower);
    max - min
}

#[allow(dead_code)]
pub fn fdc_profile_angle_rad(state: &FaceDepthState) -> f32 {
    fdc_average(state) * FRAC_PI_4
}

#[allow(dead_code)]
pub fn fdc_to_weights(state: &FaceDepthState) -> [f32; 3] {
    let m = state.config.max_depth;
    let n = |v: f32| {
        if m > 1e-9 {
            (v / m).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    };
    [n(state.upper), n(state.middle), n(state.lower)]
}

#[allow(dead_code)]
pub fn fdc_blend(a: &FaceDepthState, b: &FaceDepthState, t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a.upper * (1.0 - t) + b.upper * t,
        a.middle * (1.0 - t) + b.middle * t,
        a.lower * (1.0 - t) + b.lower * t,
    ]
}

#[allow(dead_code)]
pub fn fdc_to_json(state: &FaceDepthState) -> String {
    format!(
        "{{\"upper\":{:.4},\"middle\":{:.4},\"lower\":{:.4}}}",
        state.upper, state.middle, state.lower
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_neutral() {
        assert!(fdc_is_neutral(&new_face_depth_state(
            default_face_depth_config()
        )));
    }
    #[test]
    fn set_upper_clamps() {
        let mut s = new_face_depth_state(default_face_depth_config());
        fdc_set_upper(&mut s, 5.0);
        assert!((0.0..=1.0).contains(&s.upper));
    }
    #[test]
    fn set_all_applies() {
        let mut s = new_face_depth_state(default_face_depth_config());
        fdc_set_all(&mut s, 0.5);
        assert!((s.upper - 0.5).abs() < 1e-5 && (s.lower - 0.5).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_face_depth_state(default_face_depth_config());
        fdc_set_all(&mut s, 0.5);
        fdc_reset(&mut s);
        assert!(fdc_is_neutral(&s));
    }
    #[test]
    fn average_three_vals() {
        let mut s = new_face_depth_state(default_face_depth_config());
        fdc_set_upper(&mut s, 0.3);
        fdc_set_middle(&mut s, 0.6);
        fdc_set_lower(&mut s, 0.9);
        assert!((fdc_average(&s) - 0.6).abs() < 1e-5);
    }
    #[test]
    fn range_max_minus_min() {
        let mut s = new_face_depth_state(default_face_depth_config());
        fdc_set_upper(&mut s, 0.2);
        fdc_set_middle(&mut s, 0.2);
        fdc_set_lower(&mut s, 0.8);
        assert!((fdc_range(&s) - 0.6).abs() < 1e-5);
    }
    #[test]
    fn profile_angle_nonneg() {
        let s = new_face_depth_state(default_face_depth_config());
        assert!(fdc_profile_angle_rad(&s) >= -1.0);
    }
    #[test]
    fn to_weights_at_max() {
        let mut s = new_face_depth_state(default_face_depth_config());
        fdc_set_upper(&mut s, 1.0);
        assert!((fdc_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_mid() {
        let mut a = new_face_depth_state(default_face_depth_config());
        let b = new_face_depth_state(default_face_depth_config());
        fdc_set_upper(&mut a, 0.4);
        let w = fdc_blend(&a, &b, 0.5);
        assert!((w[0] - 0.2).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_middle() {
        assert!(
            fdc_to_json(&new_face_depth_state(default_face_depth_config())).contains("\"middle\"")
        );
    }
}
