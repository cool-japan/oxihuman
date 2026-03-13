// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dental/teeth shape morph stub.

/// Dental alignment type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DentalAlignment {
    Perfect,
    SlightOverjet,
    Crowded,
    Gapped,
    Underbite,
    Overbite,
}

/// Dental morph controller.
#[derive(Debug, Clone)]
pub struct DentalMorph {
    pub alignment: DentalAlignment,
    pub tooth_size: f32,
    pub gum_exposure: f32,
    pub whitening: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl DentalMorph {
    pub fn new(morph_count: usize) -> Self {
        DentalMorph {
            alignment: DentalAlignment::Perfect,
            tooth_size: 1.0,
            gum_exposure: 0.3,
            whitening: 0.8,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new dental morph controller.
pub fn new_dental_morph(morph_count: usize) -> DentalMorph {
    DentalMorph::new(morph_count)
}

/// Set dental alignment.
pub fn dm_set_alignment(morph: &mut DentalMorph, alignment: DentalAlignment) {
    morph.alignment = alignment;
}

/// Set tooth size scale.
pub fn dm_set_tooth_size(morph: &mut DentalMorph, size: f32) {
    morph.tooth_size = size.clamp(0.5, 2.0);
}

/// Set gum exposure.
pub fn dm_set_gum_exposure(morph: &mut DentalMorph, exposure: f32) {
    morph.gum_exposure = exposure.clamp(0.0, 1.0);
}

/// Set whitening level.
pub fn dm_set_whitening(morph: &mut DentalMorph, whitening: f32) {
    morph.whitening = whitening.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: tooth_size-normalized).
pub fn dm_evaluate(morph: &DentalMorph) -> Vec<f32> {
    /* Stub: weight from tooth_size and gum_exposure */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = ((morph.tooth_size - 0.5) / 1.5) * (1.0 - morph.gum_exposure);
    vec![w.clamp(0.0, 1.0); morph.morph_count]
}

/// Enable or disable.
pub fn dm_set_enabled(morph: &mut DentalMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn dm_to_json(morph: &DentalMorph) -> String {
    let align = match morph.alignment {
        DentalAlignment::Perfect => "perfect",
        DentalAlignment::SlightOverjet => "slight_overjet",
        DentalAlignment::Crowded => "crowded",
        DentalAlignment::Gapped => "gapped",
        DentalAlignment::Underbite => "underbite",
        DentalAlignment::Overbite => "overbite",
    };
    format!(
        r#"{{"alignment":"{}","tooth_size":{},"gum_exposure":{},"enabled":{}}}"#,
        align, morph.tooth_size, morph.gum_exposure, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_alignment() {
        let m = new_dental_morph(4);
        assert_eq!(
            m.alignment,
            DentalAlignment::Perfect /* default must be Perfect */
        );
    }

    #[test]
    fn test_set_alignment() {
        let mut m = new_dental_morph(4);
        dm_set_alignment(&mut m, DentalAlignment::Crowded);
        assert_eq!(
            m.alignment,
            DentalAlignment::Crowded /* alignment must be set */
        );
    }

    #[test]
    fn test_tooth_size_clamped() {
        let mut m = new_dental_morph(4);
        dm_set_tooth_size(&mut m, 5.0);
        assert!((m.tooth_size - 2.0).abs() < 1e-6 /* tooth_size clamped to 2.0 */);
    }

    #[test]
    fn test_gum_exposure_clamped() {
        let mut m = new_dental_morph(4);
        dm_set_gum_exposure(&mut m, -0.5);
        assert!((m.gum_exposure).abs() < 1e-6 /* gum_exposure clamped to 0.0 */);
    }

    #[test]
    fn test_whitening_clamped() {
        let mut m = new_dental_morph(4);
        dm_set_whitening(&mut m, 1.5);
        assert!((m.whitening - 1.0).abs() < 1e-6 /* whitening clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_dental_morph(5);
        assert_eq!(
            dm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_dental_morph(4);
        dm_set_enabled(&mut m, false);
        assert!(dm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_alignment() {
        let m = new_dental_morph(4);
        let j = dm_to_json(&m);
        assert!(j.contains("\"alignment\"") /* JSON must have alignment */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_dental_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }
}
