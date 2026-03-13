// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Chin width morph control: adjusts the lateral span of the chin.

/// Configuration for chin width morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinWidthConfig {
    pub min_width: f32,
    pub max_width: f32,
}

/// Runtime state for chin width morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinWidthState {
    pub width: f32,
    pub taper: f32,
    pub cleft_depth: f32,
}

#[allow(dead_code)]
pub fn default_chin_width_config() -> ChinWidthConfig {
    ChinWidthConfig {
        min_width: 0.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_chin_width_state() -> ChinWidthState {
    ChinWidthState {
        width: 0.5,
        taper: 0.5,
        cleft_depth: 0.0,
    }
}

#[allow(dead_code)]
pub fn cw_set_width(state: &mut ChinWidthState, cfg: &ChinWidthConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn cw_set_taper(state: &mut ChinWidthState, v: f32) {
    state.taper = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cw_set_cleft(state: &mut ChinWidthState, v: f32) {
    state.cleft_depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cw_reset(state: &mut ChinWidthState) {
    *state = new_chin_width_state();
}

#[allow(dead_code)]
pub fn cw_to_weights(state: &ChinWidthState) -> Vec<(String, f32)> {
    vec![
        ("chin_width".to_string(), state.width),
        ("chin_taper".to_string(), state.taper),
        ("chin_cleft".to_string(), state.cleft_depth),
    ]
}

#[allow(dead_code)]
pub fn cw_to_json(state: &ChinWidthState) -> String {
    format!(
        r#"{{"width":{:.4},"taper":{:.4},"cleft_depth":{:.4}}}"#,
        state.width, state.taper, state.cleft_depth
    )
}

#[allow(dead_code)]
pub fn cw_blend(a: &ChinWidthState, b: &ChinWidthState, t: f32) -> ChinWidthState {
    let t = t.clamp(0.0, 1.0);
    ChinWidthState {
        width: a.width + (b.width - a.width) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        cleft_depth: a.cleft_depth + (b.cleft_depth - a.cleft_depth) * t,
    }
}

#[allow(dead_code)]
pub fn cw_effective_width(state: &ChinWidthState) -> f32 {
    state.width * (1.0 - state.taper * 0.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_chin_width_config();
        assert!(cfg.min_width.abs() < 1e-6);
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_chin_width_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!(s.cleft_depth.abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_chin_width_config();
        let mut s = new_chin_width_state();
        cw_set_width(&mut s, &cfg, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_chin_width_state();
        cw_set_taper(&mut s, 0.8);
        assert!((s.taper - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_cleft() {
        let mut s = new_chin_width_state();
        cw_set_cleft(&mut s, 0.4);
        assert!((s.cleft_depth - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_chin_width_config();
        let mut s = new_chin_width_state();
        cw_set_width(&mut s, &cfg, 0.9);
        cw_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_chin_width_state();
        assert_eq!(cw_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_chin_width_state();
        let j = cw_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("taper"));
    }

    #[test]
    fn test_blend() {
        let a = new_chin_width_state();
        let mut b = new_chin_width_state();
        b.width = 1.0;
        let mid = cw_blend(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_effective_width() {
        let s = new_chin_width_state();
        let w = cw_effective_width(&s);
        assert!(w > 0.0);
    }
}
