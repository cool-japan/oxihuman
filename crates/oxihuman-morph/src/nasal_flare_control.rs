// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nasal alar flare (nostril-wing spread) control.

/// Side.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NasalFlareSide {
    Left,
    Right,
    Both,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NasalFlareState {
    pub flare_left: f32,
    pub flare_right: f32,
    /// Elevation of the alar base (positive = lifted).
    pub base_elevation: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NasalFlareConfig {
    pub max_flare: f32,
}

impl Default for NasalFlareConfig {
    fn default() -> Self {
        Self { max_flare: 1.0 }
    }
}
impl Default for NasalFlareState {
    fn default() -> Self {
        Self {
            flare_left: 0.0,
            flare_right: 0.0,
            base_elevation: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_nasal_flare_state() -> NasalFlareState {
    NasalFlareState::default()
}

#[allow(dead_code)]
pub fn default_nasal_flare_config() -> NasalFlareConfig {
    NasalFlareConfig::default()
}

#[allow(dead_code)]
pub fn nf_set_flare(
    state: &mut NasalFlareState,
    cfg: &NasalFlareConfig,
    side: NasalFlareSide,
    v: f32,
) {
    let v = v.clamp(-cfg.max_flare, cfg.max_flare);
    match side {
        NasalFlareSide::Left => state.flare_left = v,
        NasalFlareSide::Right => state.flare_right = v,
        NasalFlareSide::Both => {
            state.flare_left = v;
            state.flare_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn nf_set_base_elevation(state: &mut NasalFlareState, v: f32) {
    state.base_elevation = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nf_reset(state: &mut NasalFlareState) {
    *state = NasalFlareState::default();
}

#[allow(dead_code)]
pub fn nf_is_neutral(state: &NasalFlareState) -> bool {
    state.flare_left.abs() < 1e-4
        && state.flare_right.abs() < 1e-4
        && state.base_elevation.abs() < 1e-4
}

#[allow(dead_code)]
pub fn nf_blend(a: &NasalFlareState, b: &NasalFlareState, t: f32) -> NasalFlareState {
    let t = t.clamp(0.0, 1.0);
    NasalFlareState {
        flare_left: a.flare_left + (b.flare_left - a.flare_left) * t,
        flare_right: a.flare_right + (b.flare_right - a.flare_right) * t,
        base_elevation: a.base_elevation + (b.base_elevation - a.base_elevation) * t,
    }
}

#[allow(dead_code)]
pub fn nf_symmetry(state: &NasalFlareState) -> f32 {
    1.0 - (state.flare_left - state.flare_right).abs().min(1.0)
}

#[allow(dead_code)]
pub fn nf_average_flare(state: &NasalFlareState) -> f32 {
    (state.flare_left + state.flare_right) * 0.5
}

#[allow(dead_code)]
pub fn nf_to_weights(state: &NasalFlareState) -> [f32; 3] {
    [state.flare_left, state.flare_right, state.base_elevation]
}

#[allow(dead_code)]
pub fn nf_to_json(state: &NasalFlareState) -> String {
    format!(
        "{{\"flare_l\":{:.4},\"flare_r\":{:.4},\"base_elev\":{:.4}}}",
        state.flare_left, state.flare_right, state.base_elevation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(nf_is_neutral(&new_nasal_flare_state()));
    }

    #[test]
    fn flare_clamps_max() {
        let mut s = new_nasal_flare_state();
        let cfg = default_nasal_flare_config();
        nf_set_flare(&mut s, &cfg, NasalFlareSide::Left, 5.0);
        assert!(s.flare_left <= cfg.max_flare);
    }

    #[test]
    fn flare_clamps_min() {
        let mut s = new_nasal_flare_state();
        let cfg = default_nasal_flare_config();
        nf_set_flare(&mut s, &cfg, NasalFlareSide::Right, -5.0);
        assert!(s.flare_right >= -cfg.max_flare);
    }

    #[test]
    fn both_sides() {
        let mut s = new_nasal_flare_state();
        let cfg = default_nasal_flare_config();
        nf_set_flare(&mut s, &cfg, NasalFlareSide::Both, 0.4);
        assert!((s.flare_left - s.flare_right).abs() < 1e-5);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_nasal_flare_state();
        let cfg = default_nasal_flare_config();
        nf_set_flare(&mut s, &cfg, NasalFlareSide::Both, 0.5);
        nf_reset(&mut s);
        assert!(nf_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_nasal_flare_config();
        let mut a = new_nasal_flare_state();
        let mut b = new_nasal_flare_state();
        nf_set_flare(&mut a, &cfg, NasalFlareSide::Left, 0.0);
        nf_set_flare(&mut b, &cfg, NasalFlareSide::Left, 1.0);
        let m = nf_blend(&a, &b, 0.5);
        assert!((m.flare_left - 0.5).abs() < 1e-4);
    }

    #[test]
    fn symmetry_equal() {
        let s = new_nasal_flare_state();
        assert!((nf_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_zero() {
        assert!((nf_average_flare(&new_nasal_flare_state())).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(nf_to_weights(&new_nasal_flare_state()).len(), 3);
    }

    #[test]
    fn json_contains_flare() {
        assert!(nf_to_json(&new_nasal_flare_state()).contains("flare"));
    }
}
