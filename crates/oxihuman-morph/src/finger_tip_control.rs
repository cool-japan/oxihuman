// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Finger tip control: adjusts the shape and width of fingertips.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerTipConfig {
    pub min_width: f32,
    pub max_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerTipState {
    pub width: f32,
    pub taper: f32,
    pub nail_length: f32,
}

#[allow(dead_code)]
pub fn default_finger_tip_config() -> FingerTipConfig {
    FingerTipConfig {
        min_width: 0.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_finger_tip_state() -> FingerTipState {
    FingerTipState {
        width: 0.5,
        taper: 0.3,
        nail_length: 0.4,
    }
}

#[allow(dead_code)]
pub fn ft_set_width(state: &mut FingerTipState, cfg: &FingerTipConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn ft_set_taper(state: &mut FingerTipState, v: f32) {
    state.taper = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ft_set_nail_length(state: &mut FingerTipState, v: f32) {
    state.nail_length = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ft_reset(state: &mut FingerTipState) {
    *state = new_finger_tip_state();
}

/// Cross-section area of fingertip (approx ellipse).
#[allow(dead_code)]
pub fn ft_cross_section_area(state: &FingerTipState) -> f32 {
    let a = state.width * 0.5;
    let b = a * (1.0 - state.taper * 0.5);
    PI * a * b
}

#[allow(dead_code)]
pub fn ft_to_weights(state: &FingerTipState) -> Vec<(String, f32)> {
    vec![
        ("fingertip_width".to_string(), state.width),
        ("fingertip_taper".to_string(), state.taper),
        ("fingertip_nail_length".to_string(), state.nail_length),
    ]
}

#[allow(dead_code)]
pub fn ft_to_json(state: &FingerTipState) -> String {
    format!(
        r#"{{"width":{:.4},"taper":{:.4},"nail_length":{:.4}}}"#,
        state.width, state.taper, state.nail_length
    )
}

#[allow(dead_code)]
pub fn ft_blend(a: &FingerTipState, b: &FingerTipState, t: f32) -> FingerTipState {
    let t = t.clamp(0.0, 1.0);
    FingerTipState {
        width: a.width + (b.width - a.width) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        nail_length: a.nail_length + (b.nail_length - a.nail_length) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_finger_tip_config();
        assert!(cfg.min_width.abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_finger_tip_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_finger_tip_config();
        let mut s = new_finger_tip_state();
        ft_set_width(&mut s, &cfg, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_finger_tip_state();
        ft_set_taper(&mut s, 0.7);
        assert!((s.taper - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_nail_length() {
        let mut s = new_finger_tip_state();
        ft_set_nail_length(&mut s, 0.9);
        assert!((s.nail_length - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_finger_tip_config();
        let mut s = new_finger_tip_state();
        ft_set_width(&mut s, &cfg, 0.9);
        ft_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cross_section_area() {
        let s = new_finger_tip_state();
        assert!(ft_cross_section_area(&s) > 0.0);
    }

    #[test]
    fn test_to_weights() {
        let s = new_finger_tip_state();
        assert_eq!(ft_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_finger_tip_state();
        let mut b = new_finger_tip_state();
        b.width = 1.0;
        let mid = ft_blend(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_finger_tip_state();
        let j = ft_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("taper"));
    }
}
