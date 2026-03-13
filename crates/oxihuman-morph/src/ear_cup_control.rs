// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear-cup (protruding ear angle) control.

/// Side.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EarCupSide {
    Left,
    Right,
    Both,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EarCupState {
    /// Cup/protrusion angle in normalised [0..1] range (0 = flat, 1 = max cup).
    pub cup_left: f32,
    pub cup_right: f32,
    /// Top vs bottom cup bias (-1..1; 0 = uniform).
    pub bias_left: f32,
    pub bias_right: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EarCupConfig {
    pub max_cup: f32,
}

impl Default for EarCupConfig {
    fn default() -> Self {
        Self { max_cup: 1.0 }
    }
}

impl Default for EarCupState {
    fn default() -> Self {
        Self {
            cup_left: 0.0,
            cup_right: 0.0,
            bias_left: 0.0,
            bias_right: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_ear_cup_state() -> EarCupState {
    EarCupState::default()
}

#[allow(dead_code)]
pub fn default_ear_cup_config() -> EarCupConfig {
    EarCupConfig::default()
}

#[allow(dead_code)]
pub fn ec_set_cup(state: &mut EarCupState, cfg: &EarCupConfig, side: EarCupSide, v: f32) {
    let v = v.clamp(0.0, cfg.max_cup);
    match side {
        EarCupSide::Left => state.cup_left = v,
        EarCupSide::Right => state.cup_right = v,
        EarCupSide::Both => {
            state.cup_left = v;
            state.cup_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn ec_set_bias(state: &mut EarCupState, side: EarCupSide, bias: f32) {
    let b = bias.clamp(-1.0, 1.0);
    match side {
        EarCupSide::Left => state.bias_left = b,
        EarCupSide::Right => state.bias_right = b,
        EarCupSide::Both => {
            state.bias_left = b;
            state.bias_right = b;
        }
    }
}

#[allow(dead_code)]
pub fn ec_reset(state: &mut EarCupState) {
    *state = EarCupState::default();
}

#[allow(dead_code)]
pub fn ec_is_neutral(state: &EarCupState) -> bool {
    state.cup_left < 1e-4 && state.cup_right < 1e-4
}

#[allow(dead_code)]
pub fn ec_blend(a: &EarCupState, b: &EarCupState, t: f32) -> EarCupState {
    let t = t.clamp(0.0, 1.0);
    EarCupState {
        cup_left: a.cup_left + (b.cup_left - a.cup_left) * t,
        cup_right: a.cup_right + (b.cup_right - a.cup_right) * t,
        bias_left: a.bias_left + (b.bias_left - a.bias_left) * t,
        bias_right: a.bias_right + (b.bias_right - a.bias_right) * t,
    }
}

#[allow(dead_code)]
pub fn ec_symmetry(state: &EarCupState) -> f32 {
    1.0 - (state.cup_left - state.cup_right).abs().min(1.0)
}

#[allow(dead_code)]
pub fn ec_average_cup(state: &EarCupState) -> f32 {
    (state.cup_left + state.cup_right) * 0.5
}

#[allow(dead_code)]
pub fn ec_to_weights(state: &EarCupState) -> [f32; 4] {
    [
        state.cup_left,
        state.cup_right,
        state.bias_left,
        state.bias_right,
    ]
}

#[allow(dead_code)]
pub fn ec_to_json(state: &EarCupState) -> String {
    format!(
        "{{\"cup_left\":{:.4},\"cup_right\":{:.4},\"bias_left\":{:.4},\"bias_right\":{:.4}}}",
        state.cup_left, state.cup_right, state.bias_left, state.bias_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(ec_is_neutral(&new_ear_cup_state()));
    }

    #[test]
    fn set_cup_clamps_max() {
        let mut s = new_ear_cup_state();
        let cfg = default_ear_cup_config();
        ec_set_cup(&mut s, &cfg, EarCupSide::Left, 99.0);
        assert!(s.cup_left <= cfg.max_cup);
    }

    #[test]
    fn set_cup_not_negative() {
        let mut s = new_ear_cup_state();
        let cfg = default_ear_cup_config();
        ec_set_cup(&mut s, &cfg, EarCupSide::Right, -1.0);
        assert!(s.cup_right >= 0.0);
    }

    #[test]
    fn both_sides_set() {
        let mut s = new_ear_cup_state();
        let cfg = default_ear_cup_config();
        ec_set_cup(&mut s, &cfg, EarCupSide::Both, 0.5);
        assert!((s.cup_left - s.cup_right).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_ear_cup_state();
        let cfg = default_ear_cup_config();
        ec_set_cup(&mut s, &cfg, EarCupSide::Both, 0.8);
        ec_reset(&mut s);
        assert!(ec_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_ear_cup_config();
        let mut a = new_ear_cup_state();
        let mut b = new_ear_cup_state();
        ec_set_cup(&mut a, &cfg, EarCupSide::Left, 0.0);
        ec_set_cup(&mut b, &cfg, EarCupSide::Left, 1.0);
        let m = ec_blend(&a, &b, 0.5);
        assert!((m.cup_left - 0.5).abs() < 1e-4);
    }

    #[test]
    fn symmetry_one_equal() {
        let s = new_ear_cup_state();
        assert!((ec_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_cup_zero_default() {
        assert!((ec_average_cup(&new_ear_cup_state())).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(ec_to_weights(&new_ear_cup_state()).len(), 4);
    }

    #[test]
    fn json_has_cup_left() {
        assert!(ec_to_json(&new_ear_cup_state()).contains("cup_left"));
    }
}
