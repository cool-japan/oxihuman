// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Eye fold (epicanthic fold) morph control.

/// Configuration for eye fold morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeFoldConfig {
    pub min_fold: f32,
    pub max_fold: f32,
}

/// Runtime state for eye fold morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeFoldState {
    pub left_fold: f32,
    pub right_fold: f32,
    pub crease_depth: f32,
}

#[allow(dead_code)]
pub fn default_eye_fold_config() -> EyeFoldConfig {
    EyeFoldConfig {
        min_fold: 0.0,
        max_fold: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_eye_fold_state() -> EyeFoldState {
    EyeFoldState {
        left_fold: 0.0,
        right_fold: 0.0,
        crease_depth: 0.3,
    }
}

#[allow(dead_code)]
pub fn ef_set_left(state: &mut EyeFoldState, cfg: &EyeFoldConfig, v: f32) {
    state.left_fold = v.clamp(cfg.min_fold, cfg.max_fold);
}

#[allow(dead_code)]
pub fn ef_set_right(state: &mut EyeFoldState, cfg: &EyeFoldConfig, v: f32) {
    state.right_fold = v.clamp(cfg.min_fold, cfg.max_fold);
}

#[allow(dead_code)]
pub fn ef_set_both(state: &mut EyeFoldState, cfg: &EyeFoldConfig, v: f32) {
    let clamped = v.clamp(cfg.min_fold, cfg.max_fold);
    state.left_fold = clamped;
    state.right_fold = clamped;
}

#[allow(dead_code)]
pub fn ef_set_crease(state: &mut EyeFoldState, v: f32) {
    state.crease_depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ef_reset(state: &mut EyeFoldState) {
    *state = new_eye_fold_state();
}

#[allow(dead_code)]
pub fn ef_to_weights(state: &EyeFoldState) -> Vec<(String, f32)> {
    vec![
        ("eye_fold_left".to_string(), state.left_fold),
        ("eye_fold_right".to_string(), state.right_fold),
        ("eye_fold_crease".to_string(), state.crease_depth),
    ]
}

#[allow(dead_code)]
pub fn ef_to_json(state: &EyeFoldState) -> String {
    format!(
        r#"{{"left_fold":{:.4},"right_fold":{:.4},"crease_depth":{:.4}}}"#,
        state.left_fold, state.right_fold, state.crease_depth
    )
}

#[allow(dead_code)]
pub fn ef_blend(a: &EyeFoldState, b: &EyeFoldState, t: f32) -> EyeFoldState {
    let t = t.clamp(0.0, 1.0);
    EyeFoldState {
        left_fold: a.left_fold + (b.left_fold - a.left_fold) * t,
        right_fold: a.right_fold + (b.right_fold - a.right_fold) * t,
        crease_depth: a.crease_depth + (b.crease_depth - a.crease_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_fold_config();
        assert!(cfg.min_fold.abs() < 1e-6);
        assert!((cfg.max_fold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_eye_fold_state();
        assert!(s.left_fold.abs() < 1e-6);
        assert!((s.crease_depth - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_clamps() {
        let cfg = default_eye_fold_config();
        let mut s = new_eye_fold_state();
        ef_set_left(&mut s, &cfg, 5.0);
        assert!((s.left_fold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_both() {
        let cfg = default_eye_fold_config();
        let mut s = new_eye_fold_state();
        ef_set_both(&mut s, &cfg, 0.6);
        assert!((s.left_fold - 0.6).abs() < 1e-6);
        assert!((s.right_fold - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_crease() {
        let mut s = new_eye_fold_state();
        ef_set_crease(&mut s, 0.9);
        assert!((s.crease_depth - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_eye_fold_config();
        let mut s = new_eye_fold_state();
        ef_set_left(&mut s, &cfg, 0.8);
        ef_reset(&mut s);
        assert!(s.left_fold.abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_eye_fold_state();
        assert_eq!(ef_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_eye_fold_state();
        let j = ef_to_json(&s);
        assert!(j.contains("left_fold"));
    }

    #[test]
    fn test_blend() {
        let a = new_eye_fold_state();
        let mut b = new_eye_fold_state();
        b.left_fold = 1.0;
        let mid = ef_blend(&a, &b, 0.5);
        assert!((mid.left_fold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_right() {
        let cfg = default_eye_fold_config();
        let mut s = new_eye_fold_state();
        ef_set_right(&mut s, &cfg, 0.4);
        assert!((s.right_fold - 0.4).abs() < 1e-6);
    }
}
