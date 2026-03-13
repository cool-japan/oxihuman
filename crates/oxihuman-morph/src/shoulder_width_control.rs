// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Shoulder width/breadth morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderWidthConfig {
    pub min_width: f32,
    pub max_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderWidthState {
    pub width: f32,
    pub slope: f32,
    pub roll: f32,
}

#[allow(dead_code)]
pub fn default_shoulder_width_config() -> ShoulderWidthConfig {
    ShoulderWidthConfig { min_width: 0.0, max_width: 1.0 }
}

#[allow(dead_code)]
pub fn new_shoulder_width_state() -> ShoulderWidthState {
    ShoulderWidthState { width: 0.5, slope: 0.0, roll: 0.0 }
}

#[allow(dead_code)]
pub fn sw_set_width(state: &mut ShoulderWidthState, cfg: &ShoulderWidthConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn sw_set_slope(state: &mut ShoulderWidthState, v: f32) {
    state.slope = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn sw_set_roll(state: &mut ShoulderWidthState, v: f32) {
    state.roll = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn sw_reset(state: &mut ShoulderWidthState) {
    *state = new_shoulder_width_state();
}

#[allow(dead_code)]
pub fn sw_to_weights(state: &ShoulderWidthState) -> [f32; 3] {
    [state.width, state.slope, state.roll]
}

#[allow(dead_code)]
pub fn sw_to_json(state: &ShoulderWidthState) -> String {
    format!(
        r#"{{"width":{:.4},"slope":{:.4},"roll":{:.4}}}"#,
        state.width, state.slope, state.roll
    )
}

#[allow(dead_code)]
pub fn sw_clamp(state: &mut ShoulderWidthState, cfg: &ShoulderWidthConfig) {
    state.width = state.width.clamp(cfg.min_width, cfg.max_width);
    state.slope = state.slope.clamp(-1.0, 1.0);
    state.roll = state.roll.clamp(-1.0, 1.0);
}

// ── New canonical structs/functions required by lib.rs re-export ──────────────

/// Canonical shoulder width struct (forward_roll renamed to forward_roll field).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderWidth {
    pub width: f32,
    pub slope: f32,
    pub forward_roll: f32,
}

/// Returns a default `ShoulderWidth`.
#[allow(dead_code)]
pub fn default_shoulder_width() -> ShoulderWidth {
    ShoulderWidth { width: 0.5, slope: 0.0, forward_roll: 0.0 }
}

/// Applies shoulder width values to a weight slice.
#[allow(dead_code)]
pub fn apply_shoulder_width(weights: &mut [f32], sw: &ShoulderWidth) {
    if !weights.is_empty() {
        weights[0] = sw.width;
    }
    if weights.len() > 1 {
        weights[1] = sw.slope;
    }
    if weights.len() > 2 {
        weights[2] = sw.forward_roll;
    }
}

/// Linearly blends two `ShoulderWidth` values by `t` in [0, 1].
#[allow(dead_code)]
pub fn shoulder_width_blend(a: &ShoulderWidth, b: &ShoulderWidth, t: f32) -> ShoulderWidth {
    let t = t.clamp(0.0, 1.0);
    ShoulderWidth {
        width: a.width + (b.width - a.width) * t,
        slope: a.slope + (b.slope - a.slope) * t,
        forward_roll: a.forward_roll + (b.forward_roll - a.forward_roll) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_shoulder_width_config();
        assert!((cfg.min_width - 0.0).abs() < 1e-6);
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_shoulder_width_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.slope - 0.0).abs() < 1e-6);
        assert!((s.roll - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_sw_set_width_clamps() {
        let cfg = default_shoulder_width_config();
        let mut s = new_shoulder_width_state();
        sw_set_width(&mut s, &cfg, 2.0);
        assert!((s.width - 1.0).abs() < 1e-6);
        sw_set_width(&mut s, &cfg, -1.0);
        assert!((s.width - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_sw_set_slope_clamps() {
        let mut s = new_shoulder_width_state();
        sw_set_slope(&mut s, 5.0);
        assert!((s.slope - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sw_set_roll_clamps() {
        let mut s = new_shoulder_width_state();
        sw_set_roll(&mut s, -5.0);
        assert!((s.roll - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_sw_reset() {
        let mut s = new_shoulder_width_state();
        s.width = 0.9;
        s.slope = 0.7;
        sw_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.slope - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_sw_to_weights() {
        let s = new_shoulder_width_state();
        let w = sw_to_weights(&s);
        assert_eq!(w.len(), 3);
        assert!((w[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sw_to_json() {
        let s = new_shoulder_width_state();
        let j = sw_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("slope"));
        assert!(j.contains("roll"));
    }

    #[test]
    fn test_sw_clamp() {
        let cfg = default_shoulder_width_config();
        let mut s = new_shoulder_width_state();
        s.width = 5.0;
        s.slope = 5.0;
        sw_clamp(&mut s, &cfg);
        assert!((s.width - 1.0).abs() < 1e-6);
        assert!((s.slope - 1.0).abs() < 1e-6);
    }
}
