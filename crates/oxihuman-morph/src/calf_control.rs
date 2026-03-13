// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Calf muscle shape morph control.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalfConfig {
    pub max_muscle_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalfState {
    pub muscle_l: f32,
    pub muscle_r: f32,
    pub definition_l: f32,
    pub definition_r: f32,
}

#[allow(dead_code)]
pub fn default_calf_config() -> CalfConfig {
    CalfConfig {
        max_muscle_size: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_calf_state() -> CalfState {
    CalfState {
        muscle_l: 0.0,
        muscle_r: 0.0,
        definition_l: 0.0,
        definition_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn calf_set_muscle(state: &mut CalfState, cfg: &CalfConfig, left: f32, right: f32) {
    state.muscle_l = left.clamp(0.0, cfg.max_muscle_size);
    state.muscle_r = right.clamp(0.0, cfg.max_muscle_size);
}

#[allow(dead_code)]
pub fn calf_set_definition(state: &mut CalfState, left: f32, right: f32) {
    state.definition_l = left.clamp(0.0, 1.0);
    state.definition_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn calf_mirror(state: &mut CalfState) {
    let avg_m = (state.muscle_l + state.muscle_r) * 0.5;
    let avg_d = (state.definition_l + state.definition_r) * 0.5;
    state.muscle_l = avg_m;
    state.muscle_r = avg_m;
    state.definition_l = avg_d;
    state.definition_r = avg_d;
}

#[allow(dead_code)]
pub fn calf_reset(state: &mut CalfState) {
    *state = new_calf_state();
}

#[allow(dead_code)]
pub fn calf_to_weights(state: &CalfState) -> Vec<(String, f32)> {
    vec![
        ("calf_muscle_l".to_string(), state.muscle_l),
        ("calf_muscle_r".to_string(), state.muscle_r),
        ("calf_definition_l".to_string(), state.definition_l),
        ("calf_definition_r".to_string(), state.definition_r),
    ]
}

#[allow(dead_code)]
pub fn calf_to_json(state: &CalfState) -> String {
    format!(
        r#"{{"muscle_l":{:.4},"muscle_r":{:.4},"definition_l":{:.4},"definition_r":{:.4}}}"#,
        state.muscle_l, state.muscle_r, state.definition_l, state.definition_r
    )
}

// ── New canonical structs/functions required by lib.rs re-export ──────────────

/// Canonical calf control struct.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalfControl {
    pub width: f32,
    pub length: f32,
    pub muscle_tone: f32,
}

/// Returns a default `CalfControl`.
#[allow(dead_code)]
pub fn default_calf_control() -> CalfControl {
    CalfControl {
        width: 0.5,
        length: 0.5,
        muscle_tone: 0.0,
    }
}

/// Applies calf control values to a weight slice.
#[allow(dead_code)]
pub fn apply_calf_control(weights: &mut [f32], cc: &CalfControl) {
    if !weights.is_empty() {
        weights[0] = cc.width;
    }
    if weights.len() > 1 {
        weights[1] = cc.length;
    }
    if weights.len() > 2 {
        weights[2] = cc.muscle_tone;
    }
}

/// Linearly blends two `CalfControl` values by `t` in [0, 1].
#[allow(dead_code)]
pub fn calf_control_blend(a: &CalfControl, b: &CalfControl, t: f32) -> CalfControl {
    let t = t.clamp(0.0, 1.0);
    CalfControl {
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
        muscle_tone: a.muscle_tone + (b.muscle_tone - a.muscle_tone) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_calf_config();
        assert_eq!(cfg.max_muscle_size, 1.0);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_calf_state();
        assert_eq!(s.muscle_l, 0.0);
        assert_eq!(s.definition_r, 0.0);
    }

    #[test]
    fn test_set_muscle_clamps() {
        let cfg = default_calf_config();
        let mut s = new_calf_state();
        calf_set_muscle(&mut s, &cfg, 2.0, -0.5);
        assert_eq!(s.muscle_l, 1.0);
        assert_eq!(s.muscle_r, 0.0);
    }

    #[test]
    fn test_set_definition_clamps() {
        let mut s = new_calf_state();
        calf_set_definition(&mut s, 0.6, 1.5);
        assert!((s.definition_l - 0.6).abs() < 1e-6);
        assert_eq!(s.definition_r, 1.0);
    }

    #[test]
    fn test_mirror_averages() {
        let mut s = new_calf_state();
        s.muscle_l = 0.3;
        s.muscle_r = 0.7;
        calf_mirror(&mut s);
        assert!((s.muscle_l - 0.5).abs() < 1e-6);
        assert!((s.muscle_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_calf_config();
        let mut s = new_calf_state();
        calf_set_muscle(&mut s, &cfg, 0.8, 0.8);
        calf_reset(&mut s);
        assert_eq!(s.muscle_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_calf_state();
        assert_eq!(calf_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_to_json_has_keys() {
        let s = new_calf_state();
        let j = calf_to_json(&s);
        assert!(j.contains("muscle_l"));
        assert!(j.contains("definition_r"));
    }

    #[test]
    fn test_set_muscle_valid() {
        let cfg = default_calf_config();
        let mut s = new_calf_state();
        calf_set_muscle(&mut s, &cfg, 0.4, 0.6);
        assert!((s.muscle_l - 0.4).abs() < 1e-6);
        assert!((s.muscle_r - 0.6).abs() < 1e-6);
    }
}
