// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edema/swelling morph stub.

/// Edema type classification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdemaType {
    Pitting,
    NonPitting,
    Lymphedema,
    Angioedema,
}

/// Edema region entry.
#[derive(Debug, Clone)]
pub struct EdemaRegion {
    pub edema_type: EdemaType,
    pub position: [f32; 3],
    pub radius: f32,
    pub swelling: f32,
}

/// Edema morph controller.
#[derive(Debug, Clone)]
pub struct EdemaMorph {
    pub regions: Vec<EdemaRegion>,
    pub global_intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl EdemaMorph {
    pub fn new(morph_count: usize) -> Self {
        EdemaMorph {
            regions: Vec::new(),
            global_intensity: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new edema morph.
pub fn new_edema_morph(morph_count: usize) -> EdemaMorph {
    EdemaMorph::new(morph_count)
}

/// Add an edema region.
pub fn edm_add_region(morph: &mut EdemaMorph, region: EdemaRegion) {
    morph.regions.push(region);
}

/// Set global intensity.
pub fn edm_set_intensity(morph: &mut EdemaMorph, intensity: f32) {
    morph.global_intensity = intensity.clamp(0.0, 1.0);
}

/// Clear all regions.
pub fn edm_clear(morph: &mut EdemaMorph) {
    morph.regions.clear();
}

/// Evaluate morph weights (stub: uniform from global_intensity).
pub fn edm_evaluate(morph: &EdemaMorph) -> Vec<f32> {
    /* Stub: uniform weight from global_intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_intensity; morph.morph_count]
}

/// Enable or disable.
pub fn edm_set_enabled(morph: &mut EdemaMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return region count.
pub fn edm_region_count(morph: &EdemaMorph) -> usize {
    morph.regions.len()
}

/// Serialize to JSON-like string.
pub fn edm_to_json(morph: &EdemaMorph) -> String {
    format!(
        r#"{{"region_count":{},"global_intensity":{},"morph_count":{},"enabled":{}}}"#,
        morph.regions.len(),
        morph.global_intensity,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_region() -> EdemaRegion {
        EdemaRegion {
            edema_type: EdemaType::Pitting,
            position: [0.0, 0.0, 0.0],
            radius: 0.1,
            swelling: 0.5,
        }
    }

    #[test]
    fn test_initial_empty() {
        let m = new_edema_morph(4);
        assert_eq!(edm_region_count(&m), 0 /* no regions initially */);
    }

    #[test]
    fn test_add_region() {
        let mut m = new_edema_morph(4);
        edm_add_region(&mut m, make_region());
        assert_eq!(edm_region_count(&m), 1 /* one region after add */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_edema_morph(4);
        edm_add_region(&mut m, make_region());
        edm_clear(&mut m);
        assert_eq!(edm_region_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_edema_morph(4);
        edm_set_intensity(&mut m, 1.5);
        assert!((m.global_intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_edema_morph(5);
        edm_set_intensity(&mut m, 0.8);
        assert_eq!(
            edm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_edema_morph(4);
        edm_set_enabled(&mut m, false);
        assert!(edm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_region_count() {
        let m = new_edema_morph(4);
        let j = edm_to_json(&m);
        assert!(j.contains("\"region_count\"") /* JSON must have region_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_edema_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_edema_morph(3);
        edm_set_intensity(&mut m, 0.3);
        let out = edm_evaluate(&m);
        assert!((out[0] - 0.3).abs() < 1e-5 /* evaluate must match global_intensity */);
    }
}
