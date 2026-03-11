// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bruise/contusion discoloration morph stub.

/// Bruise age stage (healing progression).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BruiseStage {
    Fresh,
    Day1to2,
    Day3to5,
    Day6to10,
    Fading,
}

/// A bruise entry.
#[derive(Debug, Clone)]
pub struct BruiseEntry {
    pub stage: BruiseStage,
    pub position: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
}

/// Bruise morph controller.
#[derive(Debug, Clone)]
pub struct BruiseMorph {
    pub bruises: Vec<BruiseEntry>,
    pub global_intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl BruiseMorph {
    pub fn new(morph_count: usize) -> Self {
        BruiseMorph {
            bruises: Vec::new(),
            global_intensity: 1.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new bruise morph.
pub fn new_bruise_morph(morph_count: usize) -> BruiseMorph {
    BruiseMorph::new(morph_count)
}

/// Add a bruise entry.
pub fn brm_add_bruise(morph: &mut BruiseMorph, entry: BruiseEntry) {
    morph.bruises.push(entry);
}

/// Set global intensity.
pub fn brm_set_intensity(morph: &mut BruiseMorph, intensity: f32) {
    morph.global_intensity = intensity.clamp(0.0, 1.0);
}

/// Clear all bruises.
pub fn brm_clear(morph: &mut BruiseMorph) {
    morph.bruises.clear();
}

/// Evaluate morph weights (stub: uniform from global_intensity).
pub fn brm_evaluate(morph: &BruiseMorph) -> Vec<f32> {
    /* Stub: uniform weight from global_intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_intensity; morph.morph_count]
}

/// Enable or disable.
pub fn brm_set_enabled(morph: &mut BruiseMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return bruise count.
pub fn brm_bruise_count(morph: &BruiseMorph) -> usize {
    morph.bruises.len()
}

/// Serialize to JSON-like string.
pub fn brm_to_json(morph: &BruiseMorph) -> String {
    format!(
        r#"{{"bruise_count":{},"global_intensity":{},"morph_count":{},"enabled":{}}}"#,
        morph.bruises.len(),
        morph.global_intensity,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bruise() -> BruiseEntry {
        BruiseEntry {
            stage: BruiseStage::Fresh,
            position: [0.0, 0.5, 0.0],
            radius: 0.05,
            intensity: 0.8,
        }
    }

    #[test]
    fn test_initial_empty() {
        let m = new_bruise_morph(4);
        assert_eq!(brm_bruise_count(&m), 0 /* no bruises initially */);
    }

    #[test]
    fn test_add_bruise() {
        let mut m = new_bruise_morph(4);
        brm_add_bruise(&mut m, make_bruise());
        assert_eq!(brm_bruise_count(&m), 1 /* one bruise after add */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_bruise_morph(4);
        brm_add_bruise(&mut m, make_bruise());
        brm_clear(&mut m);
        assert_eq!(brm_bruise_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_bruise_morph(4);
        brm_set_intensity(&mut m, 2.0);
        assert!((m.global_intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_bruise_morph(6);
        assert_eq!(
            brm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_bruise_morph(4);
        brm_set_enabled(&mut m, false);
        assert!(brm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_bruise_count() {
        let m = new_bruise_morph(4);
        let j = brm_to_json(&m);
        assert!(j.contains("\"bruise_count\"") /* JSON must have bruise_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_bruise_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_stage_variant() {
        let e = make_bruise();
        assert_eq!(e.stage, BruiseStage::Fresh /* stage must be Fresh */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_bruise_morph(2);
        brm_set_intensity(&mut m, 0.5);
        let out = brm_evaluate(&m);
        assert!((out[0] - 0.5).abs() < 1e-5 /* evaluate must match global_intensity */);
    }
}
