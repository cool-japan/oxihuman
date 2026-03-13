// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Scar/keloid surface morph stub.

/// Type of scar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScarType {
    Atrophic,
    Hypertrophic,
    Keloid,
    Contracture,
}

/// A single scar region entry.
#[derive(Debug, Clone)]
pub struct ScarRegion {
    pub scar_type: ScarType,
    pub position: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
}

/// Scar tissue morph controller.
#[derive(Debug, Clone)]
pub struct ScarTissueMorph {
    pub scars: Vec<ScarRegion>,
    pub global_intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl ScarTissueMorph {
    pub fn new(morph_count: usize) -> Self {
        ScarTissueMorph {
            scars: Vec::new(),
            global_intensity: 1.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new scar tissue morph.
pub fn new_scar_tissue_morph(morph_count: usize) -> ScarTissueMorph {
    ScarTissueMorph::new(morph_count)
}

/// Add a scar region.
pub fn scm_add_scar(morph: &mut ScarTissueMorph, region: ScarRegion) {
    morph.scars.push(region);
}

/// Set global intensity.
pub fn scm_set_intensity(morph: &mut ScarTissueMorph, intensity: f32) {
    morph.global_intensity = intensity.clamp(0.0, 1.0);
}

/// Remove all scars.
pub fn scm_clear(morph: &mut ScarTissueMorph) {
    morph.scars.clear();
}

/// Evaluate morph weights (stub: uniform from global_intensity).
pub fn scm_evaluate(morph: &ScarTissueMorph) -> Vec<f32> {
    /* Stub: uniform weight from global_intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_intensity; morph.morph_count]
}

/// Enable or disable.
pub fn scm_set_enabled(morph: &mut ScarTissueMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return scar count.
pub fn scm_scar_count(morph: &ScarTissueMorph) -> usize {
    morph.scars.len()
}

/// Serialize to JSON-like string.
pub fn scm_to_json(morph: &ScarTissueMorph) -> String {
    format!(
        r#"{{"scar_count":{},"global_intensity":{},"morph_count":{},"enabled":{}}}"#,
        morph.scars.len(),
        morph.global_intensity,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scar(t: ScarType) -> ScarRegion {
        ScarRegion {
            scar_type: t,
            position: [0.0, 0.0, 0.0],
            intensity: 0.5,
            radius: 0.1,
        }
    }

    #[test]
    fn test_initial_scar_count() {
        let m = new_scar_tissue_morph(4);
        assert_eq!(scm_scar_count(&m), 0 /* no scars initially */);
    }

    #[test]
    fn test_add_scar() {
        let mut m = new_scar_tissue_morph(4);
        scm_add_scar(&mut m, make_scar(ScarType::Keloid));
        assert_eq!(scm_scar_count(&m), 1 /* one scar after add */);
    }

    #[test]
    fn test_clear_scars() {
        let mut m = new_scar_tissue_morph(4);
        scm_add_scar(&mut m, make_scar(ScarType::Atrophic));
        scm_clear(&mut m);
        assert_eq!(scm_scar_count(&m), 0 /* scars cleared */);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut m = new_scar_tissue_morph(4);
        scm_set_intensity(&mut m, 1.5);
        assert!((m.global_intensity - 1.0).abs() < 1e-6 /* intensity clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_scar_tissue_morph(5);
        assert_eq!(
            scm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_scar_tissue_morph(4);
        scm_set_enabled(&mut m, false);
        assert!(scm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_scar_count() {
        let m = new_scar_tissue_morph(4);
        let j = scm_to_json(&m);
        assert!(j.contains("\"scar_count\"") /* JSON must have scar_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_scar_tissue_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_scar_tissue_morph(3);
        scm_set_intensity(&mut m, 0.6);
        let out = scm_evaluate(&m);
        assert!((out[0] - 0.6).abs() < 1e-5 /* evaluate must match global_intensity */);
    }
}
