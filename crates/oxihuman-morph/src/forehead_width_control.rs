// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Forehead width morph control: adjusts lateral span of the forehead.

/// Configuration for forehead width morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadWidthConfig {
    pub min_width: f32,
    pub max_width: f32,
}

/// Runtime state for forehead width morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadWidthState {
    pub width: f32,
    pub temple_depth: f32,
    pub bossing: f32,
}

#[allow(dead_code)]
pub fn default_forehead_width_config() -> ForeheadWidthConfig {
    ForeheadWidthConfig {
        min_width: 0.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_forehead_width_state() -> ForeheadWidthState {
    ForeheadWidthState {
        width: 0.5,
        temple_depth: 0.3,
        bossing: 0.0,
    }
}

#[allow(dead_code)]
pub fn fw_set_width(state: &mut ForeheadWidthState, cfg: &ForeheadWidthConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn fw_set_temple(state: &mut ForeheadWidthState, v: f32) {
    state.temple_depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fw_set_bossing(state: &mut ForeheadWidthState, v: f32) {
    state.bossing = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fw_reset(state: &mut ForeheadWidthState) {
    *state = new_forehead_width_state();
}

#[allow(dead_code)]
pub fn fw_to_weights(state: &ForeheadWidthState) -> Vec<(String, f32)> {
    vec![
        ("forehead_width".to_string(), state.width),
        ("forehead_temple_depth".to_string(), state.temple_depth),
        ("forehead_bossing".to_string(), state.bossing),
    ]
}

#[allow(dead_code)]
pub fn fw_to_json(state: &ForeheadWidthState) -> String {
    format!(
        r#"{{"width":{:.4},"temple_depth":{:.4},"bossing":{:.4}}}"#,
        state.width, state.temple_depth, state.bossing
    )
}

#[allow(dead_code)]
pub fn fw_blend(a: &ForeheadWidthState, b: &ForeheadWidthState, t: f32) -> ForeheadWidthState {
    let t = t.clamp(0.0, 1.0);
    ForeheadWidthState {
        width: a.width + (b.width - a.width) * t,
        temple_depth: a.temple_depth + (b.temple_depth - a.temple_depth) * t,
        bossing: a.bossing + (b.bossing - a.bossing) * t,
    }
}

#[allow(dead_code)]
pub fn fw_effective_width(state: &ForeheadWidthState) -> f32 {
    state.width * (1.0 + state.bossing * 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_forehead_width_config();
        assert!(cfg.min_width.abs() < 1e-6);
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_forehead_width_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!(s.bossing.abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_forehead_width_config();
        let mut s = new_forehead_width_state();
        fw_set_width(&mut s, &cfg, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_temple() {
        let mut s = new_forehead_width_state();
        fw_set_temple(&mut s, 0.8);
        assert!((s.temple_depth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_bossing() {
        let mut s = new_forehead_width_state();
        fw_set_bossing(&mut s, 0.5);
        assert!((s.bossing - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_forehead_width_config();
        let mut s = new_forehead_width_state();
        fw_set_width(&mut s, &cfg, 0.9);
        fw_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_forehead_width_state();
        assert_eq!(fw_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_forehead_width_state();
        let j = fw_to_json(&s);
        assert!(j.contains("width"));
    }

    #[test]
    fn test_blend() {
        let a = new_forehead_width_state();
        let mut b = new_forehead_width_state();
        b.width = 1.0;
        let mid = fw_blend(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_effective_width() {
        let s = new_forehead_width_state();
        let w = fw_effective_width(&s);
        assert!(w > 0.0);
    }
}
