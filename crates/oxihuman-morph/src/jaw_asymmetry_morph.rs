// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Jaw deviation and asymmetry morph.

/// Jaw asymmetry configuration.
#[derive(Debug, Clone)]
pub struct JawAsymmetryMorphConfig {
    pub lateral_deviation: f32,
    pub chin_shift: f32,
    pub ramus_asymmetry: f32,
}

impl Default for JawAsymmetryMorphConfig {
    fn default() -> Self {
        Self {
            lateral_deviation: 0.0,
            chin_shift: 0.0,
            ramus_asymmetry: 0.0,
        }
    }
}

/// Jaw asymmetry morph state.
#[derive(Debug, Clone)]
pub struct JawAsymmetryMorph {
    pub config: JawAsymmetryMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl JawAsymmetryMorph {
    pub fn new() -> Self {
        Self {
            config: JawAsymmetryMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for JawAsymmetryMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new JawAsymmetryMorph.
pub fn new_jaw_asymmetry_morph() -> JawAsymmetryMorph {
    JawAsymmetryMorph::new()
}

/// Set lateral jaw deviation (-1.0 left, 1.0 right).
pub fn jaw_asym_set_lateral(morph: &mut JawAsymmetryMorph, v: f32) {
    morph.config.lateral_deviation = v.clamp(-1.0, 1.0);
}

/// Set chin horizontal shift.
pub fn jaw_asym_set_chin_shift(morph: &mut JawAsymmetryMorph, v: f32) {
    morph.config.chin_shift = v.clamp(-1.0, 1.0);
}

/// Set mandibular ramus asymmetry factor.
pub fn jaw_asym_set_ramus(morph: &mut JawAsymmetryMorph, v: f32) {
    morph.config.ramus_asymmetry = v.clamp(0.0, 1.0);
}

/// Compute total jaw deviation magnitude.
pub fn jaw_asym_deviation_magnitude(morph: &JawAsymmetryMorph) -> f32 {
    (morph.config.lateral_deviation.abs() + morph.config.chin_shift.abs() * 0.5) * morph.intensity
}

/// Serialize to JSON.
pub fn jaw_asymmetry_to_json(morph: &JawAsymmetryMorph) -> String {
    format!(
        r#"{{"intensity":{},"lateral_deviation":{},"chin_shift":{},"ramus_asymmetry":{}}}"#,
        morph.intensity,
        morph.config.lateral_deviation,
        morph.config.chin_shift,
        morph.config.ramus_asymmetry,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_jaw_asymmetry_morph();
        assert!((m.config.lateral_deviation - 0.0).abs() < 1e-6 /* default */);
    }

    #[test]
    fn test_lateral_clamp_high() {
        let mut m = new_jaw_asymmetry_morph();
        jaw_asym_set_lateral(&mut m, 5.0);
        assert!((m.config.lateral_deviation - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_lateral_clamp_low() {
        let mut m = new_jaw_asymmetry_morph();
        jaw_asym_set_lateral(&mut m, -5.0);
        assert!((m.config.lateral_deviation - (-1.0)).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_chin_shift() {
        let mut m = new_jaw_asymmetry_morph();
        jaw_asym_set_chin_shift(&mut m, 0.4);
        assert!((m.config.chin_shift - 0.4).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_ramus() {
        let mut m = new_jaw_asymmetry_morph();
        jaw_asym_set_ramus(&mut m, 0.6);
        assert!((m.config.ramus_asymmetry - 0.6).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_deviation_zero() {
        let m = new_jaw_asymmetry_morph();
        assert!((jaw_asym_deviation_magnitude(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_deviation_nonzero() {
        let mut m = new_jaw_asymmetry_morph();
        jaw_asym_set_lateral(&mut m, 1.0);
        m.intensity = 1.0;
        assert!(jaw_asym_deviation_magnitude(&m) > 0.0 /* nonzero */);
    }

    #[test]
    fn test_json_key() {
        let m = new_jaw_asymmetry_morph();
        let j = jaw_asymmetry_to_json(&m);
        assert!(j.contains("lateral_deviation") /* key */);
    }

    #[test]
    fn test_default_enabled() {
        let m = JawAsymmetryMorph::default();
        assert!(m.enabled /* enabled */);
    }
}
