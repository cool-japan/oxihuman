// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ear helix fold — controls the roll and definition of the helical rim.

/// Configuration for ear helix fold.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarHelixConfig {
    pub max_fold: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarHelixSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarHelixState {
    pub left_fold: f32,
    pub right_fold: f32,
    pub definition: f32,
}

#[allow(dead_code)]
pub fn default_ear_helix_config() -> EarHelixConfig {
    EarHelixConfig { max_fold: 1.0 }
}

#[allow(dead_code)]
pub fn new_ear_helix_state() -> EarHelixState {
    EarHelixState {
        left_fold: 0.0,
        right_fold: 0.0,
        definition: 0.0,
    }
}

#[allow(dead_code)]
pub fn ehf_set_fold(state: &mut EarHelixState, cfg: &EarHelixConfig, side: EarHelixSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_fold);
    match side {
        EarHelixSide::Left => state.left_fold = clamped,
        EarHelixSide::Right => state.right_fold = clamped,
    }
}

#[allow(dead_code)]
pub fn ehf_set_both(state: &mut EarHelixState, cfg: &EarHelixConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_fold);
    state.left_fold = clamped;
    state.right_fold = clamped;
}

#[allow(dead_code)]
pub fn ehf_set_definition(state: &mut EarHelixState, v: f32) {
    state.definition = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ehf_reset(state: &mut EarHelixState) {
    *state = new_ear_helix_state();
}

#[allow(dead_code)]
pub fn ehf_is_neutral(state: &EarHelixState) -> bool {
    state.left_fold.abs() < 1e-6 && state.right_fold.abs() < 1e-6 && state.definition.abs() < 1e-6
}

#[allow(dead_code)]
pub fn ehf_average_fold(state: &EarHelixState) -> f32 {
    (state.left_fold + state.right_fold) * 0.5
}

#[allow(dead_code)]
pub fn ehf_symmetry(state: &EarHelixState) -> f32 {
    (state.left_fold - state.right_fold).abs()
}

#[allow(dead_code)]
pub fn ehf_blend(a: &EarHelixState, b: &EarHelixState, t: f32) -> EarHelixState {
    let t = t.clamp(0.0, 1.0);
    EarHelixState {
        left_fold: a.left_fold + (b.left_fold - a.left_fold) * t,
        right_fold: a.right_fold + (b.right_fold - a.right_fold) * t,
        definition: a.definition + (b.definition - a.definition) * t,
    }
}

#[allow(dead_code)]
pub fn ehf_to_weights(state: &EarHelixState) -> Vec<(String, f32)> {
    vec![
        ("ear_helix_fold_l".to_string(), state.left_fold),
        ("ear_helix_fold_r".to_string(), state.right_fold),
        ("ear_helix_definition".to_string(), state.definition),
    ]
}

#[allow(dead_code)]
pub fn ehf_to_json(state: &EarHelixState) -> String {
    format!(
        r#"{{"left_fold":{:.4},"right_fold":{:.4},"definition":{:.4}}}"#,
        state.left_fold, state.right_fold, state.definition
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_ear_helix_config();
        assert!((cfg.max_fold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_ear_helix_state();
        assert!(ehf_is_neutral(&s));
    }

    #[test]
    fn set_fold_left() {
        let cfg = default_ear_helix_config();
        let mut s = new_ear_helix_state();
        ehf_set_fold(&mut s, &cfg, EarHelixSide::Left, 0.5);
        assert!((s.left_fold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_fold_clamps() {
        let cfg = default_ear_helix_config();
        let mut s = new_ear_helix_state();
        ehf_set_fold(&mut s, &cfg, EarHelixSide::Right, 10.0);
        assert!((s.right_fold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_symmetric() {
        let cfg = default_ear_helix_config();
        let mut s = new_ear_helix_state();
        ehf_set_both(&mut s, &cfg, 0.7);
        assert!(ehf_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_definition() {
        let mut s = new_ear_helix_state();
        ehf_set_definition(&mut s, 0.8);
        assert!((s.definition - 0.8).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_ear_helix_config();
        let mut s = new_ear_helix_state();
        ehf_set_both(&mut s, &cfg, 0.9);
        ehf_reset(&mut s);
        assert!(ehf_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_ear_helix_state();
        let cfg = default_ear_helix_config();
        let mut b = new_ear_helix_state();
        ehf_set_both(&mut b, &cfg, 1.0);
        let m = ehf_blend(&a, &b, 0.5);
        assert!((m.left_fold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_ear_helix_state();
        assert_eq!(ehf_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_ear_helix_state();
        let j = ehf_to_json(&s);
        assert!(j.contains("left_fold"));
    }
}
