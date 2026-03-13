// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Heel morphology controls for heel shape, width, and height.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeelConfig {
    pub width: f32,
    pub height: f32,
    pub roundness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeelState {
    pub width: f32,
    pub height: f32,
    pub roundness: f32,
    pub callus: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeelWeights {
    pub wide: f32,
    pub narrow: f32,
    pub tall: f32,
    pub round: f32,
    pub calloused: f32,
}

#[allow(dead_code)]
pub fn default_heel_config() -> HeelConfig {
    HeelConfig { width: 0.5, height: 0.5, roundness: 0.5 }
}

#[allow(dead_code)]
pub fn new_heel_state() -> HeelState {
    HeelState { width: 0.5, height: 0.5, roundness: 0.5, callus: 0.0 }
}

#[allow(dead_code)]
pub fn set_heel_width(state: &mut HeelState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_heel_height(state: &mut HeelState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_heel_roundness(state: &mut HeelState, value: f32) {
    state.roundness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_heel_callus(state: &mut HeelState, value: f32) {
    state.callus = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_heel_weights(state: &HeelState, cfg: &HeelConfig) -> HeelWeights {
    let w = state.width * cfg.width;
    let wide = (w * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let narrow = (1.0 - w).clamp(0.0, 1.0);
    let tall = (state.height * cfg.height).clamp(0.0, 1.0);
    let round = (state.roundness * cfg.roundness).clamp(0.0, 1.0);
    let calloused = state.callus.clamp(0.0, 1.0);
    HeelWeights { wide, narrow, tall, round, calloused }
}

#[allow(dead_code)]
pub fn heel_to_json(state: &HeelState) -> String {
    format!(
        r#"{{"width":{},"height":{},"roundness":{},"callus":{}}}"#,
        state.width, state.height, state.roundness, state.callus
    )
}

#[allow(dead_code)]
pub fn blend_heel_states(a: &HeelState, b: &HeelState, t: f32) -> HeelState {
    let t = t.clamp(0.0, 1.0);
    HeelState {
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        callus: a.callus + (b.callus - a.callus) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_heel_config();
        assert!((0.0..=1.0).contains(&cfg.width));
    }

    #[test]
    fn test_new_state() {
        let s = new_heel_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp() {
        let mut s = new_heel_state();
        set_heel_width(&mut s, 1.5);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut s = new_heel_state();
        set_heel_height(&mut s, 0.8);
        assert!((s.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness() {
        let mut s = new_heel_state();
        set_heel_roundness(&mut s, 0.7);
        assert!((s.roundness - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_callus() {
        let mut s = new_heel_state();
        set_heel_callus(&mut s, 0.4);
        assert!((s.callus - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_heel_state();
        let cfg = default_heel_config();
        let w = compute_heel_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.wide));
        assert!((0.0..=1.0).contains(&w.tall));
    }

    #[test]
    fn test_to_json() {
        let s = new_heel_state();
        let json = heel_to_json(&s);
        assert!(json.contains("width"));
        assert!(json.contains("callus"));
    }

    #[test]
    fn test_blend() {
        let a = new_heel_state();
        let mut b = new_heel_state();
        b.width = 1.0;
        let mid = blend_heel_states(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_heel_state();
        let r = blend_heel_states(&a, &a, 0.5);
        assert!((r.width - a.width).abs() < 1e-6);
    }
}
