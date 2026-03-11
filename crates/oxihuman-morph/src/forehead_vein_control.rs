// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Forehead vein (temporal / supraorbital vein) prominence control.

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ForeheadVeinState {
    /// Temple vein prominence (0..1).
    pub temple_left: f32,
    pub temple_right: f32,
    /// Central supraorbital vein (0..1).
    pub central: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ForeheadVeinConfig {
    pub max_prominence: f32,
}

impl Default for ForeheadVeinConfig {
    fn default() -> Self {
        Self {
            max_prominence: 1.0,
        }
    }
}
impl Default for ForeheadVeinState {
    fn default() -> Self {
        Self {
            temple_left: 0.0,
            temple_right: 0.0,
            central: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_forehead_vein_state() -> ForeheadVeinState {
    ForeheadVeinState::default()
}

#[allow(dead_code)]
pub fn default_forehead_vein_config() -> ForeheadVeinConfig {
    ForeheadVeinConfig::default()
}

#[allow(dead_code)]
pub fn fv_set_temple(
    state: &mut ForeheadVeinState,
    cfg: &ForeheadVeinConfig,
    left: f32,
    right: f32,
) {
    state.temple_left = left.clamp(0.0, cfg.max_prominence);
    state.temple_right = right.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn fv_set_central(state: &mut ForeheadVeinState, cfg: &ForeheadVeinConfig, v: f32) {
    state.central = v.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn fv_reset(state: &mut ForeheadVeinState) {
    *state = ForeheadVeinState::default();
}

#[allow(dead_code)]
pub fn fv_is_neutral(state: &ForeheadVeinState) -> bool {
    state.temple_left < 1e-4 && state.temple_right < 1e-4 && state.central < 1e-4
}

#[allow(dead_code)]
pub fn fv_blend(a: &ForeheadVeinState, b: &ForeheadVeinState, t: f32) -> ForeheadVeinState {
    let t = t.clamp(0.0, 1.0);
    ForeheadVeinState {
        temple_left: a.temple_left + (b.temple_left - a.temple_left) * t,
        temple_right: a.temple_right + (b.temple_right - a.temple_right) * t,
        central: a.central + (b.central - a.central) * t,
    }
}

#[allow(dead_code)]
pub fn fv_total_prominence(state: &ForeheadVeinState) -> f32 {
    (state.temple_left + state.temple_right + state.central) / 3.0
}

#[allow(dead_code)]
pub fn fv_to_weights(state: &ForeheadVeinState) -> [f32; 3] {
    [state.temple_left, state.temple_right, state.central]
}

#[allow(dead_code)]
pub fn fv_to_json(state: &ForeheadVeinState) -> String {
    format!(
        "{{\"temple_l\":{:.4},\"temple_r\":{:.4},\"central\":{:.4}}}",
        state.temple_left, state.temple_right, state.central
    )
}

#[allow(dead_code)]
pub fn fv_symmetry(state: &ForeheadVeinState) -> f32 {
    1.0 - (state.temple_left - state.temple_right).abs().min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(fv_is_neutral(&new_forehead_vein_state()));
    }

    #[test]
    fn set_temple_clamps() {
        let mut s = new_forehead_vein_state();
        let cfg = default_forehead_vein_config();
        fv_set_temple(&mut s, &cfg, 2.0, -1.0);
        assert!(s.temple_left <= cfg.max_prominence);
        assert!(s.temple_right >= 0.0);
    }

    #[test]
    fn central_clamp() {
        let mut s = new_forehead_vein_state();
        let cfg = default_forehead_vein_config();
        fv_set_central(&mut s, &cfg, 5.0);
        assert!(s.central <= cfg.max_prominence);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_forehead_vein_state();
        let cfg = default_forehead_vein_config();
        fv_set_temple(&mut s, &cfg, 0.5, 0.5);
        fv_reset(&mut s);
        assert!(fv_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_forehead_vein_config();
        let mut a = new_forehead_vein_state();
        let mut b = new_forehead_vein_state();
        fv_set_central(&mut a, &cfg, 0.0);
        fv_set_central(&mut b, &cfg, 1.0);
        let m = fv_blend(&a, &b, 0.5);
        assert!((m.central - 0.5).abs() < 1e-4);
    }

    #[test]
    fn total_prominence_zero() {
        assert!((fv_total_prominence(&new_forehead_vein_state())).abs() < 1e-5);
    }

    #[test]
    fn symmetry_one_equal() {
        let s = new_forehead_vein_state();
        assert!((fv_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(fv_to_weights(&new_forehead_vein_state()).len(), 3);
    }

    #[test]
    fn json_has_central() {
        assert!(fv_to_json(&new_forehead_vein_state()).contains("central"));
    }

    #[test]
    fn not_neutral_after_set() {
        let mut s = new_forehead_vein_state();
        let cfg = default_forehead_vein_config();
        fv_set_central(&mut s, &cfg, 0.5);
        assert!(!fv_is_neutral(&s));
    }
}
