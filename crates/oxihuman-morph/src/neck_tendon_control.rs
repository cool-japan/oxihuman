// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Neck tendon control — sternocleidomastoid and platysma definition morphs.

use std::f32::consts::FRAC_1_SQRT_2;

/// Neck tendon configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct NeckTendonConfig {
    pub definition_min: f32,
    pub definition_max: f32,
}

impl Default for NeckTendonConfig {
    fn default() -> Self {
        Self {
            definition_min: 0.0,
            definition_max: 1.0,
        }
    }
}

/// Neck tendon state.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct NeckTendonState {
    pub scm_left: f32,
    pub scm_right: f32,
    pub platysma: f32,
    pub atlas_protrusion: f32,
}

/// Morph weight output.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct NeckTendonWeights {
    pub scm_definition_l: f32,
    pub scm_definition_r: f32,
    pub platysma_weight: f32,
    pub atlas_weight: f32,
}

/// Create default config.
#[allow(dead_code)]
pub fn default_neck_tendon_config() -> NeckTendonConfig {
    NeckTendonConfig::default()
}

/// Create new state.
#[allow(dead_code)]
pub fn new_neck_tendon_state() -> NeckTendonState {
    NeckTendonState::default()
}

/// Set SCM left definition.
#[allow(dead_code)]
pub fn nt_set_scm_left(s: &mut NeckTendonState, cfg: &NeckTendonConfig, v: f32) {
    s.scm_left = v.clamp(cfg.definition_min, cfg.definition_max);
}

/// Set SCM right definition.
#[allow(dead_code)]
pub fn nt_set_scm_right(s: &mut NeckTendonState, cfg: &NeckTendonConfig, v: f32) {
    s.scm_right = v.clamp(cfg.definition_min, cfg.definition_max);
}

/// Set both SCM sides equally.
#[allow(dead_code)]
pub fn nt_set_scm_both(s: &mut NeckTendonState, cfg: &NeckTendonConfig, v: f32) {
    let v = v.clamp(cfg.definition_min, cfg.definition_max);
    s.scm_left = v;
    s.scm_right = v;
}

/// Set platysma definition.
#[allow(dead_code)]
pub fn nt_set_platysma(s: &mut NeckTendonState, v: f32) {
    s.platysma = v.clamp(0.0, 1.0);
}

/// Set atlas protrusion.
#[allow(dead_code)]
pub fn nt_set_atlas(s: &mut NeckTendonState, v: f32) {
    s.atlas_protrusion = v.clamp(0.0, 1.0);
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn nt_reset(s: &mut NeckTendonState) {
    *s = NeckTendonState::default();
}

/// Blend two states.
#[allow(dead_code)]
pub fn nt_blend(a: &NeckTendonState, b: &NeckTendonState, t: f32) -> NeckTendonState {
    let t = t.clamp(0.0, 1.0);
    NeckTendonState {
        scm_left: a.scm_left + (b.scm_left - a.scm_left) * t,
        scm_right: a.scm_right + (b.scm_right - a.scm_right) * t,
        platysma: a.platysma + (b.platysma - a.platysma) * t,
        atlas_protrusion: a.atlas_protrusion + (b.atlas_protrusion - a.atlas_protrusion) * t,
    }
}

/// Convert state to weights.
#[allow(dead_code)]
pub fn nt_to_weights(s: &NeckTendonState) -> NeckTendonWeights {
    NeckTendonWeights {
        scm_definition_l: s.scm_left,
        scm_definition_r: s.scm_right,
        platysma_weight: s.platysma,
        atlas_weight: s.atlas_protrusion,
    }
}

/// Bilateral asymmetry score using FRAC_1_SQRT_2 as normalization.
#[allow(dead_code)]
pub fn nt_asymmetry(s: &NeckTendonState) -> f32 {
    ((s.scm_left - s.scm_right).abs() * FRAC_1_SQRT_2).min(1.0)
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn nt_to_json(s: &NeckTendonState) -> String {
    format!(
        r#"{{"scm_left":{:.4},"scm_right":{:.4},"platysma":{:.4},"atlas":{:.4}}}"#,
        s.scm_left, s.scm_right, s.platysma, s.atlas_protrusion
    )
}

/// Check if state is neutral.
#[allow(dead_code)]
pub fn nt_is_neutral(s: &NeckTendonState) -> bool {
    [s.scm_left, s.scm_right, s.platysma, s.atlas_protrusion]
        .iter()
        .all(|v| v.abs() < 1e-6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(nt_is_neutral(&new_neck_tendon_state()));
    }

    #[test]
    fn set_scm_left_clamped() {
        let cfg = default_neck_tendon_config();
        let mut s = new_neck_tendon_state();
        nt_set_scm_left(&mut s, &cfg, 2.0);
        assert!((s.scm_left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_scm_both() {
        let cfg = default_neck_tendon_config();
        let mut s = new_neck_tendon_state();
        nt_set_scm_both(&mut s, &cfg, 0.6);
        assert!((s.scm_left - 0.6).abs() < 1e-6);
        assert!((s.scm_right - 0.6).abs() < 1e-6);
    }

    #[test]
    fn platysma_clamped() {
        let mut s = new_neck_tendon_state();
        nt_set_platysma(&mut s, 1.5);
        assert!((s.platysma - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_works() {
        let cfg = default_neck_tendon_config();
        let mut s = new_neck_tendon_state();
        nt_set_scm_left(&mut s, &cfg, 0.8);
        nt_reset(&mut s);
        assert!(nt_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = NeckTendonState {
            scm_left: 0.0,
            scm_right: 0.0,
            platysma: 0.0,
            atlas_protrusion: 0.0,
        };
        let b = NeckTendonState {
            scm_left: 1.0,
            scm_right: 1.0,
            platysma: 1.0,
            atlas_protrusion: 1.0,
        };
        let m = nt_blend(&a, &b, 0.5);
        assert!((m.scm_left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn weights_correct() {
        let s = NeckTendonState {
            scm_left: 0.3,
            scm_right: 0.7,
            platysma: 0.5,
            atlas_protrusion: 0.2,
        };
        let w = nt_to_weights(&s);
        assert!((w.scm_definition_l - 0.3).abs() < 1e-6);
        assert!((w.scm_definition_r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn asymmetry_symmetric_is_zero() {
        let s = NeckTendonState {
            scm_left: 0.5,
            scm_right: 0.5,
            platysma: 0.0,
            atlas_protrusion: 0.0,
        };
        assert!(nt_asymmetry(&s) < 1e-6);
    }

    #[test]
    fn asymmetry_uses_frac1sqrt2() {
        let s = NeckTendonState {
            scm_left: 1.0,
            scm_right: 0.0,
            platysma: 0.0,
            atlas_protrusion: 0.0,
        };
        let a = nt_asymmetry(&s);
        assert!((a - FRAC_1_SQRT_2).abs() < 1e-5);
    }

    #[test]
    fn json_contains_platysma() {
        let s = NeckTendonState {
            scm_left: 0.0,
            scm_right: 0.0,
            platysma: 0.4,
            atlas_protrusion: 0.0,
        };
        assert!(nt_to_json(&s).contains("platysma"));
    }

    #[test]
    fn contains_range_check() {
        let v = 0.7f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
