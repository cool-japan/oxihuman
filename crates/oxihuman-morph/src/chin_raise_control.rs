// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chin raise and dimple morph control (FACS AU17 area).

/// State for chin raise control.
#[derive(Debug, Clone, Default)]
pub struct ChinRaiseState {
    pub raise: f32,
    pub dimple: f32,
    pub lip_chin_couple: f32,
}

/// Set the chin raise amount.
pub fn cr_set_raise(state: &mut ChinRaiseState, amount: f32) {
    state.raise = amount.clamp(0.0, 1.0);
}

/// Set the chin dimple intensity.
pub fn cr_set_dimple(state: &mut ChinRaiseState, amount: f32) {
    state.dimple = amount.clamp(0.0, 1.0);
}

/// Return the lower-lip push weight driven by chin raise.
pub fn cr_lower_lip_push(state: &ChinRaiseState) -> f32 {
    state.raise * state.lip_chin_couple.max(0.25)
}

/// Return true if the chin is near neutral.
pub fn cr_is_neutral(state: &ChinRaiseState) -> bool {
    state.raise < 0.05 && state.dimple < 0.05
}

/// Reset chin to neutral.
pub fn cr_reset(state: &mut ChinRaiseState) {
    state.raise = 0.0;
    state.dimple = 0.0;
}

/// Blend chin raise toward a target.
pub fn cr_blend_toward(state: &mut ChinRaiseState, target_raise: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    let target = target_raise.clamp(0.0, 1.0);
    state.raise += (target - state.raise) * t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_neutral() {
        /* default state should be neutral */
        let s = ChinRaiseState::default();
        assert!(cr_is_neutral(&s));
    }

    #[test]
    fn test_set_raise() {
        /* raise should be set correctly */
        let mut s = ChinRaiseState::default();
        cr_set_raise(&mut s, 0.7);
        assert!((s.raise - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_dimple() {
        /* dimple should be set correctly */
        let mut s = ChinRaiseState::default();
        cr_set_dimple(&mut s, 0.4);
        assert!((s.dimple - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_lower_lip_push_nonzero() {
        /* lower lip push should be nonzero when raised */
        let mut s = ChinRaiseState::default();
        cr_set_raise(&mut s, 1.0);
        assert!(cr_lower_lip_push(&s) > 0.0);
    }

    #[test]
    fn test_not_neutral_when_raised() {
        /* raised chin should not be neutral */
        let mut s = ChinRaiseState::default();
        cr_set_raise(&mut s, 0.5);
        assert!(!cr_is_neutral(&s));
    }

    #[test]
    fn test_reset() {
        /* reset should restore neutral */
        let mut s = ChinRaiseState::default();
        cr_set_raise(&mut s, 0.8);
        cr_set_dimple(&mut s, 0.5);
        cr_reset(&mut s);
        assert!(cr_is_neutral(&s));
    }

    #[test]
    fn test_blend_toward() {
        /* blend should move toward target */
        let mut s = ChinRaiseState::default();
        cr_blend_toward(&mut s, 1.0, 0.5);
        assert!(s.raise > 0.0 && s.raise < 1.0);
    }

    #[test]
    fn test_clamp_above_one() {
        /* raise above 1.0 should be clamped */
        let mut s = ChinRaiseState::default();
        cr_set_raise(&mut s, 9.9);
        assert!((s.raise - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lower_lip_zero_when_relaxed() {
        /* lower lip push should be zero at rest */
        let s = ChinRaiseState::default();
        assert_eq!(cr_lower_lip_push(&s), 0.0);
    }
}
