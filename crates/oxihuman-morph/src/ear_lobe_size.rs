// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear lobe size control — volume and pendulousness of the ear lobe.

/// Ear side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EarLobeSizeConfig {
    pub max_size_m: f32,
    pub max_droop_m: f32,
}

impl Default for EarLobeSizeConfig {
    fn default() -> Self {
        Self {
            max_size_m: 0.010,
            max_droop_m: 0.008,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EarLobeSizeState {
    pub left_size: f32,
    pub right_size: f32,
    pub left_droop: f32,
    pub right_droop: f32,
}

#[allow(dead_code)]
pub fn new_ear_lobe_size_state() -> EarLobeSizeState {
    EarLobeSizeState::default()
}

#[allow(dead_code)]
pub fn default_ear_lobe_size_config() -> EarLobeSizeConfig {
    EarLobeSizeConfig::default()
}

#[allow(dead_code)]
pub fn els_set_size(state: &mut EarLobeSizeState, side: EarSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        EarSide::Left => state.left_size = v,
        EarSide::Right => state.right_size = v,
    }
}

#[allow(dead_code)]
pub fn els_set_droop(state: &mut EarLobeSizeState, side: EarSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        EarSide::Left => state.left_droop = v,
        EarSide::Right => state.right_droop = v,
    }
}

#[allow(dead_code)]
pub fn els_set_both_size(state: &mut EarLobeSizeState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left_size = v;
    state.right_size = v;
}

#[allow(dead_code)]
pub fn els_reset(state: &mut EarLobeSizeState) {
    *state = EarLobeSizeState::default();
}

#[allow(dead_code)]
pub fn els_is_neutral(state: &EarLobeSizeState) -> bool {
    state.left_size < 1e-4
        && state.right_size < 1e-4
        && state.left_droop < 1e-4
        && state.right_droop < 1e-4
}

#[allow(dead_code)]
pub fn els_symmetry(state: &EarLobeSizeState) -> f32 {
    1.0 - (state.left_size - state.right_size).abs()
}

#[allow(dead_code)]
pub fn els_to_weights(state: &EarLobeSizeState, cfg: &EarLobeSizeConfig) -> [f32; 4] {
    [
        state.left_size * cfg.max_size_m,
        state.right_size * cfg.max_size_m,
        state.left_droop * cfg.max_droop_m,
        state.right_droop * cfg.max_droop_m,
    ]
}

#[allow(dead_code)]
pub fn els_blend(a: &EarLobeSizeState, b: &EarLobeSizeState, t: f32) -> EarLobeSizeState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    EarLobeSizeState {
        left_size: a.left_size * inv + b.left_size * t,
        right_size: a.right_size * inv + b.right_size * t,
        left_droop: a.left_droop * inv + b.left_droop * t,
        right_droop: a.right_droop * inv + b.right_droop * t,
    }
}

#[allow(dead_code)]
pub fn els_to_json(state: &EarLobeSizeState) -> String {
    format!(
        "{{\"left_size\":{:.4},\"right_size\":{:.4},\"left_droop\":{:.4},\"right_droop\":{:.4}}}",
        state.left_size, state.right_size, state.left_droop, state.right_droop
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(els_is_neutral(&new_ear_lobe_size_state()));
    }

    #[test]
    fn size_clamps() {
        let mut s = new_ear_lobe_size_state();
        els_set_size(&mut s, EarSide::Left, 5.0);
        assert!((s.left_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn droop_clamps() {
        let mut s = new_ear_lobe_size_state();
        els_set_droop(&mut s, EarSide::Right, -2.0);
        assert!(s.right_droop < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_ear_lobe_size_state();
        els_set_both_size(&mut s, 0.8);
        els_reset(&mut s);
        assert!(els_is_neutral(&s));
    }

    #[test]
    fn symmetry_one_when_equal() {
        let mut s = new_ear_lobe_size_state();
        els_set_both_size(&mut s, 0.5);
        assert!((els_symmetry(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn symmetry_less_when_asymmetric() {
        let mut s = new_ear_lobe_size_state();
        els_set_size(&mut s, EarSide::Left, 1.0);
        assert!(els_symmetry(&s) < 1.0);
    }

    #[test]
    fn weights_four_values() {
        let w = els_to_weights(&new_ear_lobe_size_state(), &default_ear_lobe_size_config());
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_ear_lobe_size_state();
        els_set_both_size(&mut b, 1.0);
        let r = els_blend(&new_ear_lobe_size_state(), &b, 0.5);
        assert!((r.left_size - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = els_to_json(&new_ear_lobe_size_state());
        assert!(j.contains("left_size"));
    }

    #[test]
    fn set_both_size_equal() {
        let mut s = new_ear_lobe_size_state();
        els_set_both_size(&mut s, 0.6);
        assert!((s.left_size - s.right_size).abs() < 1e-6);
    }
}
