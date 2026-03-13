// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Inner thigh morphology controls for gap, fullness, and muscle tone.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InnerThighConfig {
    pub fullness: f32,
    pub gap: f32,
    pub muscle_tone: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InnerThighState {
    pub fullness: f32,
    pub gap: f32,
    pub muscle_tone: f32,
    pub skin_tightness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InnerThighWeights {
    pub full: f32,
    pub slim: f32,
    pub gap_present: f32,
    pub toned: f32,
    pub tight_skin: f32,
}

#[allow(dead_code)]
pub fn default_inner_thigh_config() -> InnerThighConfig {
    InnerThighConfig { fullness: 0.5, gap: 0.3, muscle_tone: 0.5 }
}

#[allow(dead_code)]
pub fn new_inner_thigh_state() -> InnerThighState {
    InnerThighState { fullness: 0.5, gap: 0.3, muscle_tone: 0.5, skin_tightness: 0.5 }
}

#[allow(dead_code)]
pub fn set_inner_thigh_fullness(state: &mut InnerThighState, value: f32) {
    state.fullness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_inner_thigh_gap(state: &mut InnerThighState, value: f32) {
    state.gap = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_inner_thigh_tone(state: &mut InnerThighState, value: f32) {
    state.muscle_tone = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_inner_thigh_tightness(state: &mut InnerThighState, value: f32) {
    state.skin_tightness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_inner_thigh_weights(state: &InnerThighState, cfg: &InnerThighConfig) -> InnerThighWeights {
    let f = state.fullness * cfg.fullness;
    let full = (f * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let slim = (1.0 - f).clamp(0.0, 1.0);
    let gap_present = (state.gap * cfg.gap).clamp(0.0, 1.0);
    let toned = (state.muscle_tone * cfg.muscle_tone).clamp(0.0, 1.0);
    let tight_skin = state.skin_tightness.clamp(0.0, 1.0);
    InnerThighWeights { full, slim, gap_present, toned, tight_skin }
}

#[allow(dead_code)]
pub fn inner_thigh_to_json(state: &InnerThighState) -> String {
    format!(
        r#"{{"fullness":{},"gap":{},"muscle_tone":{},"skin_tightness":{}}}"#,
        state.fullness, state.gap, state.muscle_tone, state.skin_tightness
    )
}

#[allow(dead_code)]
pub fn blend_inner_thighs(a: &InnerThighState, b: &InnerThighState, t: f32) -> InnerThighState {
    let t = t.clamp(0.0, 1.0);
    InnerThighState {
        fullness: a.fullness + (b.fullness - a.fullness) * t,
        gap: a.gap + (b.gap - a.gap) * t,
        muscle_tone: a.muscle_tone + (b.muscle_tone - a.muscle_tone) * t,
        skin_tightness: a.skin_tightness + (b.skin_tightness - a.skin_tightness) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_inner_thigh_config();
        assert!((0.0..=1.0).contains(&cfg.fullness));
    }

    #[test]
    fn test_new_state() {
        let s = new_inner_thigh_state();
        assert!((s.fullness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_fullness_clamp() {
        let mut s = new_inner_thigh_state();
        set_inner_thigh_fullness(&mut s, 1.5);
        assert!((s.fullness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_gap() {
        let mut s = new_inner_thigh_state();
        set_inner_thigh_gap(&mut s, 0.8);
        assert!((s.gap - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_tone() {
        let mut s = new_inner_thigh_state();
        set_inner_thigh_tone(&mut s, 0.7);
        assert!((s.muscle_tone - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_tightness() {
        let mut s = new_inner_thigh_state();
        set_inner_thigh_tightness(&mut s, 0.6);
        assert!((s.skin_tightness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_inner_thigh_state();
        let cfg = default_inner_thigh_config();
        let w = compute_inner_thigh_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.full));
        assert!((0.0..=1.0).contains(&w.toned));
    }

    #[test]
    fn test_to_json() {
        let s = new_inner_thigh_state();
        let json = inner_thigh_to_json(&s);
        assert!(json.contains("fullness"));
        assert!(json.contains("skin_tightness"));
    }

    #[test]
    fn test_blend() {
        let a = new_inner_thigh_state();
        let mut b = new_inner_thigh_state();
        b.fullness = 1.0;
        let mid = blend_inner_thighs(&a, &b, 0.5);
        assert!((mid.fullness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_inner_thigh_state();
        let r = blend_inner_thighs(&a, &a, 0.5);
        assert!((r.fullness - a.fullness).abs() < 1e-6);
    }
}
