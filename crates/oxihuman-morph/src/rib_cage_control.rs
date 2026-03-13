// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rib cage control — width, depth, and flare morphs.

use std::f32::consts::FRAC_PI_6;

/// Rib cage configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct RibCageConfig {
    pub width_min: f32,
    pub width_max: f32,
    pub depth_min: f32,
    pub depth_max: f32,
}

impl Default for RibCageConfig {
    fn default() -> Self {
        Self {
            width_min: -1.0,
            width_max: 1.0,
            depth_min: -1.0,
            depth_max: 1.0,
        }
    }
}

/// Rib cage state.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct RibCageState {
    pub width: f32,
    pub depth: f32,
    pub flare: f32,
    pub barrel: f32,
}

/// Morph weight output.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct RibCageWeights {
    pub wide: f32,
    pub narrow: f32,
    pub deep: f32,
    pub shallow: f32,
    pub flare_weight: f32,
    pub barrel_weight: f32,
}

/// Create default config.
#[allow(dead_code)]
pub fn default_rib_cage_config() -> RibCageConfig {
    RibCageConfig::default()
}

/// Create new state.
#[allow(dead_code)]
pub fn new_rib_cage_state() -> RibCageState {
    RibCageState::default()
}

/// Set width, clamped.
#[allow(dead_code)]
pub fn rc_set_width(s: &mut RibCageState, cfg: &RibCageConfig, v: f32) {
    s.width = v.clamp(cfg.width_min, cfg.width_max);
}

/// Set depth, clamped.
#[allow(dead_code)]
pub fn rc_set_depth(s: &mut RibCageState, cfg: &RibCageConfig, v: f32) {
    s.depth = v.clamp(cfg.depth_min, cfg.depth_max);
}

/// Set flare (0..=1).
#[allow(dead_code)]
pub fn rc_set_flare(s: &mut RibCageState, v: f32) {
    s.flare = v.clamp(0.0, 1.0);
}

/// Set barrel chest (0..=1).
#[allow(dead_code)]
pub fn rc_set_barrel(s: &mut RibCageState, v: f32) {
    s.barrel = v.clamp(0.0, 1.0);
}

/// Reset state.
#[allow(dead_code)]
pub fn rc_reset(s: &mut RibCageState) {
    *s = RibCageState::default();
}

/// Blend two states.
#[allow(dead_code)]
pub fn rc_blend(a: &RibCageState, b: &RibCageState, t: f32) -> RibCageState {
    let t = t.clamp(0.0, 1.0);
    RibCageState {
        width: a.width + (b.width - a.width) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        flare: a.flare + (b.flare - a.flare) * t,
        barrel: a.barrel + (b.barrel - a.barrel) * t,
    }
}

/// Convert state to weights.
#[allow(dead_code)]
pub fn rc_to_weights(s: &RibCageState) -> RibCageWeights {
    RibCageWeights {
        wide: s.width.max(0.0),
        narrow: (-s.width).max(0.0),
        deep: s.depth.max(0.0),
        shallow: (-s.depth).max(0.0),
        flare_weight: s.flare,
        barrel_weight: s.barrel,
    }
}

/// Approximate rib flare angle in radians using FRAC_PI_6 as the base.
#[allow(dead_code)]
pub fn rc_flare_angle_rad(s: &RibCageState) -> f32 {
    s.flare * FRAC_PI_6
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn rc_to_json(s: &RibCageState) -> String {
    format!(
        r#"{{"width":{:.4},"depth":{:.4},"flare":{:.4},"barrel":{:.4}}}"#,
        s.width, s.depth, s.flare, s.barrel
    )
}

/// Check if state is neutral.
#[allow(dead_code)]
pub fn rc_is_neutral(s: &RibCageState) -> bool {
    [s.width, s.depth, s.flare, s.barrel]
        .iter()
        .all(|v| v.abs() < 1e-6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(rc_is_neutral(&new_rib_cage_state()));
    }

    #[test]
    fn width_clamped_positive() {
        let cfg = default_rib_cage_config();
        let mut s = new_rib_cage_state();
        rc_set_width(&mut s, &cfg, 3.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn depth_clamped_negative() {
        let cfg = default_rib_cage_config();
        let mut s = new_rib_cage_state();
        rc_set_depth(&mut s, &cfg, -3.0);
        assert!((s.depth + 1.0).abs() < 1e-6);
    }

    #[test]
    fn flare_clamped() {
        let mut s = new_rib_cage_state();
        rc_set_flare(&mut s, 2.0);
        assert!((s.flare - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_works() {
        let cfg = default_rib_cage_config();
        let mut s = new_rib_cage_state();
        rc_set_width(&mut s, &cfg, 0.5);
        rc_reset(&mut s);
        assert!(rc_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = RibCageState::default();
        let b = RibCageState {
            width: 1.0,
            depth: 1.0,
            flare: 1.0,
            barrel: 1.0,
        };
        let m = rc_blend(&a, &b, 0.5);
        assert!((m.width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn weights_wide() {
        let s = RibCageState {
            width: 0.8,
            depth: 0.0,
            flare: 0.0,
            barrel: 0.0,
        };
        let w = rc_to_weights(&s);
        assert!(w.wide > 0.0 && w.narrow < 1e-6);
    }

    #[test]
    fn flare_angle_full() {
        let s = RibCageState {
            width: 0.0,
            depth: 0.0,
            flare: 1.0,
            barrel: 0.0,
        };
        let a = rc_flare_angle_rad(&s);
        assert!((a - FRAC_PI_6).abs() < 1e-6);
    }

    #[test]
    fn json_contains_barrel() {
        let s = RibCageState {
            width: 0.0,
            depth: 0.0,
            flare: 0.0,
            barrel: 0.5,
        };
        assert!(rc_to_json(&s).contains("barrel"));
    }

    #[test]
    fn slice_not_empty_check() {
        let weights = [0.1f32, 0.2, 0.3];
        assert!(!weights.is_empty());
    }
}
