// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Waist circumference/shape morph control.

#![allow(dead_code)]

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaistConfig {
    pub min_width: f32,
    pub max_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaistState {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
}

#[allow(dead_code)]
pub fn default_waist_config() -> WaistConfig {
    WaistConfig {
        min_width: 0.2,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_waist_state() -> WaistState {
    WaistState {
        width: 0.5,
        depth: 0.5,
        height: 0.5,
    }
}

#[allow(dead_code)]
pub fn waist_set_width(state: &mut WaistState, cfg: &WaistConfig, value: f32) {
    state.width = value.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn waist_set_depth(state: &mut WaistState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
}

/// Approximate circumference as an ellipse perimeter (Ramanujan approximation).
#[allow(dead_code)]
pub fn waist_compute_circumference(state: &WaistState, scale_m: f32) -> f32 {
    let a = state.width * scale_m;
    let b = state.depth * scale_m;
    let h = (a - b).powi(2) / (a + b).powi(2);
    PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
}

#[allow(dead_code)]
pub fn waist_reset(state: &mut WaistState) {
    *state = new_waist_state();
}

#[allow(dead_code)]
pub fn waist_to_weights(state: &WaistState) -> Vec<(String, f32)> {
    vec![
        ("waist_width".to_string(), state.width),
        ("waist_depth".to_string(), state.depth),
        ("waist_height".to_string(), state.height),
    ]
}

#[allow(dead_code)]
pub fn waist_to_json(state: &WaistState) -> String {
    format!(
        r#"{{"width":{:.4},"depth":{:.4},"height":{:.4}}}"#,
        state.width, state.depth, state.height
    )
}

#[allow(dead_code)]
pub fn waist_clamp(state: &mut WaistState, cfg: &WaistConfig) {
    state.width = state.width.clamp(cfg.min_width, cfg.max_width);
    state.depth = state.depth.clamp(0.0, 1.0);
    state.height = state.height.clamp(0.0, 1.0);
}

// ── New canonical structs/functions required by lib.rs re-export ──────────────

/// Canonical waist control struct.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaistControl {
    pub width: f32,
    pub depth: f32,
    pub height_offset: f32,
}

/// Returns a default `WaistControl`.
#[allow(dead_code)]
pub fn default_waist_control() -> WaistControl {
    WaistControl {
        width: 0.5,
        depth: 0.5,
        height_offset: 0.0,
    }
}

/// Applies waist control values to a weight slice.
#[allow(dead_code)]
pub fn apply_waist_control(weights: &mut [f32], wc: &WaistControl) {
    if !weights.is_empty() {
        weights[0] = wc.width;
    }
    if weights.len() > 1 {
        weights[1] = wc.depth;
    }
    if weights.len() > 2 {
        weights[2] = wc.height_offset;
    }
}

/// Linearly blends two `WaistControl` values by `t` in [0, 1].
#[allow(dead_code)]
pub fn waist_control_blend(a: &WaistControl, b: &WaistControl, t: f32) -> WaistControl {
    let t = t.clamp(0.0, 1.0);
    WaistControl {
        width: a.width + (b.width - a.width) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        height_offset: a.height_offset + (b.height_offset - a.height_offset) * t,
    }
}

/// Create a default `WaistControl` (alias for `default_waist_control`).
#[allow(dead_code)]
pub fn new_waist_control() -> WaistControl {
    default_waist_control()
}

/// Set the width field on a `WaistControl`.
#[allow(dead_code)]
pub fn set_waist_width(wc: &mut WaistControl, w: f32) {
    wc.width = w;
}

/// Set the depth field on a `WaistControl`.
#[allow(dead_code)]
pub fn set_waist_depth(wc: &mut WaistControl, d: f32) {
    wc.depth = d;
}

/// Return the waist-to-hip ratio given a hip width `hip_w`.
#[allow(dead_code)]
pub fn waist_ratio_to_hip(wc: &WaistControl, hip_w: f32) -> f32 {
    wc.width / hip_w.max(f32::EPSILON)
}

/// Convert waist width to a scalar parameter in [0, 1].
#[allow(dead_code)]
pub fn waist_to_param(wc: &WaistControl, min_w: f32, max_w: f32) -> f32 {
    let range = (max_w - min_w).max(f32::EPSILON);
    ((wc.width - min_w) / range).clamp(0.0, 1.0)
}

/// Reconstruct a `WaistControl` from a scalar parameter in [0, 1].
#[allow(dead_code)]
pub fn waist_from_param(param: f32, min_w: f32, max_w: f32) -> WaistControl {
    WaistControl {
        width: min_w + param.clamp(0.0, 1.0) * (max_w - min_w),
        depth: 0.5,
        height_offset: 0.0,
    }
}

/// Approximate waist circumference using ellipse perimeter (Ramanujan approximation).
#[allow(dead_code)]
pub fn waist_circumference_approx(wc: &WaistControl) -> f32 {
    let a = wc.width;
    let b = wc.depth;
    let h = (a - b).powi(2) / ((a + b).powi(2) + f32::EPSILON);
    PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_waist_config();
        assert!((cfg.min_width - 0.2).abs() < 1e-6);
        assert_eq!(cfg.max_width, 1.0);
    }

    #[test]
    fn test_new_state() {
        let s = new_waist_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_waist_config();
        let mut s = new_waist_state();
        waist_set_width(&mut s, &cfg, 0.0);
        assert!((s.width - cfg.min_width).abs() < 1e-6);
        waist_set_width(&mut s, &cfg, 5.0);
        assert_eq!(s.width, cfg.max_width);
    }

    #[test]
    fn test_set_depth_clamps() {
        let mut s = new_waist_state();
        waist_set_depth(&mut s, 1.5);
        assert_eq!(s.depth, 1.0);
        waist_set_depth(&mut s, -0.5);
        assert_eq!(s.depth, 0.0);
    }

    #[test]
    fn test_circumference_positive() {
        let s = new_waist_state();
        let c = waist_compute_circumference(&s, 0.4);
        assert!(c > 0.0);
    }

    #[test]
    fn test_reset() {
        let cfg = default_waist_config();
        let mut s = new_waist_state();
        waist_set_width(&mut s, &cfg, 0.9);
        waist_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_waist_state();
        assert_eq!(waist_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json_has_keys() {
        let s = new_waist_state();
        let j = waist_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("height"));
    }

    #[test]
    fn test_clamp_enforces_bounds() {
        let cfg = default_waist_config();
        let mut s = WaistState {
            width: 0.0,
            depth: 5.0,
            height: -1.0,
        };
        waist_clamp(&mut s, &cfg);
        assert!((s.width - cfg.min_width).abs() < 1e-6);
        assert_eq!(s.depth, 1.0);
        assert_eq!(s.height, 0.0);
    }
}
