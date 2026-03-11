// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cyanosis (blue tint) morph stub.

/// Cyanosis type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CyanosisType {
    Central,
    Peripheral,
    Differential,
}

/// Cyanosis morph controller.
#[derive(Debug, Clone)]
pub struct CyanosisMorph {
    pub cyanosis_type: CyanosisType,
    pub oxygen_saturation: f32,
    pub intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl CyanosisMorph {
    pub fn new(morph_count: usize) -> Self {
        CyanosisMorph {
            cyanosis_type: CyanosisType::Peripheral,
            oxygen_saturation: 1.0,
            intensity: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new cyanosis morph.
pub fn new_cyanosis_morph(morph_count: usize) -> CyanosisMorph {
    CyanosisMorph::new(morph_count)
}

/// Set cyanosis type.
pub fn cym_set_type(morph: &mut CyanosisMorph, cyanosis_type: CyanosisType) {
    morph.cyanosis_type = cyanosis_type;
}

/// Set oxygen saturation (0.0 = no O2, 1.0 = full).
pub fn cym_set_oxygen_saturation(morph: &mut CyanosisMorph, sat: f32) {
    morph.oxygen_saturation = sat.clamp(0.0, 1.0);
}

/// Set blue tint intensity.
pub fn cym_set_intensity(morph: &mut CyanosisMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn cym_evaluate(morph: &CyanosisMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn cym_set_enabled(morph: &mut CyanosisMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn cym_to_json(morph: &CyanosisMorph) -> String {
    let t = match morph.cyanosis_type {
        CyanosisType::Central => "central",
        CyanosisType::Peripheral => "peripheral",
        CyanosisType::Differential => "differential",
    };
    format!(
        r#"{{"type":"{}","oxygen_saturation":{},"intensity":{},"morph_count":{},"enabled":{}}}"#,
        t, morph.oxygen_saturation, morph.intensity, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_type() {
        let m = new_cyanosis_morph(4);
        assert_eq!(
            m.cyanosis_type,
            CyanosisType::Peripheral /* default must be Peripheral */
        );
    }

    #[test]
    fn test_set_type() {
        let mut m = new_cyanosis_morph(4);
        cym_set_type(&mut m, CyanosisType::Central);
        assert_eq!(
            m.cyanosis_type,
            CyanosisType::Central /* type must be set */
        );
    }

    #[test]
    fn test_oxygen_saturation_clamp() {
        let mut m = new_cyanosis_morph(4);
        cym_set_oxygen_saturation(&mut m, 1.5);
        assert!((m.oxygen_saturation - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_cyanosis_morph(4);
        cym_set_intensity(&mut m, -0.5);
        assert!(m.intensity.abs() < 1e-6 /* clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_cyanosis_morph(5);
        cym_set_intensity(&mut m, 0.6);
        assert_eq!(
            cym_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_cyanosis_morph(4);
        cym_set_enabled(&mut m, false);
        assert!(cym_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_type() {
        let m = new_cyanosis_morph(4);
        let j = cym_to_json(&m);
        assert!(j.contains("\"type\"") /* JSON must have type */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_cyanosis_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_cyanosis_morph(3);
        cym_set_intensity(&mut m, 0.5);
        let out = cym_evaluate(&m);
        assert!((out[0] - 0.5).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
