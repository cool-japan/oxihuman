// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Forehead horizontal crease control — depth and count of forehead lines.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ForeheadCreaseConfig {
    pub max_depth_m: f32,
    pub max_lines: u32,
}

impl Default for ForeheadCreaseConfig {
    fn default() -> Self {
        Self {
            max_depth_m: 0.002,
            max_lines: 5,
        }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ForeheadCreaseState {
    /// Overall crease depth, 0..=1.
    pub depth: f32,
    /// Lateral spread of creases, 0..=1.
    pub spread: f32,
    /// Number of lines active (0..=max_lines).
    pub line_count: u32,
}

#[allow(dead_code)]
pub fn new_forehead_crease_state() -> ForeheadCreaseState {
    ForeheadCreaseState::default()
}

#[allow(dead_code)]
pub fn default_forehead_crease_config() -> ForeheadCreaseConfig {
    ForeheadCreaseConfig::default()
}

#[allow(dead_code)]
pub fn fhc_set_depth(state: &mut ForeheadCreaseState, v: f32) {
    state.depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fhc_set_spread(state: &mut ForeheadCreaseState, v: f32) {
    state.spread = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fhc_set_lines(state: &mut ForeheadCreaseState, n: u32, cfg: &ForeheadCreaseConfig) {
    state.line_count = n.min(cfg.max_lines);
}

#[allow(dead_code)]
pub fn fhc_reset(state: &mut ForeheadCreaseState) {
    *state = ForeheadCreaseState::default();
}

#[allow(dead_code)]
pub fn fhc_is_neutral(state: &ForeheadCreaseState) -> bool {
    state.depth < 1e-4 && state.spread < 1e-4 && state.line_count == 0
}

/// Effective depth in metres.
#[allow(dead_code)]
pub fn fhc_effective_depth(state: &ForeheadCreaseState, cfg: &ForeheadCreaseConfig) -> f32 {
    state.depth * cfg.max_depth_m
}

/// Wrinkle intensity (depth × spread).
#[allow(dead_code)]
pub fn fhc_intensity(state: &ForeheadCreaseState) -> f32 {
    state.depth * state.spread
}

#[allow(dead_code)]
pub fn fhc_blend(a: &ForeheadCreaseState, b: &ForeheadCreaseState, t: f32) -> ForeheadCreaseState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let lines = if t >= 0.5 { b.line_count } else { a.line_count };
    ForeheadCreaseState {
        depth: a.depth * inv + b.depth * t,
        spread: a.spread * inv + b.spread * t,
        line_count: lines,
    }
}

#[allow(dead_code)]
pub fn fhc_to_json(state: &ForeheadCreaseState) -> String {
    format!(
        "{{\"depth\":{:.4},\"spread\":{:.4},\"line_count\":{}}}",
        state.depth, state.spread, state.line_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(fhc_is_neutral(&new_forehead_crease_state()));
    }

    #[test]
    fn depth_clamps_high() {
        let mut s = new_forehead_crease_state();
        fhc_set_depth(&mut s, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn depth_clamps_low() {
        let mut s = new_forehead_crease_state();
        fhc_set_depth(&mut s, -1.0);
        assert!(s.depth < 1e-6);
    }

    #[test]
    fn spread_clamps() {
        let mut s = new_forehead_crease_state();
        fhc_set_spread(&mut s, 3.0);
        assert!((s.spread - 1.0).abs() < 1e-6);
    }

    #[test]
    fn line_count_clamps_to_max() {
        let cfg = default_forehead_crease_config();
        let mut s = new_forehead_crease_state();
        fhc_set_lines(&mut s, 100, &cfg);
        assert!(s.line_count <= cfg.max_lines);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_forehead_crease_config();
        let mut s = new_forehead_crease_state();
        fhc_set_depth(&mut s, 0.8);
        fhc_set_lines(&mut s, 3, &cfg);
        fhc_reset(&mut s);
        assert!(fhc_is_neutral(&s));
    }

    #[test]
    fn effective_depth_zero_at_neutral() {
        let cfg = default_forehead_crease_config();
        let s = new_forehead_crease_state();
        assert!(fhc_effective_depth(&s, &cfg) < 1e-8);
    }

    #[test]
    fn intensity_product_of_depth_and_spread() {
        let mut s = new_forehead_crease_state();
        fhc_set_depth(&mut s, 0.5);
        fhc_set_spread(&mut s, 0.8);
        assert!((fhc_intensity(&s) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn blend_midpoint() {
        let b = ForeheadCreaseState {
            depth: 1.0,
            spread: 1.0,
            line_count: 4,
        };
        let r = fhc_blend(&new_forehead_crease_state(), &b, 0.5);
        assert!((r.depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_depth() {
        let j = fhc_to_json(&new_forehead_crease_state());
        assert!(j.contains("depth") && j.contains("line_count"));
    }
}
