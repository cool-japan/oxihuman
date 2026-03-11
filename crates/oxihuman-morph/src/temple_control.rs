// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Temple region morph (width/prominence of temporal area).

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TempleConfig {
    pub max_prominence: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TempleState {
    pub prominence_l: f32,
    pub prominence_r: f32,
    pub hollow_l: f32,
    pub hollow_r: f32,
}

#[allow(dead_code)]
pub fn default_temple_config() -> TempleConfig {
    TempleConfig {
        max_prominence: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_temple_state() -> TempleState {
    TempleState {
        prominence_l: 0.0,
        prominence_r: 0.0,
        hollow_l: 0.0,
        hollow_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn temple_set_prominence(state: &mut TempleState, cfg: &TempleConfig, left: f32, right: f32) {
    state.prominence_l = left.clamp(0.0, cfg.max_prominence);
    state.prominence_r = right.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn temple_set_hollow(state: &mut TempleState, left: f32, right: f32) {
    state.hollow_l = left.clamp(0.0, 1.0);
    state.hollow_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn temple_mirror(state: &mut TempleState) {
    let avg_p = (state.prominence_l + state.prominence_r) * 0.5;
    let avg_h = (state.hollow_l + state.hollow_r) * 0.5;
    state.prominence_l = avg_p;
    state.prominence_r = avg_p;
    state.hollow_l = avg_h;
    state.hollow_r = avg_h;
}

#[allow(dead_code)]
pub fn temple_reset(state: &mut TempleState) {
    *state = new_temple_state();
}

#[allow(dead_code)]
pub fn temple_to_weights(state: &TempleState) -> Vec<(String, f32)> {
    vec![
        ("temple_prominence_l".to_string(), state.prominence_l),
        ("temple_prominence_r".to_string(), state.prominence_r),
        ("temple_hollow_l".to_string(), state.hollow_l),
        ("temple_hollow_r".to_string(), state.hollow_r),
    ]
}

#[allow(dead_code)]
pub fn temple_to_json(state: &TempleState) -> String {
    format!(
        r#"{{"prominence_l":{:.4},"prominence_r":{:.4},"hollow_l":{:.4},"hollow_r":{:.4}}}"#,
        state.prominence_l, state.prominence_r, state.hollow_l, state.hollow_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_temple_config();
        assert_eq!(cfg.max_prominence, 1.0);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_temple_state();
        assert_eq!(s.prominence_l, 0.0);
        assert_eq!(s.hollow_r, 0.0);
    }

    #[test]
    fn test_set_prominence_clamps() {
        let cfg = default_temple_config();
        let mut s = new_temple_state();
        temple_set_prominence(&mut s, &cfg, 2.0, -1.0);
        assert_eq!(s.prominence_l, 1.0);
        assert_eq!(s.prominence_r, 0.0);
    }

    #[test]
    fn test_set_hollow_clamps() {
        let mut s = new_temple_state();
        temple_set_hollow(&mut s, 0.4, 1.5);
        assert!((s.hollow_l - 0.4).abs() < 1e-6);
        assert_eq!(s.hollow_r, 1.0);
    }

    #[test]
    fn test_mirror_averages() {
        let mut s = new_temple_state();
        s.prominence_l = 0.2;
        s.prominence_r = 0.6;
        temple_mirror(&mut s);
        assert!((s.prominence_l - 0.4).abs() < 1e-6);
        assert!((s.prominence_r - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_temple_config();
        let mut s = new_temple_state();
        temple_set_prominence(&mut s, &cfg, 0.8, 0.8);
        temple_reset(&mut s);
        assert_eq!(s.prominence_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_temple_state();
        assert_eq!(temple_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_temple_state();
        let j = temple_to_json(&s);
        assert!(j.contains("prominence_l"));
        assert!(j.contains("hollow_r"));
    }

    #[test]
    fn test_set_prominence_valid() {
        let cfg = default_temple_config();
        let mut s = new_temple_state();
        temple_set_prominence(&mut s, &cfg, 0.5, 0.3);
        assert!((s.prominence_l - 0.5).abs() < 1e-6);
        assert!((s.prominence_r - 0.3).abs() < 1e-6);
    }
}

// ── TempleControl (simple blend API) ──────────────────────────────────────────

/// Simple temple morph parameters (blend API).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TempleControl {
    /// Width of the temporal region, normalised 0..1.
    pub width: f32,
    /// Flatness of the temple surface (0 = curved, 1 = flat).
    pub flatness: f32,
    /// Prominence of the temporal region, normalised 0..1.
    pub prominence: f32,
}

/// Return a default temple control.
#[allow(dead_code)]
pub fn default_temple_control() -> TempleControl {
    TempleControl {
        width: 0.5,
        flatness: 0.0,
        prominence: 0.5,
    }
}

/// Apply temple control to a morph-weight slice.
#[allow(dead_code)]
pub fn apply_temple_control(weights: &mut [f32], tc: &TempleControl) {
    if !weights.is_empty() {
        weights[0] = tc.width;
    }
    if weights.len() > 1 {
        weights[1] = tc.flatness;
    }
    if weights.len() > 2 {
        weights[2] = tc.prominence;
    }
}

/// Linear blend between two temple controls.
#[allow(dead_code)]
pub fn temple_blend(a: &TempleControl, b: &TempleControl, t: f32) -> TempleControl {
    let t = t.clamp(0.0, 1.0);
    TempleControl {
        width: a.width + (b.width - a.width) * t,
        flatness: a.flatness + (b.flatness - a.flatness) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}
