// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Shin morph — controls shin shape, curvature and calf definition.

/// Configuration for shin control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShinConfig {
    pub max_girth: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShinSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShinState {
    pub left_girth: f32,
    pub right_girth: f32,
    pub left_curvature: f32,
    pub right_curvature: f32,
}

#[allow(dead_code)]
pub fn default_shin_config() -> ShinConfig {
    ShinConfig { max_girth: 1.0 }
}

#[allow(dead_code)]
pub fn new_shin_state() -> ShinState {
    ShinState {
        left_girth: 0.0,
        right_girth: 0.0,
        left_curvature: 0.0,
        right_curvature: 0.0,
    }
}

#[allow(dead_code)]
pub fn shn_set_girth(state: &mut ShinState, cfg: &ShinConfig, side: ShinSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_girth);
    match side {
        ShinSide::Left => state.left_girth = clamped,
        ShinSide::Right => state.right_girth = clamped,
    }
}

#[allow(dead_code)]
pub fn shn_set_both_girth(state: &mut ShinState, cfg: &ShinConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_girth);
    state.left_girth = clamped;
    state.right_girth = clamped;
}

#[allow(dead_code)]
pub fn shn_set_curvature(state: &mut ShinState, side: ShinSide, v: f32) {
    let clamped = v.clamp(-1.0, 1.0);
    match side {
        ShinSide::Left => state.left_curvature = clamped,
        ShinSide::Right => state.right_curvature = clamped,
    }
}

#[allow(dead_code)]
pub fn shn_reset(state: &mut ShinState) {
    *state = new_shin_state();
}

#[allow(dead_code)]
pub fn shn_is_neutral(state: &ShinState) -> bool {
    let vals = [
        state.left_girth,
        state.right_girth,
        state.left_curvature,
        state.right_curvature,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn shn_average_girth(state: &ShinState) -> f32 {
    (state.left_girth + state.right_girth) * 0.5
}

#[allow(dead_code)]
pub fn shn_symmetry(state: &ShinState) -> f32 {
    (state.left_girth - state.right_girth).abs()
}

#[allow(dead_code)]
pub fn shn_blend(a: &ShinState, b: &ShinState, t: f32) -> ShinState {
    let t = t.clamp(0.0, 1.0);
    ShinState {
        left_girth: a.left_girth + (b.left_girth - a.left_girth) * t,
        right_girth: a.right_girth + (b.right_girth - a.right_girth) * t,
        left_curvature: a.left_curvature + (b.left_curvature - a.left_curvature) * t,
        right_curvature: a.right_curvature + (b.right_curvature - a.right_curvature) * t,
    }
}

#[allow(dead_code)]
pub fn shn_to_weights(state: &ShinState) -> Vec<(String, f32)> {
    vec![
        ("shin_girth_l".to_string(), state.left_girth),
        ("shin_girth_r".to_string(), state.right_girth),
        ("shin_curvature_l".to_string(), state.left_curvature),
        ("shin_curvature_r".to_string(), state.right_curvature),
    ]
}

#[allow(dead_code)]
pub fn shn_to_json(state: &ShinState) -> String {
    format!(
        r#"{{"left_girth":{:.4},"right_girth":{:.4},"left_curvature":{:.4},"right_curvature":{:.4}}}"#,
        state.left_girth, state.right_girth, state.left_curvature, state.right_curvature
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_shin_config();
        assert!((cfg.max_girth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_shin_state();
        assert!(shn_is_neutral(&s));
    }

    #[test]
    fn set_girth_left() {
        let cfg = default_shin_config();
        let mut s = new_shin_state();
        shn_set_girth(&mut s, &cfg, ShinSide::Left, 0.5);
        assert!((s.left_girth - 0.5).abs() < 1e-6);
        assert_eq!(s.right_girth, 0.0);
    }

    #[test]
    fn set_girth_clamps() {
        let cfg = default_shin_config();
        let mut s = new_shin_state();
        shn_set_girth(&mut s, &cfg, ShinSide::Right, 10.0);
        assert!((s.right_girth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_girth() {
        let cfg = default_shin_config();
        let mut s = new_shin_state();
        shn_set_both_girth(&mut s, &cfg, 0.6);
        assert!(shn_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_curvature_signed() {
        let mut s = new_shin_state();
        shn_set_curvature(&mut s, ShinSide::Left, -0.3);
        assert!((s.left_curvature + 0.3).abs() < 1e-6);
    }

    #[test]
    fn average_girth() {
        let cfg = default_shin_config();
        let mut s = new_shin_state();
        shn_set_girth(&mut s, &cfg, ShinSide::Left, 0.4);
        shn_set_girth(&mut s, &cfg, ShinSide::Right, 0.6);
        assert!((shn_average_girth(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_shin_config();
        let mut s = new_shin_state();
        shn_set_both_girth(&mut s, &cfg, 0.8);
        shn_reset(&mut s);
        assert!(shn_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_shin_state();
        let cfg = default_shin_config();
        let mut b = new_shin_state();
        shn_set_both_girth(&mut b, &cfg, 1.0);
        let mid = shn_blend(&a, &b, 0.5);
        assert!((mid.left_girth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_shin_state();
        assert_eq!(shn_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_shin_state();
        let j = shn_to_json(&s);
        assert!(j.contains("left_girth"));
        assert!(j.contains("right_curvature"));
    }
}
