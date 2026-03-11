// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cheek dimple morph control.

#![allow(dead_code)]

/// Configuration for dimple morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DimpleConfig {
    pub max_depth: f32,
}

/// Runtime state for dimple morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DimpleState {
    pub depth_l: f32,
    pub depth_r: f32,
    pub position_l: f32,
    pub position_r: f32,
}

#[allow(dead_code)]
pub fn default_dimple_config() -> DimpleConfig {
    DimpleConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_dimple_state() -> DimpleState {
    DimpleState {
        depth_l: 0.0,
        depth_r: 0.0,
        position_l: 0.5,
        position_r: 0.5,
    }
}

#[allow(dead_code)]
pub fn dimple_set_depth(state: &mut DimpleState, cfg: &DimpleConfig, left: f32, right: f32) {
    state.depth_l = left.clamp(0.0, cfg.max_depth);
    state.depth_r = right.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn dimple_set_position(state: &mut DimpleState, left: f32, right: f32) {
    state.position_l = left.clamp(0.0, 1.0);
    state.position_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn dimple_mirror(state: &mut DimpleState) {
    let avg_d = (state.depth_l + state.depth_r) * 0.5;
    let avg_p = (state.position_l + state.position_r) * 0.5;
    state.depth_l = avg_d;
    state.depth_r = avg_d;
    state.position_l = avg_p;
    state.position_r = avg_p;
}

#[allow(dead_code)]
pub fn dimple_reset(state: &mut DimpleState) {
    *state = new_dimple_state();
}

#[allow(dead_code)]
pub fn dimple_to_weights(state: &DimpleState) -> Vec<(String, f32)> {
    vec![
        ("dimple_depth_l".to_string(), state.depth_l),
        ("dimple_depth_r".to_string(), state.depth_r),
        ("dimple_position_l".to_string(), state.position_l),
        ("dimple_position_r".to_string(), state.position_r),
    ]
}

#[allow(dead_code)]
pub fn dimple_to_json(state: &DimpleState) -> String {
    format!(
        r#"{{"depth_l":{:.4},"depth_r":{:.4},"position_l":{:.4},"position_r":{:.4}}}"#,
        state.depth_l, state.depth_r, state.position_l, state.position_r
    )
}

#[allow(dead_code)]
pub fn dimple_clamp(state: &mut DimpleState, cfg: &DimpleConfig) {
    state.depth_l = state.depth_l.clamp(0.0, cfg.max_depth);
    state.depth_r = state.depth_r.clamp(0.0, cfg.max_depth);
    state.position_l = state.position_l.clamp(0.0, 1.0);
    state.position_r = state.position_r.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_dimple_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_dimple_state();
        assert_eq!(s.depth_l, 0.0);
        assert_eq!(s.depth_r, 0.0);
        assert!((s.position_l - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_dimple_config();
        let mut s = new_dimple_state();
        dimple_set_depth(&mut s, &cfg, 2.0, -1.0);
        assert!((s.depth_l - 1.0).abs() < 1e-6);
        assert_eq!(s.depth_r, 0.0);
    }

    #[test]
    fn test_set_position_clamps() {
        let mut s = new_dimple_state();
        dimple_set_position(&mut s, -0.5, 1.5);
        assert_eq!(s.position_l, 0.0);
        assert_eq!(s.position_r, 1.0);
    }

    #[test]
    fn test_mirror() {
        let mut s = new_dimple_state();
        s.depth_l = 0.2;
        s.depth_r = 0.8;
        dimple_mirror(&mut s);
        assert!((s.depth_l - 0.5).abs() < 1e-6);
        assert!((s.depth_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_dimple_config();
        let mut s = new_dimple_state();
        dimple_set_depth(&mut s, &cfg, 0.5, 0.5);
        dimple_reset(&mut s);
        assert_eq!(s.depth_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_dimple_state();
        let w = dimple_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_dimple_state();
        let j = dimple_to_json(&s);
        assert!(j.contains("depth_l"));
        assert!(j.contains("position_r"));
    }
}
