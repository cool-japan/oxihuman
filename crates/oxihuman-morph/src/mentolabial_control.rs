// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mentolabial sulcus (chin-lip groove) morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MentolabialConfig {
    pub max_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MentolabialState {
    pub depth: f32,
    pub width: f32,
    pub position: f32,
}

#[allow(dead_code)]
pub fn default_mentolabial_config() -> MentolabialConfig {
    MentolabialConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_mentolabial_state() -> MentolabialState {
    MentolabialState { depth: 0.0, width: 0.5, position: 0.5 }
}

#[allow(dead_code)]
pub fn mento_set_depth(state: &mut MentolabialState, cfg: &MentolabialConfig, v: f32) {
    state.depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn mento_set_width(state: &mut MentolabialState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn mento_set_position(state: &mut MentolabialState, v: f32) {
    state.position = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn mento_reset(state: &mut MentolabialState) {
    *state = new_mentolabial_state();
}

#[allow(dead_code)]
pub fn mento_to_weights(state: &MentolabialState) -> [f32; 3] {
    [state.depth, state.width, state.position]
}

#[allow(dead_code)]
pub fn mento_to_json(state: &MentolabialState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"position":{:.4}}}"#,
        state.depth, state.width, state.position
    )
}

#[allow(dead_code)]
pub fn mento_clamp(state: &mut MentolabialState, cfg: &MentolabialConfig) {
    state.depth = state.depth.clamp(0.0, cfg.max_depth);
    state.width = state.width.clamp(0.0, 1.0);
    state.position = state.position.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_mentolabial_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_mentolabial_state();
        assert!((s.depth - 0.0).abs() < 1e-6);
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.position - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_mentolabial_config();
        let mut s = new_mentolabial_state();
        mento_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
        mento_set_depth(&mut s, &cfg, -1.0);
        assert!((s.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let mut s = new_mentolabial_state();
        mento_set_width(&mut s, 2.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_position_clamps() {
        let mut s = new_mentolabial_state();
        mento_set_position(&mut s, -0.5);
        assert!((s.position - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_mentolabial_config();
        let mut s = new_mentolabial_state();
        mento_set_depth(&mut s, &cfg, 0.9);
        mento_reset(&mut s);
        assert!((s.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_mentolabial_state();
        let w = mento_to_weights(&s);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_mentolabial_state();
        let j = mento_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("position"));
    }

    #[test]
    fn test_clamp() {
        let cfg = default_mentolabial_config();
        let mut s = new_mentolabial_state();
        s.depth = 5.0;
        s.position = 5.0;
        mento_clamp(&mut s, &cfg);
        assert!((s.depth - 1.0).abs() < 1e-6);
        assert!((s.position - 1.0).abs() < 1e-6);
    }
}
