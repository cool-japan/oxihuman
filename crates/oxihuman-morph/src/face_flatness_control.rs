// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face flatness control — antero-posterior depth compression of the face.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceFlatnessConfig {
    pub max_flatten_m: f32,
}

impl Default for FaceFlatnessConfig {
    fn default() -> Self {
        Self {
            max_flatten_m: 0.018,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FaceFlatnessState {
    /// 0 = full depth, 1 = fully flattened.
    pub flatness: f32,
    /// Mid-face compression (cheek/zygomatic region), 0..=1.
    pub mid_compress: f32,
}

#[allow(dead_code)]
pub fn new_face_flatness_state() -> FaceFlatnessState {
    FaceFlatnessState::default()
}

#[allow(dead_code)]
pub fn default_face_flatness_config() -> FaceFlatnessConfig {
    FaceFlatnessConfig::default()
}

#[allow(dead_code)]
pub fn ffl_set_flatness(state: &mut FaceFlatnessState, v: f32) {
    state.flatness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ffl_set_mid(state: &mut FaceFlatnessState, v: f32) {
    state.mid_compress = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ffl_reset(state: &mut FaceFlatnessState) {
    *state = FaceFlatnessState::default();
}

#[allow(dead_code)]
pub fn ffl_is_neutral(state: &FaceFlatnessState) -> bool {
    state.flatness < 1e-4 && state.mid_compress < 1e-4
}

/// Depth scale factor (1 = no change, 0 = fully flat).
#[allow(dead_code)]
pub fn ffl_depth_scale(state: &FaceFlatnessState) -> f32 {
    1.0 - state.flatness * 0.6
}

#[allow(dead_code)]
pub fn ffl_to_weights(state: &FaceFlatnessState, cfg: &FaceFlatnessConfig) -> [f32; 2] {
    [
        state.flatness * cfg.max_flatten_m,
        state.mid_compress * cfg.max_flatten_m * 0.6,
    ]
}

#[allow(dead_code)]
pub fn ffl_blend(a: &FaceFlatnessState, b: &FaceFlatnessState, t: f32) -> FaceFlatnessState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    FaceFlatnessState {
        flatness: a.flatness * inv + b.flatness * t,
        mid_compress: a.mid_compress * inv + b.mid_compress * t,
    }
}

#[allow(dead_code)]
pub fn ffl_to_json(state: &FaceFlatnessState) -> String {
    format!(
        "{{\"flatness\":{:.4},\"mid_compress\":{:.4}}}",
        state.flatness, state.mid_compress
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(ffl_is_neutral(&new_face_flatness_state()));
    }

    #[test]
    fn flatness_clamps() {
        let mut s = new_face_flatness_state();
        ffl_set_flatness(&mut s, 5.0);
        assert!((s.flatness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn flatness_clamps_low() {
        let mut s = new_face_flatness_state();
        ffl_set_flatness(&mut s, -1.0);
        assert!(s.flatness < 1e-6);
    }

    #[test]
    fn reset_works() {
        let mut s = new_face_flatness_state();
        ffl_set_flatness(&mut s, 0.9);
        ffl_reset(&mut s);
        assert!(ffl_is_neutral(&s));
    }

    #[test]
    fn depth_scale_one_at_neutral() {
        let s = new_face_flatness_state();
        assert!((ffl_depth_scale(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn depth_scale_less_at_max() {
        let mut s = new_face_flatness_state();
        ffl_set_flatness(&mut s, 1.0);
        assert!(ffl_depth_scale(&s) < 1.0);
    }

    #[test]
    fn weights_positive() {
        let cfg = default_face_flatness_config();
        let mut s = new_face_flatness_state();
        ffl_set_flatness(&mut s, 1.0);
        let w = ffl_to_weights(&s, &cfg);
        assert!(w[0] > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let b = FaceFlatnessState {
            flatness: 1.0,
            mid_compress: 0.0,
        };
        let r = ffl_blend(&new_face_flatness_state(), &b, 0.5);
        assert!((r.flatness - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_keys_present() {
        let j = ffl_to_json(&new_face_flatness_state());
        assert!(j.contains("flatness") && j.contains("mid_compress"));
    }

    #[test]
    fn mid_clamps() {
        let mut s = new_face_flatness_state();
        ffl_set_mid(&mut s, -5.0);
        assert!(s.mid_compress < 1e-6);
    }
}
