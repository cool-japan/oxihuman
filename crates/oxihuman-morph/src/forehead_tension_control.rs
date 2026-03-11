// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Forehead tension control — frontalis muscle tension and skin compression.

/// Configuration for forehead tension.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadTensionConfig {
    pub max_tension: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadTensionState {
    pub central_tension: f32,
    pub left_tension: f32,
    pub right_tension: f32,
    pub skin_compression: f32,
}

#[allow(dead_code)]
pub fn default_forehead_tension_config() -> ForeheadTensionConfig {
    ForeheadTensionConfig { max_tension: 1.0 }
}

#[allow(dead_code)]
pub fn new_forehead_tension_state() -> ForeheadTensionState {
    ForeheadTensionState {
        central_tension: 0.0,
        left_tension: 0.0,
        right_tension: 0.0,
        skin_compression: 0.0,
    }
}

#[allow(dead_code)]
pub fn ften_set_central(state: &mut ForeheadTensionState, cfg: &ForeheadTensionConfig, v: f32) {
    state.central_tension = v.clamp(0.0, cfg.max_tension);
}

#[allow(dead_code)]
pub fn ften_set_lateral(
    state: &mut ForeheadTensionState,
    cfg: &ForeheadTensionConfig,
    left: f32,
    right: f32,
) {
    state.left_tension = left.clamp(0.0, cfg.max_tension);
    state.right_tension = right.clamp(0.0, cfg.max_tension);
}

#[allow(dead_code)]
pub fn ften_set_compression(state: &mut ForeheadTensionState, v: f32) {
    state.skin_compression = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ften_set_all(state: &mut ForeheadTensionState, cfg: &ForeheadTensionConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_tension);
    state.central_tension = clamped;
    state.left_tension = clamped;
    state.right_tension = clamped;
}

#[allow(dead_code)]
pub fn ften_reset(state: &mut ForeheadTensionState) {
    *state = new_forehead_tension_state();
}

#[allow(dead_code)]
pub fn ften_is_neutral(state: &ForeheadTensionState) -> bool {
    let vals = [
        state.central_tension,
        state.left_tension,
        state.right_tension,
        state.skin_compression,
    ];
    vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn ften_average(state: &ForeheadTensionState) -> f32 {
    (state.central_tension + state.left_tension + state.right_tension) / 3.0
}

#[allow(dead_code)]
pub fn ften_blend(
    a: &ForeheadTensionState,
    b: &ForeheadTensionState,
    t: f32,
) -> ForeheadTensionState {
    let t = t.clamp(0.0, 1.0);
    ForeheadTensionState {
        central_tension: a.central_tension + (b.central_tension - a.central_tension) * t,
        left_tension: a.left_tension + (b.left_tension - a.left_tension) * t,
        right_tension: a.right_tension + (b.right_tension - a.right_tension) * t,
        skin_compression: a.skin_compression + (b.skin_compression - a.skin_compression) * t,
    }
}

#[allow(dead_code)]
pub fn ften_to_weights(state: &ForeheadTensionState) -> Vec<(String, f32)> {
    vec![
        (
            "forehead_tension_central".to_string(),
            state.central_tension,
        ),
        ("forehead_tension_left".to_string(), state.left_tension),
        ("forehead_tension_right".to_string(), state.right_tension),
        (
            "forehead_skin_compression".to_string(),
            state.skin_compression,
        ),
    ]
}

#[allow(dead_code)]
pub fn ften_to_json(state: &ForeheadTensionState) -> String {
    format!(
        r#"{{"central":{:.4},"left":{:.4},"right":{:.4},"compression":{:.4}}}"#,
        state.central_tension, state.left_tension, state.right_tension, state.skin_compression
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_forehead_tension_config();
        assert!((cfg.max_tension - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_forehead_tension_state();
        assert!(ften_is_neutral(&s));
    }

    #[test]
    fn set_central_clamps() {
        let cfg = default_forehead_tension_config();
        let mut s = new_forehead_tension_state();
        ften_set_central(&mut s, &cfg, 3.0);
        assert!((s.central_tension - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_lateral() {
        let cfg = default_forehead_tension_config();
        let mut s = new_forehead_tension_state();
        ften_set_lateral(&mut s, &cfg, 0.3, 0.7);
        assert!((s.left_tension - 0.3).abs() < 1e-6);
        assert!((s.right_tension - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_all() {
        let cfg = default_forehead_tension_config();
        let mut s = new_forehead_tension_state();
        ften_set_all(&mut s, &cfg, 0.5);
        assert!((ften_average(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_compression() {
        let mut s = new_forehead_tension_state();
        ften_set_compression(&mut s, 0.6);
        assert!((s.skin_compression - 0.6).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_forehead_tension_config();
        let mut s = new_forehead_tension_state();
        ften_set_all(&mut s, &cfg, 0.8);
        ften_reset(&mut s);
        assert!(ften_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_forehead_tension_state();
        let cfg = default_forehead_tension_config();
        let mut b = new_forehead_tension_state();
        ften_set_all(&mut b, &cfg, 1.0);
        let m = ften_blend(&a, &b, 0.5);
        assert!((m.central_tension - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_forehead_tension_state();
        assert_eq!(ften_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_forehead_tension_state();
        let j = ften_to_json(&s);
        assert!(j.contains("central"));
    }
}
