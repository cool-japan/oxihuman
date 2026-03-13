// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Body weight morph control: adjusts overall body mass distribution.

use std::f32::consts::PI;

/// Configuration for body weight morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyWeightConfig {
    pub min_weight: f32,
    pub max_weight: f32,
    pub default_weight: f32,
}

/// Runtime state for body weight morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyWeightState {
    pub overall: f32,
    pub upper_body: f32,
    pub lower_body: f32,
    pub belly: f32,
}

#[allow(dead_code)]
pub fn default_body_weight_config() -> BodyWeightConfig {
    BodyWeightConfig {
        min_weight: 0.0,
        max_weight: 1.0,
        default_weight: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_body_weight_state() -> BodyWeightState {
    BodyWeightState {
        overall: 0.5,
        upper_body: 0.5,
        lower_body: 0.5,
        belly: 0.3,
    }
}

#[allow(dead_code)]
pub fn bw_set_overall(state: &mut BodyWeightState, cfg: &BodyWeightConfig, v: f32) {
    state.overall = v.clamp(cfg.min_weight, cfg.max_weight);
}

#[allow(dead_code)]
pub fn bw_set_upper(state: &mut BodyWeightState, v: f32) {
    state.upper_body = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn bw_set_lower(state: &mut BodyWeightState, v: f32) {
    state.lower_body = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn bw_set_belly(state: &mut BodyWeightState, v: f32) {
    state.belly = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn bw_reset(state: &mut BodyWeightState) {
    *state = new_body_weight_state();
}

#[allow(dead_code)]
pub fn bw_to_weights(state: &BodyWeightState) -> Vec<(String, f32)> {
    vec![
        ("body_weight_overall".to_string(), state.overall),
        ("body_weight_upper".to_string(), state.upper_body),
        ("body_weight_lower".to_string(), state.lower_body),
        ("body_weight_belly".to_string(), state.belly),
    ]
}

#[allow(dead_code)]
pub fn bw_to_json(state: &BodyWeightState) -> String {
    format!(
        r#"{{"overall":{:.4},"upper_body":{:.4},"lower_body":{:.4},"belly":{:.4}}}"#,
        state.overall, state.upper_body, state.lower_body, state.belly
    )
}

#[allow(dead_code)]
pub fn bw_blend(a: &BodyWeightState, b: &BodyWeightState, t: f32) -> BodyWeightState {
    let t = t.clamp(0.0, 1.0);
    BodyWeightState {
        overall: a.overall + (b.overall - a.overall) * t,
        upper_body: a.upper_body + (b.upper_body - a.upper_body) * t,
        lower_body: a.lower_body + (b.lower_body - a.lower_body) * t,
        belly: a.belly + (b.belly - a.belly) * t,
    }
}

/// Compute a sine-based distribution curve for weight placement.
#[allow(dead_code)]
pub fn bw_distribution_curve(state: &BodyWeightState, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    state.overall * (PI * t).sin() * (state.upper_body + state.lower_body) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_body_weight_config();
        assert!((cfg.min_weight).abs() < 1e-6);
        assert!((cfg.max_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_body_weight_state();
        assert!((s.overall - 0.5).abs() < 1e-6);
        assert!((s.belly - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_overall_clamps() {
        let cfg = default_body_weight_config();
        let mut s = new_body_weight_state();
        bw_set_overall(&mut s, &cfg, 5.0);
        assert!((s.overall - 1.0).abs() < 1e-6);
        bw_set_overall(&mut s, &cfg, -1.0);
        assert!(s.overall.abs() < 1e-6);
    }

    #[test]
    fn test_set_upper() {
        let mut s = new_body_weight_state();
        bw_set_upper(&mut s, 0.8);
        assert!((s.upper_body - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_lower() {
        let mut s = new_body_weight_state();
        bw_set_lower(&mut s, 0.2);
        assert!((s.lower_body - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_set_belly() {
        let mut s = new_body_weight_state();
        bw_set_belly(&mut s, 0.9);
        assert!((s.belly - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_body_weight_config();
        let mut s = new_body_weight_state();
        bw_set_overall(&mut s, &cfg, 0.9);
        bw_reset(&mut s);
        assert!((s.overall - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_body_weight_state();
        assert_eq!(bw_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = new_body_weight_state();
        let mut b = new_body_weight_state();
        b.overall = 1.0;
        let mid = bw_blend(&a, &b, 0.5);
        assert!((mid.overall - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_distribution_curve() {
        let s = new_body_weight_state();
        let v = bw_distribution_curve(&s, 0.5);
        assert!(v > 0.0);
    }
}
