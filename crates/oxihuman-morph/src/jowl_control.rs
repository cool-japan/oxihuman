// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Jowl/cheek sag morph control (age-related sagging).

#![allow(dead_code)]

/// Configuration for jowl morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JowlConfig {
    pub max_sag: f32,
}

/// Runtime state for jowl morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JowlState {
    pub sag_l: f32,
    pub sag_r: f32,
    pub width_l: f32,
    pub width_r: f32,
}

#[allow(dead_code)]
pub fn default_jowl_config() -> JowlConfig {
    JowlConfig { max_sag: 1.0 }
}

#[allow(dead_code)]
pub fn new_jowl_state() -> JowlState {
    JowlState {
        sag_l: 0.0,
        sag_r: 0.0,
        width_l: 0.0,
        width_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn jowl_set_sag(state: &mut JowlState, cfg: &JowlConfig, left: f32, right: f32) {
    state.sag_l = left.clamp(0.0, cfg.max_sag);
    state.sag_r = right.clamp(0.0, cfg.max_sag);
}

#[allow(dead_code)]
pub fn jowl_set_width(state: &mut JowlState, left: f32, right: f32) {
    state.width_l = left.clamp(0.0, 1.0);
    state.width_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn jowl_mirror(state: &mut JowlState) {
    let avg_s = (state.sag_l + state.sag_r) * 0.5;
    let avg_w = (state.width_l + state.width_r) * 0.5;
    state.sag_l = avg_s;
    state.sag_r = avg_s;
    state.width_l = avg_w;
    state.width_r = avg_w;
}

#[allow(dead_code)]
pub fn jowl_reset(state: &mut JowlState) {
    *state = new_jowl_state();
}

#[allow(dead_code)]
pub fn jowl_to_weights(state: &JowlState) -> Vec<(String, f32)> {
    vec![
        ("jowl_sag_l".to_string(), state.sag_l),
        ("jowl_sag_r".to_string(), state.sag_r),
        ("jowl_width_l".to_string(), state.width_l),
        ("jowl_width_r".to_string(), state.width_r),
    ]
}

#[allow(dead_code)]
pub fn jowl_to_json(state: &JowlState) -> String {
    format!(
        r#"{{"sag_l":{:.4},"sag_r":{:.4},"width_l":{:.4},"width_r":{:.4}}}"#,
        state.sag_l, state.sag_r, state.width_l, state.width_r
    )
}

#[allow(dead_code)]
pub fn jowl_clamp(state: &mut JowlState, cfg: &JowlConfig) {
    state.sag_l = state.sag_l.clamp(0.0, cfg.max_sag);
    state.sag_r = state.sag_r.clamp(0.0, cfg.max_sag);
    state.width_l = state.width_l.clamp(0.0, 1.0);
    state.width_r = state.width_r.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jowl_config();
        assert!((cfg.max_sag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_jowl_state();
        assert_eq!(s.sag_l, 0.0);
        assert_eq!(s.width_r, 0.0);
    }

    #[test]
    fn test_set_sag_clamps() {
        let cfg = default_jowl_config();
        let mut s = new_jowl_state();
        jowl_set_sag(&mut s, &cfg, 2.0, -0.5);
        assert!((s.sag_l - 1.0).abs() < 1e-6);
        assert_eq!(s.sag_r, 0.0);
    }

    #[test]
    fn test_set_width_clamps() {
        let mut s = new_jowl_state();
        jowl_set_width(&mut s, 1.5, -0.2);
        assert_eq!(s.width_l, 1.0);
        assert_eq!(s.width_r, 0.0);
    }

    #[test]
    fn test_mirror() {
        let mut s = new_jowl_state();
        s.sag_l = 0.3;
        s.sag_r = 0.7;
        jowl_mirror(&mut s);
        assert!((s.sag_l - 0.5).abs() < 1e-6);
        assert!((s.sag_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_jowl_config();
        let mut s = new_jowl_state();
        jowl_set_sag(&mut s, &cfg, 0.5, 0.5);
        jowl_reset(&mut s);
        assert_eq!(s.sag_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_jowl_state();
        let w = jowl_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_jowl_state();
        let j = jowl_to_json(&s);
        assert!(j.contains("sag_l"));
        assert!(j.contains("width_r"));
    }
}
