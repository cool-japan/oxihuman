// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin thickness deformation morph stub.

/// Body region for skin thickness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkinRegion {
    Face,
    Scalp,
    Hands,
    Feet,
    Torso,
    Limbs,
}

/// Skin thickness morph controller.
#[derive(Debug, Clone)]
pub struct SkinThicknessMorph {
    pub global_thickness: f32,
    pub region_values: Vec<(SkinRegion, f32)>,
    pub morph_count: usize,
    pub enabled: bool,
}

impl SkinThicknessMorph {
    pub fn new(morph_count: usize) -> Self {
        SkinThicknessMorph {
            global_thickness: 0.5,
            region_values: Vec::new(),
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new skin thickness morph.
pub fn new_skin_thickness_morph(morph_count: usize) -> SkinThicknessMorph {
    SkinThicknessMorph::new(morph_count)
}

/// Set global skin thickness.
pub fn stm_set_thickness(morph: &mut SkinThicknessMorph, thickness: f32) {
    morph.global_thickness = thickness.clamp(0.0, 1.0);
}

/// Set per-region thickness override.
pub fn stm_set_region(morph: &mut SkinThicknessMorph, region: SkinRegion, thickness: f32) {
    let v = thickness.clamp(0.0, 1.0);
    if let Some(e) = morph.region_values.iter_mut().find(|(r, _)| *r == region) {
        e.1 = v;
    } else {
        morph.region_values.push((region, v));
    }
}

/// Evaluate morph weights (stub: uniform from thickness).
pub fn stm_evaluate(morph: &SkinThicknessMorph) -> Vec<f32> {
    /* Stub: all targets scaled by global_thickness */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_thickness; morph.morph_count]
}

/// Enable or disable.
pub fn stm_set_enabled(morph: &mut SkinThicknessMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return region count.
pub fn stm_region_count(morph: &SkinThicknessMorph) -> usize {
    morph.region_values.len()
}

/// Serialize to JSON-like string.
pub fn stm_to_json(morph: &SkinThicknessMorph) -> String {
    format!(
        r#"{{"global_thickness":{},"morph_count":{},"regions":{},"enabled":{}}}"#,
        morph.global_thickness,
        morph.morph_count,
        morph.region_values.len(),
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thickness() {
        let m = new_skin_thickness_morph(4);
        assert!((m.global_thickness - 0.5).abs() < 1e-6 /* default thickness must be 0.5 */);
    }

    #[test]
    fn test_set_thickness_clamps() {
        let mut m = new_skin_thickness_morph(4);
        stm_set_thickness(&mut m, -1.0);
        assert!((m.global_thickness).abs() < 1e-6 /* thickness clamped to 0.0 */);
    }

    #[test]
    fn test_region_added() {
        let mut m = new_skin_thickness_morph(4);
        stm_set_region(&mut m, SkinRegion::Face, 0.3);
        assert_eq!(stm_region_count(&m), 1 /* one region must be added */);
    }

    #[test]
    fn test_region_updated() {
        let mut m = new_skin_thickness_morph(4);
        stm_set_region(&mut m, SkinRegion::Hands, 0.2);
        stm_set_region(&mut m, SkinRegion::Hands, 0.9);
        assert_eq!(
            stm_region_count(&m),
            1 /* same region must update not duplicate */
        );
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_skin_thickness_morph(7);
        let out = stm_evaluate(&m);
        assert_eq!(out.len(), 7 /* output must match morph_count */);
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_skin_thickness_morph(4);
        stm_set_enabled(&mut m, false);
        assert!(stm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_global_thickness() {
        let m = new_skin_thickness_morph(4);
        let j = stm_to_json(&m);
        assert!(j.contains("\"global_thickness\"") /* JSON must have global_thickness */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_skin_thickness_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_value_matches_thickness() {
        let mut m = new_skin_thickness_morph(3);
        stm_set_thickness(&mut m, 0.7);
        let out = stm_evaluate(&m);
        assert!((out[0] - 0.7).abs() < 1e-5 /* evaluated weight must equal thickness */);
    }
}
