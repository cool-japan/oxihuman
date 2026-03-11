// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Cheek fullness control: adjusts the volume and projection of the cheeks.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekFullnessConfig {
    pub min_fullness: f32,
    pub max_fullness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekFullnessState {
    pub fullness: f32,
    pub projection: f32,
    pub symmetry: f32,
}

#[allow(dead_code)]
pub fn default_cheek_fullness_config() -> CheekFullnessConfig {
    CheekFullnessConfig {
        min_fullness: 0.0,
        max_fullness: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_cheek_fullness_state() -> CheekFullnessState {
    CheekFullnessState {
        fullness: 0.5,
        projection: 0.4,
        symmetry: 1.0,
    }
}

#[allow(dead_code)]
pub fn cf_set_fullness(state: &mut CheekFullnessState, cfg: &CheekFullnessConfig, v: f32) {
    state.fullness = v.clamp(cfg.min_fullness, cfg.max_fullness);
}

#[allow(dead_code)]
pub fn cf_set_projection(state: &mut CheekFullnessState, v: f32) {
    state.projection = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cf_set_symmetry(state: &mut CheekFullnessState, v: f32) {
    state.symmetry = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cf_reset(state: &mut CheekFullnessState) {
    *state = new_cheek_fullness_state();
}

#[allow(dead_code)]
pub fn cf_volume_estimate(state: &CheekFullnessState) -> f32 {
    (4.0 / 3.0) * PI * state.fullness * state.projection * state.fullness
}

#[allow(dead_code)]
pub fn cf_to_weights(state: &CheekFullnessState) -> Vec<(String, f32)> {
    vec![
        ("cheek_fullness".to_string(), state.fullness),
        ("cheek_projection".to_string(), state.projection),
        ("cheek_symmetry".to_string(), state.symmetry),
    ]
}

#[allow(dead_code)]
pub fn cf_to_json(state: &CheekFullnessState) -> String {
    format!(
        r#"{{"fullness":{:.4},"projection":{:.4},"symmetry":{:.4}}}"#,
        state.fullness, state.projection, state.symmetry
    )
}

#[allow(dead_code)]
pub fn cf_blend(a: &CheekFullnessState, b: &CheekFullnessState, t: f32) -> CheekFullnessState {
    let t = t.clamp(0.0, 1.0);
    CheekFullnessState {
        fullness: a.fullness + (b.fullness - a.fullness) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
    }
}

#[allow(dead_code)]
pub fn cf_effective_left(state: &CheekFullnessState) -> f32 {
    state.fullness * state.symmetry
}

#[allow(dead_code)]
pub fn cf_effective_right(state: &CheekFullnessState) -> f32 {
    state.fullness * (2.0 - state.symmetry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheek_fullness_config();
        assert!(cfg.min_fullness.abs() < 1e-6);
        assert!((cfg.max_fullness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_cheek_fullness_state();
        assert!((s.fullness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_fullness_clamps() {
        let cfg = default_cheek_fullness_config();
        let mut s = new_cheek_fullness_state();
        cf_set_fullness(&mut s, &cfg, 5.0);
        assert!((s.fullness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection() {
        let mut s = new_cheek_fullness_state();
        cf_set_projection(&mut s, 0.8);
        assert!((s.projection - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_symmetry() {
        let mut s = new_cheek_fullness_state();
        cf_set_symmetry(&mut s, 0.6);
        assert!((s.symmetry - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_cheek_fullness_config();
        let mut s = new_cheek_fullness_state();
        cf_set_fullness(&mut s, &cfg, 0.9);
        cf_reset(&mut s);
        assert!((s.fullness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_volume_estimate() {
        let s = new_cheek_fullness_state();
        assert!(cf_volume_estimate(&s) > 0.0);
    }

    #[test]
    fn test_to_weights() {
        let s = new_cheek_fullness_state();
        assert_eq!(cf_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_cheek_fullness_state();
        let mut b = new_cheek_fullness_state();
        b.fullness = 1.0;
        let mid = cf_blend(&a, &b, 0.5);
        assert!((mid.fullness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_effective_sides() {
        let s = new_cheek_fullness_state();
        let l = cf_effective_left(&s);
        let r = cf_effective_right(&s);
        assert!((l - r).abs() < 1e-6);
    }
}
