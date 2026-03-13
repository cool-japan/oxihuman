// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brow raise / furrow morph control (FACS AU1/2/4 area).

/// State for the brow raise and furrow control.
#[derive(Debug, Clone)]
pub struct BrowRaiseState {
    pub inner_left: f32,
    pub inner_right: f32,
    pub outer_left: f32,
    pub outer_right: f32,
    pub furrow: f32,
}

impl Default for BrowRaiseState {
    fn default() -> Self {
        Self { inner_left: 0.0, inner_right: 0.0, outer_left: 0.0, outer_right: 0.0, furrow: 0.0 }
    }
}

/// Raise both inner brow sections.
pub fn br_raise_inner(state: &mut BrowRaiseState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.inner_left = a;
    state.inner_right = a;
}

/// Raise both outer brow sections.
pub fn br_raise_outer(state: &mut BrowRaiseState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.outer_left = a;
    state.outer_right = a;
}

/// Set furrow amount (corrugator contraction).
pub fn br_set_furrow(state: &mut BrowRaiseState, amount: f32) {
    state.furrow = amount.clamp(0.0, 1.0);
}

/// Return the mean brow raise across all four zones.
pub fn br_mean_raise(state: &BrowRaiseState) -> f32 {
    (state.inner_left + state.inner_right + state.outer_left + state.outer_right) * 0.25
}

/// Return the asymmetry between left and right sides (left - right).
pub fn br_asymmetry(state: &BrowRaiseState) -> f32 {
    let left = (state.inner_left + state.outer_left) * 0.5;
    let right = (state.inner_right + state.outer_right) * 0.5;
    left - right
}

/// Reset all brow weights to zero.
pub fn br_reset(state: &mut BrowRaiseState) {
    *state = BrowRaiseState::default();
}

/// Raise all four zones simultaneously.
pub fn br_raise_all(state: &mut BrowRaiseState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.inner_left = a;
    state.inner_right = a;
    state.outer_left = a;
    state.outer_right = a;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        /* default state should be zero everywhere */
        let s = BrowRaiseState::default();
        assert_eq!(s.inner_left, 0.0);
        assert_eq!(s.furrow, 0.0);
    }

    #[test]
    fn test_raise_inner() {
        /* inner raise should set both inner values */
        let mut s = BrowRaiseState::default();
        br_raise_inner(&mut s, 0.6);
        assert!((s.inner_left - 0.6).abs() < 1e-6);
        assert!((s.inner_right - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_raise_outer() {
        /* outer raise should set both outer values */
        let mut s = BrowRaiseState::default();
        br_raise_outer(&mut s, 0.4);
        assert!((s.outer_left - 0.4).abs() < 1e-6);
        assert!((s.outer_right - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_furrow() {
        /* furrow should set the furrow value */
        let mut s = BrowRaiseState::default();
        br_set_furrow(&mut s, 0.7);
        assert!((s.furrow - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_mean_raise() {
        /* mean should average all four zones */
        let mut s = BrowRaiseState::default();
        br_raise_all(&mut s, 0.8);
        assert!((br_mean_raise(&s) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_asymmetry_symmetric() {
        /* symmetric state should have zero asymmetry */
        let mut s = BrowRaiseState::default();
        br_raise_all(&mut s, 1.0);
        assert!(br_asymmetry(&s).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry_left_high() {
        /* left-biased state should have positive asymmetry */
        let s = BrowRaiseState { inner_left: 1.0, outer_left: 1.0, ..Default::default() };
        assert!(br_asymmetry(&s) > 0.0);
    }

    #[test]
    fn test_reset() {
        /* reset should zero all fields */
        let mut s = BrowRaiseState::default();
        br_raise_all(&mut s, 0.9);
        br_set_furrow(&mut s, 0.5);
        br_reset(&mut s);
        assert_eq!(s.inner_left, 0.0);
        assert_eq!(s.furrow, 0.0);
    }

    #[test]
    fn test_clamp_above_one() {
        /* values above 1 should be clamped */
        let mut s = BrowRaiseState::default();
        br_raise_all(&mut s, 2.5);
        assert!((s.inner_left - 1.0).abs() < 1e-6);
    }
}
