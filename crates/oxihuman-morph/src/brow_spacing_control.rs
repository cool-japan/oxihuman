// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Brow spacing control: adjusts the horizontal distance between eyebrows.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowSpacingConfig {
    pub min_spacing: f32,
    pub max_spacing: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowSpacingState {
    pub spacing: f32,
    pub arch_offset: f32,
    pub symmetry: f32,
}

#[allow(dead_code)]
pub fn default_brow_spacing_config() -> BrowSpacingConfig {
    BrowSpacingConfig {
        min_spacing: 0.0,
        max_spacing: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_brow_spacing_state() -> BrowSpacingState {
    BrowSpacingState {
        spacing: 0.5,
        arch_offset: 0.0,
        symmetry: 1.0,
    }
}

#[allow(dead_code)]
pub fn bs_set_spacing(state: &mut BrowSpacingState, cfg: &BrowSpacingConfig, v: f32) {
    state.spacing = v.clamp(cfg.min_spacing, cfg.max_spacing);
}

#[allow(dead_code)]
pub fn bs_set_arch_offset(state: &mut BrowSpacingState, v: f32) {
    state.arch_offset = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn bs_set_symmetry(state: &mut BrowSpacingState, v: f32) {
    state.symmetry = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn bs_reset(state: &mut BrowSpacingState) {
    *state = new_brow_spacing_state();
}

#[allow(dead_code)]
pub fn bs_effective_left(state: &BrowSpacingState) -> f32 {
    state.spacing * state.symmetry
}

#[allow(dead_code)]
pub fn bs_effective_right(state: &BrowSpacingState) -> f32 {
    state.spacing * (2.0 - state.symmetry)
}

#[allow(dead_code)]
pub fn bs_to_weights(state: &BrowSpacingState) -> Vec<(String, f32)> {
    vec![
        ("brow_spacing".to_string(), state.spacing),
        ("brow_arch_offset".to_string(), state.arch_offset),
        ("brow_symmetry".to_string(), state.symmetry),
    ]
}

#[allow(dead_code)]
pub fn bs_to_json(state: &BrowSpacingState) -> String {
    format!(
        r#"{{"spacing":{:.4},"arch_offset":{:.4},"symmetry":{:.4}}}"#,
        state.spacing, state.arch_offset, state.symmetry
    )
}

#[allow(dead_code)]
pub fn bs_blend(a: &BrowSpacingState, b: &BrowSpacingState, t: f32) -> BrowSpacingState {
    let t = t.clamp(0.0, 1.0);
    BrowSpacingState {
        spacing: a.spacing + (b.spacing - a.spacing) * t,
        arch_offset: a.arch_offset + (b.arch_offset - a.arch_offset) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
    }
}

/// Compute arch angle from offset.
#[allow(dead_code)]
pub fn bs_arch_angle(state: &BrowSpacingState) -> f32 {
    state.arch_offset * FRAC_PI_4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_spacing_config();
        assert!(cfg.min_spacing.abs() < 1e-6);
        assert!((cfg.max_spacing - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_brow_spacing_state();
        assert!((s.spacing - 0.5).abs() < 1e-6);
        assert!((s.symmetry - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_spacing_clamps() {
        let cfg = default_brow_spacing_config();
        let mut s = new_brow_spacing_state();
        bs_set_spacing(&mut s, &cfg, 5.0);
        assert!((s.spacing - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch_offset() {
        let mut s = new_brow_spacing_state();
        bs_set_arch_offset(&mut s, 0.7);
        assert!((s.arch_offset - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_symmetry() {
        let mut s = new_brow_spacing_state();
        bs_set_symmetry(&mut s, 0.8);
        assert!((s.symmetry - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_brow_spacing_config();
        let mut s = new_brow_spacing_state();
        bs_set_spacing(&mut s, &cfg, 0.9);
        bs_reset(&mut s);
        assert!((s.spacing - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_effective_sides() {
        let s = new_brow_spacing_state();
        let l = bs_effective_left(&s);
        let r = bs_effective_right(&s);
        assert!((l - r).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_brow_spacing_state();
        assert_eq!(bs_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_blend() {
        let a = new_brow_spacing_state();
        let mut b = new_brow_spacing_state();
        b.spacing = 1.0;
        let mid = bs_blend(&a, &b, 0.5);
        assert!((mid.spacing - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_arch_angle() {
        let mut s = new_brow_spacing_state();
        s.arch_offset = 1.0;
        let angle = bs_arch_angle(&s);
        assert!((angle - FRAC_PI_4).abs() < 1e-6);
    }
}
