// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Brow asymmetry morph — independent L/R brow adjustment.

#![allow(dead_code)]

/// Configuration for brow asymmetry morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowAsymConfig {
    pub max_delta: f32,
}

/// Runtime state for brow asymmetry morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowAsymState {
    pub height_delta: f32,
    pub arch_delta: f32,
    pub inner_delta: f32,
    pub outer_delta: f32,
}

#[allow(dead_code)]
pub fn default_brow_asym_config() -> BrowAsymConfig {
    BrowAsymConfig { max_delta: 1.0 }
}

#[allow(dead_code)]
pub fn new_brow_asym_state() -> BrowAsymState {
    BrowAsymState {
        height_delta: 0.0,
        arch_delta: 0.0,
        inner_delta: 0.0,
        outer_delta: 0.0,
    }
}

#[allow(dead_code)]
pub fn brow_asym_set_height_delta(state: &mut BrowAsymState, cfg: &BrowAsymConfig, v: f32) {
    state.height_delta = v.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn brow_asym_set_arch_delta(state: &mut BrowAsymState, cfg: &BrowAsymConfig, v: f32) {
    state.arch_delta = v.clamp(-cfg.max_delta, cfg.max_delta);
}

#[allow(dead_code)]
pub fn brow_asym_reset(state: &mut BrowAsymState) {
    *state = new_brow_asym_state();
}

#[allow(dead_code)]
pub fn brow_asym_to_weights(state: &BrowAsymState) -> Vec<(String, f32)> {
    vec![
        ("brow_height_delta".to_string(), state.height_delta),
        ("brow_arch_delta".to_string(), state.arch_delta),
        ("brow_inner_delta".to_string(), state.inner_delta),
        ("brow_outer_delta".to_string(), state.outer_delta),
    ]
}

#[allow(dead_code)]
pub fn brow_asym_to_json(state: &BrowAsymState) -> String {
    format!(
        r#"{{"height_delta":{:.4},"arch_delta":{:.4},"inner_delta":{:.4},"outer_delta":{:.4}}}"#,
        state.height_delta, state.arch_delta, state.inner_delta, state.outer_delta
    )
}

#[allow(dead_code)]
pub fn brow_asym_magnitude(state: &BrowAsymState) -> f32 {
    (state.height_delta * state.height_delta
        + state.arch_delta * state.arch_delta
        + state.inner_delta * state.inner_delta
        + state.outer_delta * state.outer_delta)
        .sqrt()
}

#[allow(dead_code)]
pub fn brow_asym_clamp(state: &mut BrowAsymState, cfg: &BrowAsymConfig) {
    state.height_delta = state.height_delta.clamp(-cfg.max_delta, cfg.max_delta);
    state.arch_delta = state.arch_delta.clamp(-cfg.max_delta, cfg.max_delta);
    state.inner_delta = state.inner_delta.clamp(-cfg.max_delta, cfg.max_delta);
    state.outer_delta = state.outer_delta.clamp(-cfg.max_delta, cfg.max_delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_asym_config();
        assert!((cfg.max_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_brow_asym_state();
        assert_eq!(s.height_delta, 0.0);
        assert_eq!(s.arch_delta, 0.0);
    }

    #[test]
    fn test_set_height_delta_clamps() {
        let cfg = default_brow_asym_config();
        let mut s = new_brow_asym_state();
        brow_asym_set_height_delta(&mut s, &cfg, 2.0);
        assert!((s.height_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch_delta_clamps_negative() {
        let cfg = default_brow_asym_config();
        let mut s = new_brow_asym_state();
        brow_asym_set_arch_delta(&mut s, &cfg, -3.0);
        assert!((s.arch_delta + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_brow_asym_config();
        let mut s = new_brow_asym_state();
        brow_asym_set_height_delta(&mut s, &cfg, 0.5);
        brow_asym_reset(&mut s);
        assert_eq!(s.height_delta, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_brow_asym_state();
        let w = brow_asym_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_brow_asym_state();
        let j = brow_asym_to_json(&s);
        assert!(j.contains("height_delta"));
        assert!(j.contains("outer_delta"));
    }

    #[test]
    fn test_magnitude_zero() {
        let s = new_brow_asym_state();
        assert!((brow_asym_magnitude(&s)).abs() < 1e-6);
    }
}
