// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Nostril flare and nose bridge morph control.

/// Which side of the nose to target.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoseSide {
    Left,
    Right,
    Both,
}

/// Configuration for nose morph behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoseConfig {
    /// Maximum nostril flare delta (0..1 maps to 0..max_flare).
    pub max_flare: f32,
    /// Speed multiplier for smooth transition updates.
    pub transition_speed: f32,
    /// Maximum nose wrinkle intensity.
    pub max_wrinkle: f32,
    /// Maximum bridge width adjustment (symmetric around 0).
    pub max_bridge_delta: f32,
}

/// Live state of the nose morphs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoseState {
    pub flare_left: f32,
    pub flare_right: f32,
    pub wrinkle: f32,
    pub bridge_width: f32,
    pub config: NoseConfig,
}

/// Morph weight output produced by [`nose_to_morph_weights`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoseMorphWeights {
    pub nostril_flare_l: f32,
    pub nostril_flare_r: f32,
    pub nose_wrinkle: f32,
    pub bridge_narrow: f32,
    pub bridge_wide: f32,
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Return a sensible default [`NoseConfig`].
#[allow(dead_code)]
pub fn default_nose_config() -> NoseConfig {
    NoseConfig {
        max_flare: 1.0,
        transition_speed: 5.0,
        max_wrinkle: 1.0,
        max_bridge_delta: 0.5,
    }
}

/// Create a zeroed-out [`NoseState`] using the given config.
#[allow(dead_code)]
pub fn new_nose_state(config: NoseConfig) -> NoseState {
    NoseState {
        flare_left: 0.0,
        flare_right: 0.0,
        wrinkle: 0.0,
        bridge_width: 0.0,
        config,
    }
}

/// Set nostril flare for the specified side(s). Values clamped to 0..1.
#[allow(dead_code)]
pub fn set_nostril_flare(state: &mut NoseState, side: NoseSide, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match side {
        NoseSide::Left => state.flare_left = v,
        NoseSide::Right => state.flare_right = v,
        NoseSide::Both => {
            state.flare_left = v;
            state.flare_right = v;
        }
    }
}

/// Set nose wrinkle intensity (0..1).
#[allow(dead_code)]
pub fn set_nose_wrinkle(state: &mut NoseState, value: f32) {
    state.wrinkle = value.clamp(0.0, 1.0);
}

/// Set nose bridge width offset (-1..1, negative = narrower, positive = wider).
#[allow(dead_code)]
pub fn set_nose_bridge_width(state: &mut NoseState, value: f32) {
    state.bridge_width = value.clamp(-1.0, 1.0);
}

/// Smoothly advance `current` towards `target` by `dt` using the config speed.
#[allow(dead_code)]
pub fn update_nose(current: &mut NoseState, target: &NoseState, dt: f32) {
    let speed = current.config.transition_speed;
    let alpha = (speed * dt).clamp(0.0, 1.0);
    current.flare_left += (target.flare_left - current.flare_left) * alpha;
    current.flare_right += (target.flare_right - current.flare_right) * alpha;
    current.wrinkle += (target.wrinkle - current.wrinkle) * alpha;
    current.bridge_width += (target.bridge_width - current.bridge_width) * alpha;
}

/// Return the left nostril flare value.
#[allow(dead_code)]
pub fn nostril_flare_left(state: &NoseState) -> f32 {
    state.flare_left
}

/// Return the right nostril flare value.
#[allow(dead_code)]
pub fn nostril_flare_right(state: &NoseState) -> f32 {
    state.flare_right
}

/// Return the nose wrinkle amount.
#[allow(dead_code)]
pub fn nose_wrinkle_amount(state: &NoseState) -> f32 {
    state.wrinkle
}

/// Return the bridge width offset.
#[allow(dead_code)]
pub fn nose_bridge_width(state: &NoseState) -> f32 {
    state.bridge_width
}

/// Linearly interpolate between two nose states by `t` (0..1).
#[allow(dead_code)]
pub fn blend_nose_states(a: &NoseState, b: &NoseState, t: f32) -> NoseState {
    let t = t.clamp(0.0, 1.0);
    NoseState {
        flare_left: a.flare_left + (b.flare_left - a.flare_left) * t,
        flare_right: a.flare_right + (b.flare_right - a.flare_right) * t,
        wrinkle: a.wrinkle + (b.wrinkle - a.wrinkle) * t,
        bridge_width: a.bridge_width + (b.bridge_width - a.bridge_width) * t,
        config: a.config.clone(),
    }
}

/// Convert a [`NoseState`] to a [`NoseMorphWeights`] struct.
#[allow(dead_code)]
pub fn nose_to_morph_weights(state: &NoseState) -> NoseMorphWeights {
    let bridge_wide = state.bridge_width.max(0.0);
    let bridge_narrow = (-state.bridge_width).max(0.0);
    NoseMorphWeights {
        nostril_flare_l: state.flare_left * state.config.max_flare,
        nostril_flare_r: state.flare_right * state.config.max_flare,
        nose_wrinkle: state.wrinkle * state.config.max_wrinkle,
        bridge_narrow: bridge_narrow * state.config.max_bridge_delta,
        bridge_wide: bridge_wide * state.config.max_bridge_delta,
    }
}

/// Reset all nose morph values to zero.
#[allow(dead_code)]
pub fn reset_nose(state: &mut NoseState) {
    state.flare_left = 0.0;
    state.flare_right = 0.0;
    state.wrinkle = 0.0;
    state.bridge_width = 0.0;
}

