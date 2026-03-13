// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nasolabial fold control — nasolabial fold depth morph (cheek-to-mouth crease).

/// Which side of the face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NlSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NasolabialFoldConfig {
    /// Maximum depth displacement in normalised units.
    pub max_depth: f32,
    /// Maximum length influence along the fold.
    pub max_length: f32,
}

impl Default for NasolabialFoldConfig {
    fn default() -> Self {
        Self {
            max_depth: 1.0,
            max_length: 1.0,
        }
    }
}

/// Nasolabial fold state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct NasolabialFoldState {
    /// Fold depth: 0 (flat) .. 1 (deep crease).
    pub depth_left: f32,
    pub depth_right: f32,
    /// Fold length emphasis: 0..=1.
    pub length_left: f32,
    pub length_right: f32,
}

#[allow(dead_code)]
pub fn new_nasolabial_fold_state() -> NasolabialFoldState {
    NasolabialFoldState::default()
}

#[allow(dead_code)]
pub fn default_nasolabial_fold_config() -> NasolabialFoldConfig {
    NasolabialFoldConfig::default()
}

#[allow(dead_code)]
pub fn nlf_set_depth(state: &mut NasolabialFoldState, side: NlSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        NlSide::Left => state.depth_left = v,
        NlSide::Right => state.depth_right = v,
    }
}

#[allow(dead_code)]
pub fn nlf_set_length(state: &mut NasolabialFoldState, side: NlSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        NlSide::Left => state.length_left = v,
        NlSide::Right => state.length_right = v,
    }
}

#[allow(dead_code)]
pub fn nlf_set_both(state: &mut NasolabialFoldState, depth: f32) {
    let depth = depth.clamp(0.0, 1.0);
    state.depth_left = depth;
    state.depth_right = depth;
}

#[allow(dead_code)]
pub fn nlf_reset(state: &mut NasolabialFoldState) {
    *state = NasolabialFoldState::default();
}

#[allow(dead_code)]
pub fn nlf_is_neutral(state: &NasolabialFoldState) -> bool {
    state.depth_left < 1e-4 && state.depth_right < 1e-4
}

/// Asymmetry between left and right fold depth.
#[allow(dead_code)]
pub fn nlf_asymmetry(state: &NasolabialFoldState) -> f32 {
    (state.depth_left - state.depth_right).abs()
}

/// Depth in normalised units for a side.
#[allow(dead_code)]
pub fn nlf_depth(state: &NasolabialFoldState, side: NlSide, cfg: &NasolabialFoldConfig) -> f32 {
    let v = match side {
        NlSide::Left => state.depth_left,
        NlSide::Right => state.depth_right,
    };
    v * cfg.max_depth
}

/// Returns morph weights \[depth_l, depth_r, length_l, length_r\].
#[allow(dead_code)]
pub fn nlf_to_weights(state: &NasolabialFoldState) -> [f32; 4] {
    [
        state.depth_left,
        state.depth_right,
        state.length_left,
        state.length_right,
    ]
}

#[allow(dead_code)]
pub fn nlf_blend(a: &NasolabialFoldState, b: &NasolabialFoldState, t: f32) -> NasolabialFoldState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    NasolabialFoldState {
        depth_left: a.depth_left * inv + b.depth_left * t,
        depth_right: a.depth_right * inv + b.depth_right * t,
        length_left: a.length_left * inv + b.length_left * t,
        length_right: a.length_right * inv + b.length_right * t,
    }
}

#[allow(dead_code)]
pub fn nlf_to_json(state: &NasolabialFoldState) -> String {
    format!(
        "{{\"depth_l\":{:.4},\"depth_r\":{:.4}}}",
        state.depth_left, state.depth_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(nlf_is_neutral(&new_nasolabial_fold_state()));
    }

    #[test]
    fn depth_clamps_high() {
        let mut s = new_nasolabial_fold_state();
        nlf_set_depth(&mut s, NlSide::Left, 5.0);
        assert!((s.depth_left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn depth_clamps_low() {
        let mut s = new_nasolabial_fold_state();
        nlf_set_depth(&mut s, NlSide::Right, -1.0);
        assert!(s.depth_right < 1e-6);
    }

    #[test]
    fn set_both_symmetric() {
        let mut s = new_nasolabial_fold_state();
        nlf_set_both(&mut s, 0.6);
        assert!((s.depth_left - s.depth_right).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_nasolabial_fold_state();
        nlf_set_both(&mut s, 1.0);
        nlf_reset(&mut s);
        assert!(nlf_is_neutral(&s));
    }

    #[test]
    fn asymmetry_nonzero_when_different() {
        let mut s = new_nasolabial_fold_state();
        nlf_set_depth(&mut s, NlSide::Left, 0.8);
        nlf_set_depth(&mut s, NlSide::Right, 0.2);
        assert!((nlf_asymmetry(&s) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn depth_scaled_by_config() {
        let cfg = default_nasolabial_fold_config();
        let mut s = new_nasolabial_fold_state();
        nlf_set_depth(&mut s, NlSide::Left, 1.0);
        assert!((nlf_depth(&s, NlSide::Left, &cfg) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_four_elements() {
        let w = nlf_to_weights(&new_nasolabial_fold_state());
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_nasolabial_fold_state();
        nlf_set_both(&mut b, 1.0);
        let r = nlf_blend(&new_nasolabial_fold_state(), &b, 0.5);
        assert!((r.depth_left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = nlf_to_json(&new_nasolabial_fold_state());
        assert!(j.contains("depth_l") && j.contains("depth_r"));
    }
}
