// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin pore size morph stub.

/// Facial zone for pore size control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoreZone {
    TZone,
    Cheeks,
    Forehead,
    Chin,
    Nose,
}

/// Pore size morph controller.
#[derive(Debug, Clone)]
pub struct PoreSizeMorph {
    pub global_size: f32,
    pub zone_overrides: Vec<(PoreZone, f32)>,
    pub depth: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl PoreSizeMorph {
    pub fn new(morph_count: usize) -> Self {
        PoreSizeMorph {
            global_size: 0.3,
            zone_overrides: Vec::new(),
            depth: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new pore size morph controller.
pub fn new_pore_size_morph(morph_count: usize) -> PoreSizeMorph {
    PoreSizeMorph::new(morph_count)
}

/// Set global pore size.
pub fn psm_set_size(morph: &mut PoreSizeMorph, size: f32) {
    morph.global_size = size.clamp(0.0, 1.0);
}

/// Set per-zone size override.
pub fn psm_set_zone(morph: &mut PoreSizeMorph, zone: PoreZone, size: f32) {
    let v = size.clamp(0.0, 1.0);
    if let Some(e) = morph.zone_overrides.iter_mut().find(|(z, _)| *z == zone) {
        e.1 = v;
    } else {
        morph.zone_overrides.push((zone, v));
    }
}

/// Set pore depth (affects displacement intensity).
pub fn psm_set_depth(morph: &mut PoreSizeMorph, depth: f32) {
    morph.depth = depth.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: size × depth).
pub fn psm_evaluate(morph: &PoreSizeMorph) -> Vec<f32> {
    /* Stub: weight is size multiplied by depth */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = morph.global_size * morph.depth;
    vec![w; morph.morph_count]
}

/// Enable or disable.
pub fn psm_set_enabled(morph: &mut PoreSizeMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return zone override count.
pub fn psm_zone_count(morph: &PoreSizeMorph) -> usize {
    morph.zone_overrides.len()
}

/// Serialize to JSON-like string.
pub fn psm_to_json(morph: &PoreSizeMorph) -> String {
    format!(
        r#"{{"global_size":{},"depth":{},"zones":{},"morph_count":{},"enabled":{}}}"#,
        morph.global_size,
        morph.depth,
        morph.zone_overrides.len(),
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_size() {
        let m = new_pore_size_morph(4);
        assert!((m.global_size - 0.3).abs() < 1e-6 /* default size must be 0.3 */);
    }

    #[test]
    fn test_set_size_clamps() {
        let mut m = new_pore_size_morph(4);
        psm_set_size(&mut m, 2.0);
        assert!((m.global_size - 1.0).abs() < 1e-6 /* size clamped to 1.0 */);
    }

    #[test]
    fn test_zone_added() {
        let mut m = new_pore_size_morph(4);
        psm_set_zone(&mut m, PoreZone::TZone, 0.7);
        assert_eq!(psm_zone_count(&m), 1 /* one zone override added */);
    }

    #[test]
    fn test_depth_clamped() {
        let mut m = new_pore_size_morph(4);
        psm_set_depth(&mut m, -1.0);
        assert!((m.depth).abs() < 1e-6 /* depth clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_pore_size_morph(5);
        assert_eq!(
            psm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_pore_size_morph(4);
        psm_set_enabled(&mut m, false);
        assert!(psm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_evaluate_product() {
        let mut m = new_pore_size_morph(2);
        psm_set_size(&mut m, 0.5);
        psm_set_depth(&mut m, 0.4);
        let out = psm_evaluate(&m);
        assert!((out[0] - 0.2).abs() < 1e-5 /* weight must be size * depth */);
    }

    #[test]
    fn test_to_json_has_size() {
        let m = new_pore_size_morph(4);
        let j = psm_to_json(&m);
        assert!(j.contains("\"global_size\"") /* JSON must have global_size */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_pore_size_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_zone_update_not_duplicate() {
        let mut m = new_pore_size_morph(4);
        psm_set_zone(&mut m, PoreZone::Nose, 0.3);
        psm_set_zone(&mut m, PoreZone::Nose, 0.7);
        assert_eq!(
            psm_zone_count(&m),
            1 /* same zone must update not duplicate */
        );
    }
}
