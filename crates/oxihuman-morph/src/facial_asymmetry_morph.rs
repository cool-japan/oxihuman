// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Facial asymmetry adjustment morph.

/// Facial asymmetry configuration.
#[derive(Debug, Clone)]
pub struct FacialAsymmetryMorphConfig {
    pub left_scale: f32,
    pub right_scale: f32,
    pub vertical_offset: f32,
    pub horizontal_shift: f32,
}

impl Default for FacialAsymmetryMorphConfig {
    fn default() -> Self {
        Self {
            left_scale: 1.0,
            right_scale: 1.0,
            vertical_offset: 0.0,
            horizontal_shift: 0.0,
        }
    }
}

/// Facial asymmetry morph state.
#[derive(Debug, Clone)]
pub struct FacialAsymmetryMorph {
    pub config: FacialAsymmetryMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl FacialAsymmetryMorph {
    pub fn new() -> Self {
        Self {
            config: FacialAsymmetryMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for FacialAsymmetryMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new FacialAsymmetryMorph.
pub fn new_facial_asymmetry_morph() -> FacialAsymmetryMorph {
    FacialAsymmetryMorph::new()
}

/// Set left facial half scale.
pub fn facial_asym_set_left_scale(morph: &mut FacialAsymmetryMorph, v: f32) {
    morph.config.left_scale = v.clamp(0.5, 1.5);
}

/// Set right facial half scale.
pub fn facial_asym_set_right_scale(morph: &mut FacialAsymmetryMorph, v: f32) {
    morph.config.right_scale = v.clamp(0.5, 1.5);
}

/// Set vertical offset between halves.
pub fn facial_asym_set_vertical_offset(morph: &mut FacialAsymmetryMorph, v: f32) {
    morph.config.vertical_offset = v.clamp(-1.0, 1.0);
}

/// Set horizontal shift.
pub fn facial_asym_set_horizontal_shift(morph: &mut FacialAsymmetryMorph, v: f32) {
    morph.config.horizontal_shift = v.clamp(-1.0, 1.0);
}

/// Compute overall asymmetry score.
pub fn facial_asym_score(morph: &FacialAsymmetryMorph) -> f32 {
    let scale_diff = (morph.config.left_scale - morph.config.right_scale).abs();
    (scale_diff + morph.config.vertical_offset.abs() * 0.5) * morph.intensity
}

/// Serialize to JSON.
pub fn facial_asymmetry_to_json(morph: &FacialAsymmetryMorph) -> String {
    format!(
        r#"{{"intensity":{},"left_scale":{},"right_scale":{},"vertical_offset":{},"horizontal_shift":{}}}"#,
        morph.intensity,
        morph.config.left_scale,
        morph.config.right_scale,
        morph.config.vertical_offset,
        morph.config.horizontal_shift,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_facial_asymmetry_morph();
        assert!((m.config.left_scale - 1.0).abs() < 1e-6 /* default 1 */);
    }

    #[test]
    fn test_left_scale_clamp() {
        let mut m = new_facial_asymmetry_morph();
        facial_asym_set_left_scale(&mut m, 3.0);
        assert!((m.config.left_scale - 1.5).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_right_scale() {
        let mut m = new_facial_asymmetry_morph();
        facial_asym_set_right_scale(&mut m, 0.9);
        assert!((m.config.right_scale - 0.9).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_vertical_offset() {
        let mut m = new_facial_asymmetry_morph();
        facial_asym_set_vertical_offset(&mut m, 0.3);
        assert!((m.config.vertical_offset - 0.3).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_horizontal_shift() {
        let mut m = new_facial_asymmetry_morph();
        facial_asym_set_horizontal_shift(&mut m, -0.2);
        assert!((m.config.horizontal_shift - (-0.2)).abs() < 1e-6 /* negative ok */);
    }

    #[test]
    fn test_score_symmetric() {
        let m = new_facial_asymmetry_morph();
        assert!((facial_asym_score(&m) - 0.0).abs() < 1e-6 /* zero for symmetric */);
    }

    #[test]
    fn test_score_asymmetric() {
        let mut m = new_facial_asymmetry_morph();
        facial_asym_set_left_scale(&mut m, 1.2);
        facial_asym_set_right_scale(&mut m, 0.8);
        m.intensity = 1.0;
        assert!(facial_asym_score(&m) > 0.0 /* nonzero */);
    }

    #[test]
    fn test_json_key() {
        let m = new_facial_asymmetry_morph();
        let j = facial_asymmetry_to_json(&m);
        assert!(j.contains("left_scale") /* key */);
    }

    #[test]
    fn test_default_enabled() {
        let m = FacialAsymmetryMorph::default();
        assert!(m.enabled /* enabled */);
    }
}
