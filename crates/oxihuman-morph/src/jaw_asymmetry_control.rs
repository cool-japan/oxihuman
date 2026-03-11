// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Jaw asymmetry morph (lateral shift/rotation).

#![allow(dead_code)]

/// Configuration for jaw asymmetry morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawAsymConfig {
    pub max_shift: f32,
    pub max_rotation: f32,
}

/// Runtime state for jaw asymmetry morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawAsymState {
    pub lateral_shift: f32,
    pub rotation: f32,
    pub vertical_delta: f32,
}

#[allow(dead_code)]
pub fn default_jaw_asym_config() -> JawAsymConfig {
    JawAsymConfig {
        max_shift: 1.0,
        max_rotation: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_jaw_asym_state() -> JawAsymState {
    JawAsymState {
        lateral_shift: 0.0,
        rotation: 0.0,
        vertical_delta: 0.0,
    }
}

#[allow(dead_code)]
pub fn ja_set_lateral_shift(state: &mut JawAsymState, cfg: &JawAsymConfig, v: f32) {
    state.lateral_shift = v.clamp(-cfg.max_shift, cfg.max_shift);
}

#[allow(dead_code)]
pub fn ja_set_rotation(state: &mut JawAsymState, cfg: &JawAsymConfig, v: f32) {
    state.rotation = v.clamp(-cfg.max_rotation, cfg.max_rotation);
}

#[allow(dead_code)]
pub fn ja_set_vertical_delta(state: &mut JawAsymState, v: f32) {
    state.vertical_delta = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ja_reset(state: &mut JawAsymState) {
    *state = new_jaw_asym_state();
}

#[allow(dead_code)]
pub fn ja_to_weights(state: &JawAsymState) -> Vec<(String, f32)> {
    vec![
        ("jaw_asym_lateral_shift".to_string(), state.lateral_shift),
        ("jaw_asym_rotation".to_string(), state.rotation),
        ("jaw_asym_vertical_delta".to_string(), state.vertical_delta),
    ]
}

#[allow(dead_code)]
pub fn ja_to_json(state: &JawAsymState) -> String {
    format!(
        r#"{{"lateral_shift":{:.4},"rotation":{:.4},"vertical_delta":{:.4}}}"#,
        state.lateral_shift, state.rotation, state.vertical_delta
    )
}

#[allow(dead_code)]
pub fn ja_clamp(state: &mut JawAsymState, cfg: &JawAsymConfig) {
    state.lateral_shift = state.lateral_shift.clamp(-cfg.max_shift, cfg.max_shift);
    state.rotation = state.rotation.clamp(-cfg.max_rotation, cfg.max_rotation);
    state.vertical_delta = state.vertical_delta.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ja_magnitude(state: &JawAsymState) -> f32 {
    (state.lateral_shift * state.lateral_shift
        + state.rotation * state.rotation
        + state.vertical_delta * state.vertical_delta)
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jaw_asym_config();
        assert!((cfg.max_shift - 1.0).abs() < 1e-6);
        assert!((cfg.max_rotation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_jaw_asym_state();
        assert_eq!(s.lateral_shift, 0.0);
        assert_eq!(s.rotation, 0.0);
        assert_eq!(s.vertical_delta, 0.0);
    }

    #[test]
    fn test_set_lateral_shift_clamps() {
        let cfg = default_jaw_asym_config();
        let mut s = new_jaw_asym_state();
        ja_set_lateral_shift(&mut s, &cfg, 5.0);
        assert!((s.lateral_shift - 1.0).abs() < 1e-6);
        ja_set_lateral_shift(&mut s, &cfg, -5.0);
        assert!((s.lateral_shift + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_rotation() {
        let cfg = default_jaw_asym_config();
        let mut s = new_jaw_asym_state();
        ja_set_rotation(&mut s, &cfg, 0.4);
        assert!((s.rotation - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical_delta_clamps() {
        let mut s = new_jaw_asym_state();
        ja_set_vertical_delta(&mut s, 2.0);
        assert!((s.vertical_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_jaw_asym_config();
        let mut s = new_jaw_asym_state();
        ja_set_lateral_shift(&mut s, &cfg, 0.5);
        ja_reset(&mut s);
        assert_eq!(s.lateral_shift, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_jaw_asym_state();
        assert_eq!(ja_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_jaw_asym_state();
        let j = ja_to_json(&s);
        assert!(j.contains("lateral_shift"));
        assert!(j.contains("rotation"));
        assert!(j.contains("vertical_delta"));
    }

    #[test]
    fn test_magnitude_zero() {
        let s = new_jaw_asym_state();
        assert!((ja_magnitude(&s)).abs() < 1e-6);
    }

    #[test]
    fn test_magnitude_nonzero() {
        let cfg = default_jaw_asym_config();
        let mut s = new_jaw_asym_state();
        ja_set_lateral_shift(&mut s, &cfg, 1.0);
        assert!(ja_magnitude(&s) > 0.9);
    }
}
