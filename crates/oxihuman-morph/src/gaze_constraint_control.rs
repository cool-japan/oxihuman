// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gaze constraint — limits gaze direction and corrects morph weights.

/// Limits applied to the gaze cone.
#[derive(Debug, Clone)]
pub struct GazeConstraintConfig {
    pub max_yaw_deg: f32,
    pub max_pitch_deg: f32,
    pub soft_limit_ratio: f32,
}

impl Default for GazeConstraintConfig {
    fn default() -> Self {
        Self { max_yaw_deg: 35.0, max_pitch_deg: 25.0, soft_limit_ratio: 0.8 }
    }
}

/// Current constrained gaze state.
#[derive(Debug, Clone, Default)]
pub struct GazeConstraintState {
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub config: GazeConstraintConfig,
}

impl GazeConstraintState {
    /// Create a new gaze constraint state with default config.
    pub fn new() -> Self {
        Self { config: GazeConstraintConfig::default(), ..Default::default() }
    }
}

/// Apply gaze direction, clamping to configured limits.
pub fn gc_set_gaze(state: &mut GazeConstraintState, yaw: f32, pitch: f32) {
    state.yaw_deg = yaw.clamp(-state.config.max_yaw_deg, state.config.max_yaw_deg);
    state.pitch_deg = pitch.clamp(-state.config.max_pitch_deg, state.config.max_pitch_deg);
}

/// Return true if the gaze is inside the soft limit cone.
pub fn gc_is_comfortable(state: &GazeConstraintState) -> bool {
    let r = state.config.soft_limit_ratio;
    state.yaw_deg.abs() <= state.config.max_yaw_deg * r
        && state.pitch_deg.abs() <= state.config.max_pitch_deg * r
}

/// Return normalised yaw in [-1, 1] relative to max.
pub fn gc_normalised_yaw(state: &GazeConstraintState) -> f32 {
    if state.config.max_yaw_deg < 1e-6 {
        return 0.0;
    }
    state.yaw_deg / state.config.max_yaw_deg
}

/// Return normalised pitch in [-1, 1] relative to max.
pub fn gc_normalised_pitch(state: &GazeConstraintState) -> f32 {
    if state.config.max_pitch_deg < 1e-6 {
        return 0.0;
    }
    state.pitch_deg / state.config.max_pitch_deg
}

/// Reset gaze to neutral.
pub fn gc_reset(state: &mut GazeConstraintState) {
    state.yaw_deg = 0.0;
    state.pitch_deg = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_neutral() {
        /* default state should be neutral */
        let s = GazeConstraintState::new();
        assert_eq!(s.yaw_deg, 0.0);
        assert_eq!(s.pitch_deg, 0.0);
    }

    #[test]
    fn test_set_gaze_clamps() {
        /* gaze beyond limit should be clamped */
        let mut s = GazeConstraintState::new();
        gc_set_gaze(&mut s, 999.0, -999.0);
        assert!((s.yaw_deg - s.config.max_yaw_deg).abs() < 1e-6);
        assert!((s.pitch_deg + s.config.max_pitch_deg).abs() < 1e-6);
    }

    #[test]
    fn test_comfortable() {
        /* small gaze should be comfortable */
        let mut s = GazeConstraintState::new();
        gc_set_gaze(&mut s, 5.0, 5.0);
        assert!(gc_is_comfortable(&s));
    }

    #[test]
    fn test_uncomfortable() {
        /* gaze at max limit should not be comfortable */
        let mut s = GazeConstraintState::new();
        let max = s.config.max_yaw_deg;
        gc_set_gaze(&mut s, max, 0.0);
        assert!(!gc_is_comfortable(&s));
    }

    #[test]
    fn test_normalised_yaw() {
        /* half-max yaw should normalise to 0.5 */
        let mut s = GazeConstraintState::new();
        let half = s.config.max_yaw_deg / 2.0;
        gc_set_gaze(&mut s, half, 0.0);
        assert!((gc_normalised_yaw(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_normalised_pitch() {
        /* half-max pitch should normalise to -0.5 */
        let mut s = GazeConstraintState::new();
        let half = s.config.max_pitch_deg / 2.0;
        gc_set_gaze(&mut s, 0.0, -half);
        assert!((gc_normalised_pitch(&s) + 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_reset() {
        /* reset should zero both angles */
        let mut s = GazeConstraintState::new();
        gc_set_gaze(&mut s, 20.0, 10.0);
        gc_reset(&mut s);
        assert_eq!(s.yaw_deg, 0.0);
        assert_eq!(s.pitch_deg, 0.0);
    }

    #[test]
    fn test_default_config_limits() {
        /* config limits should be positive */
        let cfg = GazeConstraintConfig::default();
        assert!(cfg.max_yaw_deg > 0.0);
        assert!(cfg.max_pitch_deg > 0.0);
    }

    #[test]
    fn test_soft_limit_ratio_range() {
        /* soft limit ratio should be within (0, 1) */
        let cfg = GazeConstraintConfig::default();
        assert!((0.0..=1.0).contains(&cfg.soft_limit_ratio));
    }
}
