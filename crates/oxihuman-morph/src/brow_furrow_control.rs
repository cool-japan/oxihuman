// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brow furrow and frown line morph controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowFurrowConfig {
    pub max_depth: f32,
    pub lateral_spread: f32,
    pub corrugator_strength: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowFurrowState {
    pub furrow_depth: f32,
    pub left_intensity: f32,
    pub right_intensity: f32,
    pub vertical_lines: f32,
    pub horizontal_lines: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowFurrowWeights {
    pub corrugator_left: f32,
    pub corrugator_right: f32,
    pub procerus: f32,
    pub vertical_crease: f32,
    pub horizontal_crease: f32,
}

#[allow(dead_code)]
pub fn default_brow_furrow_config() -> BrowFurrowConfig {
    BrowFurrowConfig {
        max_depth: 1.0,
        lateral_spread: 0.5,
        corrugator_strength: 0.8,
    }
}

#[allow(dead_code)]
pub fn new_brow_furrow_state() -> BrowFurrowState {
    BrowFurrowState {
        furrow_depth: 0.0,
        left_intensity: 0.0,
        right_intensity: 0.0,
        vertical_lines: 0.0,
        horizontal_lines: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_furrow_depth(state: &mut BrowFurrowState, value: f32) {
    state.furrow_depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_furrow_asymmetry(state: &mut BrowFurrowState, left: f32, right: f32) {
    state.left_intensity = left.clamp(0.0, 1.0);
    state.right_intensity = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_vertical_lines(state: &mut BrowFurrowState, value: f32) {
    state.vertical_lines = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_furrow_weights(state: &BrowFurrowState, cfg: &BrowFurrowConfig) -> BrowFurrowWeights {
    let depth = state.furrow_depth * cfg.max_depth;
    let cl = (state.left_intensity * cfg.corrugator_strength + depth * 0.5).clamp(0.0, 1.0);
    let cr = (state.right_intensity * cfg.corrugator_strength + depth * 0.5).clamp(0.0, 1.0);
    let proc = (depth * cfg.lateral_spread).clamp(0.0, 1.0);
    let vc = (state.vertical_lines * depth).clamp(0.0, 1.0);
    let hc = (state.horizontal_lines * depth * 0.5).clamp(0.0, 1.0);
    BrowFurrowWeights {
        corrugator_left: cl,
        corrugator_right: cr,
        procerus: proc,
        vertical_crease: vc,
        horizontal_crease: hc,
    }
}

#[allow(dead_code)]
pub fn furrow_to_json(state: &BrowFurrowState) -> String {
    format!(
        r#"{{"furrow_depth":{},"left":{},"right":{},"vertical":{},"horizontal":{}}}"#,
        state.furrow_depth, state.left_intensity, state.right_intensity,
        state.vertical_lines, state.horizontal_lines
    )
}

#[allow(dead_code)]
pub fn blend_furrow_states(a: &BrowFurrowState, b: &BrowFurrowState, t: f32) -> BrowFurrowState {
    let t = t.clamp(0.0, 1.0);
    BrowFurrowState {
        furrow_depth: a.furrow_depth + (b.furrow_depth - a.furrow_depth) * t,
        left_intensity: a.left_intensity + (b.left_intensity - a.left_intensity) * t,
        right_intensity: a.right_intensity + (b.right_intensity - a.right_intensity) * t,
        vertical_lines: a.vertical_lines + (b.vertical_lines - a.vertical_lines) * t,
        horizontal_lines: a.horizontal_lines + (b.horizontal_lines - a.horizontal_lines) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_furrow_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeroed() {
        let s = new_brow_furrow_state();
        assert!(s.furrow_depth.abs() < 1e-6);
    }

    #[test]
    fn test_set_furrow_depth() {
        let mut s = new_brow_furrow_state();
        set_furrow_depth(&mut s, 0.7);
        assert!((s.furrow_depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_depth() {
        let mut s = new_brow_furrow_state();
        set_furrow_depth(&mut s, 5.0);
        assert!((s.furrow_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry() {
        let mut s = new_brow_furrow_state();
        set_furrow_asymmetry(&mut s, 0.8, 0.3);
        assert!((s.left_intensity - 0.8).abs() < 1e-6);
        assert!((s.right_intensity - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_range() {
        let mut s = new_brow_furrow_state();
        s.furrow_depth = 0.8;
        s.left_intensity = 0.6;
        s.right_intensity = 0.4;
        let cfg = default_brow_furrow_config();
        let w = compute_furrow_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.corrugator_left));
        assert!((0.0..=1.0).contains(&w.procerus));
    }

    #[test]
    fn test_to_json() {
        let s = new_brow_furrow_state();
        let j = furrow_to_json(&s);
        assert!(j.contains("furrow_depth"));
    }

    #[test]
    fn test_blend() {
        let a = new_brow_furrow_state();
        let mut b = new_brow_furrow_state();
        b.furrow_depth = 1.0;
        let mid = blend_furrow_states(&a, &b, 0.5);
        assert!((mid.furrow_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical_lines() {
        let mut s = new_brow_furrow_state();
        set_vertical_lines(&mut s, 0.6);
        assert!((s.vertical_lines - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_brow_furrow_state();
        let r = blend_furrow_states(&a, &a, 0.5);
        assert!((r.furrow_depth - a.furrow_depth).abs() < 1e-6);
    }
}
