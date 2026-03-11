// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Ear helix morph controls for outer ear curvature and fold.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarHelixControlConfig {
    pub curl: f32,
    pub fold: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarHelixControlState {
    pub curl: f32,
    pub fold: f32,
    pub width: f32,
    pub lobe_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarHelixControlWeights {
    pub curled: f32,
    pub folded: f32,
    pub wide: f32,
    pub large_lobe: f32,
    pub flat: f32,
}

#[allow(dead_code)]
pub fn default_ear_helix_control_config() -> EarHelixControlConfig {
    EarHelixControlConfig { curl: 0.5, fold: 0.5, width: 0.5 }
}

#[allow(dead_code)]
pub fn new_ear_helix_control_state() -> EarHelixControlState {
    EarHelixControlState { curl: 0.5, fold: 0.5, width: 0.5, lobe_size: 0.5 }
}

#[allow(dead_code)]
pub fn set_ear_helix_control_curl(state: &mut EarHelixControlState, value: f32) {
    state.curl = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ear_helix_control_fold(state: &mut EarHelixControlState, value: f32) {
    state.fold = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ear_helix_control_width(state: &mut EarHelixControlState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ear_helix_control_lobe_size(state: &mut EarHelixControlState, value: f32) {
    state.lobe_size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_ear_helix_control_weights(state: &EarHelixControlState, cfg: &EarHelixControlConfig) -> EarHelixControlWeights {
    let curled = (state.curl * cfg.curl * (PI * 0.25).sin()).clamp(0.0, 1.0);
    let folded = (state.fold * cfg.fold).clamp(0.0, 1.0);
    let wide = (state.width * cfg.width).clamp(0.0, 1.0);
    let large_lobe = state.lobe_size.clamp(0.0, 1.0);
    let flat = (1.0 - state.curl).clamp(0.0, 1.0);
    EarHelixControlWeights { curled, folded, wide, large_lobe, flat }
}

#[allow(dead_code)]
pub fn ear_helix_control_to_json(state: &EarHelixControlState) -> String {
    format!(
        r#"{{\"curl\":{},\"fold\":{},\"width\":{},\"lobe_size\":{}}}"#,
        state.curl, state.fold, state.width, state.lobe_size
    )
}

#[allow(dead_code)]
pub fn blend_ear_helix_controls(a: &EarHelixControlState, b: &EarHelixControlState, t: f32) -> EarHelixControlState {
    let t = t.clamp(0.0, 1.0);
    EarHelixControlState {
        curl: a.curl + (b.curl - a.curl) * t,
        fold: a.fold + (b.fold - a.fold) * t,
        width: a.width + (b.width - a.width) * t,
        lobe_size: a.lobe_size + (b.lobe_size - a.lobe_size) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ear_helix_control_config();
        assert!((0.0..=1.0).contains(&cfg.curl));
    }

    #[test]
    fn test_new_state() {
        let s = new_ear_helix_control_state();
        assert!((s.curl - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_curl_clamp() {
        let mut s = new_ear_helix_control_state();
        set_ear_helix_control_curl(&mut s, 1.5);
        assert!((s.curl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_fold() {
        let mut s = new_ear_helix_control_state();
        set_ear_helix_control_fold(&mut s, 0.8);
        assert!((s.fold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_ear_helix_control_state();
        set_ear_helix_control_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_lobe_size() {
        let mut s = new_ear_helix_control_state();
        set_ear_helix_control_lobe_size(&mut s, 0.6);
        assert!((s.lobe_size - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_ear_helix_control_state();
        let cfg = default_ear_helix_control_config();
        let w = compute_ear_helix_control_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.curled));
        assert!((0.0..=1.0).contains(&w.folded));
    }

    #[test]
    fn test_to_json() {
        let s = new_ear_helix_control_state();
        let json = ear_helix_control_to_json(&s);
        assert!(json.contains("curl"));
        assert!(json.contains("lobe_size"));
    }

    #[test]
    fn test_blend() {
        let a = new_ear_helix_control_state();
        let mut b = new_ear_helix_control_state();
        b.curl = 1.0;
        let mid = blend_ear_helix_controls(&a, &b, 0.5);
        assert!((mid.curl - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_ear_helix_control_state();
        let r = blend_ear_helix_controls(&a, &a, 0.5);
        assert!((r.curl - a.curl).abs() < 1e-6);
    }
}
