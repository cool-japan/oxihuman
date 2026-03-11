// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Ear concha control: adjusts the depth and shape of the ear concha cavity.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarConchaConfig {
    pub min_depth: f32,
    pub max_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarConchaState {
    pub depth: f32,
    pub width: f32,
    pub symmetry: f32,
}

#[allow(dead_code)]
pub fn default_ear_concha_config() -> EarConchaConfig {
    EarConchaConfig {
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_ear_concha_state() -> EarConchaState {
    EarConchaState {
        depth: 0.5,
        width: 0.5,
        symmetry: 1.0,
    }
}

#[allow(dead_code)]
pub fn ec_set_depth(state: &mut EarConchaState, cfg: &EarConchaConfig, v: f32) {
    state.depth = v.clamp(cfg.min_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn ec_set_width(state: &mut EarConchaState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ec_set_symmetry(state: &mut EarConchaState, v: f32) {
    state.symmetry = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ec_reset(state: &mut EarConchaState) {
    *state = new_ear_concha_state();
}

#[allow(dead_code)]
pub fn ec_cavity_volume(state: &EarConchaState) -> f32 {
    PI * state.width * state.width * state.depth * 0.25
}

#[allow(dead_code)]
pub fn ec_to_weights(state: &EarConchaState) -> Vec<(String, f32)> {
    vec![
        ("ear_concha_depth".to_string(), state.depth),
        ("ear_concha_width".to_string(), state.width),
        ("ear_concha_symmetry".to_string(), state.symmetry),
    ]
}

#[allow(dead_code)]
pub fn ec_to_json(state: &EarConchaState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"symmetry":{:.4}}}"#,
        state.depth, state.width, state.symmetry
    )
}

#[allow(dead_code)]
pub fn ec_blend(a: &EarConchaState, b: &EarConchaState, t: f32) -> EarConchaState {
    let t = t.clamp(0.0, 1.0);
    EarConchaState {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
    }
}

#[allow(dead_code)]
pub fn ec_effective_left(state: &EarConchaState) -> f32 {
    state.depth * state.symmetry
}

#[allow(dead_code)]
pub fn ec_effective_right(state: &EarConchaState) -> f32 {
    state.depth * (2.0 - state.symmetry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ear_concha_config();
        assert!(cfg.min_depth.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_ear_concha_state();
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_ear_concha_config();
        let mut s = new_ear_concha_state();
        ec_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_ear_concha_state();
        ec_set_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_symmetry() {
        let mut s = new_ear_concha_state();
        ec_set_symmetry(&mut s, 0.7);
        assert!((s.symmetry - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_ear_concha_config();
        let mut s = new_ear_concha_state();
        ec_set_depth(&mut s, &cfg, 0.9);
        ec_reset(&mut s);
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cavity_volume() {
        let s = new_ear_concha_state();
        assert!(ec_cavity_volume(&s) > 0.0);
    }

    #[test]
    fn test_to_weights() {
        let s = new_ear_concha_state();
        assert_eq!(ec_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_ear_concha_state();
        let mut b = new_ear_concha_state();
        b.depth = 1.0;
        let mid = ec_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_effective_sides() {
        let s = new_ear_concha_state();
        assert!((ec_effective_left(&s) - ec_effective_right(&s)).abs() < 1e-6);
    }
}
