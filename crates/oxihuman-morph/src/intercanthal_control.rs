// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Intercanthal distance morph control: adjusts the distance between inner eye corners.

/// Configuration for intercanthal morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntercanthalConfig {
    pub min_distance: f32,
    pub max_distance: f32,
}

/// Runtime state for intercanthal morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntercanthalState {
    pub distance: f32,
    pub bridge_width: f32,
    pub tilt: f32,
}

#[allow(dead_code)]
pub fn default_intercanthal_config() -> IntercanthalConfig {
    IntercanthalConfig {
        min_distance: 0.0,
        max_distance: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_intercanthal_state() -> IntercanthalState {
    IntercanthalState {
        distance: 0.5,
        bridge_width: 0.5,
        tilt: 0.0,
    }
}

#[allow(dead_code)]
pub fn ic_set_distance(state: &mut IntercanthalState, cfg: &IntercanthalConfig, v: f32) {
    state.distance = v.clamp(cfg.min_distance, cfg.max_distance);
}

#[allow(dead_code)]
pub fn ic_set_bridge_width(state: &mut IntercanthalState, v: f32) {
    state.bridge_width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ic_set_tilt(state: &mut IntercanthalState, v: f32) {
    state.tilt = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ic_reset(state: &mut IntercanthalState) {
    *state = new_intercanthal_state();
}

#[allow(dead_code)]
pub fn ic_to_weights(state: &IntercanthalState) -> Vec<(String, f32)> {
    vec![
        ("intercanthal_distance".to_string(), state.distance),
        ("intercanthal_bridge_width".to_string(), state.bridge_width),
        ("intercanthal_tilt".to_string(), state.tilt),
    ]
}

#[allow(dead_code)]
pub fn ic_to_json(state: &IntercanthalState) -> String {
    format!(
        r#"{{"distance":{:.4},"bridge_width":{:.4},"tilt":{:.4}}}"#,
        state.distance, state.bridge_width, state.tilt
    )
}

#[allow(dead_code)]
pub fn ic_blend(a: &IntercanthalState, b: &IntercanthalState, t: f32) -> IntercanthalState {
    let t = t.clamp(0.0, 1.0);
    IntercanthalState {
        distance: a.distance + (b.distance - a.distance) * t,
        bridge_width: a.bridge_width + (b.bridge_width - a.bridge_width) * t,
        tilt: a.tilt + (b.tilt - a.tilt) * t,
    }
}

#[allow(dead_code)]
pub fn ic_effective_distance(state: &IntercanthalState) -> f32 {
    state.distance + state.bridge_width * 0.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_intercanthal_config();
        assert!(cfg.min_distance.abs() < 1e-6);
        assert!((cfg.max_distance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_intercanthal_state();
        assert!((s.distance - 0.5).abs() < 1e-6);
        assert!(s.tilt.abs() < 1e-6);
    }

    #[test]
    fn test_set_distance_clamps() {
        let cfg = default_intercanthal_config();
        let mut s = new_intercanthal_state();
        ic_set_distance(&mut s, &cfg, 5.0);
        assert!((s.distance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_bridge_width() {
        let mut s = new_intercanthal_state();
        ic_set_bridge_width(&mut s, 0.8);
        assert!((s.bridge_width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_tilt() {
        let mut s = new_intercanthal_state();
        ic_set_tilt(&mut s, -0.5);
        assert!((s.tilt + 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_intercanthal_config();
        let mut s = new_intercanthal_state();
        ic_set_distance(&mut s, &cfg, 0.9);
        ic_reset(&mut s);
        assert!((s.distance - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_intercanthal_state();
        assert_eq!(ic_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_intercanthal_state();
        let j = ic_to_json(&s);
        assert!(j.contains("distance"));
    }

    #[test]
    fn test_blend() {
        let a = new_intercanthal_state();
        let mut b = new_intercanthal_state();
        b.distance = 1.0;
        let mid = ic_blend(&a, &b, 0.5);
        assert!((mid.distance - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_effective_distance() {
        let s = new_intercanthal_state();
        let d = ic_effective_distance(&s);
        assert!(d > 0.0);
    }
}
