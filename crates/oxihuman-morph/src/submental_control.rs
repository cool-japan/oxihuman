// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Submental area (under chin / neck-chin junction) morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubmentalConfig {
    pub max_fullness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubmentalState {
    pub fullness: f32,
    pub angle: f32,
    pub definition: f32,
}

#[allow(dead_code)]
pub fn default_submental_config() -> SubmentalConfig {
    SubmentalConfig { max_fullness: 1.0 }
}

#[allow(dead_code)]
pub fn new_submental_state() -> SubmentalState {
    SubmentalState { fullness: 0.0, angle: 0.0, definition: 0.5 }
}

#[allow(dead_code)]
pub fn subm_set_fullness(state: &mut SubmentalState, cfg: &SubmentalConfig, v: f32) {
    state.fullness = v.clamp(0.0, cfg.max_fullness);
}

#[allow(dead_code)]
pub fn subm_set_angle(state: &mut SubmentalState, v: f32) {
    state.angle = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn subm_set_definition(state: &mut SubmentalState, v: f32) {
    state.definition = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn subm_reset(state: &mut SubmentalState) {
    *state = new_submental_state();
}

#[allow(dead_code)]
pub fn subm_to_weights(state: &SubmentalState) -> [f32; 3] {
    [state.fullness, state.angle, state.definition]
}

#[allow(dead_code)]
pub fn subm_to_json(state: &SubmentalState) -> String {
    format!(
        r#"{{"fullness":{:.4},"angle":{:.4},"definition":{:.4}}}"#,
        state.fullness, state.angle, state.definition
    )
}

#[allow(dead_code)]
pub fn subm_clamp(state: &mut SubmentalState, cfg: &SubmentalConfig) {
    state.fullness = state.fullness.clamp(0.0, cfg.max_fullness);
    state.angle = state.angle.clamp(-1.0, 1.0);
    state.definition = state.definition.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_submental_config();
        assert!((cfg.max_fullness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_submental_state();
        assert!((s.fullness - 0.0).abs() < 1e-6);
        assert!((s.angle - 0.0).abs() < 1e-6);
        assert!((s.definition - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_fullness_clamps() {
        let cfg = default_submental_config();
        let mut s = new_submental_state();
        subm_set_fullness(&mut s, &cfg, 5.0);
        assert!((s.fullness - 1.0).abs() < 1e-6);
        subm_set_fullness(&mut s, &cfg, -1.0);
        assert!((s.fullness - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_clamps() {
        let mut s = new_submental_state();
        subm_set_angle(&mut s, 5.0);
        assert!((s.angle - 1.0).abs() < 1e-6);
        subm_set_angle(&mut s, -5.0);
        assert!((s.angle - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_set_definition_clamps() {
        let mut s = new_submental_state();
        subm_set_definition(&mut s, 2.0);
        assert!((s.definition - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_submental_config();
        let mut s = new_submental_state();
        subm_set_fullness(&mut s, &cfg, 0.9);
        subm_reset(&mut s);
        assert!((s.fullness - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_submental_state();
        let w = subm_to_weights(&s);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_submental_state();
        let j = subm_to_json(&s);
        assert!(j.contains("fullness"));
        assert!(j.contains("definition"));
    }

    #[test]
    fn test_clamp() {
        let cfg = default_submental_config();
        let mut s = new_submental_state();
        s.fullness = 10.0;
        s.definition = -1.0;
        subm_clamp(&mut s, &cfg);
        assert!((s.fullness - 1.0).abs() < 1e-6);
        assert!((s.definition - 0.0).abs() < 1e-6);
    }
}
