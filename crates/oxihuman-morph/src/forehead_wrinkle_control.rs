// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Forehead wrinkle morph control.

#![allow(dead_code)]

/// Configuration for forehead wrinkle morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadWrinkleConfig {
    pub max_depth: f32,
    pub max_count: u32,
}

/// Runtime state for forehead wrinkle morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadWrinkleState {
    pub depth: f32,
    pub horizontal_lines: f32,
    pub vertical_lines: f32,
}

#[allow(dead_code)]
pub fn default_forehead_wrinkle_config() -> ForeheadWrinkleConfig {
    ForeheadWrinkleConfig {
        max_depth: 1.0,
        max_count: 5,
    }
}

#[allow(dead_code)]
pub fn new_forehead_wrinkle_state() -> ForeheadWrinkleState {
    ForeheadWrinkleState {
        depth: 0.0,
        horizontal_lines: 0.0,
        vertical_lines: 0.0,
    }
}

#[allow(dead_code)]
pub fn fw_set_depth(state: &mut ForeheadWrinkleState, cfg: &ForeheadWrinkleConfig, v: f32) {
    state.depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn fw_set_horizontal(state: &mut ForeheadWrinkleState, v: f32) {
    state.horizontal_lines = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fw_set_vertical(state: &mut ForeheadWrinkleState, v: f32) {
    state.vertical_lines = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fw_reset(state: &mut ForeheadWrinkleState) {
    *state = new_forehead_wrinkle_state();
}

#[allow(dead_code)]
pub fn fw_to_weights(state: &ForeheadWrinkleState) -> Vec<(String, f32)> {
    vec![
        ("fw_depth".to_string(), state.depth),
        ("fw_horizontal_lines".to_string(), state.horizontal_lines),
        ("fw_vertical_lines".to_string(), state.vertical_lines),
    ]
}

#[allow(dead_code)]
pub fn fw_to_json(state: &ForeheadWrinkleState) -> String {
    format!(
        r#"{{"depth":{:.4},"horizontal_lines":{:.4},"vertical_lines":{:.4}}}"#,
        state.depth, state.horizontal_lines, state.vertical_lines
    )
}

#[allow(dead_code)]
pub fn fw_clamp(state: &mut ForeheadWrinkleState, cfg: &ForeheadWrinkleConfig) {
    state.depth = state.depth.clamp(0.0, cfg.max_depth);
    state.horizontal_lines = state.horizontal_lines.clamp(0.0, 1.0);
    state.vertical_lines = state.vertical_lines.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_forehead_wrinkle_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
        assert_eq!(cfg.max_count, 5);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_forehead_wrinkle_state();
        assert_eq!(s.depth, 0.0);
        assert_eq!(s.horizontal_lines, 0.0);
        assert_eq!(s.vertical_lines, 0.0);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_forehead_wrinkle_config();
        let mut s = new_forehead_wrinkle_state();
        fw_set_depth(&mut s, &cfg, 2.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_horizontal_clamps() {
        let mut s = new_forehead_wrinkle_state();
        fw_set_horizontal(&mut s, 1.5);
        assert!((s.horizontal_lines - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical_clamps_neg() {
        let mut s = new_forehead_wrinkle_state();
        fw_set_vertical(&mut s, -0.5);
        assert_eq!(s.vertical_lines, 0.0);
    }

    #[test]
    fn test_reset() {
        let cfg = default_forehead_wrinkle_config();
        let mut s = new_forehead_wrinkle_state();
        fw_set_depth(&mut s, &cfg, 0.7);
        fw_reset(&mut s);
        assert_eq!(s.depth, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_forehead_wrinkle_state();
        let w = fw_to_weights(&s);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_forehead_wrinkle_state();
        let j = fw_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("horizontal_lines"));
        assert!(j.contains("vertical_lines"));
    }
}
