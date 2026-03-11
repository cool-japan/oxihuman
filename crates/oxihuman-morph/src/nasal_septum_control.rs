// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Nasal septum control — deviation, width, and depth morphs.

use std::f32::consts::FRAC_PI_4;

/// Nasal septum configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct NasalSeptumConfig {
    pub deviation_min: f32,
    pub deviation_max: f32,
    pub width_min: f32,
    pub width_max: f32,
}

impl Default for NasalSeptumConfig {
    fn default() -> Self {
        Self {
            deviation_min: -1.0,
            deviation_max: 1.0,
            width_min: 0.0,
            width_max: 1.0,
        }
    }
}

/// Nasal septum state.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct NasalSeptumState {
    pub deviation: f32,
    pub width: f32,
    pub depth: f32,
}

/// Morph weight output.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct NasalSeptumWeights {
    pub deviate_left: f32,
    pub deviate_right: f32,
    pub widen: f32,
    pub narrow: f32,
    pub deepen: f32,
}

/// Create default config.
#[allow(dead_code)]
pub fn default_nasal_septum_config() -> NasalSeptumConfig {
    NasalSeptumConfig::default()
}

/// Create new state.
#[allow(dead_code)]
pub fn new_nasal_septum_state() -> NasalSeptumState {
    NasalSeptumState::default()
}

/// Set deviation, clamped to config range.
#[allow(dead_code)]
pub fn ns_set_deviation(s: &mut NasalSeptumState, cfg: &NasalSeptumConfig, v: f32) {
    s.deviation = v.clamp(cfg.deviation_min, cfg.deviation_max);
}

/// Set width, clamped to `[0,1]`.
#[allow(dead_code)]
pub fn ns_set_width(s: &mut NasalSeptumState, cfg: &NasalSeptumConfig, v: f32) {
    s.width = v.clamp(cfg.width_min, cfg.width_max);
}

/// Set depth, clamped to `[0,1]`.
#[allow(dead_code)]
pub fn ns_set_depth(s: &mut NasalSeptumState, v: f32) {
    s.depth = v.clamp(0.0, 1.0);
}

/// Reset state to defaults.
#[allow(dead_code)]
pub fn ns_reset(s: &mut NasalSeptumState) {
    *s = NasalSeptumState::default();
}

/// Blend two states.
#[allow(dead_code)]
pub fn ns_blend(a: &NasalSeptumState, b: &NasalSeptumState, t: f32) -> NasalSeptumState {
    let t = t.clamp(0.0, 1.0);
    NasalSeptumState {
        deviation: a.deviation + (b.deviation - a.deviation) * t,
        width: a.width + (b.width - a.width) * t,
        depth: a.depth + (b.depth - a.depth) * t,
    }
}

/// Convert state to weights.
#[allow(dead_code)]
pub fn ns_to_weights(s: &NasalSeptumState) -> NasalSeptumWeights {
    NasalSeptumWeights {
        deviate_left: (-s.deviation).max(0.0),
        deviate_right: s.deviation.max(0.0),
        widen: s.width,
        narrow: (1.0 - s.width).max(0.0),
        deepen: s.depth,
    }
}

/// Approximate angular deviation using FRAC_PI_4 as reference range.
#[allow(dead_code)]
pub fn ns_angular_deviation_rad(s: &NasalSeptumState) -> f32 {
    s.deviation * FRAC_PI_4
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn ns_to_json(s: &NasalSeptumState) -> String {
    format!(
        r#"{{"deviation":{:.4},"width":{:.4},"depth":{:.4}}}"#,
        s.deviation, s.width, s.depth
    )
}

/// Check if state is neutral.
#[allow(dead_code)]
pub fn ns_is_neutral(s: &NasalSeptumState) -> bool {
    s.deviation.abs() < 1e-6 && s.width.abs() < 1e-6 && s.depth.abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(ns_is_neutral(&new_nasal_septum_state()));
    }

    #[test]
    fn deviation_clamped_positive() {
        let cfg = default_nasal_septum_config();
        let mut s = new_nasal_septum_state();
        ns_set_deviation(&mut s, &cfg, 5.0);
        assert!((s.deviation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn deviation_clamped_negative() {
        let cfg = default_nasal_septum_config();
        let mut s = new_nasal_septum_state();
        ns_set_deviation(&mut s, &cfg, -5.0);
        assert!((s.deviation + 1.0).abs() < 1e-6);
    }

    #[test]
    fn width_clamped() {
        let cfg = default_nasal_septum_config();
        let mut s = new_nasal_septum_state();
        ns_set_width(&mut s, &cfg, 2.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn depth_clamped() {
        let mut s = new_nasal_septum_state();
        ns_set_depth(&mut s, -0.5);
        assert!((s.depth).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_nasal_septum_config();
        let mut s = new_nasal_septum_state();
        ns_set_deviation(&mut s, &cfg, 0.5);
        ns_reset(&mut s);
        assert!(ns_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = NasalSeptumState {
            deviation: 0.0,
            width: 0.0,
            depth: 0.0,
        };
        let b = NasalSeptumState {
            deviation: 1.0,
            width: 1.0,
            depth: 1.0,
        };
        let m = ns_blend(&a, &b, 0.5);
        assert!((m.deviation - 0.5).abs() < 1e-5);
    }

    #[test]
    fn weights_deviate_right() {
        let s = NasalSeptumState {
            deviation: 0.7,
            width: 0.0,
            depth: 0.0,
        };
        let w = ns_to_weights(&s);
        assert!(w.deviate_right > 0.0 && w.deviate_left < 1e-6);
    }

    #[test]
    fn angular_deviation_uses_frac_pi_4() {
        let s = NasalSeptumState {
            deviation: 1.0,
            width: 0.0,
            depth: 0.0,
        };
        let ang = ns_angular_deviation_rad(&s);
        assert!((ang - FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn json_contains_deviation() {
        let s = NasalSeptumState {
            deviation: 0.3,
            width: 0.1,
            depth: 0.2,
        };
        assert!(ns_to_json(&s).contains("deviation"));
    }

    #[test]
    fn contains_check_range() {
        let v = 0.5f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
