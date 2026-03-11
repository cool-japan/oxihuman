// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Forehead protrusion control: adjusts frontal bossing and forehead slope.

use std::f32::consts::FRAC_PI_3;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadProtrusionConfig {
    pub min_protrusion: f32,
    pub max_protrusion: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadProtrusionState {
    pub protrusion: f32,
    pub slope: f32,
    pub bossing: f32,
}

#[allow(dead_code)]
pub fn default_forehead_protrusion_config() -> ForeheadProtrusionConfig {
    ForeheadProtrusionConfig {
        min_protrusion: 0.0,
        max_protrusion: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_forehead_protrusion_state() -> ForeheadProtrusionState {
    ForeheadProtrusionState {
        protrusion: 0.5,
        slope: 0.5,
        bossing: 0.3,
    }
}

#[allow(dead_code)]
pub fn fp_set_protrusion(
    state: &mut ForeheadProtrusionState,
    cfg: &ForeheadProtrusionConfig,
    v: f32,
) {
    state.protrusion = v.clamp(cfg.min_protrusion, cfg.max_protrusion);
}

#[allow(dead_code)]
pub fn fp_set_slope(state: &mut ForeheadProtrusionState, v: f32) {
    state.slope = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fp_set_bossing(state: &mut ForeheadProtrusionState, v: f32) {
    state.bossing = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fp_reset(state: &mut ForeheadProtrusionState) {
    *state = new_forehead_protrusion_state();
}

#[allow(dead_code)]
pub fn fp_slope_angle(state: &ForeheadProtrusionState) -> f32 {
    state.slope * FRAC_PI_3
}

#[allow(dead_code)]
pub fn fp_to_weights(state: &ForeheadProtrusionState) -> Vec<(String, f32)> {
    vec![
        ("forehead_protrusion".to_string(), state.protrusion),
        ("forehead_slope".to_string(), state.slope),
        ("forehead_bossing".to_string(), state.bossing),
    ]
}

#[allow(dead_code)]
pub fn fp_to_json(state: &ForeheadProtrusionState) -> String {
    format!(
        r#"{{"protrusion":{:.4},"slope":{:.4},"bossing":{:.4}}}"#,
        state.protrusion, state.slope, state.bossing
    )
}

#[allow(dead_code)]
pub fn fp_blend(
    a: &ForeheadProtrusionState,
    b: &ForeheadProtrusionState,
    t: f32,
) -> ForeheadProtrusionState {
    let t = t.clamp(0.0, 1.0);
    ForeheadProtrusionState {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        slope: a.slope + (b.slope - a.slope) * t,
        bossing: a.bossing + (b.bossing - a.bossing) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_forehead_protrusion_config();
        assert!(cfg.min_protrusion.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_forehead_protrusion_state();
        assert!((s.protrusion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion_clamps() {
        let cfg = default_forehead_protrusion_config();
        let mut s = new_forehead_protrusion_state();
        fp_set_protrusion(&mut s, &cfg, 5.0);
        assert!((s.protrusion - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_slope() {
        let mut s = new_forehead_protrusion_state();
        fp_set_slope(&mut s, 0.8);
        assert!((s.slope - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_bossing() {
        let mut s = new_forehead_protrusion_state();
        fp_set_bossing(&mut s, 0.7);
        assert!((s.bossing - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_forehead_protrusion_config();
        let mut s = new_forehead_protrusion_state();
        fp_set_protrusion(&mut s, &cfg, 0.9);
        fp_reset(&mut s);
        assert!((s.protrusion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_slope_angle() {
        let mut s = new_forehead_protrusion_state();
        s.slope = 1.0;
        assert!((fp_slope_angle(&s) - FRAC_PI_3).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_forehead_protrusion_state();
        assert_eq!(fp_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_forehead_protrusion_state();
        let mut b = new_forehead_protrusion_state();
        b.protrusion = 1.0;
        let mid = fp_blend(&a, &b, 0.5);
        assert!((mid.protrusion - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_forehead_protrusion_state();
        let j = fp_to_json(&s);
        assert!(j.contains("protrusion"));
    }
}
