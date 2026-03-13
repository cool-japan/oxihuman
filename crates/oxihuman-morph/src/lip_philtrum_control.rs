// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Lip philtrum morph control: adjusts the groove between nose and upper lip.

/// Configuration for philtrum morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipPhiltrumConfig {
    pub min_depth: f32,
    pub max_depth: f32,
}

/// Runtime state for philtrum morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipPhiltrumState {
    pub depth: f32,
    pub width: f32,
    pub length: f32,
}

#[allow(dead_code)]
pub fn default_lip_philtrum_config() -> LipPhiltrumConfig {
    LipPhiltrumConfig {
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_lip_philtrum_state() -> LipPhiltrumState {
    LipPhiltrumState {
        depth: 0.4,
        width: 0.5,
        length: 0.5,
    }
}

#[allow(dead_code)]
pub fn lp_set_depth(state: &mut LipPhiltrumState, cfg: &LipPhiltrumConfig, v: f32) {
    state.depth = v.clamp(cfg.min_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn lp_set_width(state: &mut LipPhiltrumState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn lp_set_length(state: &mut LipPhiltrumState, v: f32) {
    state.length = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn lp_reset(state: &mut LipPhiltrumState) {
    *state = new_lip_philtrum_state();
}

#[allow(dead_code)]
pub fn lp_to_weights(state: &LipPhiltrumState) -> Vec<(String, f32)> {
    vec![
        ("philtrum_depth".to_string(), state.depth),
        ("philtrum_width".to_string(), state.width),
        ("philtrum_length".to_string(), state.length),
    ]
}

#[allow(dead_code)]
pub fn lp_to_json(state: &LipPhiltrumState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"length":{:.4}}}"#,
        state.depth, state.width, state.length
    )
}

#[allow(dead_code)]
pub fn lp_blend(a: &LipPhiltrumState, b: &LipPhiltrumState, t: f32) -> LipPhiltrumState {
    let t = t.clamp(0.0, 1.0);
    LipPhiltrumState {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}

#[allow(dead_code)]
pub fn lp_prominence(state: &LipPhiltrumState) -> f32 {
    state.depth * state.length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_philtrum_config();
        assert!(cfg.min_depth.abs() < 1e-6);
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_lip_philtrum_state();
        assert!((s.depth - 0.4).abs() < 1e-6);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_lip_philtrum_config();
        let mut s = new_lip_philtrum_state();
        lp_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_lip_philtrum_state();
        lp_set_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_length() {
        let mut s = new_lip_philtrum_state();
        lp_set_length(&mut s, 0.2);
        assert!((s.length - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_lip_philtrum_config();
        let mut s = new_lip_philtrum_state();
        lp_set_depth(&mut s, &cfg, 0.9);
        lp_reset(&mut s);
        assert!((s.depth - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_lip_philtrum_state();
        assert_eq!(lp_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_lip_philtrum_state();
        let j = lp_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("width"));
    }

    #[test]
    fn test_blend() {
        let a = new_lip_philtrum_state();
        let mut b = new_lip_philtrum_state();
        b.depth = 1.0;
        let mid = lp_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_prominence() {
        let s = new_lip_philtrum_state();
        let p = lp_prominence(&s);
        assert!(p > 0.0);
    }
}
