// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eye corner (canthal) position and shape morph controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeCornerConfig {
    pub inner_range: f32,
    pub outer_range: f32,
    pub tilt_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeCornerState {
    pub inner_height: f32,
    pub outer_height: f32,
    pub inner_width: f32,
    pub outer_width: f32,
    pub tilt: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeCornerMorphWeights {
    pub inner_up: f32,
    pub inner_down: f32,
    pub outer_up: f32,
    pub outer_down: f32,
    pub tilt_positive: f32,
    pub tilt_negative: f32,
}

#[allow(dead_code)]
pub fn default_eye_corner_config() -> EyeCornerConfig {
    EyeCornerConfig {
        inner_range: 0.5,
        outer_range: 0.5,
        tilt_range: 0.4,
    }
}

#[allow(dead_code)]
pub fn new_eye_corner_state() -> EyeCornerState {
    EyeCornerState {
        inner_height: 0.5,
        outer_height: 0.5,
        inner_width: 0.5,
        outer_width: 0.5,
        tilt: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_inner_corner_height(state: &mut EyeCornerState, value: f32) {
    state.inner_height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_outer_corner_height(state: &mut EyeCornerState, value: f32) {
    state.outer_height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_eye_tilt(state: &mut EyeCornerState, value: f32) {
    state.tilt = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_inner_width(state: &mut EyeCornerState, value: f32) {
    state.inner_width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_eye_corner_weights(state: &EyeCornerState, cfg: &EyeCornerConfig) -> EyeCornerMorphWeights {
    let ih = (state.inner_height - 0.5) * 2.0 * cfg.inner_range;
    let oh = (state.outer_height - 0.5) * 2.0 * cfg.outer_range;
    let tilt_val = (state.tilt - 0.5) * 2.0 * cfg.tilt_range;
    EyeCornerMorphWeights {
        inner_up: ih.max(0.0).clamp(0.0, 1.0),
        inner_down: (-ih).max(0.0).clamp(0.0, 1.0),
        outer_up: oh.max(0.0).clamp(0.0, 1.0),
        outer_down: (-oh).max(0.0).clamp(0.0, 1.0),
        tilt_positive: tilt_val.max(0.0).clamp(0.0, 1.0),
        tilt_negative: (-tilt_val).max(0.0).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn eye_corner_to_json(state: &EyeCornerState) -> String {
    format!(
        r#"{{"inner_h":{},"outer_h":{},"inner_w":{},"outer_w":{},"tilt":{}}}"#,
        state.inner_height, state.outer_height, state.inner_width, state.outer_width, state.tilt
    )
}

#[allow(dead_code)]
pub fn blend_eye_corner_states(a: &EyeCornerState, b: &EyeCornerState, t: f32) -> EyeCornerState {
    let t = t.clamp(0.0, 1.0);
    EyeCornerState {
        inner_height: a.inner_height + (b.inner_height - a.inner_height) * t,
        outer_height: a.outer_height + (b.outer_height - a.outer_height) * t,
        inner_width: a.inner_width + (b.inner_width - a.inner_width) * t,
        outer_width: a.outer_width + (b.outer_width - a.outer_width) * t,
        tilt: a.tilt + (b.tilt - a.tilt) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_eye_corner_config();
        assert!((0.0..=1.0).contains(&c.inner_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_eye_corner_state();
        assert!((s.tilt - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_inner_height() {
        let mut s = new_eye_corner_state();
        set_inner_corner_height(&mut s, 0.8);
        assert!((s.inner_height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_outer_height_clamp() {
        let mut s = new_eye_corner_state();
        set_outer_corner_height(&mut s, -1.0);
        assert!(s.outer_height.abs() < 1e-6);
    }

    #[test]
    fn test_set_tilt() {
        let mut s = new_eye_corner_state();
        set_eye_tilt(&mut s, 0.7);
        assert!((s.tilt - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_weights_neutral() {
        let s = new_eye_corner_state();
        let cfg = default_eye_corner_config();
        let w = compute_eye_corner_weights(&s, &cfg);
        assert!(w.inner_up.abs() < 1e-6);
        assert!(w.inner_down.abs() < 1e-6);
    }

    #[test]
    fn test_weights_range() {
        let mut s = new_eye_corner_state();
        s.inner_height = 1.0;
        s.outer_height = 0.0;
        let cfg = default_eye_corner_config();
        let w = compute_eye_corner_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.inner_up));
        assert!((0.0..=1.0).contains(&w.outer_down));
    }

    #[test]
    fn test_to_json() {
        let s = new_eye_corner_state();
        let j = eye_corner_to_json(&s);
        assert!(j.contains("inner_h"));
    }

    #[test]
    fn test_blend() {
        let a = new_eye_corner_state();
        let mut b = new_eye_corner_state();
        b.tilt = 1.0;
        let mid = blend_eye_corner_states(&a, &b, 0.5);
        assert!((mid.tilt - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_inner_width() {
        let mut s = new_eye_corner_state();
        set_inner_width(&mut s, 0.2);
        assert!((s.inner_width - 0.2).abs() < 1e-6);
    }
}
