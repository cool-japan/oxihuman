// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Hand thickness morph control: adjusts the palm/back thickness and finger girth.

/// Configuration for hand thickness morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandThicknessConfig {
    pub min_thickness: f32,
    pub max_thickness: f32,
}

/// Runtime state for hand thickness morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandThicknessState {
    pub left_thickness: f32,
    pub right_thickness: f32,
    pub finger_girth: f32,
    pub palm_width: f32,
}

#[allow(dead_code)]
pub fn default_hand_thickness_config() -> HandThicknessConfig {
    HandThicknessConfig {
        min_thickness: 0.0,
        max_thickness: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_hand_thickness_state() -> HandThicknessState {
    HandThicknessState {
        left_thickness: 0.5,
        right_thickness: 0.5,
        finger_girth: 0.5,
        palm_width: 0.5,
    }
}

#[allow(dead_code)]
pub fn ht_set_left(state: &mut HandThicknessState, cfg: &HandThicknessConfig, v: f32) {
    state.left_thickness = v.clamp(cfg.min_thickness, cfg.max_thickness);
}

#[allow(dead_code)]
pub fn ht_set_right(state: &mut HandThicknessState, cfg: &HandThicknessConfig, v: f32) {
    state.right_thickness = v.clamp(cfg.min_thickness, cfg.max_thickness);
}

#[allow(dead_code)]
pub fn ht_set_finger_girth(state: &mut HandThicknessState, v: f32) {
    state.finger_girth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_set_palm_width(state: &mut HandThicknessState, v: f32) {
    state.palm_width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_reset(state: &mut HandThicknessState) {
    *state = new_hand_thickness_state();
}

#[allow(dead_code)]
pub fn ht_to_weights(state: &HandThicknessState) -> Vec<(String, f32)> {
    vec![
        ("hand_thickness_left".to_string(), state.left_thickness),
        ("hand_thickness_right".to_string(), state.right_thickness),
        ("hand_finger_girth".to_string(), state.finger_girth),
        ("hand_palm_width".to_string(), state.palm_width),
    ]
}

#[allow(dead_code)]
pub fn ht_to_json(state: &HandThicknessState) -> String {
    format!(
        r#"{{"left_thickness":{:.4},"right_thickness":{:.4},"finger_girth":{:.4},"palm_width":{:.4}}}"#,
        state.left_thickness, state.right_thickness, state.finger_girth, state.palm_width
    )
}

#[allow(dead_code)]
pub fn ht_blend(a: &HandThicknessState, b: &HandThicknessState, t: f32) -> HandThicknessState {
    let t = t.clamp(0.0, 1.0);
    HandThicknessState {
        left_thickness: a.left_thickness + (b.left_thickness - a.left_thickness) * t,
        right_thickness: a.right_thickness + (b.right_thickness - a.right_thickness) * t,
        finger_girth: a.finger_girth + (b.finger_girth - a.finger_girth) * t,
        palm_width: a.palm_width + (b.palm_width - a.palm_width) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_hand_thickness_config();
        assert!(cfg.min_thickness.abs() < 1e-6);
        assert!((cfg.max_thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_hand_thickness_state();
        assert!((s.left_thickness - 0.5).abs() < 1e-6);
        assert!((s.finger_girth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_clamps() {
        let cfg = default_hand_thickness_config();
        let mut s = new_hand_thickness_state();
        ht_set_left(&mut s, &cfg, 5.0);
        assert!((s.left_thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_right() {
        let cfg = default_hand_thickness_config();
        let mut s = new_hand_thickness_state();
        ht_set_right(&mut s, &cfg, 0.3);
        assert!((s.right_thickness - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_finger_girth() {
        let mut s = new_hand_thickness_state();
        ht_set_finger_girth(&mut s, 0.8);
        assert!((s.finger_girth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_palm_width() {
        let mut s = new_hand_thickness_state();
        ht_set_palm_width(&mut s, 0.2);
        assert!((s.palm_width - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_hand_thickness_config();
        let mut s = new_hand_thickness_state();
        ht_set_left(&mut s, &cfg, 0.9);
        ht_reset(&mut s);
        assert!((s.left_thickness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_hand_thickness_state();
        assert_eq!(ht_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_to_json() {
        let s = new_hand_thickness_state();
        let j = ht_to_json(&s);
        assert!(j.contains("left_thickness"));
    }

    #[test]
    fn test_blend() {
        let a = new_hand_thickness_state();
        let mut b = new_hand_thickness_state();
        b.left_thickness = 1.0;
        let mid = ht_blend(&a, &b, 0.5);
        assert!((mid.left_thickness - 0.75).abs() < 1e-6);
    }
}
