// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Abdomen shape morphology controls for belly protrusion, waist width, and core definition.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AbdomenConfig {
    pub belly_protrusion: f32,
    pub waist_width: f32,
    pub core_definition: f32,
    pub navel_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AbdomenState {
    pub protrusion: f32,
    pub width: f32,
    pub definition: f32,
    pub navel: f32,
    pub tuck: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AbdomenMorphWeights {
    pub belly_out: f32,
    pub belly_in: f32,
    pub waist_wide: f32,
    pub waist_narrow: f32,
    pub core_defined: f32,
    pub navel_deep: f32,
}

#[allow(dead_code)]
pub fn default_abdomen_config() -> AbdomenConfig {
    AbdomenConfig {
        belly_protrusion: 0.5,
        waist_width: 0.5,
        core_definition: 0.5,
        navel_depth: 0.3,
    }
}

#[allow(dead_code)]
pub fn new_abdomen_state() -> AbdomenState {
    AbdomenState {
        protrusion: 0.5,
        width: 0.5,
        definition: 0.5,
        navel: 0.3,
        tuck: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_belly_protrusion(state: &mut AbdomenState, value: f32) {
    state.protrusion = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_abdomen_width(state: &mut AbdomenState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_core_definition(state: &mut AbdomenState, value: f32) {
    state.definition = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_abdomen_tuck(state: &mut AbdomenState, value: f32) {
    state.tuck = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_abdomen_weights(state: &AbdomenState, cfg: &AbdomenConfig) -> AbdomenMorphWeights {
    let p = state.protrusion * cfg.belly_protrusion;
    let belly_out = (p * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let belly_in = ((1.0 - p) * state.tuck).clamp(0.0, 1.0);
    let w = state.width * cfg.waist_width;
    let waist_wide = w.clamp(0.0, 1.0);
    let waist_narrow = (1.0 - w).clamp(0.0, 1.0);
    let core_defined = (state.definition * cfg.core_definition).clamp(0.0, 1.0);
    let navel_deep = (state.navel * cfg.navel_depth).clamp(0.0, 1.0);
    AbdomenMorphWeights {
        belly_out,
        belly_in,
        waist_wide,
        waist_narrow,
        core_defined,
        navel_deep,
    }
}

#[allow(dead_code)]
pub fn abdomen_to_json(state: &AbdomenState) -> String {
    format!(
        r#"{{"protrusion":{},"width":{},"definition":{},"navel":{},"tuck":{}}}"#,
        state.protrusion, state.width, state.definition, state.navel, state.tuck
    )
}

#[allow(dead_code)]
pub fn blend_abdomen_states(a: &AbdomenState, b: &AbdomenState, t: f32) -> AbdomenState {
    let t = t.clamp(0.0, 1.0);
    AbdomenState {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        width: a.width + (b.width - a.width) * t,
        definition: a.definition + (b.definition - a.definition) * t,
        navel: a.navel + (b.navel - a.navel) * t,
        tuck: a.tuck + (b.tuck - a.tuck) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_abdomen_config();
        assert!((0.0..=1.0).contains(&cfg.belly_protrusion));
        assert!((0.0..=1.0).contains(&cfg.waist_width));
    }

    #[test]
    fn test_new_state() {
        let s = new_abdomen_state();
        assert!((s.protrusion - 0.5).abs() < 1e-6);
        assert!((s.tuck).abs() < 1e-6);
    }

    #[test]
    fn test_set_belly_protrusion_clamp() {
        let mut s = new_abdomen_state();
        set_belly_protrusion(&mut s, 1.5);
        assert!((s.protrusion - 1.0).abs() < 1e-6);
        set_belly_protrusion(&mut s, -0.5);
        assert!(s.protrusion.abs() < 1e-6);
    }

    #[test]
    fn test_set_abdomen_width_clamp() {
        let mut s = new_abdomen_state();
        set_abdomen_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_core_definition() {
        let mut s = new_abdomen_state();
        set_core_definition(&mut s, 0.9);
        assert!((s.definition - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_in_range() {
        let s = new_abdomen_state();
        let cfg = default_abdomen_config();
        let w = compute_abdomen_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.belly_out));
        assert!((0.0..=1.0).contains(&w.waist_wide));
        assert!((0.0..=1.0).contains(&w.core_defined));
    }

    #[test]
    fn test_abdomen_to_json() {
        let s = new_abdomen_state();
        let json = abdomen_to_json(&s);
        assert!(json.contains("protrusion"));
        assert!(json.contains("tuck"));
    }

    #[test]
    fn test_blend_states() {
        let a = new_abdomen_state();
        let mut b = new_abdomen_state();
        b.protrusion = 1.0;
        let mid = blend_abdomen_states(&a, &b, 0.5);
        assert!((mid.protrusion - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_tuck() {
        let mut s = new_abdomen_state();
        set_abdomen_tuck(&mut s, 0.7);
        assert!((s.tuck - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_abdomen_state();
        let result = blend_abdomen_states(&a, &a, 0.5);
        assert!((result.protrusion - a.protrusion).abs() < 1e-6);
    }
}
