// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear rim (helix) morph — controls the sharpness and roll of the ear rim.

/// Configuration for ear rim control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarRimConfig {
    pub max_roll: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarRimSide {
    Left,
    Right,
}

/// Runtime state for ear rim morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarRimState {
    pub left_roll: f32,
    pub right_roll: f32,
    pub left_sharpness: f32,
    pub right_sharpness: f32,
}

#[allow(dead_code)]
pub fn default_ear_rim_config() -> EarRimConfig {
    EarRimConfig { max_roll: 1.0 }
}

#[allow(dead_code)]
pub fn new_ear_rim_state() -> EarRimState {
    EarRimState {
        left_roll: 0.0,
        right_roll: 0.0,
        left_sharpness: 0.0,
        right_sharpness: 0.0,
    }
}

#[allow(dead_code)]
pub fn er_set_roll(state: &mut EarRimState, cfg: &EarRimConfig, side: EarRimSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_roll);
    match side {
        EarRimSide::Left => state.left_roll = clamped,
        EarRimSide::Right => state.right_roll = clamped,
    }
}

#[allow(dead_code)]
pub fn er_set_sharpness(state: &mut EarRimState, side: EarRimSide, v: f32) {
    let clamped = v.clamp(0.0, 1.0);
    match side {
        EarRimSide::Left => state.left_sharpness = clamped,
        EarRimSide::Right => state.right_sharpness = clamped,
    }
}

#[allow(dead_code)]
pub fn er_set_both_roll(state: &mut EarRimState, cfg: &EarRimConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_roll);
    state.left_roll = clamped;
    state.right_roll = clamped;
}

#[allow(dead_code)]
pub fn er_reset(state: &mut EarRimState) {
    *state = new_ear_rim_state();
}

#[allow(dead_code)]
pub fn er_is_neutral(state: &EarRimState) -> bool {
    let vals = [
        state.left_roll,
        state.right_roll,
        state.left_sharpness,
        state.right_sharpness,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn er_average_roll(state: &EarRimState) -> f32 {
    (state.left_roll + state.right_roll) * 0.5
}

#[allow(dead_code)]
pub fn er_symmetry(state: &EarRimState) -> f32 {
    (state.left_roll - state.right_roll).abs()
}

#[allow(dead_code)]
pub fn er_blend(a: &EarRimState, b: &EarRimState, t: f32) -> EarRimState {
    let t = t.clamp(0.0, 1.0);
    EarRimState {
        left_roll: a.left_roll + (b.left_roll - a.left_roll) * t,
        right_roll: a.right_roll + (b.right_roll - a.right_roll) * t,
        left_sharpness: a.left_sharpness + (b.left_sharpness - a.left_sharpness) * t,
        right_sharpness: a.right_sharpness + (b.right_sharpness - a.right_sharpness) * t,
    }
}

#[allow(dead_code)]
pub fn er_to_weights(state: &EarRimState) -> Vec<(String, f32)> {
    vec![
        ("ear_rim_roll_l".to_string(), state.left_roll),
        ("ear_rim_roll_r".to_string(), state.right_roll),
        ("ear_rim_sharp_l".to_string(), state.left_sharpness),
        ("ear_rim_sharp_r".to_string(), state.right_sharpness),
    ]
}

#[allow(dead_code)]
pub fn er_to_json(state: &EarRimState) -> String {
    format!(
        r#"{{"left_roll":{:.4},"right_roll":{:.4},"left_sharpness":{:.4},"right_sharpness":{:.4}}}"#,
        state.left_roll, state.right_roll, state.left_sharpness, state.right_sharpness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_ear_rim_config();
        assert!((cfg.max_roll - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_ear_rim_state();
        assert!(er_is_neutral(&s));
    }

    #[test]
    fn set_roll_left() {
        let cfg = default_ear_rim_config();
        let mut s = new_ear_rim_state();
        er_set_roll(&mut s, &cfg, EarRimSide::Left, 0.5);
        assert!((s.left_roll - 0.5).abs() < 1e-6);
        assert_eq!(s.right_roll, 0.0);
    }

    #[test]
    fn set_roll_clamps() {
        let cfg = default_ear_rim_config();
        let mut s = new_ear_rim_state();
        er_set_roll(&mut s, &cfg, EarRimSide::Right, 5.0);
        assert!((s.right_roll - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_sharpness() {
        let mut s = new_ear_rim_state();
        er_set_sharpness(&mut s, EarRimSide::Left, 0.8);
        assert!((s.left_sharpness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn set_both_roll_equal() {
        let cfg = default_ear_rim_config();
        let mut s = new_ear_rim_state();
        er_set_both_roll(&mut s, &cfg, 0.6);
        assert!((s.left_roll - 0.6).abs() < 1e-6);
        assert!((s.right_roll - 0.6).abs() < 1e-6);
    }

    #[test]
    fn symmetry_zero_when_equal() {
        let cfg = default_ear_rim_config();
        let mut s = new_ear_rim_state();
        er_set_both_roll(&mut s, &cfg, 0.5);
        assert!(er_symmetry(&s) < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_ear_rim_config();
        let mut s = new_ear_rim_state();
        er_set_both_roll(&mut s, &cfg, 0.7);
        er_reset(&mut s);
        assert!(er_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_ear_rim_state();
        let cfg = default_ear_rim_config();
        let mut b = new_ear_rim_state();
        er_set_both_roll(&mut b, &cfg, 1.0);
        let mid = er_blend(&a, &b, 0.5);
        assert!((mid.left_roll - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_ear_rim_state();
        assert_eq!(er_to_weights(&s).len(), 4);
    }
}
