// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Torso shape morphology controls for body customization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorsoConfig {
    pub chest_depth: f32,
    pub waist_width: f32,
    pub hip_width: f32,
    pub torso_length: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorsoState {
    pub chest_size: f32,
    pub waist_size: f32,
    pub hip_size: f32,
    pub torso_length: f32,
    pub posture: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorsoMorphWeights {
    pub chest_wide: f32,
    pub chest_narrow: f32,
    pub waist_wide: f32,
    pub waist_narrow: f32,
    pub hip_wide: f32,
    pub hip_narrow: f32,
}

#[allow(dead_code)]
pub fn default_torso_config() -> TorsoConfig {
    TorsoConfig {
        chest_depth: 0.5,
        waist_width: 0.5,
        hip_width: 0.5,
        torso_length: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_torso_state() -> TorsoState {
    TorsoState {
        chest_size: 0.5,
        waist_size: 0.5,
        hip_size: 0.5,
        torso_length: 1.0,
        posture: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_chest_size(state: &mut TorsoState, size: f32) {
    state.chest_size = size.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_waist_size(state: &mut TorsoState, size: f32) {
    state.waist_size = size.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_hip_size(state: &mut TorsoState, size: f32) {
    state.hip_size = size.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_torso_length(state: &mut TorsoState, len: f32) {
    state.torso_length = len.clamp(0.1, 3.0);
}

#[allow(dead_code)]
pub fn set_posture(state: &mut TorsoState, posture: f32) {
    state.posture = posture.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_torso_weights(state: &TorsoState, _cfg: &TorsoConfig) -> TorsoMorphWeights {
    let chest_wide = (state.chest_size - 0.5).max(0.0) * 2.0;
    let chest_narrow = (0.5 - state.chest_size).max(0.0) * 2.0;
    let waist_wide = (state.waist_size - 0.5).max(0.0) * 2.0;
    let waist_narrow = (0.5 - state.waist_size).max(0.0) * 2.0;
    let hip_wide = (state.hip_size - 0.5).max(0.0) * 2.0;
    let hip_narrow = (0.5 - state.hip_size).max(0.0) * 2.0;
    TorsoMorphWeights {
        chest_wide,
        chest_narrow,
        waist_wide,
        waist_narrow,
        hip_wide,
        hip_narrow,
    }
}

#[allow(dead_code)]
pub fn blend_torso(a: &TorsoState, b: &TorsoState, t: f32) -> TorsoState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    TorsoState {
        chest_size: a.chest_size * u + b.chest_size * t,
        waist_size: a.waist_size * u + b.waist_size * t,
        hip_size: a.hip_size * u + b.hip_size * t,
        torso_length: a.torso_length * u + b.torso_length * t,
        posture: a.posture * u + b.posture * t,
    }
}

#[allow(dead_code)]
pub fn reset_torso(state: &mut TorsoState) {
    *state = new_torso_state();
}

#[allow(dead_code)]
pub fn torso_state_to_json(state: &TorsoState) -> String {
    format!(
        r#"{{"chest_size":{:.4},"waist_size":{:.4},"hip_size":{:.4},"torso_length":{:.4},"posture":{:.4}}}"#,
        state.chest_size, state.waist_size, state.hip_size, state.torso_length, state.posture
    )
}

#[allow(dead_code)]
pub fn waist_to_hip_ratio(state: &TorsoState) -> f32 {
    if state.hip_size == 0.0 {
        0.0
    } else {
        state.waist_size / state.hip_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_torso_config() {
        let cfg = default_torso_config();
        assert!((cfg.chest_depth - 0.5).abs() < 1e-5);
        assert!((cfg.torso_length - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_torso_state() {
        let s = new_torso_state();
        assert!((s.chest_size - 0.5).abs() < 1e-5);
        assert!((s.posture).abs() < 1e-5);
    }

    #[test]
    fn test_set_chest_size_clamp() {
        let mut s = new_torso_state();
        set_chest_size(&mut s, 2.0);
        assert!((s.chest_size - 1.0).abs() < 1e-5);
        set_chest_size(&mut s, -1.0);
        assert!(s.chest_size.abs() < 1e-5);
    }

    #[test]
    fn test_set_waist_and_hip() {
        let mut s = new_torso_state();
        set_waist_size(&mut s, 0.3);
        set_hip_size(&mut s, 0.7);
        assert!((s.waist_size - 0.3).abs() < 1e-5);
        assert!((s.hip_size - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_set_posture_clamp() {
        let mut s = new_torso_state();
        set_posture(&mut s, 5.0);
        assert!((s.posture - 1.0).abs() < 1e-5);
        set_posture(&mut s, -5.0);
        assert!((s.posture + 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_torso_weights() {
        let cfg = default_torso_config();
        let mut s = new_torso_state();
        set_chest_size(&mut s, 1.0);
        let w = compute_torso_weights(&s, &cfg);
        assert!((w.chest_wide - 1.0).abs() < 1e-5);
        assert!(w.chest_narrow.abs() < 1e-5);
    }

    #[test]
    fn test_blend_torso() {
        let a = new_torso_state();
        let mut b = new_torso_state();
        set_chest_size(&mut b, 1.0);
        let blended = blend_torso(&a, &b, 0.5);
        assert!((blended.chest_size - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_reset_torso() {
        let mut s = new_torso_state();
        set_chest_size(&mut s, 0.9);
        reset_torso(&mut s);
        assert!((s.chest_size - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_torso_state_to_json() {
        let s = new_torso_state();
        let json = torso_state_to_json(&s);
        assert!(json.contains("chest_size"));
        assert!(json.contains("waist_size"));
    }

    #[test]
    fn test_waist_to_hip_ratio() {
        let mut s = new_torso_state();
        set_waist_size(&mut s, 0.7);
        set_hip_size(&mut s, 1.0);
        let ratio = waist_to_hip_ratio(&s);
        assert!((ratio - 0.7).abs() < 1e-4);
    }

    #[test]
    fn test_waist_to_hip_ratio_zero_hip() {
        let mut s = new_torso_state();
        set_hip_size(&mut s, 0.0);
        let ratio = waist_to_hip_ratio(&s);
        assert!(ratio.abs() < 1e-5);
    }
}
