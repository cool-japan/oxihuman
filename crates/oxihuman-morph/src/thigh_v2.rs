// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Inner/outer thigh separation and shape morph (v2).

/// Configuration for thigh v2 morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThighV2Config {
    pub inner_min: f32,
    pub inner_max: f32,
    pub outer_min: f32,
    pub outer_max: f32,
}

impl Default for ThighV2Config {
    fn default() -> Self {
        ThighV2Config {
            inner_min: -1.0,
            inner_max: 1.0,
            outer_min: -1.0,
            outer_max: 1.0,
        }
    }
}

/// Morph weights for inner/outer thigh separation.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ThighV2Weights {
    pub inner_left: f32,
    pub inner_right: f32,
    pub outer_left: f32,
    pub outer_right: f32,
}

/// State for thigh v2 morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThighV2State {
    pub config: ThighV2Config,
    pub inner: f32,
    pub outer: f32,
    pub symmetry: f32,
}

#[allow(dead_code)]
pub fn default_thigh_v2_config() -> ThighV2Config {
    ThighV2Config::default()
}

#[allow(dead_code)]
pub fn new_thigh_v2_state(config: ThighV2Config) -> ThighV2State {
    ThighV2State {
        config,
        inner: 0.0,
        outer: 0.0,
        symmetry: 1.0,
    }
}

#[allow(dead_code)]
pub fn tv2_set_inner(state: &mut ThighV2State, v: f32) {
    state.inner = v.clamp(state.config.inner_min, state.config.inner_max);
}

#[allow(dead_code)]
pub fn tv2_set_outer(state: &mut ThighV2State, v: f32) {
    state.outer = v.clamp(state.config.outer_min, state.config.outer_max);
}

#[allow(dead_code)]
pub fn tv2_set_symmetry(state: &mut ThighV2State, s: f32) {
    state.symmetry = s.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn tv2_reset(state: &mut ThighV2State) {
    state.inner = 0.0;
    state.outer = 0.0;
    state.symmetry = 1.0;
}

#[allow(dead_code)]
pub fn tv2_is_neutral(state: &ThighV2State) -> bool {
    state.inner.abs() < 1e-6 && state.outer.abs() < 1e-6
}

#[allow(dead_code)]
pub fn tv2_to_weights(state: &ThighV2State) -> ThighV2Weights {
    let asym = 1.0 - state.symmetry;
    ThighV2Weights {
        inner_left: state.inner,
        inner_right: state.inner * (1.0 - asym * 0.3),
        outer_left: state.outer,
        outer_right: state.outer * (1.0 - asym * 0.3),
    }
}

#[allow(dead_code)]
pub fn tv2_blend(a: &ThighV2State, b: &ThighV2State, t: f32) -> ThighV2State {
    let t = t.clamp(0.0, 1.0);
    ThighV2State {
        config: a.config.clone(),
        inner: a.inner + (b.inner - a.inner) * t,
        outer: a.outer + (b.outer - a.outer) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
    }
}

#[allow(dead_code)]
pub fn tv2_average_girth(state: &ThighV2State) -> f32 {
    (state.inner.abs() + state.outer.abs()) * 0.5
}

#[allow(dead_code)]
pub fn tv2_to_json(state: &ThighV2State) -> String {
    format!(
        r#"{{"inner":{:.4},"outer":{:.4},"symmetry":{:.4}}}"#,
        state.inner, state.outer, state.symmetry
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_ranges() {
        let cfg = default_thigh_v2_config();
        assert!((cfg.inner_min - (-1.0)).abs() < 1e-6);
        assert!((cfg.inner_max - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let state = new_thigh_v2_state(default_thigh_v2_config());
        assert!(tv2_is_neutral(&state));
    }

    #[test]
    fn set_inner_clamps() {
        let mut s = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_inner(&mut s, 5.0);
        assert!((s.inner - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_outer_clamps() {
        let mut s = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_outer(&mut s, -5.0);
        assert!((s.outer - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn set_symmetry_clamps() {
        let mut s = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_symmetry(&mut s, 2.0);
        assert!((s.symmetry - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_inner(&mut s, 0.5);
        tv2_reset(&mut s);
        assert!(tv2_is_neutral(&s));
    }

    #[test]
    fn weights_symmetry_effect() {
        let mut s = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_inner(&mut s, 0.8);
        tv2_set_symmetry(&mut s, 0.0);
        let w = tv2_to_weights(&s);
        assert!(w.inner_left > w.inner_right);
    }

    #[test]
    fn blend_midpoint() {
        let a = new_thigh_v2_state(default_thigh_v2_config());
        let mut b = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_inner(&mut b, 1.0);
        let m = tv2_blend(&a, &b, 0.5);
        assert!((m.inner - 0.5).abs() < 1e-5);
    }

    #[test]
    fn average_girth_zero_when_neutral() {
        let s = new_thigh_v2_state(default_thigh_v2_config());
        assert!(tv2_average_girth(&s).abs() < 1e-6);
    }

    #[test]
    fn to_json_contains_inner() {
        let mut s = new_thigh_v2_state(default_thigh_v2_config());
        tv2_set_inner(&mut s, 0.3);
        assert!(tv2_to_json(&s).contains("inner"));
    }
}
