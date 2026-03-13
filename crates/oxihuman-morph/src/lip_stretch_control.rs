// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lip corner stretch morph control (FACS AU20 area).

/// State for lip stretch (lip corner puller) control.
#[derive(Debug, Clone, Default)]
pub struct LipStretchState {
    pub left: f32,
    pub right: f32,
    pub upper_retract: f32,
    pub lower_retract: f32,
}

/// Set lip stretch symmetrically.
pub fn ls_set_both(state: &mut LipStretchState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.left = a;
    state.right = a;
}

/// Set left corner stretch.
pub fn ls_set_left(state: &mut LipStretchState, amount: f32) {
    state.left = amount.clamp(0.0, 1.0);
}

/// Set right corner stretch.
pub fn ls_set_right(state: &mut LipStretchState, amount: f32) {
    state.right = amount.clamp(0.0, 1.0);
}

/// Derive the upper lip retraction from the mean stretch.
pub fn ls_upper_retract(state: &LipStretchState) -> f32 {
    let mean = (state.left + state.right) * 0.5;
    mean * state.upper_retract.max(0.2)
}

/// Derive the lower lip retraction from the mean stretch.
pub fn ls_lower_retract(state: &LipStretchState) -> f32 {
    let mean = (state.left + state.right) * 0.5;
    mean * state.lower_retract.max(0.15)
}

/// Return the asymmetry between left and right corners.
pub fn ls_asymmetry(state: &LipStretchState) -> f32 {
    state.left - state.right
}

/// Reset all lip stretch weights.
pub fn ls_reset(state: &mut LipStretchState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        /* default should be zero */
        let s = LipStretchState::default();
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_both() {
        /* set_both should update both corners */
        let mut s = LipStretchState::default();
        ls_set_both(&mut s, 0.6);
        assert!((s.left - 0.6).abs() < 1e-6);
        assert!((s.right - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_only() {
        /* left set should not affect right */
        let mut s = LipStretchState::default();
        ls_set_left(&mut s, 0.4);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_right_only() {
        /* right set should not affect left */
        let mut s = LipStretchState::default();
        ls_set_right(&mut s, 0.5);
        assert_eq!(s.left, 0.0);
    }

    #[test]
    fn test_upper_retract_nonzero() {
        /* upper retract should be nonzero when stretch is active */
        let mut s = LipStretchState::default();
        ls_set_both(&mut s, 1.0);
        assert!(ls_upper_retract(&s) > 0.0);
    }

    #[test]
    fn test_lower_retract_nonzero() {
        /* lower retract should be nonzero when stretch is active */
        let mut s = LipStretchState::default();
        ls_set_both(&mut s, 1.0);
        assert!(ls_lower_retract(&s) > 0.0);
    }

    #[test]
    fn test_asymmetry() {
        /* asymmetry should reflect left minus right */
        let mut s = LipStretchState::default();
        ls_set_left(&mut s, 0.8);
        ls_set_right(&mut s, 0.2);
        assert!((ls_asymmetry(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        /* reset should zero both corners */
        let mut s = LipStretchState::default();
        ls_set_both(&mut s, 0.9);
        ls_reset(&mut s);
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_clamp_negative() {
        /* negative values should be clamped to zero */
        let mut s = LipStretchState::default();
        ls_set_both(&mut s, -1.0);
        assert_eq!(s.left, 0.0);
    }
}
