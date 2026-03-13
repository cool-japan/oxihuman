// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Ear tragus morph control: adjusts the shape and size of the ear tragus.

/// Configuration for ear tragus morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarTragusConfig {
    pub min_size: f32,
    pub max_size: f32,
}

/// Runtime state for ear tragus morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarTragusState {
    pub left_size: f32,
    pub right_size: f32,
    pub protrusion: f32,
}

#[allow(dead_code)]
pub fn default_ear_tragus_config() -> EarTragusConfig {
    EarTragusConfig {
        min_size: 0.0,
        max_size: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_ear_tragus_state() -> EarTragusState {
    EarTragusState {
        left_size: 0.5,
        right_size: 0.5,
        protrusion: 0.3,
    }
}

#[allow(dead_code)]
pub fn et_set_left(state: &mut EarTragusState, cfg: &EarTragusConfig, v: f32) {
    state.left_size = v.clamp(cfg.min_size, cfg.max_size);
}

#[allow(dead_code)]
pub fn et_set_right(state: &mut EarTragusState, cfg: &EarTragusConfig, v: f32) {
    state.right_size = v.clamp(cfg.min_size, cfg.max_size);
}

#[allow(dead_code)]
pub fn et_set_both(state: &mut EarTragusState, cfg: &EarTragusConfig, v: f32) {
    let clamped = v.clamp(cfg.min_size, cfg.max_size);
    state.left_size = clamped;
    state.right_size = clamped;
}

#[allow(dead_code)]
pub fn et_set_protrusion(state: &mut EarTragusState, v: f32) {
    state.protrusion = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn et_reset(state: &mut EarTragusState) {
    *state = new_ear_tragus_state();
}

#[allow(dead_code)]
pub fn et_to_weights(state: &EarTragusState) -> Vec<(String, f32)> {
    vec![
        ("ear_tragus_left".to_string(), state.left_size),
        ("ear_tragus_right".to_string(), state.right_size),
        ("ear_tragus_protrusion".to_string(), state.protrusion),
    ]
}

#[allow(dead_code)]
pub fn et_to_json(state: &EarTragusState) -> String {
    format!(
        r#"{{"left_size":{:.4},"right_size":{:.4},"protrusion":{:.4}}}"#,
        state.left_size, state.right_size, state.protrusion
    )
}

#[allow(dead_code)]
pub fn et_blend(a: &EarTragusState, b: &EarTragusState, t: f32) -> EarTragusState {
    let t = t.clamp(0.0, 1.0);
    EarTragusState {
        left_size: a.left_size + (b.left_size - a.left_size) * t,
        right_size: a.right_size + (b.right_size - a.right_size) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ear_tragus_config();
        assert!(cfg.min_size.abs() < 1e-6);
        assert!((cfg.max_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_ear_tragus_state();
        assert!((s.left_size - 0.5).abs() < 1e-6);
        assert!((s.protrusion - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_clamps() {
        let cfg = default_ear_tragus_config();
        let mut s = new_ear_tragus_state();
        et_set_left(&mut s, &cfg, 5.0);
        assert!((s.left_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_both() {
        let cfg = default_ear_tragus_config();
        let mut s = new_ear_tragus_state();
        et_set_both(&mut s, &cfg, 0.7);
        assert!((s.left_size - 0.7).abs() < 1e-6);
        assert!((s.right_size - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion() {
        let mut s = new_ear_tragus_state();
        et_set_protrusion(&mut s, 0.8);
        assert!((s.protrusion - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_ear_tragus_config();
        let mut s = new_ear_tragus_state();
        et_set_left(&mut s, &cfg, 0.9);
        et_reset(&mut s);
        assert!((s.left_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_ear_tragus_state();
        assert_eq!(et_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_ear_tragus_state();
        let j = et_to_json(&s);
        assert!(j.contains("left_size"));
    }

    #[test]
    fn test_blend() {
        let a = new_ear_tragus_state();
        let mut b = new_ear_tragus_state();
        b.left_size = 1.0;
        let mid = et_blend(&a, &b, 0.5);
        assert!((mid.left_size - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_right() {
        let cfg = default_ear_tragus_config();
        let mut s = new_ear_tragus_state();
        et_set_right(&mut s, &cfg, 0.3);
        assert!((s.right_size - 0.3).abs() < 1e-6);
    }
}
