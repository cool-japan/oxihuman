// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lip asymmetry morph — independent L/R lip adjustment.

#![allow(dead_code)]

/// Configuration for lip asymmetry morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipAsymConfig {
    pub max_delta: f32,
}

/// Runtime state for lip asymmetry morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipAsymState {
    pub corner_l: f32,
    pub corner_r: f32,
    pub upper_l: f32,
    pub upper_r: f32,
}

#[allow(dead_code)]
pub fn default_lip_asym_config() -> LipAsymConfig {
    LipAsymConfig { max_delta: 1.0 }
}

#[allow(dead_code)]
pub fn new_lip_asym_state() -> LipAsymState {
    LipAsymState {
        corner_l: 0.0,
        corner_r: 0.0,
        upper_l: 0.0,
        upper_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn lip_asym_set_corner(state: &mut LipAsymState, cfg: &LipAsymConfig, left: f32, right: f32) {
    state.corner_l = left.clamp(-cfg.max_delta, cfg.max_delta);
    state.corner_r = right.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn lip_asym_set_upper(state: &mut LipAsymState, cfg: &LipAsymConfig, left: f32, right: f32) {
    state.upper_l = left.clamp(-cfg.max_delta, cfg.max_delta);
    state.upper_r = right.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn lip_asym_reset(state: &mut LipAsymState) {
    *state = new_lip_asym_state();
}

#[allow(dead_code)]
pub fn lip_asym_to_weights(state: &LipAsymState) -> Vec<(String, f32)> {
    vec![
        ("lip_corner_l".to_string(), state.corner_l),
        ("lip_corner_r".to_string(), state.corner_r),
        ("lip_upper_l".to_string(), state.upper_l),
        ("lip_upper_r".to_string(), state.upper_r),
    ]
}

#[allow(dead_code)]
pub fn lip_asym_to_json(state: &LipAsymState) -> String {
    format!(
        r#"{{"corner_l":{:.4},"corner_r":{:.4},"upper_l":{:.4},"upper_r":{:.4}}}"#,
        state.corner_l, state.corner_r, state.upper_l, state.upper_r
    )
}

#[allow(dead_code)]
pub fn lip_asym_magnitude(state: &LipAsymState) -> f32 {
    (state.corner_l * state.corner_l
        + state.corner_r * state.corner_r
        + state.upper_l * state.upper_l
        + state.upper_r * state.upper_r)
        .sqrt()
}

#[allow(dead_code)]
pub fn lip_asym_clamp(state: &mut LipAsymState, cfg: &LipAsymConfig) {
    state.corner_l = state.corner_l.clamp(-cfg.max_delta, cfg.max_delta);
    state.corner_r = state.corner_r.clamp(-cfg.max_delta, cfg.max_delta);
    state.upper_l = state.upper_l.clamp(-cfg.max_delta, cfg.max_delta);
    state.upper_r = state.upper_r.clamp(-cfg.max_delta, cfg.max_delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_asym_config();
        assert!((cfg.max_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_lip_asym_state();
        assert_eq!(s.corner_l, 0.0);
        assert_eq!(s.upper_r, 0.0);
    }

    #[test]
    fn test_set_corner_clamps() {
        let cfg = default_lip_asym_config();
        let mut s = new_lip_asym_state();
        lip_asym_set_corner(&mut s, &cfg, 2.0, -2.0);
        assert!((s.corner_l - 1.0).abs() < 1e-6);
        assert!((s.corner_r + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper_valid() {
        let cfg = default_lip_asym_config();
        let mut s = new_lip_asym_state();
        lip_asym_set_upper(&mut s, &cfg, 0.3, 0.6);
        assert!((s.upper_l - 0.3).abs() < 1e-6);
        assert!((s.upper_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_lip_asym_config();
        let mut s = new_lip_asym_state();
        lip_asym_set_corner(&mut s, &cfg, 0.5, 0.5);
        lip_asym_reset(&mut s);
        assert_eq!(s.corner_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_lip_asym_state();
        let w = lip_asym_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_lip_asym_state();
        let j = lip_asym_to_json(&s);
        assert!(j.contains("corner_l"));
        assert!(j.contains("upper_r"));
    }

    #[test]
    fn test_magnitude_nonzero() {
        let cfg = default_lip_asym_config();
        let mut s = new_lip_asym_state();
        lip_asym_set_corner(&mut s, &cfg, 0.5, 0.0);
        assert!(lip_asym_magnitude(&s) > 0.0);
    }

    #[test]
    fn test_clamp() {
        let cfg = default_lip_asym_config();
        let mut s = new_lip_asym_state();
        s.corner_l = 5.0;
        s.corner_r = -5.0;
        lip_asym_clamp(&mut s, &cfg);
        assert!((s.corner_l - 1.0).abs() < 1e-6);
        assert!((s.corner_r + 1.0).abs() < 1e-6);
    }
}