/// Apply a sniff effect: brief bilateral flare + slight wrinkle.
/// `intensity` in 0..1.
#[allow(dead_code)]
pub fn apply_sniff_effect(state: &mut NoseState, intensity: f32) {
    let i = intensity.clamp(0.0, 1.0);
    state.flare_left = (state.flare_left + i * 0.6).clamp(0.0, 1.0);
    state.flare_right = (state.flare_right + i * 0.6).clamp(0.0, 1.0);
    state.wrinkle = (state.wrinkle + i * 0.3).clamp(0.0, 1.0);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn base_state() -> NoseState {
        new_nose_state(default_nose_config())
    }

    #[test]
    fn test_default_config_values() {
        let cfg = default_nose_config();
        assert!(cfg.max_flare > 0.0);
        assert!(cfg.transition_speed > 0.0);
    }

    #[test]
    fn test_new_nose_state_zeroed() {
        let s = base_state();
        assert_eq!(s.flare_left, 0.0);
        assert_eq!(s.flare_right, 0.0);
        assert_eq!(s.wrinkle, 0.0);
        assert_eq!(s.bridge_width, 0.0);
    }

    #[test]
    fn test_set_nostril_flare_left() {
        let mut s = base_state();
        set_nostril_flare(&mut s, NoseSide::Left, 0.7);
        assert!((s.flare_left - 0.7).abs() < 1e-6);
        assert_eq!(s.flare_right, 0.0);
    }

    #[test]
    fn test_set_nostril_flare_right() {
        let mut s = base_state();
        set_nostril_flare(&mut s, NoseSide::Right, 0.5);
        assert!((s.flare_right - 0.5).abs() < 1e-6);
        assert_eq!(s.flare_left, 0.0);
    }

    #[test]
    fn test_set_nostril_flare_both() {
        let mut s = base_state();
        set_nostril_flare(&mut s, NoseSide::Both, 0.9);
        assert!((s.flare_left - 0.9).abs() < 1e-6);
        assert!((s.flare_right - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_nostril_flare_clamps() {
        let mut s = base_state();
        set_nostril_flare(&mut s, NoseSide::Both, 2.0);
        assert_eq!(s.flare_left, 1.0);
        set_nostril_flare(&mut s, NoseSide::Both, -1.0);
        assert_eq!(s.flare_left, 0.0);
    }

    #[test]
    fn test_set_nose_wrinkle() {
        let mut s = base_state();
        set_nose_wrinkle(&mut s, 0.4);
        assert!((nose_wrinkle_amount(&s) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_nose_bridge_width() {
        let mut s = base_state();
        set_nose_bridge_width(&mut s, 0.3);
        assert!((nose_bridge_width(&s) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_update_nose_converges() {
        let mut current = base_state();
        let mut target = base_state();
        target.flare_left = 1.0;
        for _ in 0..200 {
            update_nose(&mut current, &target, 0.1);
        }
        assert!(current.flare_left > 0.99);
    }

    #[test]
    fn test_nostril_flare_accessors() {
        let mut s = base_state();
        set_nostril_flare(&mut s, NoseSide::Left, 0.2);
        set_nostril_flare(&mut s, NoseSide::Right, 0.8);
        assert!((nostril_flare_left(&s) - 0.2).abs() < 1e-6);
        assert!((nostril_flare_right(&s) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_blend_nose_states_midpoint() {
        let mut a = base_state();
        let mut b = base_state();
        a.flare_left = 0.0;
        b.flare_left = 1.0;
        let mid = blend_nose_states(&a, &b, 0.5);
        assert!((mid.flare_left - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_nose_states_at_zero() {
        let a = base_state();
        let b = base_state();
        let result = blend_nose_states(&a, &b, 0.0);
        assert_eq!(result.flare_left, a.flare_left);
    }

    #[test]
    fn test_nose_to_morph_weights_bridge_wide() {
        let mut s = base_state();
        set_nose_bridge_width(&mut s, 1.0);
        let w = nose_to_morph_weights(&s);
        assert!(w.bridge_wide > 0.0);
        assert_eq!(w.bridge_narrow, 0.0);
    }

    #[test]
    fn test_nose_to_morph_weights_bridge_narrow() {
        let mut s = base_state();
        set_nose_bridge_width(&mut s, -1.0);
        let w = nose_to_morph_weights(&s);
        assert!(w.bridge_narrow > 0.0);
        assert_eq!(w.bridge_wide, 0.0);
    }

    #[test]
    fn test_reset_nose() {
        let mut s = base_state();
        set_nostril_flare(&mut s, NoseSide::Both, 0.9);
        set_nose_wrinkle(&mut s, 0.8);
        reset_nose(&mut s);
        assert_eq!(s.flare_left, 0.0);
        assert_eq!(s.flare_right, 0.0);
        assert_eq!(s.wrinkle, 0.0);
    }

    #[test]
    fn test_apply_sniff_effect() {
        let mut s = base_state();
        apply_sniff_effect(&mut s, 1.0);
        assert!(s.flare_left > 0.0);
        assert!(s.flare_right > 0.0);
        assert!(s.wrinkle > 0.0);
    }

    #[test]
    fn test_apply_sniff_effect_clamps() {
        let mut s = base_state();
        s.flare_left = 0.9;
        apply_sniff_effect(&mut s, 1.0);
        assert!(s.flare_left <= 1.0);
    }

    #[test]
    fn test_nose_side_debug() {
        let side = NoseSide::Both;
        let dbg = format!("{side:?}");
        assert!(dbg.contains("Both"));
    }
}
