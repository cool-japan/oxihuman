// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Lower eyelid morph controls for infraorbital contour and bags.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LowerEyelidControlConfig {
    pub puffiness: f32,
    pub tightness: f32,
    pub crease: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LowerEyelidControlState {
    pub puffiness: f32,
    pub tightness: f32,
    pub crease: f32,
    pub droop: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LowerEyelidControlWeights {
    pub puffy: f32,
    pub tight: f32,
    pub creased: f32,
    pub droopy: f32,
    pub smooth: f32,
}

#[allow(dead_code)]
pub fn default_lower_eyelid_control_config() -> LowerEyelidControlConfig {
    LowerEyelidControlConfig { puffiness: 0.5, tightness: 0.5, crease: 0.5 }
}

#[allow(dead_code)]
pub fn new_lower_eyelid_control_state() -> LowerEyelidControlState {
    LowerEyelidControlState { puffiness: 0.5, tightness: 0.5, crease: 0.5, droop: 0.5 }
}

#[allow(dead_code)]
pub fn set_lower_eyelid_control_puffiness(state: &mut LowerEyelidControlState, value: f32) {
    state.puffiness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lower_eyelid_control_tightness(state: &mut LowerEyelidControlState, value: f32) {
    state.tightness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lower_eyelid_control_crease(state: &mut LowerEyelidControlState, value: f32) {
    state.crease = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lower_eyelid_control_droop(state: &mut LowerEyelidControlState, value: f32) {
    state.droop = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_lower_eyelid_control_weights(state: &LowerEyelidControlState, cfg: &LowerEyelidControlConfig) -> LowerEyelidControlWeights {
    let puffy = (state.puffiness * cfg.puffiness * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let tight = (state.tightness * cfg.tightness).clamp(0.0, 1.0);
    let creased = (state.crease * cfg.crease).clamp(0.0, 1.0);
    let droopy = state.droop.clamp(0.0, 1.0);
    let smooth = (1.0 - state.puffiness).clamp(0.0, 1.0);
    LowerEyelidControlWeights { puffy, tight, creased, droopy, smooth }
}

#[allow(dead_code)]
pub fn lower_eyelid_control_to_json(state: &LowerEyelidControlState) -> String {
    format!(
        r#"{{\"puffiness\":{},\"tightness\":{},\"crease\":{},\"droop\":{}}}"#,
        state.puffiness, state.tightness, state.crease, state.droop
    )
}

#[allow(dead_code)]
pub fn blend_lower_eyelid_controls(a: &LowerEyelidControlState, b: &LowerEyelidControlState, t: f32) -> LowerEyelidControlState {
    let t = t.clamp(0.0, 1.0);
    LowerEyelidControlState {
        puffiness: a.puffiness + (b.puffiness - a.puffiness) * t,
        tightness: a.tightness + (b.tightness - a.tightness) * t,
        crease: a.crease + (b.crease - a.crease) * t,
        droop: a.droop + (b.droop - a.droop) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lower_eyelid_control_config();
        assert!((0.0..=1.0).contains(&cfg.puffiness));
    }

    #[test]
    fn test_new_state() {
        let s = new_lower_eyelid_control_state();
        assert!((s.puffiness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_puffiness_clamp() {
        let mut s = new_lower_eyelid_control_state();
        set_lower_eyelid_control_puffiness(&mut s, 1.5);
        assert!((s.puffiness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_tightness() {
        let mut s = new_lower_eyelid_control_state();
        set_lower_eyelid_control_tightness(&mut s, 0.8);
        assert!((s.tightness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_crease() {
        let mut s = new_lower_eyelid_control_state();
        set_lower_eyelid_control_crease(&mut s, 0.7);
        assert!((s.crease - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_droop() {
        let mut s = new_lower_eyelid_control_state();
        set_lower_eyelid_control_droop(&mut s, 0.6);
        assert!((s.droop - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_lower_eyelid_control_state();
        let cfg = default_lower_eyelid_control_config();
        let w = compute_lower_eyelid_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.puffy));
        assert!((0.0..=1.0).contains(&w.tight));
    }

    #[test]
    fn test_to_json() {
        let s = new_lower_eyelid_control_state();
        let json = lower_eyelid_control_to_json(&s);
        assert!(json.contains("puffiness"));
        assert!(json.contains("droop"));
    }

    #[test]
    fn test_blend() {
        let a = new_lower_eyelid_control_state();
        let mut b = new_lower_eyelid_control_state();
        b.puffiness = 1.0;
        let mid = blend_lower_eyelid_controls(&a, &b, 0.5);
        assert!((mid.puffiness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_lower_eyelid_control_state();
        let r = blend_lower_eyelid_controls(&a, &a, 0.5);
        assert!((r.puffiness - a.puffiness).abs() < 1e-6);
    }
}
