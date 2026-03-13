// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Forearm / wrist twist morph control.

/// State for wrist twist (pronation/supination) control.
#[derive(Debug, Clone)]
pub struct WristTwistState {
    pub left_deg: f32,
    pub right_deg: f32,
    pub max_deg: f32,
    pub forearm_couple: f32,
}

impl Default for WristTwistState {
    fn default() -> Self {
        Self { left_deg: 0.0, right_deg: 0.0, max_deg: 90.0, forearm_couple: 0.7 }
    }
}

/// Set the left wrist twist in degrees (positive = pronation).
pub fn wt_set_left(state: &mut WristTwistState, deg: f32) {
    state.left_deg = deg.clamp(-state.max_deg, state.max_deg);
}

/// Set the right wrist twist in degrees.
pub fn wt_set_right(state: &mut WristTwistState, deg: f32) {
    state.right_deg = deg.clamp(-state.max_deg, state.max_deg);
}

/// Return the normalised left twist (-1 to 1).
pub fn wt_normalised_left(state: &WristTwistState) -> f32 {
    if state.max_deg < 1e-6 { return 0.0; }
    state.left_deg / state.max_deg
}

/// Return the normalised right twist (-1 to 1).
pub fn wt_normalised_right(state: &WristTwistState) -> f32 {
    if state.max_deg < 1e-6 { return 0.0; }
    state.right_deg / state.max_deg
}

/// Return the forearm twist weight for the left arm.
pub fn wt_forearm_left(state: &WristTwistState) -> f32 {
    wt_normalised_left(state).abs() * state.forearm_couple
}

/// Reset both wrists to neutral.
pub fn wt_reset(state: &mut WristTwistState) {
    state.left_deg = 0.0;
    state.right_deg = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_neutral() {
        /* default should be zero twist */
        let s = WristTwistState::default();
        assert_eq!(s.left_deg, 0.0);
        assert_eq!(s.right_deg, 0.0);
    }

    #[test]
    fn test_set_left() {
        /* left twist should be set */
        let mut s = WristTwistState::default();
        wt_set_left(&mut s, 45.0);
        assert!((s.left_deg - 45.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_right() {
        /* right twist should be set */
        let mut s = WristTwistState::default();
        wt_set_right(&mut s, -30.0);
        assert!((s.right_deg + 30.0).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_max() {
        /* values beyond max should be clamped */
        let mut s = WristTwistState::default();
        wt_set_left(&mut s, 999.0);
        assert!((s.left_deg - s.max_deg).abs() < 1e-5);
    }

    #[test]
    fn test_normalised_left() {
        /* half max should normalise to 0.5 */
        let mut s = WristTwistState::default();
        let half = s.max_deg / 2.0;
        wt_set_left(&mut s, half);
        assert!((wt_normalised_left(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_normalised_right() {
        /* normalised right should reflect sign */
        let mut s = WristTwistState::default();
        let max = s.max_deg;
        wt_set_right(&mut s, -max);
        assert!((wt_normalised_right(&s) + 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_forearm_left_nonzero() {
        /* forearm weight should be nonzero when twisted */
        let mut s = WristTwistState::default();
        wt_set_left(&mut s, 90.0);
        assert!(wt_forearm_left(&s) > 0.0);
    }

    #[test]
    fn test_forearm_left_zero_at_neutral() {
        /* forearm weight should be zero at neutral */
        let s = WristTwistState::default();
        assert_eq!(wt_forearm_left(&s), 0.0);
    }

    #[test]
    fn test_reset() {
        /* reset should zero both twists */
        let mut s = WristTwistState::default();
        wt_set_left(&mut s, 60.0);
        wt_set_right(&mut s, -45.0);
        wt_reset(&mut s);
        assert_eq!(s.left_deg, 0.0);
        assert_eq!(s.right_deg, 0.0);
    }
}
