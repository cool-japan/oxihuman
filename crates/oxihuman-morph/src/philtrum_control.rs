// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Philtrum (groove between nose and upper lip) morph control.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhiltrumConfig {
    pub max_depth: f32,
    pub max_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhiltrumState {
    pub depth: f32,
    pub width: f32,
    pub length: f32,
}

#[allow(dead_code)]
pub fn default_philtrum_config() -> PhiltrumConfig {
    PhiltrumConfig {
        max_depth: 1.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_philtrum_state() -> PhiltrumState {
    PhiltrumState {
        depth: 0.0,
        width: 0.5,
        length: 0.5,
    }
}

#[allow(dead_code)]
pub fn philtrum_set_depth(state: &mut PhiltrumState, cfg: &PhiltrumConfig, value: f32) {
    state.depth = value.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn philtrum_set_width(state: &mut PhiltrumState, cfg: &PhiltrumConfig, value: f32) {
    state.width = value.clamp(0.0, cfg.max_width);
}

#[allow(dead_code)]
pub fn philtrum_set_length(state: &mut PhiltrumState, value: f32) {
    state.length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn philtrum_reset(state: &mut PhiltrumState) {
    *state = new_philtrum_state();
}

#[allow(dead_code)]
pub fn philtrum_to_weights(state: &PhiltrumState) -> Vec<(String, f32)> {
    vec![
        ("philtrum_depth".to_string(), state.depth),
        ("philtrum_width".to_string(), state.width),
        ("philtrum_length".to_string(), state.length),
    ]
}

#[allow(dead_code)]
pub fn philtrum_to_json(state: &PhiltrumState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"length":{:.4}}}"#,
        state.depth, state.width, state.length
    )
}

#[allow(dead_code)]
pub fn philtrum_clamp(state: &mut PhiltrumState, cfg: &PhiltrumConfig) {
    state.depth = state.depth.clamp(0.0, cfg.max_depth);
    state.width = state.width.clamp(0.0, cfg.max_width);
    state.length = state.length.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_philtrum_config();
        assert_eq!(cfg.max_depth, 1.0);
        assert_eq!(cfg.max_width, 1.0);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_philtrum_state();
        assert_eq!(s.depth, 0.0);
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_philtrum_config();
        let mut s = new_philtrum_state();
        philtrum_set_depth(&mut s, &cfg, 2.0);
        assert_eq!(s.depth, 1.0);
        philtrum_set_depth(&mut s, &cfg, -0.5);
        assert_eq!(s.depth, 0.0);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_philtrum_config();
        let mut s = new_philtrum_state();
        philtrum_set_width(&mut s, &cfg, 0.6);
        assert!((s.width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamps() {
        let mut s = new_philtrum_state();
        philtrum_set_length(&mut s, 5.0);
        assert_eq!(s.length, 1.0);
        philtrum_set_length(&mut s, -1.0);
        assert_eq!(s.length, 0.0);
    }

    #[test]
    fn test_reset() {
        let cfg = default_philtrum_config();
        let mut s = new_philtrum_state();
        philtrum_set_depth(&mut s, &cfg, 0.9);
        philtrum_reset(&mut s);
        assert_eq!(s.depth, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_philtrum_state();
        let w = philtrum_to_weights(&s);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_to_json_contains_keys() {
        let s = new_philtrum_state();
        let j = philtrum_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("length"));
    }

    #[test]
    fn test_clamp_enforces_bounds() {
        let cfg = default_philtrum_config();
        let mut s = PhiltrumState {
            depth: 3.0,
            width: -1.0,
            length: 2.0,
        };
        philtrum_clamp(&mut s, &cfg);
        assert_eq!(s.depth, 1.0);
        assert_eq!(s.width, 0.0);
        assert_eq!(s.length, 1.0);
    }
}

// ── PhiltrumControl (simple blend API) ────────────────────────────────────────

/// Simple philtrum morph parameters (blend API).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PhiltrumControl {
    /// Groove depth, normalised 0..1.
    pub depth: f32,
    /// Groove width, normalised 0..1.
    pub width: f32,
    /// Groove length, normalised 0..1.
    pub length: f32,
}

/// Return a default philtrum control.
#[allow(dead_code)]
pub fn default_philtrum_control() -> PhiltrumControl {
    PhiltrumControl {
        depth: 0.5,
        width: 0.5,
        length: 0.5,
    }
}

/// Apply philtrum control to a morph-weight slice.
#[allow(dead_code)]
pub fn apply_philtrum_control(weights: &mut [f32], pc: &PhiltrumControl) {
    if !weights.is_empty() {
        weights[0] = pc.depth;
    }
    if weights.len() > 1 {
        weights[1] = pc.width;
    }
    if weights.len() > 2 {
        weights[2] = pc.length;
    }
}

/// Linear blend between two philtrum controls.
#[allow(dead_code)]
pub fn philtrum_blend(a: &PhiltrumControl, b: &PhiltrumControl, t: f32) -> PhiltrumControl {
    let t = t.clamp(0.0, 1.0);
    PhiltrumControl {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}
