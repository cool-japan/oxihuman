// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cheek rise control — zygomaticus major elevation under the eyes.

/// Configuration for cheek rise.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekRiseConfig {
    pub max_rise: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheekRiseSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekRiseState {
    pub left_rise: f32,
    pub right_rise: f32,
}

#[allow(dead_code)]
pub fn default_cheek_rise_config() -> CheekRiseConfig {
    CheekRiseConfig { max_rise: 1.0 }
}

#[allow(dead_code)]
pub fn new_cheek_rise_state() -> CheekRiseState {
    CheekRiseState {
        left_rise: 0.0,
        right_rise: 0.0,
    }
}

#[allow(dead_code)]
pub fn cr_set_rise(state: &mut CheekRiseState, cfg: &CheekRiseConfig, side: CheekRiseSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_rise);
    match side {
        CheekRiseSide::Left => state.left_rise = clamped,
        CheekRiseSide::Right => state.right_rise = clamped,
    }
}

#[allow(dead_code)]
pub fn cr_set_both(state: &mut CheekRiseState, cfg: &CheekRiseConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_rise);
    state.left_rise = clamped;
    state.right_rise = clamped;
}

#[allow(dead_code)]
pub fn cr_reset(state: &mut CheekRiseState) {
    *state = new_cheek_rise_state();
}

#[allow(dead_code)]
pub fn cr_is_neutral(state: &CheekRiseState) -> bool {
    state.left_rise.abs() < 1e-6 && state.right_rise.abs() < 1e-6
}

#[allow(dead_code)]
pub fn cr_average(state: &CheekRiseState) -> f32 {
    (state.left_rise + state.right_rise) * 0.5
}

#[allow(dead_code)]
pub fn cr_symmetry(state: &CheekRiseState) -> f32 {
    (state.left_rise - state.right_rise).abs()
}

#[allow(dead_code)]
pub fn cr_blend(a: &CheekRiseState, b: &CheekRiseState, t: f32) -> CheekRiseState {
    let t = t.clamp(0.0, 1.0);
    CheekRiseState {
        left_rise: a.left_rise + (b.left_rise - a.left_rise) * t,
        right_rise: a.right_rise + (b.right_rise - a.right_rise) * t,
    }
}

#[allow(dead_code)]
pub fn cr_to_weights(state: &CheekRiseState) -> Vec<(String, f32)> {
    vec![
        ("cheek_rise_l".to_string(), state.left_rise),
        ("cheek_rise_r".to_string(), state.right_rise),
    ]
}

#[allow(dead_code)]
pub fn cr_to_json(state: &CheekRiseState) -> String {
    format!(
        r#"{{"left_rise":{:.4},"right_rise":{:.4}}}"#,
        state.left_rise, state.right_rise
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_cheek_rise_config();
        assert!((cfg.max_rise - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_cheek_rise_state();
        assert!(cr_is_neutral(&s));
    }

    #[test]
    fn set_rise_left() {
        let cfg = default_cheek_rise_config();
        let mut s = new_cheek_rise_state();
        cr_set_rise(&mut s, &cfg, CheekRiseSide::Left, 0.7);
        assert!((s.left_rise - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_rise_clamps() {
        let cfg = default_cheek_rise_config();
        let mut s = new_cheek_rise_state();
        cr_set_rise(&mut s, &cfg, CheekRiseSide::Right, 2.0);
        assert!((s.right_rise - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_symmetric() {
        let cfg = default_cheek_rise_config();
        let mut s = new_cheek_rise_state();
        cr_set_both(&mut s, &cfg, 0.5);
        assert!(cr_symmetry(&s) < 1e-6);
    }

    #[test]
    fn average_value() {
        let cfg = default_cheek_rise_config();
        let mut s = new_cheek_rise_state();
        cr_set_rise(&mut s, &cfg, CheekRiseSide::Left, 0.2);
        cr_set_rise(&mut s, &cfg, CheekRiseSide::Right, 0.8);
        assert!((cr_average(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_cheek_rise_config();
        let mut s = new_cheek_rise_state();
        cr_set_both(&mut s, &cfg, 0.9);
        cr_reset(&mut s);
        assert!(cr_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_cheek_rise_state();
        let cfg = default_cheek_rise_config();
        let mut b = new_cheek_rise_state();
        cr_set_both(&mut b, &cfg, 1.0);
        let m = cr_blend(&a, &b, 0.5);
        assert!((m.left_rise - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_cheek_rise_state();
        assert_eq!(cr_to_weights(&s).len(), 2);
    }

    #[test]
    fn to_json_contains_fields() {
        let s = new_cheek_rise_state();
        let j = cr_to_json(&s);
        assert!(j.contains("left_rise"));
    }
}
