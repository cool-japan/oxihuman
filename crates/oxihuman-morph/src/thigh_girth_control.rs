// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Thigh girth control — medial/lateral thigh volume adjustments.

use std::f32::consts::PI;

/// Leg side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThighGirthSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ThighGirthConfig {
    pub max_radius_m: f32,
}

impl Default for ThighGirthConfig {
    fn default() -> Self {
        Self { max_radius_m: 0.04 }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ThighGirthState {
    /// Overall girth, 0..=1.
    pub left_girth: f32,
    pub right_girth: f32,
    /// Medial emphasis, 0..=1.
    pub left_medial: f32,
    pub right_medial: f32,
}

#[allow(dead_code)]
pub fn new_thigh_girth_state() -> ThighGirthState {
    ThighGirthState::default()
}

#[allow(dead_code)]
pub fn default_thigh_girth_config() -> ThighGirthConfig {
    ThighGirthConfig::default()
}

#[allow(dead_code)]
pub fn tg_set_girth(state: &mut ThighGirthState, side: ThighGirthSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        ThighGirthSide::Left => state.left_girth = v,
        ThighGirthSide::Right => state.right_girth = v,
    }
}

#[allow(dead_code)]
pub fn tg_set_medial(state: &mut ThighGirthState, side: ThighGirthSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        ThighGirthSide::Left => state.left_medial = v,
        ThighGirthSide::Right => state.right_medial = v,
    }
}

#[allow(dead_code)]
pub fn tg_set_both(state: &mut ThighGirthState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left_girth = v;
    state.right_girth = v;
}

#[allow(dead_code)]
pub fn tg_reset(state: &mut ThighGirthState) {
    *state = ThighGirthState::default();
}

#[allow(dead_code)]
pub fn tg_is_neutral(state: &ThighGirthState) -> bool {
    state.left_girth < 1e-4 && state.right_girth < 1e-4
}

/// Circumference estimate in metres for one side.
#[allow(dead_code)]
pub fn tg_circumference(
    state: &ThighGirthState,
    side: ThighGirthSide,
    cfg: &ThighGirthConfig,
) -> f32 {
    let g = match side {
        ThighGirthSide::Left => state.left_girth,
        ThighGirthSide::Right => state.right_girth,
    };
    2.0 * PI * (0.06 + g * cfg.max_radius_m)
}

#[allow(dead_code)]
pub fn tg_symmetry(state: &ThighGirthState) -> f32 {
    1.0 - (state.left_girth - state.right_girth).abs()
}

#[allow(dead_code)]
pub fn tg_to_weights(state: &ThighGirthState, cfg: &ThighGirthConfig) -> [f32; 4] {
    [
        state.left_girth * cfg.max_radius_m,
        state.right_girth * cfg.max_radius_m,
        state.left_medial * cfg.max_radius_m * 0.6,
        state.right_medial * cfg.max_radius_m * 0.6,
    ]
}

#[allow(dead_code)]
pub fn tg_blend(a: &ThighGirthState, b: &ThighGirthState, t: f32) -> ThighGirthState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ThighGirthState {
        left_girth: a.left_girth * inv + b.left_girth * t,
        right_girth: a.right_girth * inv + b.right_girth * t,
        left_medial: a.left_medial * inv + b.left_medial * t,
        right_medial: a.right_medial * inv + b.right_medial * t,
    }
}

#[allow(dead_code)]
pub fn tg_to_json(state: &ThighGirthState) -> String {
    format!(
        "{{\"left_girth\":{:.4},\"right_girth\":{:.4}}}",
        state.left_girth, state.right_girth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(tg_is_neutral(&new_thigh_girth_state()));
    }

    #[test]
    fn girth_clamps_high() {
        let mut s = new_thigh_girth_state();
        tg_set_girth(&mut s, ThighGirthSide::Left, 5.0);
        assert!((s.left_girth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn girth_clamps_low() {
        let mut s = new_thigh_girth_state();
        tg_set_girth(&mut s, ThighGirthSide::Right, -1.0);
        assert!(s.right_girth < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_thigh_girth_state();
        tg_set_both(&mut s, 0.9);
        tg_reset(&mut s);
        assert!(tg_is_neutral(&s));
    }

    #[test]
    fn circumference_at_neutral_positive() {
        let cfg = default_thigh_girth_config();
        let s = new_thigh_girth_state();
        // Even at 0 girth there's a base radius
        assert!(tg_circumference(&s, ThighGirthSide::Left, &cfg) > 0.0);
    }

    #[test]
    fn circumference_grows_with_girth() {
        let cfg = default_thigh_girth_config();
        let mut s = new_thigh_girth_state();
        let base = tg_circumference(&s, ThighGirthSide::Left, &cfg);
        tg_set_girth(&mut s, ThighGirthSide::Left, 1.0);
        let grown = tg_circumference(&s, ThighGirthSide::Left, &cfg);
        assert!(grown > base);
    }

    #[test]
    fn symmetry_one_when_equal() {
        let mut s = new_thigh_girth_state();
        tg_set_both(&mut s, 0.5);
        assert!((tg_symmetry(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weights_four_elements() {
        let w = tg_to_weights(&new_thigh_girth_state(), &default_thigh_girth_config());
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_thigh_girth_state();
        tg_set_both(&mut b, 1.0);
        let r = tg_blend(&new_thigh_girth_state(), &b, 0.5);
        assert!((r.left_girth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = tg_to_json(&new_thigh_girth_state());
        assert!(j.contains("left_girth") && j.contains("right_girth"));
    }
}
