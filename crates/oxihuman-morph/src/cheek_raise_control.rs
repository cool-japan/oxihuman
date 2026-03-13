// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cheek raising morph control (FACS AU6 area).

/// State for cheek raise control.
#[derive(Debug, Clone)]
pub struct CheekRaiseState {
    pub left: f32,
    pub right: f32,
    pub orbital_compression: f32,
}

impl Default for CheekRaiseState {
    fn default() -> Self {
        Self { left: 0.0, right: 0.0, orbital_compression: 0.4 }
    }
}

/// Set both cheek raise weights symmetrically.
pub fn cr_set_both(state: &mut CheekRaiseState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.left = a;
    state.right = a;
}

/// Set the left cheek raise independently.
pub fn cr_set_left(state: &mut CheekRaiseState, amount: f32) {
    state.left = amount.clamp(0.0, 1.0);
}

/// Set the right cheek raise independently.
pub fn cr_set_right(state: &mut CheekRaiseState, amount: f32) {
    state.right = amount.clamp(0.0, 1.0);
}

/// Return the average of left and right cheek raise.
pub fn cr_average(state: &CheekRaiseState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Return the orbital compression weight (how much the eye orbit is affected).
pub fn cr_orbital_weight(state: &CheekRaiseState) -> f32 {
    cr_average(state) * state.orbital_compression
}

/// Reset both cheeks to zero.
pub fn cr_reset(state: &mut CheekRaiseState) {
    state.left = 0.0;
    state.right = 0.0;
}

/// Blend both cheek raises toward a target by t.
pub fn cr_blend_toward(state: &mut CheekRaiseState, target: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    let target = target.clamp(0.0, 1.0);
    state.left += (target - state.left) * t;
    state.right += (target - state.right) * t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        /* default state should be zero */
        let s = CheekRaiseState::default();
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_both() {
        /* set_both should apply to both sides */
        let mut s = CheekRaiseState::default();
        cr_set_both(&mut s, 0.5);
        assert!((s.left - 0.5).abs() < 1e-6);
        assert!((s.right - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_independent() {
        /* left can be set without affecting right */
        let mut s = CheekRaiseState::default();
        cr_set_left(&mut s, 0.8);
        assert!((s.left - 0.8).abs() < 1e-6);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_right_independent() {
        /* right can be set without affecting left */
        let mut s = CheekRaiseState::default();
        cr_set_right(&mut s, 0.3);
        assert_eq!(s.left, 0.0);
        assert!((s.right - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_average() {
        /* average should be (left+right)/2 */
        let mut s = CheekRaiseState::default();
        cr_set_left(&mut s, 0.4);
        cr_set_right(&mut s, 0.8);
        assert!((cr_average(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_orbital_weight() {
        /* orbital weight should scale with average and compression factor */
        let mut s = CheekRaiseState::default();
        cr_set_both(&mut s, 1.0);
        let expected = s.orbital_compression;
        assert!((cr_orbital_weight(&s) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        /* reset should zero both cheeks */
        let mut s = CheekRaiseState::default();
        cr_set_both(&mut s, 0.9);
        cr_reset(&mut s);
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_blend_toward() {
        /* blend should move toward target */
        let mut s = CheekRaiseState::default();
        cr_blend_toward(&mut s, 1.0, 1.0);
        assert!((s.left - 1.0).abs() < 1e-6);
        assert!((s.right - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_above_one() {
        /* values above 1.0 should be clamped */
        let mut s = CheekRaiseState::default();
        cr_set_both(&mut s, 5.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }
}
