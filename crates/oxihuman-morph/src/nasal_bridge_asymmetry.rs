// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Nasal bridge asymmetry morph (L/R deviation).

#![allow(dead_code)]

/// Configuration for nasal bridge asymmetry morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalBridgeAsymConfig {
    pub max_deviation: f32,
}

/// Runtime state for nasal bridge asymmetry morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalBridgeAsymState {
    pub deviation: f32,
    pub tilt: f32,
    pub width_delta: f32,
}

#[allow(dead_code)]
pub fn default_nasal_bridge_asym_config() -> NasalBridgeAsymConfig {
    NasalBridgeAsymConfig { max_deviation: 1.0 }
}

#[allow(dead_code)]
pub fn new_nasal_bridge_asym_state() -> NasalBridgeAsymState {
    NasalBridgeAsymState {
        deviation: 0.0,
        tilt: 0.0,
        width_delta: 0.0,
    }
}

#[allow(dead_code)]
pub fn nba_set_deviation(state: &mut NasalBridgeAsymState, cfg: &NasalBridgeAsymConfig, v: f32) {
    state.deviation = v.clamp(-cfg.max_deviation, cfg.max_deviation);
}

#[allow(dead_code)]
pub fn nba_set_tilt(state: &mut NasalBridgeAsymState, v: f32) {
    state.tilt = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nba_set_width_delta(state: &mut NasalBridgeAsymState, v: f32) {
    state.width_delta = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nba_reset(state: &mut NasalBridgeAsymState) {
    *state = new_nasal_bridge_asym_state();
}

#[allow(dead_code)]
pub fn nba_to_weights(state: &NasalBridgeAsymState) -> Vec<(String, f32)> {
    vec![
        ("nba_deviation".to_string(), state.deviation),
        ("nba_tilt".to_string(), state.tilt),
        ("nba_width_delta".to_string(), state.width_delta),
    ]
}

#[allow(dead_code)]
pub fn nba_to_json(state: &NasalBridgeAsymState) -> String {
    format!(
        r#"{{"deviation":{:.4},"tilt":{:.4},"width_delta":{:.4}}}"#,
        state.deviation, state.tilt, state.width_delta
    )
}

#[allow(dead_code)]
pub fn nba_clamp(state: &mut NasalBridgeAsymState, cfg: &NasalBridgeAsymConfig) {
    state.deviation = state.deviation.clamp(-cfg.max_deviation, cfg.max_deviation);
    state.tilt = state.tilt.clamp(-1.0, 1.0);
    state.width_delta = state.width_delta.clamp(-1.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nasal_bridge_asym_config();
        assert!((cfg.max_deviation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_nasal_bridge_asym_state();
        assert_eq!(s.deviation, 0.0);
        assert_eq!(s.tilt, 0.0);
        assert_eq!(s.width_delta, 0.0);
    }

    #[test]
    fn test_set_deviation_clamps() {
        let cfg = default_nasal_bridge_asym_config();
        let mut s = new_nasal_bridge_asym_state();
        nba_set_deviation(&mut s, &cfg, 5.0);
        assert!((s.deviation - 1.0).abs() < 1e-6);
        nba_set_deviation(&mut s, &cfg, -5.0);
        assert!((s.deviation + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_tilt() {
        let mut s = new_nasal_bridge_asym_state();
        nba_set_tilt(&mut s, 0.4);
        assert!((s.tilt - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_delta_clamps() {
        let mut s = new_nasal_bridge_asym_state();
        nba_set_width_delta(&mut s, 2.0);
        assert!((s.width_delta - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_nasal_bridge_asym_config();
        let mut s = new_nasal_bridge_asym_state();
        nba_set_deviation(&mut s, &cfg, 0.5);
        nba_reset(&mut s);
        assert_eq!(s.deviation, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_nasal_bridge_asym_state();
        assert_eq!(nba_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_nasal_bridge_asym_state();
        let j = nba_to_json(&s);
        assert!(j.contains("deviation"));
        assert!(j.contains("tilt"));
        assert!(j.contains("width_delta"));
    }
}
