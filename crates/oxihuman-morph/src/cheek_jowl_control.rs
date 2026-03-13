// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cheek-jowl (lower cheek / mandibular fullness) control.

/// Side enum.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JowlSide {
    Left,
    Right,
    Both,
}

/// Jowl state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CheekJowlState {
    pub sag_left: f32,
    pub sag_right: f32,
    pub volume_left: f32,
    pub volume_right: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CheekJowlConfig {
    pub max_sag: f32,
    pub max_volume: f32,
}

impl Default for CheekJowlConfig {
    fn default() -> Self {
        Self {
            max_sag: 1.0,
            max_volume: 1.0,
        }
    }
}

impl Default for CheekJowlState {
    fn default() -> Self {
        Self {
            sag_left: 0.0,
            sag_right: 0.0,
            volume_left: 0.0,
            volume_right: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_cheek_jowl_state() -> CheekJowlState {
    CheekJowlState::default()
}

#[allow(dead_code)]
pub fn default_cheek_jowl_config() -> CheekJowlConfig {
    CheekJowlConfig::default()
}

#[allow(dead_code)]
pub fn cj_set_sag(state: &mut CheekJowlState, cfg: &CheekJowlConfig, side: JowlSide, v: f32) {
    let v = v.clamp(0.0, cfg.max_sag);
    match side {
        JowlSide::Left => state.sag_left = v,
        JowlSide::Right => state.sag_right = v,
        JowlSide::Both => {
            state.sag_left = v;
            state.sag_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn cj_set_volume(state: &mut CheekJowlState, cfg: &CheekJowlConfig, side: JowlSide, v: f32) {
    let v = v.clamp(0.0, cfg.max_volume);
    match side {
        JowlSide::Left => state.volume_left = v,
        JowlSide::Right => state.volume_right = v,
        JowlSide::Both => {
            state.volume_left = v;
            state.volume_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn cj_reset(state: &mut CheekJowlState) {
    *state = CheekJowlState::default();
}

#[allow(dead_code)]
pub fn cj_is_neutral(state: &CheekJowlState) -> bool {
    state.sag_left < 1e-4
        && state.sag_right < 1e-4
        && state.volume_left < 1e-4
        && state.volume_right < 1e-4
}

#[allow(dead_code)]
pub fn cj_blend(a: &CheekJowlState, b: &CheekJowlState, t: f32) -> CheekJowlState {
    let t = t.clamp(0.0, 1.0);
    CheekJowlState {
        sag_left: a.sag_left + (b.sag_left - a.sag_left) * t,
        sag_right: a.sag_right + (b.sag_right - a.sag_right) * t,
        volume_left: a.volume_left + (b.volume_left - a.volume_left) * t,
        volume_right: a.volume_right + (b.volume_right - a.volume_right) * t,
    }
}

#[allow(dead_code)]
pub fn cj_symmetry(state: &CheekJowlState) -> f32 {
    1.0 - (state.sag_left - state.sag_right).abs().min(1.0)
}

#[allow(dead_code)]
pub fn cj_total_volume(state: &CheekJowlState) -> f32 {
    state.volume_left + state.volume_right
}

#[allow(dead_code)]
pub fn cj_to_weights(state: &CheekJowlState) -> [f32; 4] {
    [
        state.sag_left,
        state.sag_right,
        state.volume_left,
        state.volume_right,
    ]
}

#[allow(dead_code)]
pub fn cj_to_json(state: &CheekJowlState) -> String {
    format!(
        "{{\"sag_left\":{:.4},\"sag_right\":{:.4},\"vol_left\":{:.4},\"vol_right\":{:.4}}}",
        state.sag_left, state.sag_right, state.volume_left, state.volume_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(cj_is_neutral(&new_cheek_jowl_state()));
    }

    #[test]
    fn set_sag_clamps() {
        let mut s = new_cheek_jowl_state();
        let cfg = default_cheek_jowl_config();
        cj_set_sag(&mut s, &cfg, JowlSide::Left, 5.0);
        assert!(s.sag_left <= cfg.max_sag);
    }

    #[test]
    fn set_both_sides() {
        let mut s = new_cheek_jowl_state();
        let cfg = default_cheek_jowl_config();
        cj_set_sag(&mut s, &cfg, JowlSide::Both, 0.7);
        assert!((s.sag_left - 0.7).abs() < 1e-5);
        assert!((s.sag_right - 0.7).abs() < 1e-5);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_cheek_jowl_state();
        let cfg = default_cheek_jowl_config();
        cj_set_sag(&mut s, &cfg, JowlSide::Both, 0.5);
        cj_reset(&mut s);
        assert!(cj_is_neutral(&s));
    }

    #[test]
    fn blend_t0_is_a() {
        let mut a = new_cheek_jowl_state();
        let b = new_cheek_jowl_state();
        let cfg = default_cheek_jowl_config();
        cj_set_sag(&mut a, &cfg, JowlSide::Left, 0.6);
        let r = cj_blend(&a, &b, 0.0);
        assert!((r.sag_left - 0.6).abs() < 1e-5);
    }

    #[test]
    fn symmetry_symmetric() {
        let s = new_cheek_jowl_state();
        assert!((cj_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn total_volume_zero_by_default() {
        assert!((cj_total_volume(&new_cheek_jowl_state())).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(cj_to_weights(&new_cheek_jowl_state()).len(), 4);
    }

    #[test]
    fn json_contains_sag() {
        let s = new_cheek_jowl_state();
        assert!(cj_to_json(&s).contains("sag"));
    }

    #[test]
    fn volume_not_negative() {
        let mut s = new_cheek_jowl_state();
        let cfg = default_cheek_jowl_config();
        cj_set_volume(&mut s, &cfg, JowlSide::Right, -1.0);
        assert!(s.volume_right >= 0.0);
    }
}
