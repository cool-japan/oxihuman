// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Brow height morph control: adjusts vertical position of the brow ridge.

/// Configuration for brow height morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowHeightConfig {
    pub min_height: f32,
    pub max_height: f32,
}

/// Runtime state for brow height morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowHeightState {
    pub left_height: f32,
    pub right_height: f32,
    pub symmetry: f32,
}

#[allow(dead_code)]
pub fn default_brow_height_config() -> BrowHeightConfig {
    BrowHeightConfig {
        min_height: -1.0,
        max_height: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_brow_height_state() -> BrowHeightState {
    BrowHeightState {
        left_height: 0.0,
        right_height: 0.0,
        symmetry: 1.0,
    }
}

#[allow(dead_code)]
pub fn browh_set_left(state: &mut BrowHeightState, cfg: &BrowHeightConfig, v: f32) {
    state.left_height = v.clamp(cfg.min_height, cfg.max_height);
}

#[allow(dead_code)]
pub fn browh_set_right(state: &mut BrowHeightState, cfg: &BrowHeightConfig, v: f32) {
    state.right_height = v.clamp(cfg.min_height, cfg.max_height);
}

#[allow(dead_code)]
pub fn browh_set_both(state: &mut BrowHeightState, cfg: &BrowHeightConfig, v: f32) {
    let clamped = v.clamp(cfg.min_height, cfg.max_height);
    state.left_height = clamped;
    state.right_height = clamped;
}

#[allow(dead_code)]
pub fn browh_set_symmetry(state: &mut BrowHeightState, v: f32) {
    state.symmetry = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn browh_reset(state: &mut BrowHeightState) {
    *state = new_brow_height_state();
}

#[allow(dead_code)]
pub fn browh_effective_right(state: &BrowHeightState) -> f32 {
    state.left_height * state.symmetry + state.right_height * (1.0 - state.symmetry)
}

#[allow(dead_code)]
pub fn browh_to_weights(state: &BrowHeightState) -> Vec<(String, f32)> {
    vec![
        ("brow_height_left".to_string(), state.left_height),
        (
            "brow_height_right".to_string(),
            browh_effective_right(state),
        ),
    ]
}

#[allow(dead_code)]
pub fn browh_to_json(state: &BrowHeightState) -> String {
    format!(
        r#"{{"left_height":{:.4},"right_height":{:.4},"symmetry":{:.4}}}"#,
        state.left_height, state.right_height, state.symmetry
    )
}

#[allow(dead_code)]
pub fn browh_blend(a: &BrowHeightState, b: &BrowHeightState, t: f32) -> BrowHeightState {
    let t = t.clamp(0.0, 1.0);
    BrowHeightState {
        left_height: a.left_height + (b.left_height - a.left_height) * t,
        right_height: a.right_height + (b.right_height - a.right_height) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_height_config();
        assert!((cfg.min_height + 1.0).abs() < 1e-6);
        assert!((cfg.max_height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_brow_height_state();
        assert!(s.left_height.abs() < 1e-6);
        assert!((s.symmetry - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_clamps() {
        let cfg = default_brow_height_config();
        let mut s = new_brow_height_state();
        browh_set_left(&mut s, &cfg, 5.0);
        assert!((s.left_height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_both() {
        let cfg = default_brow_height_config();
        let mut s = new_brow_height_state();
        browh_set_both(&mut s, &cfg, 0.5);
        assert!((s.left_height - 0.5).abs() < 1e-6);
        assert!((s.right_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry() {
        let mut s = new_brow_height_state();
        browh_set_symmetry(&mut s, 0.0);
        assert!(s.symmetry.abs() < 1e-6);
    }

    #[test]
    fn test_effective_right_full_symmetry() {
        let mut s = new_brow_height_state();
        s.left_height = 0.8;
        s.right_height = 0.2;
        s.symmetry = 1.0;
        assert!((browh_effective_right(&s) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_brow_height_config();
        let mut s = new_brow_height_state();
        browh_set_left(&mut s, &cfg, 0.7);
        browh_reset(&mut s);
        assert!(s.left_height.abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_brow_height_state();
        assert_eq!(browh_to_weights(&s).len(), 2);
    }

    #[test]
    fn test_to_json() {
        let s = new_brow_height_state();
        let j = browh_to_json(&s);
        assert!(j.contains("left_height"));
    }

    #[test]
    fn test_blend() {
        let a = new_brow_height_state();
        let mut b = new_brow_height_state();
        b.left_height = 1.0;
        let mid = browh_blend(&a, &b, 0.5);
        assert!((mid.left_height - 0.5).abs() < 1e-6);
    }
}
