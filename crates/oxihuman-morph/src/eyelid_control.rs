// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Eyelid shape morphs: ptosis, squint, upper/lower lid control.
//!
//! Provides per-side eyelid state management with smooth interpolation and
//! morph-target weight extraction for real-time facial animation.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Which side of the face an eyelid operation applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyelidSide {
    Left,
    Right,
    Both,
}

/// Per-eye eyelid animation state.
#[derive(Debug, Clone)]
pub struct EyelidState {
    /// Upper lid open fraction [0 = closed, 1 = wide open].
    pub upper_open_left: f32,
    /// Upper lid open fraction right eye.
    pub upper_open_right: f32,
    /// Lower lid raise fraction [0 = neutral, 1 = fully raised].
    pub lower_raise_left: f32,
    /// Lower lid raise fraction right eye.
    pub lower_raise_right: f32,
    /// Ptosis (drooping) intensity [0 = none, 1 = full droop].
    pub ptosis_left: f32,
    /// Ptosis right eye.
    pub ptosis_right: f32,
    /// Squint intensity [0 = none, 1 = fully squinted].
    pub squint_left: f32,
    /// Squint right eye.
    pub squint_right: f32,
    /// Fatigue effect [0 = awake, 1 = heavily fatigued].
    pub fatigue: f32,
    /// Smooth-interpolation target for upper_open_left.
    pub target_upper_open_left: f32,
    /// Smooth-interpolation target for upper_open_right.
    pub target_upper_open_right: f32,
}

/// Configuration parameters for eyelid animation.
#[derive(Debug, Clone)]
pub struct EyelidConfig {
    /// Smoothing speed for lid transitions (units per second).
    pub smooth_speed: f32,
    /// Maximum ptosis from fatigue (added on top of explicit ptosis).
    pub max_fatigue_ptosis: f32,
    /// Fatigue reduces upper-open by this factor at fatigue=1.
    pub fatigue_open_scale: f32,
    /// Minimum aperture enforced even at full ptosis.
    pub min_aperture: f32,
}

// ---------------------------------------------------------------------------
// Construction helpers
// ---------------------------------------------------------------------------

/// Return a sensible default `EyelidConfig`.
pub fn default_eyelid_config() -> EyelidConfig {
    EyelidConfig {
        smooth_speed: 8.0,
        max_fatigue_ptosis: 0.4,
        fatigue_open_scale: 0.5,
        min_aperture: 0.0,
    }
}

