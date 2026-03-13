// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Body fat distribution morph controls for regional adipose placement.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FatDistributionConfig {
    pub abdominal: f32,
    pub gluteal: f32,
    pub limb: f32,
    pub visceral_ratio: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FatDistributionState {
    pub abdominal: f32,
    pub gluteal: f32,
    pub limb: f32,
    pub visceral_ratio: f32,
    pub overall_fat: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FatDistributionWeights {
    pub belly: f32,
    pub hip: f32,
    pub arm: f32,
    pub leg: f32,
    pub visceral: f32,
}

#[allow(dead_code)]
pub fn default_fat_distribution_config() -> FatDistributionConfig {
    FatDistributionConfig { abdominal: 0.5, gluteal: 0.5, limb: 0.3, visceral_ratio: 0.2 }
}

#[allow(dead_code)]
pub fn new_fat_distribution_state() -> FatDistributionState {
    FatDistributionState { abdominal: 0.5, gluteal: 0.5, limb: 0.3, visceral_ratio: 0.2, overall_fat: 0.3 }
}

#[allow(dead_code)]
pub fn set_abdominal_fat(state: &mut FatDistributionState, value: f32) {
    state.abdominal = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gluteal_fat(state: &mut FatDistributionState, value: f32) {
    state.gluteal = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_limb_fat(state: &mut FatDistributionState, value: f32) {
    state.limb = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_overall_fat(state: &mut FatDistributionState, value: f32) {
    state.overall_fat = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_fat_weights(state: &FatDistributionState, cfg: &FatDistributionConfig) -> FatDistributionWeights {
    let belly = (state.abdominal * cfg.abdominal * state.overall_fat * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let hip = (state.gluteal * cfg.gluteal * state.overall_fat).clamp(0.0, 1.0);
    let arm = (state.limb * cfg.limb * state.overall_fat * 0.7).clamp(0.0, 1.0);
    let leg = (state.limb * cfg.limb * state.overall_fat).clamp(0.0, 1.0);
    let visceral = (state.visceral_ratio * cfg.visceral_ratio * state.overall_fat).clamp(0.0, 1.0);
    FatDistributionWeights { belly, hip, arm, leg, visceral }
}

#[allow(dead_code)]
pub fn fat_distribution_to_json(state: &FatDistributionState) -> String {
    format!(
        r#"{{"abdominal":{},"gluteal":{},"limb":{},"visceral_ratio":{},"overall_fat":{}}}"#,
        state.abdominal, state.gluteal, state.limb, state.visceral_ratio, state.overall_fat
    )
}

#[allow(dead_code)]
pub fn blend_fat_distributions(a: &FatDistributionState, b: &FatDistributionState, t: f32) -> FatDistributionState {
    let t = t.clamp(0.0, 1.0);
    FatDistributionState {
        abdominal: a.abdominal + (b.abdominal - a.abdominal) * t,
        gluteal: a.gluteal + (b.gluteal - a.gluteal) * t,
        limb: a.limb + (b.limb - a.limb) * t,
        visceral_ratio: a.visceral_ratio + (b.visceral_ratio - a.visceral_ratio) * t,
        overall_fat: a.overall_fat + (b.overall_fat - a.overall_fat) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_fat_distribution_config();
        assert!((0.0..=1.0).contains(&cfg.abdominal));
    }

    #[test]
    fn test_new_state() {
        let s = new_fat_distribution_state();
        assert!((s.abdominal - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_abdominal_clamp() {
        let mut s = new_fat_distribution_state();
        set_abdominal_fat(&mut s, 1.5);
        assert!((s.abdominal - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_gluteal() {
        let mut s = new_fat_distribution_state();
        set_gluteal_fat(&mut s, 0.8);
        assert!((s.gluteal - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_limb() {
        let mut s = new_fat_distribution_state();
        set_limb_fat(&mut s, 0.7);
        assert!((s.limb - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_overall() {
        let mut s = new_fat_distribution_state();
        set_overall_fat(&mut s, 0.6);
        assert!((s.overall_fat - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_fat_distribution_state();
        let cfg = default_fat_distribution_config();
        let w = compute_fat_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.belly));
        assert!((0.0..=1.0).contains(&w.hip));
    }

    #[test]
    fn test_to_json() {
        let s = new_fat_distribution_state();
        let json = fat_distribution_to_json(&s);
        assert!(json.contains("abdominal"));
        assert!(json.contains("overall_fat"));
    }

    #[test]
    fn test_blend() {
        let a = new_fat_distribution_state();
        let mut b = new_fat_distribution_state();
        b.abdominal = 1.0;
        let mid = blend_fat_distributions(&a, &b, 0.5);
        assert!((mid.abdominal - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_fat_distribution_state();
        let r = blend_fat_distributions(&a, &a, 0.5);
        assert!((r.abdominal - a.abdominal).abs() < 1e-6);
    }
}
