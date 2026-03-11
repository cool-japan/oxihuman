// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaw open/close driver morph control.

/// Configuration for jaw open/close morphing.
#[derive(Debug, Clone)]
pub struct JawOpenConfig {
    pub max_open: f32,
    pub open_weight: f32,
    pub lower_lip_follow: f32,
    pub upper_lip_depress: f32,
}

/// State for the jaw open control.
#[derive(Debug, Clone)]
pub struct JawOpenState {
    pub config: JawOpenConfig,
    pub current_open: f32,
}

impl Default for JawOpenConfig {
    fn default() -> Self {
        Self { max_open: 1.0, open_weight: 1.0, lower_lip_follow: 0.6, upper_lip_depress: 0.1 }
    }
}

impl JawOpenState {
    /// Create a new jaw open state with default config.
    pub fn new() -> Self {
        Self { config: JawOpenConfig::default(), current_open: 0.0 }
    }
}

impl Default for JawOpenState {
    fn default() -> Self {
        Self::new()
    }
}

/// Set the jaw open amount (0.0 = closed, 1.0 = fully open).
pub fn jaw_set_open(state: &mut JawOpenState, amount: f32) {
    state.current_open = amount.clamp(0.0, state.config.max_open);
}

/// Get the current jaw open amount.
pub fn jaw_get_open(state: &JawOpenState) -> f32 {
    state.current_open
}

/// Return the derived lower-lip follow weight.
pub fn jaw_lower_lip_weight(state: &JawOpenState) -> f32 {
    state.current_open * state.config.lower_lip_follow
}

/// Return the derived upper-lip depression weight.
pub fn jaw_upper_lip_weight(state: &JawOpenState) -> f32 {
    state.current_open * state.config.upper_lip_depress
}

/// Close the jaw completely.
pub fn jaw_close(state: &mut JawOpenState) {
    state.current_open = 0.0;
}

/// Blend from current open to a target open amount over t in [0,1].
pub fn jaw_blend_toward(state: &mut JawOpenState, target: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    let target = target.clamp(0.0, state.config.max_open);
    state.current_open += (target - state.current_open) * t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        /* jaw should start closed */
        let s = JawOpenState::new();
        assert_eq!(s.current_open, 0.0);
    }

    #[test]
    fn test_set_open() {
        /* set open should clamp to max */
        let mut s = JawOpenState::new();
        jaw_set_open(&mut s, 0.5);
        assert!((jaw_get_open(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_above_max() {
        /* values above max_open should be clamped */
        let mut s = JawOpenState::new();
        jaw_set_open(&mut s, 9.9);
        assert!((jaw_get_open(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_close() {
        /* close should set current_open to zero */
        let mut s = JawOpenState::new();
        jaw_set_open(&mut s, 0.8);
        jaw_close(&mut s);
        assert_eq!(s.current_open, 0.0);
    }

    #[test]
    fn test_lower_lip_weight() {
        /* lower lip weight follows open * follow factor */
        let mut s = JawOpenState::new();
        jaw_set_open(&mut s, 1.0);
        let expected = s.config.lower_lip_follow;
        assert!((jaw_lower_lip_weight(&s) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_upper_lip_weight() {
        /* upper lip weight follows open * depress factor */
        let mut s = JawOpenState::new();
        jaw_set_open(&mut s, 1.0);
        let expected = s.config.upper_lip_depress;
        assert!((jaw_upper_lip_weight(&s) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_blend_toward() {
        /* blend should move current toward target */
        let mut s = JawOpenState::new();
        jaw_blend_toward(&mut s, 1.0, 0.5);
        assert!(s.current_open > 0.0 && s.current_open < 1.0);
    }

    #[test]
    fn test_blend_full() {
        /* blend at t=1 should reach target exactly */
        let mut s = JawOpenState::new();
        jaw_blend_toward(&mut s, 0.8, 1.0);
        assert!((s.current_open - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_default_config() {
        /* default config should have sensible values */
        let cfg = JawOpenConfig::default();
        assert!(cfg.max_open > 0.0);
        assert!((0.0..=1.0).contains(&cfg.lower_lip_follow));
    }
}
