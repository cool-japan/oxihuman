// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Head lean via neck morph control.

/// Configuration for head lean.
#[derive(Debug, Clone)]
pub struct HeadLeanConfig {
    pub max_lean_deg: f32,
    pub neck_follow: f32,
}

impl Default for HeadLeanConfig {
    fn default() -> Self {
        Self { max_lean_deg: 30.0, neck_follow: 0.6 }
    }
}

/// State for head lean control.
#[derive(Debug, Clone)]
pub struct HeadLeanState {
    pub config: HeadLeanConfig,
    pub lean_deg: f32,
}

impl Default for HeadLeanState {
    fn default() -> Self {
        Self { config: HeadLeanConfig::default(), lean_deg: 0.0 }
    }
}

/// Set the head lean angle in degrees (positive = right lean).
pub fn hl_set_lean(state: &mut HeadLeanState, deg: f32) {
    state.lean_deg = deg.clamp(-state.config.max_lean_deg, state.config.max_lean_deg);
}

/// Return the normalised lean (-1 to 1 relative to max).
pub fn hl_normalised(state: &HeadLeanState) -> f32 {
    if state.config.max_lean_deg < 1e-6 {
        return 0.0;
    }
    state.lean_deg / state.config.max_lean_deg
}

/// Return the neck follow morph weight derived from lean.
pub fn hl_neck_weight(state: &HeadLeanState) -> f32 {
    hl_normalised(state).abs() * state.config.neck_follow
}

/// Return true if the lean is at a neutral pose.
pub fn hl_is_neutral(state: &HeadLeanState) -> bool {
    state.lean_deg.abs() < 1.0
}

/// Reset to upright neutral.
pub fn hl_reset(state: &mut HeadLeanState) {
    state.lean_deg = 0.0;
}

/// Blend lean angle toward a target over t.
pub fn hl_blend_toward(state: &mut HeadLeanState, target_deg: f32, t: f32) {
    let t = t.clamp(0.0, 1.0);
    let clamped = target_deg.clamp(-state.config.max_lean_deg, state.config.max_lean_deg);
    state.lean_deg += (clamped - state.lean_deg) * t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_neutral() {
        /* default state should be at neutral */
        let s = HeadLeanState::default();
        assert!(hl_is_neutral(&s));
    }

    #[test]
    fn test_set_lean() {
        /* lean should be set within limits */
        let mut s = HeadLeanState::default();
        hl_set_lean(&mut s, 15.0);
        assert!((s.lean_deg - 15.0).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_lean() {
        /* lean beyond max should be clamped */
        let mut s = HeadLeanState::default();
        hl_set_lean(&mut s, 999.0);
        assert!((s.lean_deg - s.config.max_lean_deg).abs() < 1e-5);
    }

    #[test]
    fn test_normalised() {
        /* half max lean should normalise to 0.5 */
        let mut s = HeadLeanState::default();
        let half = s.config.max_lean_deg / 2.0;
        hl_set_lean(&mut s, half);
        assert!((hl_normalised(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_neck_weight_nonzero() {
        /* neck weight should be nonzero for non-zero lean */
        let mut s = HeadLeanState::default();
        hl_set_lean(&mut s, 15.0);
        assert!(hl_neck_weight(&s) > 0.0);
    }

    #[test]
    fn test_neck_weight_zero_neutral() {
        /* neck weight should be zero at neutral */
        let s = HeadLeanState::default();
        assert_eq!(hl_neck_weight(&s), 0.0);
    }

    #[test]
    fn test_reset() {
        /* reset should bring lean to zero */
        let mut s = HeadLeanState::default();
        hl_set_lean(&mut s, 20.0);
        hl_reset(&mut s);
        assert!(hl_is_neutral(&s));
    }

    #[test]
    fn test_blend_toward() {
        /* blend should move toward target */
        let mut s = HeadLeanState::default();
        hl_blend_toward(&mut s, 20.0, 0.5);
        assert!(s.lean_deg > 0.0 && s.lean_deg < 20.0);
    }

    #[test]
    fn test_negative_lean() {
        /* negative lean should be allowed */
        let mut s = HeadLeanState::default();
        hl_set_lean(&mut s, -15.0);
        assert!(s.lean_deg < 0.0);
    }
}
