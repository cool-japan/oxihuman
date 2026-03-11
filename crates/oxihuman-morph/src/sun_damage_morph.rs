// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV/sun damage skin morph stub.

/// Sun damage severity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SunDamageSeverity {
    None,
    Mild,
    Moderate,
    Severe,
}

/// Sun damage morph controller.
#[derive(Debug, Clone)]
pub struct SunDamageMorph {
    pub severity: SunDamageSeverity,
    pub exposure_years: f32,
    pub intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl SunDamageMorph {
    pub fn new(morph_count: usize) -> Self {
        SunDamageMorph {
            severity: SunDamageSeverity::None,
            exposure_years: 0.0,
            intensity: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new sun damage morph.
pub fn new_sun_damage_morph(morph_count: usize) -> SunDamageMorph {
    SunDamageMorph::new(morph_count)
}

/// Set severity.
pub fn sdm_set_severity(morph: &mut SunDamageMorph, severity: SunDamageSeverity) {
    morph.severity = severity;
}

/// Set accumulated exposure in years.
pub fn sdm_set_exposure_years(morph: &mut SunDamageMorph, years: f32) {
    morph.exposure_years = years.max(0.0);
}

/// Set overall intensity.
pub fn sdm_set_intensity(morph: &mut SunDamageMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn sdm_evaluate(morph: &SunDamageMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn sdm_set_enabled(morph: &mut SunDamageMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn sdm_to_json(morph: &SunDamageMorph) -> String {
    let sev = match morph.severity {
        SunDamageSeverity::None => "none",
        SunDamageSeverity::Mild => "mild",
        SunDamageSeverity::Moderate => "moderate",
        SunDamageSeverity::Severe => "severe",
    };
    format!(
        r#"{{"severity":"{}","exposure_years":{},"intensity":{},"morph_count":{},"enabled":{}}}"#,
        sev, morph.exposure_years, morph.intensity, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_severity() {
        let m = new_sun_damage_morph(4);
        assert_eq!(
            m.severity,
            SunDamageSeverity::None /* default severity must be None */
        );
    }

    #[test]
    fn test_set_severity() {
        let mut m = new_sun_damage_morph(4);
        sdm_set_severity(&mut m, SunDamageSeverity::Severe);
        assert_eq!(
            m.severity,
            SunDamageSeverity::Severe /* severity must be set */
        );
    }

    #[test]
    fn test_exposure_years_clamp() {
        let mut m = new_sun_damage_morph(4);
        sdm_set_exposure_years(&mut m, -5.0);
        assert!(m.exposure_years.abs() < 1e-6 /* exposure_years must not be negative */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_sun_damage_morph(4);
        sdm_set_intensity(&mut m, 1.5);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* intensity clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_sun_damage_morph(6);
        sdm_set_intensity(&mut m, 0.5);
        assert_eq!(
            sdm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_sun_damage_morph(4);
        sdm_set_enabled(&mut m, false);
        assert!(sdm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_evaluate_zero_count() {
        let m = new_sun_damage_morph(0);
        assert!(sdm_evaluate(&m).is_empty() /* zero count must return empty */);
    }

    #[test]
    fn test_to_json_has_severity() {
        let m = new_sun_damage_morph(4);
        let j = sdm_to_json(&m);
        assert!(j.contains("\"severity\"") /* JSON must have severity */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_sun_damage_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_sun_damage_morph(3);
        sdm_set_intensity(&mut m, 0.4);
        let out = sdm_evaluate(&m);
        assert!((out[0] - 0.4).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
