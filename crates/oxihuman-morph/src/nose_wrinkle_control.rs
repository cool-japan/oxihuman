// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Nose wrinkle morph control (FACS AU9 area).

/// State for nose wrinkle control.
#[derive(Debug, Clone, Default)]
pub struct NoseWrinkleState {
    pub left: f32,
    pub right: f32,
    pub bridge_compress: f32,
}

/// Set nose wrinkle symmetrically.
pub fn nw_set_both(state: &mut NoseWrinkleState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.left = a;
    state.right = a;
}

/// Set the left nostril wrinkle.
pub fn nw_set_left(state: &mut NoseWrinkleState, amount: f32) {
    state.left = amount.clamp(0.0, 1.0);
}

/// Set the right nostril wrinkle.
pub fn nw_set_right(state: &mut NoseWrinkleState, amount: f32) {
    state.right = amount.clamp(0.0, 1.0);
}

/// Return the average wrinkle level.
pub fn nw_average(state: &NoseWrinkleState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Return bridge compression as driven by mean wrinkle.
pub fn nw_bridge_weight(state: &NoseWrinkleState) -> f32 {
    nw_average(state) * state.bridge_compress.max(0.3)
}

/// Reset all nose wrinkle weights.
pub fn nw_reset(state: &mut NoseWrinkleState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        /* default state should have zero wrinkle */
        let s = NoseWrinkleState::default();
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_both() {
        /* set_both should update both sides */
        let mut s = NoseWrinkleState::default();
        nw_set_both(&mut s, 0.7);
        assert!((s.left - 0.7).abs() < 1e-6);
        assert!((s.right - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_left() {
        /* left should be set without affecting right */
        let mut s = NoseWrinkleState::default();
        nw_set_left(&mut s, 0.5);
        assert!((s.left - 0.5).abs() < 1e-6);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_right() {
        /* right should be set without affecting left */
        let mut s = NoseWrinkleState::default();
        nw_set_right(&mut s, 0.4);
        assert_eq!(s.left, 0.0);
        assert!((s.right - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_average() {
        /* average should be (left+right)/2 */
        let mut s = NoseWrinkleState::default();
        nw_set_left(&mut s, 0.2);
        nw_set_right(&mut s, 0.8);
        assert!((nw_average(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        /* out of range values should be clamped */
        let mut s = NoseWrinkleState::default();
        nw_set_both(&mut s, 3.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        /* reset should zero left and right */
        let mut s = NoseWrinkleState::default();
        nw_set_both(&mut s, 0.6);
        nw_reset(&mut s);
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_bridge_weight_nonzero() {
        /* bridge weight should be nonzero when wrinkle is active */
        let mut s = NoseWrinkleState::default();
        nw_set_both(&mut s, 1.0);
        assert!(nw_bridge_weight(&s) > 0.0);
    }

    #[test]
    fn test_bridge_weight_zero_when_relaxed() {
        /* bridge weight should be zero when wrinkle is zero */
        let s = NoseWrinkleState::default();
        assert_eq!(nw_bridge_weight(&s), 0.0);
    }
}
