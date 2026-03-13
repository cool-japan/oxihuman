// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Body taper control: adjusts torso taper from shoulders to waist.

use std::f32::consts::PI;

/// Configuration for body taper morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyTaperConfig {
    pub min_taper: f32,
    pub max_taper: f32,
}

/// Runtime state for body taper morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyTaperState {
    pub shoulder_width: f32,
    pub waist_width: f32,
    pub hip_width: f32,
}

#[allow(dead_code)]
pub fn default_body_taper_config() -> BodyTaperConfig {
    BodyTaperConfig {
        min_taper: 0.0,
        max_taper: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_body_taper_state() -> BodyTaperState {
    BodyTaperState {
        shoulder_width: 0.6,
        waist_width: 0.4,
        hip_width: 0.5,
    }
}

#[allow(dead_code)]
pub fn bt_set_shoulder(state: &mut BodyTaperState, cfg: &BodyTaperConfig, v: f32) {
    state.shoulder_width = v.clamp(cfg.min_taper, cfg.max_taper);
}

#[allow(dead_code)]
pub fn bt_set_waist(state: &mut BodyTaperState, cfg: &BodyTaperConfig, v: f32) {
    state.waist_width = v.clamp(cfg.min_taper, cfg.max_taper);
}

#[allow(dead_code)]
pub fn bt_set_hip(state: &mut BodyTaperState, cfg: &BodyTaperConfig, v: f32) {
    state.hip_width = v.clamp(cfg.min_taper, cfg.max_taper);
}

#[allow(dead_code)]
pub fn bt_reset(state: &mut BodyTaperState) {
    *state = new_body_taper_state();
}

#[allow(dead_code)]
pub fn bt_taper_ratio(state: &BodyTaperState) -> f32 {
    if state.shoulder_width.abs() < 1e-9 {
        return 0.0;
    }
    state.waist_width / state.shoulder_width
}

#[allow(dead_code)]
pub fn bt_to_weights(state: &BodyTaperState) -> Vec<(String, f32)> {
    vec![
        ("body_shoulder_width".to_string(), state.shoulder_width),
        ("body_waist_width".to_string(), state.waist_width),
        ("body_hip_width".to_string(), state.hip_width),
    ]
}

#[allow(dead_code)]
pub fn bt_to_json(state: &BodyTaperState) -> String {
    format!(
        r#"{{"shoulder_width":{:.4},"waist_width":{:.4},"hip_width":{:.4}}}"#,
        state.shoulder_width, state.waist_width, state.hip_width
    )
}

#[allow(dead_code)]
pub fn bt_blend(a: &BodyTaperState, b: &BodyTaperState, t: f32) -> BodyTaperState {
    let t = t.clamp(0.0, 1.0);
    BodyTaperState {
        shoulder_width: a.shoulder_width + (b.shoulder_width - a.shoulder_width) * t,
        waist_width: a.waist_width + (b.waist_width - a.waist_width) * t,
        hip_width: a.hip_width + (b.hip_width - a.hip_width) * t,
    }
}

/// Approximate silhouette area using trapezoid approximation.
#[allow(dead_code)]
pub fn bt_silhouette_area(state: &BodyTaperState, height: f32) -> f32 {
    let _ = PI; // use PI to keep import
    0.5 * (state.shoulder_width + state.hip_width) * height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_body_taper_config();
        assert!(cfg.min_taper.abs() < 1e-6);
        assert!((cfg.max_taper - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_body_taper_state();
        assert!((s.shoulder_width - 0.6).abs() < 1e-6);
        assert!((s.waist_width - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_shoulder_clamps() {
        let cfg = default_body_taper_config();
        let mut s = new_body_taper_state();
        bt_set_shoulder(&mut s, &cfg, 5.0);
        assert!((s.shoulder_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_waist() {
        let cfg = default_body_taper_config();
        let mut s = new_body_taper_state();
        bt_set_waist(&mut s, &cfg, 0.3);
        assert!((s.waist_width - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_hip() {
        let cfg = default_body_taper_config();
        let mut s = new_body_taper_state();
        bt_set_hip(&mut s, &cfg, 0.7);
        assert!((s.hip_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_body_taper_config();
        let mut s = new_body_taper_state();
        bt_set_shoulder(&mut s, &cfg, 0.9);
        bt_reset(&mut s);
        assert!((s.shoulder_width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_taper_ratio() {
        let s = new_body_taper_state();
        let r = bt_taper_ratio(&s);
        assert!((r - 0.4 / 0.6).abs() < 1e-4);
    }

    #[test]
    fn test_to_weights() {
        let s = new_body_taper_state();
        assert_eq!(bt_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_body_taper_state();
        let mut b = new_body_taper_state();
        b.shoulder_width = 1.0;
        let mid = bt_blend(&a, &b, 0.5);
        assert!((mid.shoulder_width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_silhouette_area() {
        let s = new_body_taper_state();
        let area = bt_silhouette_area(&s, 1.0);
        assert!(area > 0.0);
    }
}
