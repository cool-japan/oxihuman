// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pallor/paleness morph stub.

/// Pallor cause.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PallorCause {
    Anemia,
    Shock,
    Fear,
    Cold,
    Illness,
}

/// Pallor morph controller.
#[derive(Debug, Clone)]
pub struct PallorMorph {
    pub cause: PallorCause,
    pub intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl PallorMorph {
    pub fn new(morph_count: usize) -> Self {
        PallorMorph {
            cause: PallorCause::Anemia,
            intensity: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new pallor morph.
pub fn new_pallor_morph(morph_count: usize) -> PallorMorph {
    PallorMorph::new(morph_count)
}

/// Set pallor cause.
pub fn plm_set_cause(morph: &mut PallorMorph, cause: PallorCause) {
    morph.cause = cause;
}

/// Set intensity.
pub fn plm_set_intensity(morph: &mut PallorMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn plm_evaluate(morph: &PallorMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn plm_set_enabled(morph: &mut PallorMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn plm_to_json(morph: &PallorMorph) -> String {
    let cause = match morph.cause {
        PallorCause::Anemia => "anemia",
        PallorCause::Shock => "shock",
        PallorCause::Fear => "fear",
        PallorCause::Cold => "cold",
        PallorCause::Illness => "illness",
    };
    format!(
        r#"{{"cause":"{}","intensity":{},"morph_count":{},"enabled":{}}}"#,
        cause, morph.intensity, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cause() {
        let m = new_pallor_morph(4);
        assert_eq!(
            m.cause,
            PallorCause::Anemia /* default cause must be Anemia */
        );
    }

    #[test]
    fn test_set_cause() {
        let mut m = new_pallor_morph(4);
        plm_set_cause(&mut m, PallorCause::Fear);
        assert_eq!(m.cause, PallorCause::Fear /* cause must be set */);
    }

    #[test]
    fn test_intensity_clamp_high() {
        let mut m = new_pallor_morph(4);
        plm_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_intensity_clamp_low() {
        let mut m = new_pallor_morph(4);
        plm_set_intensity(&mut m, -1.0);
        assert!(m.intensity.abs() < 1e-6 /* clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_pallor_morph(5);
        plm_set_intensity(&mut m, 0.5);
        assert_eq!(
            plm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_pallor_morph(4);
        plm_set_enabled(&mut m, false);
        assert!(plm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_cause() {
        let m = new_pallor_morph(4);
        let j = plm_to_json(&m);
        assert!(j.contains("\"cause\"") /* JSON must have cause */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_pallor_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_pallor_morph(3);
        plm_set_intensity(&mut m, 0.9);
        let out = plm_evaluate(&m);
        assert!((out[0] - 0.9).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
