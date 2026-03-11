// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Jaw depth morph control: adjusts the anterior-posterior depth of the jaw.

/// Configuration for jaw depth morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawDepthConfig {
    pub min_depth: f32,
    pub max_depth: f32,
}

/// Runtime state for jaw depth morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawDepthState {
    pub depth: f32,
    pub angle: f32,
    pub ramus_height: f32,
}

#[allow(dead_code)]
pub fn default_jaw_depth_config() -> JawDepthConfig {
    JawDepthConfig {
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_jaw_depth_state() -> JawDepthState {
    JawDepthState {
        depth: 0.5,
        angle: 0.5,
        ramus_height: 0.5,
    }
}

#[allow(dead_code)]
pub fn jd_set_depth(state: &mut JawDepthState, cfg: &JawDepthConfig, v: f32) {
    state.depth = v.clamp(cfg.min_depth, cfg.max_depth);
}

#[allow(dead_code)]
pub fn jd_set_angle(state: &mut JawDepthState, v: f32) {
    state.angle = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn jd_set_ramus(state: &mut JawDepthState, v: f32) {
    state.ramus_height = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn jd_reset(state: &mut JawDepthState) {
    *state = new_jaw_depth_state();
}

#[allow(dead_code)]
pub fn jd_to_weights(state: &JawDepthState) -> Vec<(String, f32)> {
    vec![
        ("jaw_depth".to_string(), state.depth),
        ("jaw_angle".to_string(), state.angle),
        ("jaw_ramus_height".to_string(), state.ramus_height),
    ]
}

#[allow(dead_code)]
pub fn jd_to_json(state: &JawDepthState) -> String {
    format!(
        r#"{{"depth":{:.4},"angle":{:.4},"ramus_height":{:.4}}}"#,
        state.depth, state.angle, state.ramus_height
    )
}

#[allow(dead_code)]
pub fn jd_blend(a: &JawDepthState, b: &JawDepthState, t: f32) -> JawDepthState {
    let t = t.clamp(0.0, 1.0);
    JawDepthState {
        depth: a.depth + (b.depth - a.depth) * t,
        angle: a.angle + (b.angle - a.angle) * t,
        ramus_height: a.ramus_height + (b.ramus_height - a.ramus_height) * t,
    }
}

#[allow(dead_code)]
pub fn jd_effective_depth(state: &JawDepthState) -> f32 {
    state.depth * (1.0 + state.angle * 0.15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jaw_depth_config();
        assert!(cfg.min_depth.abs() < 1e-6);
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_jaw_depth_state();
        assert!((s.depth - 0.5).abs() < 1e-6);
        assert!((s.angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_jaw_depth_config();
        let mut s = new_jaw_depth_state();
        jd_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle() {
        let mut s = new_jaw_depth_state();
        jd_set_angle(&mut s, 0.8);
        assert!((s.angle - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_ramus() {
        let mut s = new_jaw_depth_state();
        jd_set_ramus(&mut s, 0.2);
        assert!((s.ramus_height - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_jaw_depth_config();
        let mut s = new_jaw_depth_state();
        jd_set_depth(&mut s, &cfg, 0.9);
        jd_reset(&mut s);
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_jaw_depth_state();
        assert_eq!(jd_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_jaw_depth_state();
        let j = jd_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("angle"));
    }

    #[test]
    fn test_blend() {
        let a = new_jaw_depth_state();
        let mut b = new_jaw_depth_state();
        b.depth = 1.0;
        let mid = jd_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_effective_depth() {
        let s = new_jaw_depth_state();
        let d = jd_effective_depth(&s);
        assert!(d > 0.0);
    }
}
