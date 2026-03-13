// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Thigh girth and shape control.

/// Side.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThighSide {
    Left,
    Right,
    Both,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ThighState {
    /// Overall girth factor (0.5..2.0 normalised around 1.0 neutral).
    pub girth_left: f32,
    pub girth_right: f32,
    /// Inner thigh fullness (0..1).
    pub inner_left: f32,
    pub inner_right: f32,
    /// Outer thigh fullness (0..1).
    pub outer_left: f32,
    pub outer_right: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ThighConfig {
    pub min_girth: f32,
    pub max_girth: f32,
}

impl Default for ThighConfig {
    fn default() -> Self {
        Self {
            min_girth: 0.5,
            max_girth: 2.0,
        }
    }
}
impl Default for ThighState {
    fn default() -> Self {
        Self {
            girth_left: 1.0,
            girth_right: 1.0,
            inner_left: 0.0,
            inner_right: 0.0,
            outer_left: 0.0,
            outer_right: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_thigh_state() -> ThighState {
    ThighState::default()
}

#[allow(dead_code)]
pub fn default_thigh_config() -> ThighConfig {
    ThighConfig::default()
}

#[allow(dead_code)]
pub fn th_set_girth(state: &mut ThighState, cfg: &ThighConfig, side: ThighSide, v: f32) {
    let v = v.clamp(cfg.min_girth, cfg.max_girth);
    match side {
        ThighSide::Left => state.girth_left = v,
        ThighSide::Right => state.girth_right = v,
        ThighSide::Both => {
            state.girth_left = v;
            state.girth_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn th_set_inner(state: &mut ThighState, side: ThighSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        ThighSide::Left => state.inner_left = v,
        ThighSide::Right => state.inner_right = v,
        ThighSide::Both => {
            state.inner_left = v;
            state.inner_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn th_set_outer(state: &mut ThighState, side: ThighSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        ThighSide::Left => state.outer_left = v,
        ThighSide::Right => state.outer_right = v,
        ThighSide::Both => {
            state.outer_left = v;
            state.outer_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn th_reset(state: &mut ThighState) {
    *state = ThighState::default();
}

#[allow(dead_code)]
pub fn th_is_neutral(state: &ThighState) -> bool {
    (state.girth_left - 1.0).abs() < 1e-4
        && (state.girth_right - 1.0).abs() < 1e-4
        && state.inner_left < 1e-4
        && state.inner_right < 1e-4
        && state.outer_left < 1e-4
        && state.outer_right < 1e-4
}

#[allow(dead_code)]
pub fn th_blend(a: &ThighState, b: &ThighState, t: f32) -> ThighState {
    let t = t.clamp(0.0, 1.0);
    ThighState {
        girth_left: a.girth_left + (b.girth_left - a.girth_left) * t,
        girth_right: a.girth_right + (b.girth_right - a.girth_right) * t,
        inner_left: a.inner_left + (b.inner_left - a.inner_left) * t,
        inner_right: a.inner_right + (b.inner_right - a.inner_right) * t,
        outer_left: a.outer_left + (b.outer_left - a.outer_left) * t,
        outer_right: a.outer_right + (b.outer_right - a.outer_right) * t,
    }
}

#[allow(dead_code)]
pub fn th_symmetry(state: &ThighState) -> f32 {
    1.0 - (state.girth_left - state.girth_right).abs().min(1.0)
}

#[allow(dead_code)]
pub fn th_average_girth(state: &ThighState) -> f32 {
    (state.girth_left + state.girth_right) * 0.5
}

#[allow(dead_code)]
pub fn th_to_weights(state: &ThighState) -> [f32; 6] {
    [
        state.girth_left - 1.0,
        state.girth_right - 1.0,
        state.inner_left,
        state.inner_right,
        state.outer_left,
        state.outer_right,
    ]
}

#[allow(dead_code)]
pub fn th_to_json(state: &ThighState) -> String {
    format!(
        "{{\"g_l\":{:.4},\"g_r\":{:.4},\"in_l\":{:.4},\"in_r\":{:.4},\"out_l\":{:.4},\"out_r\":{:.4}}}",
        state.girth_left, state.girth_right,
        state.inner_left, state.inner_right,
        state.outer_left, state.outer_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(th_is_neutral(&new_thigh_state()));
    }

    #[test]
    fn girth_clamps_max() {
        let mut s = new_thigh_state();
        let cfg = default_thigh_config();
        th_set_girth(&mut s, &cfg, ThighSide::Left, 99.0);
        assert!(s.girth_left <= cfg.max_girth);
    }

    #[test]
    fn girth_clamps_min() {
        let mut s = new_thigh_state();
        let cfg = default_thigh_config();
        th_set_girth(&mut s, &cfg, ThighSide::Right, 0.0);
        assert!(s.girth_right >= cfg.min_girth);
    }

    #[test]
    fn both_sides_girth() {
        let mut s = new_thigh_state();
        let cfg = default_thigh_config();
        th_set_girth(&mut s, &cfg, ThighSide::Both, 1.5);
        assert!((s.girth_left - s.girth_right).abs() < 1e-5);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_thigh_state();
        let cfg = default_thigh_config();
        th_set_girth(&mut s, &cfg, ThighSide::Both, 1.8);
        th_reset(&mut s);
        assert!(th_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_thigh_config();
        let mut a = new_thigh_state();
        let mut b = new_thigh_state();
        th_set_girth(&mut a, &cfg, ThighSide::Left, 1.0);
        th_set_girth(&mut b, &cfg, ThighSide::Left, 2.0);
        let m = th_blend(&a, &b, 0.5);
        assert!((m.girth_left - 1.5).abs() < 1e-4);
    }

    #[test]
    fn symmetry_equal() {
        let s = new_thigh_state();
        assert!((th_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_girth_neutral() {
        assert!((th_average_girth(&new_thigh_state()) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(th_to_weights(&new_thigh_state()).len(), 6);
    }

    #[test]
    fn json_has_girth() {
        assert!(th_to_json(&new_thigh_state()).contains("g_l"));
    }

    #[test]
    fn inner_clamped() {
        let mut s = new_thigh_state();
        th_set_inner(&mut s, ThighSide::Left, 5.0);
        assert!(s.inner_left <= 1.0);
    }
}
