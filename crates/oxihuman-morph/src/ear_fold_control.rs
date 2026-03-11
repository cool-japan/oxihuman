// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ear fold control — the antihelical fold prominence and definition.

/// Which ear.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarFoldSide {
    Left,
    Right,
}

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarFoldConfig {
    pub max_fold: f32,
}

impl Default for EarFoldConfig {
    fn default() -> Self {
        EarFoldConfig { max_fold: 1.0 }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarFoldState {
    left_fold: f32,
    right_fold: f32,
    /// Definition sharpness in `[0.0, 1.0]`.
    definition: f32,
    config: EarFoldConfig,
}

/// Default config.
pub fn default_ear_fold_config() -> EarFoldConfig {
    EarFoldConfig::default()
}

/// New neutral state.
pub fn new_ear_fold_state(config: EarFoldConfig) -> EarFoldState {
    EarFoldState {
        left_fold: 0.0,
        right_fold: 0.0,
        definition: 0.5,
        config,
    }
}

/// Set fold on one side.
pub fn ef2_set_fold(state: &mut EarFoldState, side: EarFoldSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        EarFoldSide::Left => state.left_fold = v,
        EarFoldSide::Right => state.right_fold = v,
    }
}

/// Set fold on both sides.
pub fn ef2_set_both(state: &mut EarFoldState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left_fold = v;
    state.right_fold = v;
}

/// Set definition sharpness.
pub fn ef2_set_definition(state: &mut EarFoldState, v: f32) {
    state.definition = v.clamp(0.0, 1.0);
}

/// Reset.
pub fn ef2_reset(state: &mut EarFoldState) {
    state.left_fold = 0.0;
    state.right_fold = 0.0;
    state.definition = 0.5;
}

/// True if fold values are effectively zero.
pub fn ef2_is_neutral(state: &EarFoldState) -> bool {
    state.left_fold < 1e-5 && state.right_fold < 1e-5
}

/// Symmetry: 1.0 = perfect symmetry, 0.0 = maximally asymmetric.
pub fn ef2_symmetry(state: &EarFoldState) -> f32 {
    1.0 - (state.left_fold - state.right_fold).abs()
}

/// Average fold depth.
pub fn ef2_average_fold(state: &EarFoldState) -> f32 {
    (state.left_fold + state.right_fold) * 0.5
}

/// Morph weights: `[left_fold, right_fold, definition]`.
pub fn ef2_to_weights(state: &EarFoldState) -> [f32; 3] {
    let s = state.config.max_fold;
    [
        (state.left_fold * s).clamp(0.0, 1.0),
        (state.right_fold * s).clamp(0.0, 1.0),
        state.definition,
    ]
}

/// Blend.
pub fn ef2_blend(a: &EarFoldState, b: &EarFoldState, t: f32) -> EarFoldState {
    let t = t.clamp(0.0, 1.0);
    EarFoldState {
        left_fold: a.left_fold + (b.left_fold - a.left_fold) * t,
        right_fold: a.right_fold + (b.right_fold - a.right_fold) * t,
        definition: a.definition + (b.definition - a.definition) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn ef2_to_json(state: &EarFoldState) -> String {
    format!(
        r#"{{"left_fold":{:.4},"right_fold":{:.4},"definition":{:.4}}}"#,
        state.left_fold, state.right_fold, state.definition
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> EarFoldState {
        new_ear_fold_state(default_ear_fold_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(ef2_is_neutral(&make()));
    }

    #[test]
    fn set_one_side() {
        let mut s = make();
        ef2_set_fold(&mut s, EarFoldSide::Left, 0.6);
        assert!((s.left_fold - 0.6).abs() < 1e-5);
    }

    #[test]
    fn set_both_equal() {
        let mut s = make();
        ef2_set_both(&mut s, 0.4);
        assert!((s.left_fold - s.right_fold).abs() < 1e-5);
    }

    #[test]
    fn symmetry_when_equal() {
        let mut s = make();
        ef2_set_both(&mut s, 0.5);
        assert!((ef2_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_zeros_folds() {
        let mut s = make();
        ef2_set_both(&mut s, 0.9);
        ef2_reset(&mut s);
        assert!(ef2_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        ef2_set_both(&mut b, 1.0);
        let m = ef2_blend(&make(), &b, 0.5);
        assert!((m.left_fold - 0.5).abs() < 1e-5);
    }

    #[test]
    fn weights_in_range() {
        let mut s = make();
        ef2_set_both(&mut s, 0.7);
        for v in ef2_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn json_has_left_fold() {
        assert!(ef2_to_json(&make()).contains("left_fold"));
    }

    #[test]
    fn clamp_high() {
        let mut s = make();
        ef2_set_fold(&mut s, EarFoldSide::Right, 10.0);
        assert!((s.right_fold - 1.0).abs() < 1e-5);
    }
}
