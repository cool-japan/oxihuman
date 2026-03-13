// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Under-eye bags/hollows morph control.

#![allow(dead_code)]

/// Configuration for under-eye morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UndereyeConfig {
    pub max_puff: f32,
    pub max_hollow: f32,
}

/// Runtime state for under-eye morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UndereyeState {
    pub puff_l: f32,
    pub puff_r: f32,
    pub hollow_l: f32,
    pub hollow_r: f32,
    pub dark_circle_l: f32,
    pub dark_circle_r: f32,
}

#[allow(dead_code)]
pub fn default_undereye_config() -> UndereyeConfig {
    UndereyeConfig {
        max_puff: 1.0,
        max_hollow: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_undereye_state() -> UndereyeState {
    UndereyeState {
        puff_l: 0.0,
        puff_r: 0.0,
        hollow_l: 0.0,
        hollow_r: 0.0,
        dark_circle_l: 0.0,
        dark_circle_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn undereye_set_puff(state: &mut UndereyeState, cfg: &UndereyeConfig, left: f32, right: f32) {
    state.puff_l = left.clamp(0.0, cfg.max_puff);
    state.puff_r = right.clamp(0.0, cfg.max_puff);
}

#[allow(dead_code)]
pub fn undereye_set_hollow(state: &mut UndereyeState, cfg: &UndereyeConfig, left: f32, right: f32) {
    state.hollow_l = left.clamp(0.0, cfg.max_hollow);
    state.hollow_r = right.clamp(0.0, cfg.max_hollow);
}

#[allow(dead_code)]
pub fn undereye_mirror(state: &mut UndereyeState) {
    let avg_p = (state.puff_l + state.puff_r) * 0.5;
    let avg_h = (state.hollow_l + state.hollow_r) * 0.5;
    let avg_d = (state.dark_circle_l + state.dark_circle_r) * 0.5;
    state.puff_l = avg_p;
    state.puff_r = avg_p;
    state.hollow_l = avg_h;
    state.hollow_r = avg_h;
    state.dark_circle_l = avg_d;
    state.dark_circle_r = avg_d;
}

#[allow(dead_code)]
pub fn undereye_reset(state: &mut UndereyeState) {
    *state = new_undereye_state();
}

#[allow(dead_code)]
pub fn undereye_to_weights(state: &UndereyeState) -> Vec<(String, f32)> {
    vec![
        ("undereye_puff_l".to_string(), state.puff_l),
        ("undereye_puff_r".to_string(), state.puff_r),
        ("undereye_hollow_l".to_string(), state.hollow_l),
        ("undereye_hollow_r".to_string(), state.hollow_r),
        ("undereye_dark_circle_l".to_string(), state.dark_circle_l),
        ("undereye_dark_circle_r".to_string(), state.dark_circle_r),
    ]
}

#[allow(dead_code)]
pub fn undereye_to_json(state: &UndereyeState) -> String {
    format!(
        r#"{{"puff_l":{:.4},"puff_r":{:.4},"hollow_l":{:.4},"hollow_r":{:.4},"dark_circle_l":{:.4},"dark_circle_r":{:.4}}}"#,
        state.puff_l, state.puff_r, state.hollow_l, state.hollow_r,
        state.dark_circle_l, state.dark_circle_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_undereye_config();
        assert!((cfg.max_puff - 1.0).abs() < 1e-6);
        assert!((cfg.max_hollow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_undereye_state();
        assert_eq!(s.puff_l, 0.0);
        assert_eq!(s.dark_circle_r, 0.0);
    }

    #[test]
    fn test_set_puff_clamps() {
        let cfg = default_undereye_config();
        let mut s = new_undereye_state();
        undereye_set_puff(&mut s, &cfg, 2.0, -1.0);
        assert!((s.puff_l - 1.0).abs() < 1e-6);
        assert_eq!(s.puff_r, 0.0);
    }

    #[test]
    fn test_set_hollow_clamps() {
        let cfg = default_undereye_config();
        let mut s = new_undereye_state();
        undereye_set_hollow(&mut s, &cfg, 0.4, 0.8);
        assert!((s.hollow_l - 0.4).abs() < 1e-6);
        assert!((s.hollow_r - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_mirror() {
        let mut s = new_undereye_state();
        s.puff_l = 0.2;
        s.puff_r = 0.6;
        undereye_mirror(&mut s);
        assert!((s.puff_l - 0.4).abs() < 1e-6);
        assert!((s.puff_r - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_undereye_config();
        let mut s = new_undereye_state();
        undereye_set_puff(&mut s, &cfg, 0.5, 0.5);
        undereye_reset(&mut s);
        assert_eq!(s.puff_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_undereye_state();
        let w = undereye_to_weights(&s);
        assert_eq!(w.len(), 6);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_undereye_state();
        let j = undereye_to_json(&s);
        assert!(j.contains("puff_l"));
        assert!(j.contains("dark_circle_r"));
    }

    #[test]
    fn test_hollow_valid() {
        let cfg = default_undereye_config();
        let mut s = new_undereye_state();
        undereye_set_hollow(&mut s, &cfg, 0.3, 0.7);
        assert!((s.hollow_l - 0.3).abs() < 1e-6);
    }
}
