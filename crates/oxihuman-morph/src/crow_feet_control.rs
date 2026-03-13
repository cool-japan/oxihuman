// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Crow's feet (lateral eye wrinkle) morph control.

#![allow(dead_code)]

/// Configuration for crow's feet morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CrowFeetConfig {
    pub max_depth: f32,
}

/// Runtime state for crow's feet morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CrowFeetState {
    pub depth_l: f32,
    pub depth_r: f32,
    pub spread_l: f32,
    pub spread_r: f32,
}

#[allow(dead_code)]
pub fn default_crow_feet_config() -> CrowFeetConfig {
    CrowFeetConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_crow_feet_state() -> CrowFeetState {
    CrowFeetState {
        depth_l: 0.0,
        depth_r: 0.0,
        spread_l: 0.0,
        spread_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn cf_set_depth(state: &mut CrowFeetState, cfg: &CrowFeetConfig, left: f32, right: f32) {
    state.depth_l = left.clamp(0.0, cfg.max_depth);
    state.depth_r = right.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn cf_set_spread(state: &mut CrowFeetState, left: f32, right: f32) {
    state.spread_l = left.clamp(0.0, 1.0);
    state.spread_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cf_mirror(state: &mut CrowFeetState) {
    let avg_d = (state.depth_l + state.depth_r) * 0.5;
    let avg_s = (state.spread_l + state.spread_r) * 0.5;
    state.depth_l = avg_d;
    state.depth_r = avg_d;
    state.spread_l = avg_s;
    state.spread_r = avg_s;
}

#[allow(dead_code)]
pub fn cf_reset(state: &mut CrowFeetState) {
    *state = new_crow_feet_state();
}

#[allow(dead_code)]
pub fn cf_to_weights(state: &CrowFeetState) -> Vec<(String, f32)> {
    vec![
        ("crow_feet_depth_l".to_string(), state.depth_l),
        ("crow_feet_depth_r".to_string(), state.depth_r),
        ("crow_feet_spread_l".to_string(), state.spread_l),
        ("crow_feet_spread_r".to_string(), state.spread_r),
    ]
}

#[allow(dead_code)]
pub fn cf_to_json(state: &CrowFeetState) -> String {
    format!(
        r#"{{"depth_l":{:.4},"depth_r":{:.4},"spread_l":{:.4},"spread_r":{:.4}}}"#,
        state.depth_l, state.depth_r, state.spread_l, state.spread_r
    )
}

#[allow(dead_code)]
pub fn cf_clamp(state: &mut CrowFeetState, cfg: &CrowFeetConfig) {
    state.depth_l = state.depth_l.clamp(0.0, cfg.max_depth);
    state.depth_r = state.depth_r.clamp(0.0, cfg.max_depth);
    state.spread_l = state.spread_l.clamp(0.0, 1.0);
    state.spread_r = state.spread_r.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_crow_feet_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_crow_feet_state();
        assert_eq!(s.depth_l, 0.0);
        assert_eq!(s.spread_r, 0.0);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_crow_feet_config();
        let mut s = new_crow_feet_state();
        cf_set_depth(&mut s, &cfg, 3.0, -1.0);
        assert!((s.depth_l - 1.0).abs() < 1e-6);
        assert_eq!(s.depth_r, 0.0);
    }

    #[test]
    fn test_set_spread_valid() {
        let mut s = new_crow_feet_state();
        cf_set_spread(&mut s, 0.3, 0.7);
        assert!((s.spread_l - 0.3).abs() < 1e-6);
        assert!((s.spread_r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_mirror() {
        let mut s = new_crow_feet_state();
        s.depth_l = 0.4;
        s.depth_r = 0.8;
        cf_mirror(&mut s);
        assert!((s.depth_l - 0.6).abs() < 1e-6);
        assert!((s.depth_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_crow_feet_config();
        let mut s = new_crow_feet_state();
        cf_set_depth(&mut s, &cfg, 0.5, 0.5);
        cf_reset(&mut s);
        assert_eq!(s.depth_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_crow_feet_state();
        let w = cf_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_crow_feet_state();
        let j = cf_to_json(&s);
        assert!(j.contains("depth_l"));
        assert!(j.contains("spread_r"));
    }
}
