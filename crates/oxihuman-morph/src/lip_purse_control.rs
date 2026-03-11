// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lip purse control — orbicularis oris contraction and lip protrusion.

/// Configuration for lip purse.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipPurseConfig {
    pub max_purse: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipPurseState {
    pub upper_purse: f32,
    pub lower_purse: f32,
    pub protrusion: f32,
}

#[allow(dead_code)]
pub fn default_lip_purse_config() -> LipPurseConfig {
    LipPurseConfig { max_purse: 1.0 }
}

#[allow(dead_code)]
pub fn new_lip_purse_state() -> LipPurseState {
    LipPurseState {
        upper_purse: 0.0,
        lower_purse: 0.0,
        protrusion: 0.0,
    }
}

#[allow(dead_code)]
pub fn lpur_set_upper(state: &mut LipPurseState, cfg: &LipPurseConfig, v: f32) {
    state.upper_purse = v.clamp(0.0, cfg.max_purse);
}

#[allow(dead_code)]
pub fn lpur_set_lower(state: &mut LipPurseState, cfg: &LipPurseConfig, v: f32) {
    state.lower_purse = v.clamp(0.0, cfg.max_purse);
}

#[allow(dead_code)]
pub fn lpur_set_both(state: &mut LipPurseState, cfg: &LipPurseConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_purse);
    state.upper_purse = clamped;
    state.lower_purse = clamped;
}

#[allow(dead_code)]
pub fn lpur_set_protrusion(state: &mut LipPurseState, v: f32) {
    state.protrusion = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn lpur_reset(state: &mut LipPurseState) {
    *state = new_lip_purse_state();
}

#[allow(dead_code)]
pub fn lpur_is_neutral(state: &LipPurseState) -> bool {
    state.upper_purse.abs() < 1e-6
        && state.lower_purse.abs() < 1e-6
        && state.protrusion.abs() < 1e-6
}

#[allow(dead_code)]
pub fn lpur_intensity(state: &LipPurseState) -> f32 {
    (state.upper_purse + state.lower_purse) * 0.5
}

#[allow(dead_code)]
pub fn lpur_blend(a: &LipPurseState, b: &LipPurseState, t: f32) -> LipPurseState {
    let t = t.clamp(0.0, 1.0);
    LipPurseState {
        upper_purse: a.upper_purse + (b.upper_purse - a.upper_purse) * t,
        lower_purse: a.lower_purse + (b.lower_purse - a.lower_purse) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
    }
}

#[allow(dead_code)]
pub fn lpur_to_weights(state: &LipPurseState) -> Vec<(String, f32)> {
    vec![
        ("lip_purse_upper".to_string(), state.upper_purse),
        ("lip_purse_lower".to_string(), state.lower_purse),
        ("lip_protrusion".to_string(), state.protrusion),
    ]
}

#[allow(dead_code)]
pub fn lpur_to_json(state: &LipPurseState) -> String {
    format!(
        r#"{{"upper_purse":{:.4},"lower_purse":{:.4},"protrusion":{:.4}}}"#,
        state.upper_purse, state.lower_purse, state.protrusion
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_lip_purse_config();
        assert!((cfg.max_purse - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_lip_purse_state();
        assert!(lpur_is_neutral(&s));
    }

    #[test]
    fn set_upper_clamps() {
        let cfg = default_lip_purse_config();
        let mut s = new_lip_purse_state();
        lpur_set_upper(&mut s, &cfg, 5.0);
        assert!((s.upper_purse - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_lower() {
        let cfg = default_lip_purse_config();
        let mut s = new_lip_purse_state();
        lpur_set_lower(&mut s, &cfg, 0.4);
        assert!((s.lower_purse - 0.4).abs() < 1e-6);
    }

    #[test]
    fn set_both() {
        let cfg = default_lip_purse_config();
        let mut s = new_lip_purse_state();
        lpur_set_both(&mut s, &cfg, 0.6);
        assert!((lpur_intensity(&s) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_protrusion() {
        let mut s = new_lip_purse_state();
        lpur_set_protrusion(&mut s, 0.5);
        assert!((s.protrusion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_lip_purse_config();
        let mut s = new_lip_purse_state();
        lpur_set_both(&mut s, &cfg, 0.8);
        lpur_reset(&mut s);
        assert!(lpur_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_lip_purse_state();
        let cfg = default_lip_purse_config();
        let mut b = new_lip_purse_state();
        lpur_set_both(&mut b, &cfg, 1.0);
        let m = lpur_blend(&a, &b, 0.5);
        assert!((m.upper_purse - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_lip_purse_state();
        assert_eq!(lpur_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_lip_purse_state();
        let j = lpur_to_json(&s);
        assert!(j.contains("upper_purse"));
    }
}