/// Create a fully-open, non-fatigued eyelid state.
pub fn new_eyelid_state() -> EyelidState {
    EyelidState {
        upper_open_left: 1.0,
        upper_open_right: 1.0,
        lower_raise_left: 0.0,
        lower_raise_right: 0.0,
        ptosis_left: 0.0,
        ptosis_right: 0.0,
        squint_left: 0.0,
        squint_right: 0.0,
        fatigue: 0.0,
        target_upper_open_left: 1.0,
        target_upper_open_right: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Setters
// ---------------------------------------------------------------------------

/// Set the upper-lid open target for the given side [0..1].
pub fn set_upper_lid(state: &mut EyelidState, side: EyelidSide, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match side {
        EyelidSide::Left => {
            state.upper_open_left = v;
            state.target_upper_open_left = v;
        }
        EyelidSide::Right => {
            state.upper_open_right = v;
            state.target_upper_open_right = v;
        }
        EyelidSide::Both => {
            state.upper_open_left = v;
            state.upper_open_right = v;
            state.target_upper_open_left = v;
            state.target_upper_open_right = v;
        }
    }
}

/// Set lower-lid raise for the given side [0..1].
pub fn set_lower_lid(state: &mut EyelidState, side: EyelidSide, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match side {
        EyelidSide::Left => state.lower_raise_left = v,
        EyelidSide::Right => state.lower_raise_right = v,
        EyelidSide::Both => {
            state.lower_raise_left = v;
            state.lower_raise_right = v;
        }
    }
}

/// Set ptosis (drooping) for the given side [0..1].
pub fn set_ptosis(state: &mut EyelidState, side: EyelidSide, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match side {
        EyelidSide::Left => state.ptosis_left = v,
        EyelidSide::Right => state.ptosis_right = v,
        EyelidSide::Both => {
            state.ptosis_left = v;
            state.ptosis_right = v;
        }
    }
}

/// Set squint for the given side [0..1].
pub fn set_squint(state: &mut EyelidState, side: EyelidSide, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match side {
        EyelidSide::Left => state.squint_left = v,
        EyelidSide::Right => state.squint_right = v,
        EyelidSide::Both => {
            state.squint_left = v;
            state.squint_right = v;
        }
    }
}

/// Apply fatigue effect: reduces upper-lid opening and adds ptosis.
///
/// `fatigue` in [0..1]: 0 = fully awake, 1 = heavily fatigued.
pub fn apply_fatigue_effect(state: &mut EyelidState, config: &EyelidConfig, fatigue: f32) {
    let f = fatigue.clamp(0.0, 1.0);
    state.fatigue = f;
    let ptosis_add = f * config.max_fatigue_ptosis;
    state.ptosis_left = (state.ptosis_left + ptosis_add).clamp(0.0, 1.0);
    state.ptosis_right = (state.ptosis_right + ptosis_add).clamp(0.0, 1.0);
    let open_factor = 1.0 - f * (1.0 - config.fatigue_open_scale);
    state.upper_open_left = (state.upper_open_left * open_factor).clamp(0.0, 1.0);
    state.upper_open_right = (state.upper_open_right * open_factor).clamp(0.0, 1.0);
    state.target_upper_open_left = state.upper_open_left;
    state.target_upper_open_right = state.upper_open_right;
}

// ---------------------------------------------------------------------------
// Update / queries
// ---------------------------------------------------------------------------

/// Smoothly interpolate lid positions toward their targets.
///
/// Call once per frame with `dt` seconds elapsed.
pub fn update_eyelids(state: &mut EyelidState, config: &EyelidConfig, dt: f32) {
    let speed = config.smooth_speed * dt;
    state.upper_open_left = lerp_toward(state.upper_open_left, state.target_upper_open_left, speed);
    state.upper_open_right =
        lerp_toward(state.upper_open_right, state.target_upper_open_right, speed);
}

/// Effective aperture of the left eye, accounting for ptosis and squint.
pub fn eyelid_open_amount_left(state: &EyelidState, config: &EyelidConfig) -> f32 {
    let base = state.upper_open_left * (1.0 - state.ptosis_left * 0.8);
    let squinted = base * (1.0 - state.squint_left * 0.6);
    squinted.clamp(config.min_aperture, 1.0)
}

/// Effective aperture of the right eye.
pub fn eyelid_open_amount_right(state: &EyelidState, config: &EyelidConfig) -> f32 {
    let base = state.upper_open_right * (1.0 - state.ptosis_right * 0.8);
    let squinted = base * (1.0 - state.squint_right * 0.6);
    squinted.clamp(config.min_aperture, 1.0)
}

/// Average aperture across both eyes.
pub fn eyelid_aperture(state: &EyelidState, config: &EyelidConfig) -> f32 {
    (eyelid_open_amount_left(state, config) + eyelid_open_amount_right(state, config)) * 0.5
}

/// Linearly blend two eyelid states by `t` [0..1].
pub fn blend_eyelid_states(a: &EyelidState, b: &EyelidState, t: f32) -> EyelidState {
    let t = t.clamp(0.0, 1.0);
    EyelidState {
        upper_open_left: lerp(a.upper_open_left, b.upper_open_left, t),
        upper_open_right: lerp(a.upper_open_right, b.upper_open_right, t),
        lower_raise_left: lerp(a.lower_raise_left, b.lower_raise_left, t),
        lower_raise_right: lerp(a.lower_raise_right, b.lower_raise_right, t),
        ptosis_left: lerp(a.ptosis_left, b.ptosis_left, t),
        ptosis_right: lerp(a.ptosis_right, b.ptosis_right, t),
        squint_left: lerp(a.squint_left, b.squint_left, t),
        squint_right: lerp(a.squint_right, b.squint_right, t),
        fatigue: lerp(a.fatigue, b.fatigue, t),
        target_upper_open_left: lerp(a.target_upper_open_left, b.target_upper_open_left, t),
        target_upper_open_right: lerp(a.target_upper_open_right, b.target_upper_open_right, t),
    }
}

/// Convert eyelid state to a morph-target weight map.
pub fn eyelid_to_morph_weights(state: &EyelidState, config: &EyelidConfig) -> HashMap<String, f32> {
    let mut map = HashMap::new();
    let open_l = eyelid_open_amount_left(state, config);
    let open_r = eyelid_open_amount_right(state, config);
    // Closed = 1 − open
    map.insert("eyelid_close_L".to_string(), (1.0 - open_l).clamp(0.0, 1.0));
    map.insert("eyelid_close_R".to_string(), (1.0 - open_r).clamp(0.0, 1.0));
    map.insert("eyelid_lower_raise_L".to_string(), state.lower_raise_left);
    map.insert("eyelid_lower_raise_R".to_string(), state.lower_raise_right);
    map.insert("eyelid_ptosis_L".to_string(), state.ptosis_left);
    map.insert("eyelid_ptosis_R".to_string(), state.ptosis_right);
    map.insert("eyelid_squint_L".to_string(), state.squint_left);
    map.insert("eyelid_squint_R".to_string(), state.squint_right);
    map
}

/// Reset eyelid state to fully open, no ptosis, no squint.
pub fn reset_eyelids(state: &mut EyelidState) {
    *state = new_eyelid_state();
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Move `current` toward `target` by at most `speed`, return result.
#[inline]
fn lerp_toward(current: f32, target: f32, speed: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= speed {
        target
    } else {
        current + diff.signum() * speed
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> EyelidConfig {
        default_eyelid_config()
    }

    // 1. new_eyelid_state returns fully open eyes
    #[test]
    fn new_state_is_open() {
        let s = new_eyelid_state();
        assert_eq!(s.upper_open_left, 1.0);
        assert_eq!(s.upper_open_right, 1.0);
    }

    // 2. set_upper_lid clamps to [0, 1]
    #[test]
    fn set_upper_lid_clamps() {
        let mut s = new_eyelid_state();
        set_upper_lid(&mut s, EyelidSide::Left, 2.5);
        assert_eq!(s.upper_open_left, 1.0);
        set_upper_lid(&mut s, EyelidSide::Right, -0.5);
        assert_eq!(s.upper_open_right, 0.0);
    }

    // 3. set_upper_lid Both sets both eyes
    #[test]
    fn set_upper_lid_both() {
        let mut s = new_eyelid_state();
        set_upper_lid(&mut s, EyelidSide::Both, 0.5);
        assert!((s.upper_open_left - 0.5).abs() < 1e-6);
        assert!((s.upper_open_right - 0.5).abs() < 1e-6);
    }

    // 4. set_lower_lid works
    #[test]
    fn set_lower_lid_works() {
        let mut s = new_eyelid_state();
        set_lower_lid(&mut s, EyelidSide::Left, 0.7);
        assert!((s.lower_raise_left - 0.7).abs() < 1e-6);
        assert_eq!(s.lower_raise_right, 0.0);
    }

    // 5. set_ptosis applies per-side
    #[test]
    fn set_ptosis_per_side() {
        let mut s = new_eyelid_state();
        set_ptosis(&mut s, EyelidSide::Left, 0.3);
        assert!((s.ptosis_left - 0.3).abs() < 1e-6);
        assert_eq!(s.ptosis_right, 0.0);
    }

    // 6. set_squint Both
    #[test]
    fn set_squint_both_sides() {
        let mut s = new_eyelid_state();
        set_squint(&mut s, EyelidSide::Both, 0.6);
        assert!((s.squint_left - 0.6).abs() < 1e-6);
        assert!((s.squint_right - 0.6).abs() < 1e-6);
    }

    // 7. eyelid_open_amount_left reduces with ptosis
    #[test]
    fn ptosis_reduces_open_amount() {
        let mut s = new_eyelid_state();
        let open_before = eyelid_open_amount_left(&s, &cfg());
        set_ptosis(&mut s, EyelidSide::Left, 0.5);
        let open_after = eyelid_open_amount_left(&s, &cfg());
        assert!(open_after < open_before);
    }

    // 8. eyelid_open_amount_right reduces with squint
    #[test]
    fn squint_reduces_right_open() {
        let mut s = new_eyelid_state();
        let before = eyelid_open_amount_right(&s, &cfg());
        set_squint(&mut s, EyelidSide::Right, 1.0);
        let after = eyelid_open_amount_right(&s, &cfg());
        assert!(after < before);
    }

    // 9. eyelid_aperture is average of both eyes
    #[test]
    fn aperture_is_average() {
        let mut s = new_eyelid_state();
        set_upper_lid(&mut s, EyelidSide::Left, 0.4);
        set_upper_lid(&mut s, EyelidSide::Right, 0.8);
        let aperture = eyelid_aperture(&s, &cfg());
        assert!(aperture > 0.0 && aperture < 1.0);
    }

    // 10. blend_eyelid_states at t=0 gives a
    #[test]
    fn blend_at_t0_gives_a() {
        let a = new_eyelid_state();
        let mut b = new_eyelid_state();
        set_upper_lid(&mut b, EyelidSide::Both, 0.0);
        let result = blend_eyelid_states(&a, &b, 0.0);
        assert!((result.upper_open_left - 1.0).abs() < 1e-6);
    }

    // 11. blend_eyelid_states at t=1 gives b
    #[test]
    fn blend_at_t1_gives_b() {
        let a = new_eyelid_state();
        let mut b = new_eyelid_state();
        set_upper_lid(&mut b, EyelidSide::Both, 0.2);
        let result = blend_eyelid_states(&a, &b, 1.0);
        assert!((result.upper_open_left - 0.2).abs() < 1e-6);
    }

    // 12. eyelid_to_morph_weights contains expected keys
    #[test]
    fn morph_weights_has_expected_keys() {
        let s = new_eyelid_state();
        let weights = eyelid_to_morph_weights(&s, &cfg());
        assert!(weights.contains_key("eyelid_close_L"));
        assert!(weights.contains_key("eyelid_close_R"));
        assert!(weights.contains_key("eyelid_ptosis_L"));
        assert!(weights.contains_key("eyelid_squint_R"));
    }

    // 13. fully open eyes → close morph weight = 0
    #[test]
    fn open_eyes_close_weight_is_zero() {
        let s = new_eyelid_state();
        let weights = eyelid_to_morph_weights(&s, &cfg());
        assert!(weights["eyelid_close_L"] < 1e-5);
        assert!(weights["eyelid_close_R"] < 1e-5);
    }

    // 14. reset_eyelids clears all state
    #[test]
    fn reset_clears_state() {
        let mut s = new_eyelid_state();
        set_upper_lid(&mut s, EyelidSide::Both, 0.0);
        set_ptosis(&mut s, EyelidSide::Both, 0.9);
        reset_eyelids(&mut s);
        assert_eq!(s.upper_open_left, 1.0);
        assert_eq!(s.ptosis_left, 0.0);
    }

    // 15. update_eyelids moves toward target
    #[test]
    fn update_eyelids_moves_toward_target() {
        let mut s = new_eyelid_state();
        s.upper_open_left = 0.0;
        s.target_upper_open_left = 1.0;
        update_eyelids(&mut s, &cfg(), 0.1);
        assert!(s.upper_open_left > 0.0);
        assert!(s.upper_open_left <= 1.0);
    }

    // 16. apply_fatigue_effect reduces openness
    #[test]
    fn fatigue_reduces_openness() {
        let mut s = new_eyelid_state();
        apply_fatigue_effect(&mut s, &cfg(), 1.0);
        assert!(s.upper_open_left < 1.0);
        assert!(s.upper_open_right < 1.0);
        assert!(s.ptosis_left > 0.0);
    }

    // 17. EyelidSide equality
    #[test]
    fn eyelid_side_equality() {
        assert_eq!(EyelidSide::Left, EyelidSide::Left);
        assert_ne!(EyelidSide::Left, EyelidSide::Right);
    }

    // 18. update_eyelids with large dt reaches target
    #[test]
    fn update_large_dt_reaches_target() {
        let mut s = new_eyelid_state();
        s.upper_open_left = 0.0;
        s.target_upper_open_left = 0.5;
        update_eyelids(&mut s, &cfg(), 100.0);
        assert!((s.upper_open_left - 0.5).abs() < 1e-5);
    }
}
