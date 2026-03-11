// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lip pucker / kiss morph control.

/// Configuration for lip pucker.
#[derive(Debug, Clone)]
pub struct LipPuckerConfig {
    pub max_pucker: f32,
    pub lip_round_factor: f32,
    pub protrusion_factor: f32,
}

impl Default for LipPuckerConfig {
    fn default() -> Self {
        Self { max_pucker: 1.0, lip_round_factor: 0.8, protrusion_factor: 0.5 }
    }
}

/// State for the lip pucker control.
#[derive(Debug, Clone)]
pub struct LipPuckerState {
    pub config: LipPuckerConfig,
    pub pucker: f32,
}

impl Default for LipPuckerState {
    fn default() -> Self {
        Self { config: LipPuckerConfig::default(), pucker: 0.0 }
    }
}

/// Set the pucker amount (0=neutral, 1=full pucker).
pub fn lp_set_pucker(state: &mut LipPuckerState, amount: f32) {
    state.pucker = amount.clamp(0.0, state.config.max_pucker);
}

/// Get the current pucker amount.
pub fn lp_get_pucker(state: &LipPuckerState) -> f32 {
    state.pucker
}

/// Return the derived lip rounding weight.
pub fn lp_lip_round_weight(state: &LipPuckerState) -> f32 {
    state.pucker * state.config.lip_round_factor
}

/// Return the derived lip protrusion weight.
pub fn lp_protrusion_weight(state: &LipPuckerState) -> f32 {
    state.pucker * state.config.protrusion_factor
}

/// Blend the pucker toward a target at rate t.
pub fn lp_blend_toward(state: &mut LipPuckerState, target: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    let target = target.clamp(0.0, state.config.max_pucker);
    state.pucker += (target - state.pucker) * t;
}

/// Reset to neutral.
pub fn lp_reset(state: &mut LipPuckerState) {
    state.pucker = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_neutral() {
        /* default state should have zero pucker */
        let s = LipPuckerState::default();
        assert_eq!(s.pucker, 0.0);
    }

    #[test]
    fn test_set_get() {
        /* set and get should round-trip */
        let mut s = LipPuckerState::default();
        lp_set_pucker(&mut s, 0.6);
        assert!((lp_get_pucker(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_max() {
        /* values above max should be clamped */
        let mut s = LipPuckerState::default();
        lp_set_pucker(&mut s, 5.0);
        assert!((lp_get_pucker(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lip_round_weight() {
        /* lip round weight should scale with pucker */
        let mut s = LipPuckerState::default();
        lp_set_pucker(&mut s, 1.0);
        let expected = s.config.lip_round_factor;
        assert!((lp_lip_round_weight(&s) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_protrusion_weight() {
        /* protrusion weight should scale with pucker */
        let mut s = LipPuckerState::default();
        lp_set_pucker(&mut s, 1.0);
        let expected = s.config.protrusion_factor;
        assert!((lp_protrusion_weight(&s) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_blend_toward() {
        /* blend should move toward target */
        let mut s = LipPuckerState::default();
        lp_blend_toward(&mut s, 1.0, 0.5);
        assert!(s.pucker > 0.0 && s.pucker < 1.0);
    }

    #[test]
    fn test_reset() {
        /* reset should zero pucker */
        let mut s = LipPuckerState::default();
        lp_set_pucker(&mut s, 0.9);
        lp_reset(&mut s);
        assert_eq!(s.pucker, 0.0);
    }

    #[test]
    fn test_config_defaults_valid() {
        /* config values should be in valid range */
        let cfg = LipPuckerConfig::default();
        assert!((0.0..=1.0).contains(&cfg.lip_round_factor));
        assert!((0.0..=1.0).contains(&cfg.protrusion_factor));
    }

    #[test]
    fn test_blend_full() {
        /* blend at t=1 should reach target */
        let mut s = LipPuckerState::default();
        lp_blend_toward(&mut s, 0.75, 1.0);
        assert!((s.pucker - 0.75).abs() < 1e-6);
    }
}
