// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Scapula morph — controls scapular prominence, elevation and winging.

/// Configuration for scapula control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScapulaConfig {
    pub max_wing: f32,
}

/// Side selection.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScapulaSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScapulaState {
    pub left_wing: f32,
    pub right_wing: f32,
    pub left_elevation: f32,
    pub right_elevation: f32,
}

#[allow(dead_code)]
pub fn default_scapula_config() -> ScapulaConfig {
    ScapulaConfig { max_wing: 1.0 }
}

#[allow(dead_code)]
pub fn new_scapula_state() -> ScapulaState {
    ScapulaState {
        left_wing: 0.0,
        right_wing: 0.0,
        left_elevation: 0.0,
        right_elevation: 0.0,
    }
}

#[allow(dead_code)]
pub fn sc_set_wing(state: &mut ScapulaState, cfg: &ScapulaConfig, side: ScapulaSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_wing);
    match side {
        ScapulaSide::Left => state.left_wing = clamped,
        ScapulaSide::Right => state.right_wing = clamped,
    }
}

#[allow(dead_code)]
pub fn sc_set_elevation(state: &mut ScapulaState, side: ScapulaSide, v: f32) {
    let clamped = v.clamp(-1.0, 1.0);
    match side {
        ScapulaSide::Left => state.left_elevation = clamped,
        ScapulaSide::Right => state.right_elevation = clamped,
    }
}

#[allow(dead_code)]
pub fn sc_set_both_wing(state: &mut ScapulaState, cfg: &ScapulaConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_wing);
    state.left_wing = clamped;
    state.right_wing = clamped;
}

#[allow(dead_code)]
pub fn sc_reset(state: &mut ScapulaState) {
    *state = new_scapula_state();
}

#[allow(dead_code)]
pub fn sc_is_neutral(state: &ScapulaState) -> bool {
    let vals = [
        state.left_wing,
        state.right_wing,
        state.left_elevation,
        state.right_elevation,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn sc_average_wing(state: &ScapulaState) -> f32 {
    (state.left_wing + state.right_wing) * 0.5
}

#[allow(dead_code)]
pub fn sc_symmetry(state: &ScapulaState) -> f32 {
    (state.left_wing - state.right_wing).abs()
}

#[allow(dead_code)]
pub fn sc_blend(a: &ScapulaState, b: &ScapulaState, t: f32) -> ScapulaState {
    let t = t.clamp(0.0, 1.0);
    ScapulaState {
        left_wing: a.left_wing + (b.left_wing - a.left_wing) * t,
        right_wing: a.right_wing + (b.right_wing - a.right_wing) * t,
        left_elevation: a.left_elevation + (b.left_elevation - a.left_elevation) * t,
        right_elevation: a.right_elevation + (b.right_elevation - a.right_elevation) * t,
    }
}

#[allow(dead_code)]
pub fn sc_to_weights(state: &ScapulaState) -> Vec<(String, f32)> {
    vec![
        ("scapula_wing_l".to_string(), state.left_wing),
        ("scapula_wing_r".to_string(), state.right_wing),
        ("scapula_elevation_l".to_string(), state.left_elevation),
        ("scapula_elevation_r".to_string(), state.right_elevation),
    ]
}

#[allow(dead_code)]
pub fn sc_to_json(state: &ScapulaState) -> String {
    format!(
        r#"{{"left_wing":{:.4},"right_wing":{:.4},"left_elevation":{:.4},"right_elevation":{:.4}}}"#,
        state.left_wing, state.right_wing, state.left_elevation, state.right_elevation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_scapula_config();
        assert!((cfg.max_wing - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_scapula_state();
        assert!(sc_is_neutral(&s));
    }

    #[test]
    fn set_wing_left() {
        let cfg = default_scapula_config();
        let mut s = new_scapula_state();
        sc_set_wing(&mut s, &cfg, ScapulaSide::Left, 0.5);
        assert!((s.left_wing - 0.5).abs() < 1e-6);
        assert_eq!(s.right_wing, 0.0);
    }

    #[test]
    fn set_wing_clamps() {
        let cfg = default_scapula_config();
        let mut s = new_scapula_state();
        sc_set_wing(&mut s, &cfg, ScapulaSide::Right, 10.0);
        assert!((s.right_wing - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_wing_equal() {
        let cfg = default_scapula_config();
        let mut s = new_scapula_state();
        sc_set_both_wing(&mut s, &cfg, 0.7);
        assert!(sc_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_elevation_signed() {
        let mut s = new_scapula_state();
        sc_set_elevation(&mut s, ScapulaSide::Left, -0.5);
        assert!((s.left_elevation + 0.5).abs() < 1e-6);
    }

    #[test]
    fn average_wing() {
        let cfg = default_scapula_config();
        let mut s = new_scapula_state();
        sc_set_wing(&mut s, &cfg, ScapulaSide::Left, 0.4);
        sc_set_wing(&mut s, &cfg, ScapulaSide::Right, 0.6);
        assert!((sc_average_wing(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_scapula_config();
        let mut s = new_scapula_state();
        sc_set_both_wing(&mut s, &cfg, 0.8);
        sc_reset(&mut s);
        assert!(sc_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_scapula_state();
        let cfg = default_scapula_config();
        let mut b = new_scapula_state();
        sc_set_both_wing(&mut b, &cfg, 1.0);
        let mid = sc_blend(&a, &b, 0.5);
        assert!((mid.left_wing - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_scapula_state();
        assert_eq!(sc_to_weights(&s).len(), 4);
    }
}
