// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Toe curl / spread morph control.

/// State for toe curl control of one foot.
#[derive(Debug, Clone, Default)]
pub struct ToeCurlState {
    pub big_toe: f32,
    pub long_toe: f32,
    pub middle_toe: f32,
    pub ring_toe: f32,
    pub little_toe: f32,
    pub spread: f32,
}

/// Set all toes to the same curl amount.
pub fn tc_set_all_curl(state: &mut ToeCurlState, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    state.big_toe = a;
    state.long_toe = a;
    state.middle_toe = a;
    state.ring_toe = a;
    state.little_toe = a;
}

/// Set the spread weight (toes apart).
pub fn tc_set_spread(state: &mut ToeCurlState, amount: f32) {
    state.spread = amount.clamp(0.0, 1.0);
}

/// Return the mean curl across all toes.
pub fn tc_mean_curl(state: &ToeCurlState) -> f32 {
    (state.big_toe + state.long_toe + state.middle_toe + state.ring_toe + state.little_toe) / 5.0
}

/// Return whether the foot is in a neutral pose.
pub fn tc_is_neutral(state: &ToeCurlState) -> bool {
    tc_mean_curl(state) < 0.05 && state.spread < 0.05
}

/// Reset all weights to neutral.
pub fn tc_reset(state: &mut ToeCurlState) {
    *state = ToeCurlState::default();
}

/// Set the big toe curl independently.
pub fn tc_set_big_toe(state: &mut ToeCurlState, amount: f32) {
    state.big_toe = amount.clamp(0.0, 1.0);
}

/// Blend toward a target curl amount.
pub fn tc_blend_toward_curl(state: &mut ToeCurlState, target: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    let target = target.clamp(0.0, 1.0);
    state.big_toe += (target - state.big_toe) * t;
    state.long_toe += (target - state.long_toe) * t;
    state.middle_toe += (target - state.middle_toe) * t;
    state.ring_toe += (target - state.ring_toe) * t;
    state.little_toe += (target - state.little_toe) * t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_neutral() {
        /* default state should be neutral */
        let s = ToeCurlState::default();
        assert!(tc_is_neutral(&s));
    }

    #[test]
    fn test_set_all_curl() {
        /* set_all_curl should affect every toe */
        let mut s = ToeCurlState::default();
        tc_set_all_curl(&mut s, 0.8);
        assert!((s.big_toe - 0.8).abs() < 1e-6);
        assert!((s.little_toe - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_mean_curl() {
        /* mean should average all toes */
        let mut s = ToeCurlState::default();
        tc_set_all_curl(&mut s, 0.6);
        assert!((tc_mean_curl(&s) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_spread() {
        /* spread should be set independently */
        let mut s = ToeCurlState::default();
        tc_set_spread(&mut s, 0.5);
        assert!((s.spread - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_not_neutral_when_curled() {
        /* curled toes should not be neutral */
        let mut s = ToeCurlState::default();
        tc_set_all_curl(&mut s, 0.5);
        assert!(!tc_is_neutral(&s));
    }

    #[test]
    fn test_reset() {
        /* reset should restore neutral */
        let mut s = ToeCurlState::default();
        tc_set_all_curl(&mut s, 1.0);
        tc_reset(&mut s);
        assert!(tc_is_neutral(&s));
    }

    #[test]
    fn test_big_toe_independent() {
        /* set_big_toe should not affect other toes */
        let mut s = ToeCurlState::default();
        tc_set_big_toe(&mut s, 0.7);
        assert!((s.big_toe - 0.7).abs() < 1e-6);
        assert_eq!(s.long_toe, 0.0);
    }

    #[test]
    fn test_blend_toward() {
        /* blend should move toes toward target */
        let mut s = ToeCurlState::default();
        tc_blend_toward_curl(&mut s, 1.0, 0.5);
        assert!(tc_mean_curl(&s) > 0.0 && tc_mean_curl(&s) < 1.0);
    }

    #[test]
    fn test_clamp_above_one() {
        /* values above 1 should be clamped */
        let mut s = ToeCurlState::default();
        tc_set_all_curl(&mut s, 5.0);
        assert!((s.big_toe - 1.0).abs() < 1e-6);
    }
}
