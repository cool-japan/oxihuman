// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Eye lid crease control: adjusts the crease depth and position above the eyelid.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeLidCreaseConfig {
    pub min_depth: f32,
    pub max_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeLidCreaseState {
    pub depth: f32,
    pub height: f32,
    pub symmetry: f32,
    pub fold: f32,
}

#[allow(dead_code)]
pub fn default_eye_lid_crease_config() -> EyeLidCreaseConfig {
    EyeLidCreaseConfig {
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_eye_lid_crease_state() -> EyeLidCreaseState {
    EyeLidCreaseState {
        depth: 0.5,
        height: 0.5,
        symmetry: 1.0,
        fold: 0.0,
    }
}

#[allow(dead_code)]
pub fn elc_set_depth(state: &mut EyeLidCreaseState, cfg: &EyeLidCreaseConfig, v: f32) {
    state.depth = v.clamp(cfg.min_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn elc_set_height(state: &mut EyeLidCreaseState, v: f32) {
    state.height = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn elc_set_symmetry(state: &mut EyeLidCreaseState, v: f32) {
    state.symmetry = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn elc_set_fold(state: &mut EyeLidCreaseState, v: f32) {
    state.fold = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn elc_reset(state: &mut EyeLidCreaseState) {
    *state = new_eye_lid_crease_state();
}

#[allow(dead_code)]
pub fn elc_crease_angle(state: &EyeLidCreaseState) -> f32 {
    state.depth * FRAC_PI_4
}

#[allow(dead_code)]
pub fn elc_to_weights(state: &EyeLidCreaseState) -> Vec<(String, f32)> {
    vec![
        ("eyelid_crease_depth".to_string(), state.depth),
        ("eyelid_crease_height".to_string(), state.height),
        ("eyelid_crease_symmetry".to_string(), state.symmetry),
        ("eyelid_crease_fold".to_string(), state.fold),
    ]
}

#[allow(dead_code)]
pub fn elc_to_json(state: &EyeLidCreaseState) -> String {
    format!(
        r#"{{"depth":{:.4},"height":{:.4},"symmetry":{:.4},"fold":{:.4}}}"#,
        state.depth, state.height, state.symmetry, state.fold
    )
}

#[allow(dead_code)]
pub fn elc_blend(a: &EyeLidCreaseState, b: &EyeLidCreaseState, t: f32) -> EyeLidCreaseState {
    let t = t.clamp(0.0, 1.0);
    EyeLidCreaseState {
        depth: a.depth + (b.depth - a.depth) * t,
        height: a.height + (b.height - a.height) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
        fold: a.fold + (b.fold - a.fold) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_lid_crease_config();
        assert!(cfg.min_depth.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_eye_lid_crease_state();
        assert!((s.depth - 0.5).abs() < 1e-6);
        assert!((s.fold).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_eye_lid_crease_config();
        let mut s = new_eye_lid_crease_state();
        elc_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut s = new_eye_lid_crease_state();
        elc_set_height(&mut s, 0.8);
        assert!((s.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_fold() {
        let mut s = new_eye_lid_crease_state();
        elc_set_fold(&mut s, 0.6);
        assert!((s.fold - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_eye_lid_crease_config();
        let mut s = new_eye_lid_crease_state();
        elc_set_depth(&mut s, &cfg, 0.9);
        elc_reset(&mut s);
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_crease_angle() {
        let mut s = new_eye_lid_crease_state();
        s.depth = 1.0;
        assert!((elc_crease_angle(&s) - FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_eye_lid_crease_state();
        assert_eq!(elc_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_blend() {
        let a = new_eye_lid_crease_state();
        let mut b = new_eye_lid_crease_state();
        b.depth = 1.0;
        let mid = elc_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_eye_lid_crease_state();
        let j = elc_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("fold"));
    }
}
