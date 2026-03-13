// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cheek sag control — inferior gravitational ptosis of cheek soft tissue.

/// Side of the face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SagSide {
    Left,
    Right,
}

/// Config for cheek sag.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CheekSagConfig {
    pub max_sag_m: f32,
}

impl Default for CheekSagConfig {
    fn default() -> Self {
        Self { max_sag_m: 0.012 }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CheekSagState {
    pub left: f32,
    pub right: f32,
}

#[allow(dead_code)]
pub fn new_cheek_sag_state() -> CheekSagState {
    CheekSagState::default()
}

#[allow(dead_code)]
pub fn default_cheek_sag_config() -> CheekSagConfig {
    CheekSagConfig::default()
}

/// Set sag for one side, 0..=1.
#[allow(dead_code)]
pub fn csag_set(state: &mut CheekSagState, side: SagSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        SagSide::Left => state.left = v,
        SagSide::Right => state.right = v,
    }
}

/// Set both sides.
#[allow(dead_code)]
pub fn csag_set_both(state: &mut CheekSagState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

/// Reset.
#[allow(dead_code)]
pub fn csag_reset(state: &mut CheekSagState) {
    *state = CheekSagState::default();
}

/// Whether neutral.
#[allow(dead_code)]
pub fn csag_is_neutral(state: &CheekSagState) -> bool {
    state.left < 1e-4 && state.right < 1e-4
}

/// Asymmetry between left and right.
#[allow(dead_code)]
pub fn csag_asymmetry(state: &CheekSagState) -> f32 {
    (state.left - state.right).abs()
}

/// Average sag.
#[allow(dead_code)]
pub fn csag_average(state: &CheekSagState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Downward displacement in metres.
#[allow(dead_code)]
pub fn csag_to_weights(state: &CheekSagState, cfg: &CheekSagConfig) -> [f32; 2] {
    [state.left * cfg.max_sag_m, state.right * cfg.max_sag_m]
}

/// Blend two states.
#[allow(dead_code)]
pub fn csag_blend(a: &CheekSagState, b: &CheekSagState, t: f32) -> CheekSagState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    CheekSagState {
        left: a.left * inv + b.left * t,
        right: a.right * inv + b.right * t,
    }
}

/// JSON string.
#[allow(dead_code)]
pub fn csag_to_json(state: &CheekSagState) -> String {
    format!(
        "{{\"left\":{:.4},\"right\":{:.4}}}",
        state.left, state.right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(csag_is_neutral(&new_cheek_sag_state()));
    }

    #[test]
    fn set_clamps_above_one() {
        let mut s = new_cheek_sag_state();
        csag_set(&mut s, SagSide::Left, 5.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_below_zero() {
        let mut s = new_cheek_sag_state();
        csag_set(&mut s, SagSide::Right, -1.0);
        assert!(s.right < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_cheek_sag_state();
        csag_set_both(&mut s, 0.8);
        csag_reset(&mut s);
        assert!(csag_is_neutral(&s));
    }

    #[test]
    fn asymmetry_when_unequal() {
        let mut s = new_cheek_sag_state();
        csag_set(&mut s, SagSide::Left, 1.0);
        csag_set(&mut s, SagSide::Right, 0.0);
        assert!(csag_asymmetry(&s) > 0.9);
    }

    #[test]
    fn average_midpoint() {
        let mut s = new_cheek_sag_state();
        csag_set_both(&mut s, 0.4);
        assert!((csag_average(&s) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn weights_proportional() {
        let cfg = default_cheek_sag_config();
        let mut s = new_cheek_sag_state();
        csag_set_both(&mut s, 1.0);
        let w = csag_to_weights(&s, &cfg);
        assert!((w[0] - cfg.max_sag_m).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_cheek_sag_state();
        let mut b = new_cheek_sag_state();
        csag_set_both(&mut a, 0.0);
        csag_set_both(&mut b, 1.0);
        let r = csag_blend(&a, &b, 0.5);
        assert!((r.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = csag_to_json(&new_cheek_sag_state());
        assert!(j.contains("left") && j.contains("right"));
    }

    #[test]
    fn set_both_equal() {
        let mut s = new_cheek_sag_state();
        csag_set_both(&mut s, 0.6);
        assert!((s.left - s.right).abs() < 1e-6);
    }
}
