// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaundice (yellow tint) morph stub.

/// Jaundice severity grade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JaundiceSeverity {
    Mild,
    Moderate,
    Severe,
}

/// Jaundice morph controller.
#[derive(Debug, Clone)]
pub struct JaundiceMorph {
    pub severity: JaundiceSeverity,
    pub bilirubin_level: f32,
    pub intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl JaundiceMorph {
    pub fn new(morph_count: usize) -> Self {
        JaundiceMorph {
            severity: JaundiceSeverity::Mild,
            bilirubin_level: 0.0,
            intensity: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new jaundice morph.
pub fn new_jaundice_morph(morph_count: usize) -> JaundiceMorph {
    JaundiceMorph::new(morph_count)
}

/// Set severity.
pub fn jdm_set_severity(morph: &mut JaundiceMorph, severity: JaundiceSeverity) {
    morph.severity = severity;
}

/// Set bilirubin level (mg/dL, unclamped).
pub fn jdm_set_bilirubin(morph: &mut JaundiceMorph, bilirubin: f32) {
    morph.bilirubin_level = bilirubin.max(0.0);
}

/// Set yellow tint intensity.
pub fn jdm_set_intensity(morph: &mut JaundiceMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn jdm_evaluate(morph: &JaundiceMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn jdm_set_enabled(morph: &mut JaundiceMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn jdm_to_json(morph: &JaundiceMorph) -> String {
    let sev = match morph.severity {
        JaundiceSeverity::Mild => "mild",
        JaundiceSeverity::Moderate => "moderate",
        JaundiceSeverity::Severe => "severe",
    };
    format!(
        r#"{{"severity":"{}","bilirubin_level":{},"intensity":{},"morph_count":{},"enabled":{}}}"#,
        sev, morph.bilirubin_level, morph.intensity, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_severity() {
        let m = new_jaundice_morph(4);
        assert_eq!(
            m.severity,
            JaundiceSeverity::Mild /* default severity must be Mild */
        );
    }

    #[test]
    fn test_set_severity() {
        let mut m = new_jaundice_morph(4);
        jdm_set_severity(&mut m, JaundiceSeverity::Severe);
        assert_eq!(
            m.severity,
            JaundiceSeverity::Severe /* severity must be set */
        );
    }

    #[test]
    fn test_bilirubin_clamp() {
        let mut m = new_jaundice_morph(4);
        jdm_set_bilirubin(&mut m, -5.0);
        assert!(m.bilirubin_level.abs() < 1e-6 /* bilirubin must not be negative */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_jaundice_morph(4);
        jdm_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_jaundice_morph(6);
        jdm_set_intensity(&mut m, 0.5);
        assert_eq!(
            jdm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_jaundice_morph(4);
        jdm_set_enabled(&mut m, false);
        assert!(jdm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_severity() {
        let m = new_jaundice_morph(4);
        let j = jdm_to_json(&m);
        assert!(j.contains("\"severity\"") /* JSON must have severity */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_jaundice_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_jaundice_morph(3);
        jdm_set_intensity(&mut m, 0.3);
        let out = jdm_evaluate(&m);
        assert!((out[0] - 0.3).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
