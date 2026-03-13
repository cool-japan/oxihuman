// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin hydration level morph stub.

/// Hydration level category.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HydrationLevel {
    Dehydrated,
    Dry,
    Normal,
    Hydrated,
    Overhydrated,
}

/// Skin hydration morph controller.
#[derive(Debug, Clone)]
pub struct HydrationMorph {
    pub level: HydrationLevel,
    pub intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl HydrationMorph {
    pub fn new(morph_count: usize) -> Self {
        HydrationMorph {
            level: HydrationLevel::Normal,
            intensity: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new hydration morph.
pub fn new_hydration_morph(morph_count: usize) -> HydrationMorph {
    HydrationMorph::new(morph_count)
}

/// Set hydration level.
pub fn hym_set_level(morph: &mut HydrationMorph, level: HydrationLevel) {
    morph.level = level;
}

/// Set global intensity.
pub fn hym_set_intensity(morph: &mut HydrationMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn hym_evaluate(morph: &HydrationMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn hym_set_enabled(morph: &mut HydrationMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn hym_to_json(morph: &HydrationMorph) -> String {
    let lvl = match morph.level {
        HydrationLevel::Dehydrated => "dehydrated",
        HydrationLevel::Dry => "dry",
        HydrationLevel::Normal => "normal",
        HydrationLevel::Hydrated => "hydrated",
        HydrationLevel::Overhydrated => "overhydrated",
    };
    format!(
        r#"{{"level":"{}","intensity":{},"morph_count":{},"enabled":{}}}"#,
        lvl, morph.intensity, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_level() {
        let m = new_hydration_morph(4);
        assert_eq!(
            m.level,
            HydrationLevel::Normal /* default level must be Normal */
        );
    }

    #[test]
    fn test_set_level() {
        let mut m = new_hydration_morph(4);
        hym_set_level(&mut m, HydrationLevel::Dry);
        assert_eq!(m.level, HydrationLevel::Dry /* level must be set */);
    }

    #[test]
    fn test_intensity_clamp_high() {
        let mut m = new_hydration_morph(4);
        hym_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* intensity clamped to 1.0 */);
    }

    #[test]
    fn test_intensity_clamp_low() {
        let mut m = new_hydration_morph(4);
        hym_set_intensity(&mut m, -0.5);
        assert!(m.intensity.abs() < 1e-6 /* intensity clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_hydration_morph(5);
        assert_eq!(
            hym_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_hydration_morph(4);
        hym_set_enabled(&mut m, false);
        assert!(hym_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_evaluate_zero_count() {
        let m = new_hydration_morph(0);
        assert!(hym_evaluate(&m).is_empty() /* zero count must return empty */);
    }

    #[test]
    fn test_to_json_has_level() {
        let m = new_hydration_morph(4);
        let j = hym_to_json(&m);
        assert!(j.contains("\"level\"") /* JSON must have level */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_hydration_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_hydration_morph(3);
        hym_set_intensity(&mut m, 0.7);
        let out = hym_evaluate(&m);
        assert!((out[0] - 0.7).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
