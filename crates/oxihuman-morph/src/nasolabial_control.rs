// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Nasolabial fold depth morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasolabialConfig {
    pub max_depth: f32,
    pub max_length: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasolabialState {
    pub depth_l: f32,
    pub depth_r: f32,
    pub length_l: f32,
    pub length_r: f32,
}

#[allow(dead_code)]
pub fn default_nasolabial_config() -> NasolabialConfig {
    NasolabialConfig { max_depth: 1.0, max_length: 1.0 }
}

#[allow(dead_code)]
pub fn new_nasolabial_state() -> NasolabialState {
    NasolabialState { depth_l: 0.0, depth_r: 0.0, length_l: 0.5, length_r: 0.5 }
}

#[allow(dead_code)]
pub fn naso_set_depth(state: &mut NasolabialState, cfg: &NasolabialConfig, left: f32, right: f32) {
    state.depth_l = left.clamp(0.0, cfg.max_depth);
    state.depth_r = right.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn naso_set_length(state: &mut NasolabialState, cfg: &NasolabialConfig, left: f32, right: f32) {
    state.length_l = left.clamp(0.0, cfg.max_length);
    state.length_r = right.clamp(0.0, cfg.max_length);
}

#[allow(dead_code)]
pub fn naso_mirror(state: &mut NasolabialState) {
    let avg_d = (state.depth_l + state.depth_r) * 0.5;
    let avg_l = (state.length_l + state.length_r) * 0.5;
    state.depth_l = avg_d;
    state.depth_r = avg_d;
    state.length_l = avg_l;
    state.length_r = avg_l;
}

#[allow(dead_code)]
pub fn naso_reset(state: &mut NasolabialState) {
    *state = new_nasolabial_state();
}

#[allow(dead_code)]
pub fn naso_to_weights(state: &NasolabialState) -> [f32; 4] {
    [state.depth_l, state.depth_r, state.length_l, state.length_r]
}

#[allow(dead_code)]
pub fn naso_to_json(state: &NasolabialState) -> String {
    format!(
        r#"{{"depth_l":{:.4},"depth_r":{:.4},"length_l":{:.4},"length_r":{:.4}}}"#,
        state.depth_l, state.depth_r, state.length_l, state.length_r
    )
}

#[allow(dead_code)]
pub fn naso_clamp(state: &mut NasolabialState, cfg: &NasolabialConfig) {
    state.depth_l = state.depth_l.clamp(0.0, cfg.max_depth);
    state.depth_r = state.depth_r.clamp(0.0, cfg.max_depth);
    state.length_l = state.length_l.clamp(0.0, cfg.max_length);
    state.length_r = state.length_r.clamp(0.0, cfg.max_length);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nasolabial_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
        assert!((cfg.max_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_nasolabial_state();
        assert!((s.depth_l - 0.0).abs() < 1e-6);
        assert!((s.length_l - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_nasolabial_config();
        let mut s = new_nasolabial_state();
        naso_set_depth(&mut s, &cfg, 5.0, -1.0);
        assert!((s.depth_l - 1.0).abs() < 1e-6);
        assert!((s.depth_r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamps() {
        let cfg = default_nasolabial_config();
        let mut s = new_nasolabial_state();
        naso_set_length(&mut s, &cfg, 0.7, 0.3);
        assert!((s.length_l - 0.7).abs() < 1e-6);
        assert!((s.length_r - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_mirror() {
        let cfg = default_nasolabial_config();
        let mut s = new_nasolabial_state();
        naso_set_depth(&mut s, &cfg, 0.2, 0.8);
        naso_mirror(&mut s);
        assert!((s.depth_l - 0.5).abs() < 1e-6);
        assert!((s.depth_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_nasolabial_config();
        let mut s = new_nasolabial_state();
        naso_set_depth(&mut s, &cfg, 0.8, 0.8);
        naso_reset(&mut s);
        assert!((s.depth_l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_length() {
        let s = new_nasolabial_state();
        let w = naso_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json() {
        let s = new_nasolabial_state();
        let j = naso_to_json(&s);
        assert!(j.contains("depth_l"));
        assert!(j.contains("length_r"));
    }

    #[test]
    fn test_clamp() {
        let cfg = default_nasolabial_config();
        let mut s = new_nasolabial_state();
        s.depth_l = 5.0;
        s.length_r = -1.0;
        naso_clamp(&mut s, &cfg);
        assert!((s.depth_l - 1.0).abs() < 1e-6);
        assert!((s.length_r - 0.0).abs() < 1e-6);
    }
}
