// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Forehead raise morph — controls how raised the forehead region is.

/// Configuration for forehead raise control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadRaiseConfig {
    pub max_raise: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadRaiseState {
    pub center_raise: f32,
    pub left_raise: f32,
    pub right_raise: f32,
    pub skin_tension: f32,
}

#[allow(dead_code)]
pub fn default_forehead_raise_config() -> ForeheadRaiseConfig {
    ForeheadRaiseConfig { max_raise: 1.0 }
}

#[allow(dead_code)]
pub fn new_forehead_raise_state() -> ForeheadRaiseState {
    ForeheadRaiseState {
        center_raise: 0.0,
        left_raise: 0.0,
        right_raise: 0.0,
        skin_tension: 0.0,
    }
}

#[allow(dead_code)]
pub fn fhr_set_center(state: &mut ForeheadRaiseState, cfg: &ForeheadRaiseConfig, v: f32) {
    state.center_raise = v.clamp(0.0, cfg.max_raise);
}

#[allow(dead_code)]
pub fn fhr_set_sides(
    state: &mut ForeheadRaiseState,
    cfg: &ForeheadRaiseConfig,
    left: f32,
    right: f32,
) {
    state.left_raise = left.clamp(0.0, cfg.max_raise);
    state.right_raise = right.clamp(0.0, cfg.max_raise);
}

#[allow(dead_code)]
pub fn fhr_set_all(state: &mut ForeheadRaiseState, cfg: &ForeheadRaiseConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_raise);
    state.center_raise = clamped;
    state.left_raise = clamped;
    state.right_raise = clamped;
}

#[allow(dead_code)]
pub fn fhr_set_tension(state: &mut ForeheadRaiseState, v: f32) {
    state.skin_tension = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fhr_reset(state: &mut ForeheadRaiseState) {
    *state = new_forehead_raise_state();
}

#[allow(dead_code)]
pub fn fhr_is_neutral(state: &ForeheadRaiseState) -> bool {
    let vals = [
        state.center_raise,
        state.left_raise,
        state.right_raise,
        state.skin_tension,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn fhr_average_raise(state: &ForeheadRaiseState) -> f32 {
    (state.center_raise + state.left_raise + state.right_raise) / 3.0
}

#[allow(dead_code)]
pub fn fhr_symmetry(state: &ForeheadRaiseState) -> f32 {
    (state.left_raise - state.right_raise).abs()
}

#[allow(dead_code)]
pub fn fhr_blend(a: &ForeheadRaiseState, b: &ForeheadRaiseState, t: f32) -> ForeheadRaiseState {
    let t = t.clamp(0.0, 1.0);
    ForeheadRaiseState {
        center_raise: a.center_raise + (b.center_raise - a.center_raise) * t,
        left_raise: a.left_raise + (b.left_raise - a.left_raise) * t,
        right_raise: a.right_raise + (b.right_raise - a.right_raise) * t,
        skin_tension: a.skin_tension + (b.skin_tension - a.skin_tension) * t,
    }
}

#[allow(dead_code)]
pub fn fhr_to_weights(state: &ForeheadRaiseState) -> Vec<(String, f32)> {
    vec![
        ("forehead_raise_center".to_string(), state.center_raise),
        ("forehead_raise_left".to_string(), state.left_raise),
        ("forehead_raise_right".to_string(), state.right_raise),
        ("forehead_skin_tension".to_string(), state.skin_tension),
    ]
}

#[allow(dead_code)]
pub fn fhr_to_json(state: &ForeheadRaiseState) -> String {
    format!(
        r#"{{"center_raise":{:.4},"left_raise":{:.4},"right_raise":{:.4},"skin_tension":{:.4}}}"#,
        state.center_raise, state.left_raise, state.right_raise, state.skin_tension
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_forehead_raise_config();
        assert!((cfg.max_raise - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_forehead_raise_state();
        assert!(fhr_is_neutral(&s));
    }

    #[test]
    fn set_center_clamps() {
        let cfg = default_forehead_raise_config();
        let mut s = new_forehead_raise_state();
        fhr_set_center(&mut s, &cfg, 5.0);
        assert!((s.center_raise - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_sides() {
        let cfg = default_forehead_raise_config();
        let mut s = new_forehead_raise_state();
        fhr_set_sides(&mut s, &cfg, 0.3, 0.7);
        assert!((s.left_raise - 0.3).abs() < 1e-6);
        assert!((s.right_raise - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_all_equal() {
        let cfg = default_forehead_raise_config();
        let mut s = new_forehead_raise_state();
        fhr_set_all(&mut s, &cfg, 0.6);
        assert!((s.center_raise - 0.6).abs() < 1e-6);
        assert!((s.left_raise - 0.6).abs() < 1e-6);
        assert!((s.right_raise - 0.6).abs() < 1e-6);
    }

    #[test]
    fn symmetry_zero_when_equal() {
        let cfg = default_forehead_raise_config();
        let mut s = new_forehead_raise_state();
        fhr_set_sides(&mut s, &cfg, 0.5, 0.5);
        assert!(fhr_symmetry(&s) < 1e-6);
    }

    #[test]
    fn average_raise() {
        let cfg = default_forehead_raise_config();
        let mut s = new_forehead_raise_state();
        fhr_set_all(&mut s, &cfg, 0.9);
        assert!((fhr_average_raise(&s) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_forehead_raise_config();
        let mut s = new_forehead_raise_state();
        fhr_set_all(&mut s, &cfg, 0.5);
        fhr_reset(&mut s);
        assert!(fhr_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_forehead_raise_state();
        let cfg = default_forehead_raise_config();
        let mut b = new_forehead_raise_state();
        fhr_set_all(&mut b, &cfg, 1.0);
        let mid = fhr_blend(&a, &b, 0.5);
        assert!((mid.center_raise - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_forehead_raise_state();
        assert_eq!(fhr_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_forehead_raise_state();
        let j = fhr_to_json(&s);
        assert!(j.contains("center_raise"));
        assert!(j.contains("skin_tension"));
    }
}
