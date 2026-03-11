// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bow leg (genu varum) leg morph.

/// Bow leg configuration.
#[derive(Debug, Clone)]
pub struct BowLegMorphConfig {
    pub varus_angle: f32,
    pub tibial_torsion: f32,
    pub ankle_inversion: f32,
}

impl Default for BowLegMorphConfig {
    fn default() -> Self {
        Self {
            varus_angle: 0.0,
            tibial_torsion: 0.0,
            ankle_inversion: 0.0,
        }
    }
}

/// Bow leg morph state.
#[derive(Debug, Clone)]
pub struct BowLegMorph {
    pub config: BowLegMorphConfig,
    pub intensity: f32,
    pub symmetric: bool,
}

impl BowLegMorph {
    pub fn new() -> Self {
        Self {
            config: BowLegMorphConfig::default(),
            intensity: 0.0,
            symmetric: true,
        }
    }
}

impl Default for BowLegMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new BowLegMorph.
pub fn new_bow_leg_morph() -> BowLegMorph {
    BowLegMorph::new()
}

/// Set varus angle factor (0.0–1.0).
pub fn bow_leg_set_varus(morph: &mut BowLegMorph, v: f32) {
    morph.config.varus_angle = v.clamp(0.0, 1.0);
}

/// Set tibial torsion component.
pub fn bow_leg_set_torsion(morph: &mut BowLegMorph, v: f32) {
    morph.config.tibial_torsion = v.clamp(0.0, 1.0);
}

/// Set ankle inversion factor.
pub fn bow_leg_set_inversion(morph: &mut BowLegMorph, v: f32) {
    morph.config.ankle_inversion = v.clamp(0.0, 1.0);
}

/// Compute effective bow-out distance factor.
pub fn bow_leg_bow_out(morph: &BowLegMorph) -> f32 {
    morph.intensity * morph.config.varus_angle
}

/// Serialize to JSON.
pub fn bow_leg_to_json(morph: &BowLegMorph) -> String {
    format!(
        r#"{{"intensity":{},"varus_angle":{},"tibial_torsion":{},"ankle_inversion":{}}}"#,
        morph.intensity,
        morph.config.varus_angle,
        morph.config.tibial_torsion,
        morph.config.ankle_inversion,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_bow_leg_morph();
        assert!((m.config.varus_angle - 0.0).abs() < 1e-6 /* default */);
    }

    #[test]
    fn test_varus_clamp_high() {
        let mut m = new_bow_leg_morph();
        bow_leg_set_varus(&mut m, 10.0);
        assert!((m.config.varus_angle - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_varus_clamp_low() {
        let mut m = new_bow_leg_morph();
        bow_leg_set_varus(&mut m, -1.0);
        assert!((m.config.varus_angle - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn test_torsion() {
        let mut m = new_bow_leg_morph();
        bow_leg_set_torsion(&mut m, 0.6);
        assert!((m.config.tibial_torsion - 0.6).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_inversion() {
        let mut m = new_bow_leg_morph();
        bow_leg_set_inversion(&mut m, 0.2);
        assert!((m.config.ankle_inversion - 0.2).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_bow_out_zero() {
        let m = new_bow_leg_morph();
        assert!((bow_leg_bow_out(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_bow_out_nonzero() {
        let mut m = new_bow_leg_morph();
        bow_leg_set_varus(&mut m, 1.0);
        m.intensity = 0.5;
        assert!((bow_leg_bow_out(&m) - 0.5).abs() < 1e-6 /* 0.5 */);
    }

    #[test]
    fn test_json_key() {
        let m = new_bow_leg_morph();
        let j = bow_leg_to_json(&m);
        assert!(j.contains("varus_angle") /* key present */);
    }

    #[test]
    fn test_default_symmetric() {
        let m = BowLegMorph::default();
        assert!(m.symmetric /* symmetric */);
    }
}
