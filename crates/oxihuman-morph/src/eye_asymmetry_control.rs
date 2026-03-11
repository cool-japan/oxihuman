// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Eye asymmetry morph — independent L/R eye adjustment.

#![allow(dead_code)]

/// Configuration for eye asymmetry morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeAsymConfig {
    pub max_delta: f32,
}

/// Runtime state for eye asymmetry morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeAsymState {
    pub size_delta: f32,
    pub height_delta: f32,
    pub tilt_delta: f32,
}

#[allow(dead_code)]
pub fn default_eye_asym_config() -> EyeAsymConfig {
    EyeAsymConfig { max_delta: 1.0 }
}

#[allow(dead_code)]
pub fn new_eye_asym_state() -> EyeAsymState {
    EyeAsymState {
        size_delta: 0.0,
        height_delta: 0.0,
        tilt_delta: 0.0,
    }
}

#[allow(dead_code)]
pub fn eye_asym_set_size_delta(state: &mut EyeAsymState, cfg: &EyeAsymConfig, v: f32) {
    state.size_delta = v.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn eye_asym_set_height_delta(state: &mut EyeAsymState, cfg: &EyeAsymConfig, v: f32) {
    state.height_delta = v.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn eye_asym_set_tilt_delta(state: &mut EyeAsymState, cfg: &EyeAsymConfig, v: f32) {
    state.tilt_delta = v.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn eye_asym_reset(state: &mut EyeAsymState) {
    *state = new_eye_asym_state();
}

#[allow(dead_code)]
pub fn eye_asym_to_weights(state: &EyeAsymState) -> Vec<(String, f32)> {
    vec![
        ("eye_size_delta".to_string(), state.size_delta),
        ("eye_height_delta".to_string(), state.height_delta),
        ("eye_tilt_delta".to_string(), state.tilt_delta),
    ]
}

#[allow(dead_code)]
pub fn eye_asym_to_json(state: &EyeAsymState) -> String {
    format!(
        r#"{{"size_delta":{:.4},"height_delta":{:.4},"tilt_delta":{:.4}}}"#,
        state.size_delta, state.height_delta, state.tilt_delta
    )
}

#[allow(dead_code)]
pub fn eye_asym_magnitude(state: &EyeAsymState) -> f32 {
    (state.size_delta * state.size_delta
        + state.height_delta * state.height_delta
        + state.tilt_delta * state.tilt_delta)
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_asym_config();
        assert!((cfg.max_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_eye_asym_state();
        assert_eq!(s.size_delta, 0.0);
        assert_eq!(s.height_delta, 0.0);
        assert_eq!(s.tilt_delta, 0.0);
    }

    #[test]
    fn test_set_size_delta_clamps() {
        let cfg = default_eye_asym_config();
        let mut s = new_eye_asym_state();
        eye_asym_set_size_delta(&mut s, &cfg, 3.0);
        assert!((s.size_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_delta_valid() {
        let cfg = default_eye_asym_config();
        let mut s = new_eye_asym_state();
        eye_asym_set_height_delta(&mut s, &cfg, -0.4);
        assert!((s.height_delta + 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_tilt_delta_clamps_neg() {
        let cfg = default_eye_asym_config();
        let mut s = new_eye_asym_state();
        eye_asym_set_tilt_delta(&mut s, &cfg, -5.0);
        assert!((s.tilt_delta + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_eye_asym_config();
        let mut s = new_eye_asym_state();
        eye_asym_set_size_delta(&mut s, &cfg, 0.5);
        eye_asym_reset(&mut s);
        assert_eq!(s.size_delta, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_eye_asym_state();
        let w = eye_asym_to_weights(&s);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_eye_asym_state();
        let j = eye_asym_to_json(&s);
        assert!(j.contains("size_delta"));
        assert!(j.contains("tilt_delta"));
    }
}
