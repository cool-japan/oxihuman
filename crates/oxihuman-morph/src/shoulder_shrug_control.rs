// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Shoulder shrug pose morph control.

/// State for shoulder shrug control.
#[derive(Debug, Clone, Default)]
pub struct ShoulderShrugState {
    pub left: f32,
    pub right: f32,
    pub trapezius_compression: f32,
    pub neck_compress: f32,
}

/// Set both shoulders simultaneously.
pub fn ss_set_both(state: &mut ShoulderShrugState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.left = a;
    state.right = a;
}

/// Set the left shoulder shrug.
pub fn ss_set_left(state: &mut ShoulderShrugState, amount: f32) {
    state.left = amount.clamp(0.0, 1.0);
}

/// Set the right shoulder shrug.
pub fn ss_set_right(state: &mut ShoulderShrugState, amount: f32) {
    state.right = amount.clamp(0.0, 1.0);
}

/// Return the average shrug level.
pub fn ss_average(state: &ShoulderShrugState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Return the neck compression weight driven by mean shrug.
pub fn ss_neck_weight(state: &ShoulderShrugState) -> f32 {
    ss_average(state) * state.neck_compress.max(0.3)
}

/// Return the trapezius compression weight.
pub fn ss_trap_weight(state: &ShoulderShrugState) -> f32 {
    ss_average(state) * state.trapezius_compression.max(0.5)
}

/// Reset shrug to neutral.
pub fn ss_reset(state: &mut ShoulderShrugState) {
    state.left = 0.0;
    state.right = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        /* default should be zero on both sides */
        let s = ShoulderShrugState::default();
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_both() {
        /* set_both should update both sides */
        let mut s = ShoulderShrugState::default();
        ss_set_both(&mut s, 0.5);
        assert!((s.left - 0.5).abs() < 1e-6);
        assert!((s.right - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_only() {
        /* setting left should not touch right */
        let mut s = ShoulderShrugState::default();
        ss_set_left(&mut s, 0.7);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_set_right_only() {
        /* setting right should not touch left */
        let mut s = ShoulderShrugState::default();
        ss_set_right(&mut s, 0.3);
        assert_eq!(s.left, 0.0);
    }

    #[test]
    fn test_average() {
        /* average should be (left+right)/2 */
        let mut s = ShoulderShrugState::default();
        ss_set_left(&mut s, 0.4);
        ss_set_right(&mut s, 0.8);
        assert!((ss_average(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_neck_weight_nonzero() {
        /* neck weight should be nonzero when shruggling */
        let mut s = ShoulderShrugState::default();
        ss_set_both(&mut s, 1.0);
        assert!(ss_neck_weight(&s) > 0.0);
    }

    #[test]
    fn test_trap_weight_nonzero() {
        /* trapezius weight should be nonzero when shrugging */
        let mut s = ShoulderShrugState::default();
        ss_set_both(&mut s, 1.0);
        assert!(ss_trap_weight(&s) > 0.0);
    }

    #[test]
    fn test_reset() {
        /* reset should zero both sides */
        let mut s = ShoulderShrugState::default();
        ss_set_both(&mut s, 0.9);
        ss_reset(&mut s);
        assert_eq!(s.left, 0.0);
        assert_eq!(s.right, 0.0);
    }

    #[test]
    fn test_clamp() {
        /* values above 1 should be clamped */
        let mut s = ShoulderShrugState::default();
        ss_set_both(&mut s, 2.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }
}
