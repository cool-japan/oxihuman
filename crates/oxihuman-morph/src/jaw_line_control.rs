// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Jaw line control: adjusts definition, angle and width of the jawline.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawLineConfig {
    pub min_val: f32,
    pub max_val: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawLineState {
    pub definition: f32,
    pub angle: f32,
    pub width: f32,
}

#[allow(dead_code)]
pub fn default_jaw_line_config() -> JawLineConfig {
    JawLineConfig {
        min_val: 0.0,
        max_val: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_jaw_line_state() -> JawLineState {
    JawLineState {
        definition: 0.5,
        angle: 0.5,
        width: 0.5,
    }
}

#[allow(dead_code)]
pub fn jl_set_definition(state: &mut JawLineState, cfg: &JawLineConfig, v: f32) {
    state.definition = v.clamp(cfg.min_val, cfg.max_val);
}

#[allow(dead_code)]
pub fn jl_set_angle(state: &mut JawLineState, cfg: &JawLineConfig, v: f32) {
    state.angle = v.clamp(cfg.min_val, cfg.max_val);
}

#[allow(dead_code)]
pub fn jl_set_width(state: &mut JawLineState, cfg: &JawLineConfig, v: f32) {
    state.width = v.clamp(cfg.min_val, cfg.max_val);
}

#[allow(dead_code)]
pub fn jl_reset(state: &mut JawLineState) {
    *state = new_jaw_line_state();
}

#[allow(dead_code)]
pub fn jl_gonial_angle(state: &JawLineState) -> f32 {
    state.angle * FRAC_PI_4 + FRAC_PI_4
}

#[allow(dead_code)]
pub fn jl_to_weights(state: &JawLineState) -> Vec<(String, f32)> {
    vec![
        ("jaw_definition".to_string(), state.definition),
        ("jaw_angle".to_string(), state.angle),
        ("jaw_width".to_string(), state.width),
    ]
}

#[allow(dead_code)]
pub fn jl_to_json(state: &JawLineState) -> String {
    format!(
        r#"{{"definition":{:.4},"angle":{:.4},"width":{:.4}}}"#,
        state.definition, state.angle, state.width
    )
}

#[allow(dead_code)]
pub fn jl_blend(a: &JawLineState, b: &JawLineState, t: f32) -> JawLineState {
    let t = t.clamp(0.0, 1.0);
    JawLineState {
        definition: a.definition + (b.definition - a.definition) * t,
        angle: a.angle + (b.angle - a.angle) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[allow(dead_code)]
pub fn jl_sharpness(state: &JawLineState) -> f32 {
    state.definition * (1.0 - state.width * 0.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jaw_line_config();
        assert!(cfg.min_val.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_jaw_line_state();
        assert!((s.definition - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_definition_clamps() {
        let cfg = default_jaw_line_config();
        let mut s = new_jaw_line_state();
        jl_set_definition(&mut s, &cfg, 5.0);
        assert!((s.definition - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle() {
        let cfg = default_jaw_line_config();
        let mut s = new_jaw_line_state();
        jl_set_angle(&mut s, &cfg, 0.8);
        assert!((s.angle - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let cfg = default_jaw_line_config();
        let mut s = new_jaw_line_state();
        jl_set_width(&mut s, &cfg, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_jaw_line_config();
        let mut s = new_jaw_line_state();
        jl_set_definition(&mut s, &cfg, 0.9);
        jl_reset(&mut s);
        assert!((s.definition - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_gonial_angle() {
        let s = new_jaw_line_state();
        let a = jl_gonial_angle(&s);
        assert!(a > FRAC_PI_4);
    }

    #[test]
    fn test_blend() {
        let a = new_jaw_line_state();
        let mut b = new_jaw_line_state();
        b.definition = 1.0;
        let mid = jl_blend(&a, &b, 0.5);
        assert!((mid.definition - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_jaw_line_state();
        assert_eq!(jl_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_sharpness() {
        let s = new_jaw_line_state();
        assert!(jl_sharpness(&s) > 0.0);
    }
}
