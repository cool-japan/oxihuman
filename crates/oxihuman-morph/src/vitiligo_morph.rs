// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vitiligo depigmentation morph stub.

/// Vitiligo distribution pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VitiligoPaattern {
    Focal,
    Segmental,
    Generalized,
    Universal,
}

/// A vitiligo patch entry.
#[derive(Debug, Clone)]
pub struct VitilligoPatch {
    pub position: [f32; 3],
    pub radius: f32,
    pub depigmentation: f32,
}

/// Vitiligo morph controller.
#[derive(Debug, Clone)]
pub struct VitiligoMorph {
    pub pattern: VitiligoPaattern,
    pub patches: Vec<VitilligoPatch>,
    pub global_extent: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl VitiligoMorph {
    pub fn new(morph_count: usize) -> Self {
        VitiligoMorph {
            pattern: VitiligoPaattern::Focal,
            patches: Vec::new(),
            global_extent: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new vitiligo morph.
pub fn new_vitiligo_morph(morph_count: usize) -> VitiligoMorph {
    VitiligoMorph::new(morph_count)
}

/// Set vitiligo pattern.
pub fn vim_set_pattern(morph: &mut VitiligoMorph, pattern: VitiligoPaattern) {
    morph.pattern = pattern;
}

/// Add a depigmentation patch.
pub fn vim_add_patch(morph: &mut VitiligoMorph, patch: VitilligoPatch) {
    morph.patches.push(patch);
}

/// Set global extent (fraction of body affected).
pub fn vim_set_extent(morph: &mut VitiligoMorph, extent: f32) {
    morph.global_extent = extent.clamp(0.0, 1.0);
}

/// Clear all patches.
pub fn vim_clear(morph: &mut VitiligoMorph) {
    morph.patches.clear();
}

/// Evaluate morph weights (stub: uniform from global_extent).
pub fn vim_evaluate(morph: &VitiligoMorph) -> Vec<f32> {
    /* Stub: uniform weight from global_extent */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_extent; morph.morph_count]
}

/// Enable or disable.
pub fn vim_set_enabled(morph: &mut VitiligoMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return patch count.
pub fn vim_patch_count(morph: &VitiligoMorph) -> usize {
    morph.patches.len()
}

/// Serialize to JSON-like string.
pub fn vim_to_json(morph: &VitiligoMorph) -> String {
    let pat = match morph.pattern {
        VitiligoPaattern::Focal => "focal",
        VitiligoPaattern::Segmental => "segmental",
        VitiligoPaattern::Generalized => "generalized",
        VitiligoPaattern::Universal => "universal",
    };
    format!(
        r#"{{"pattern":"{}","patch_count":{},"global_extent":{},"morph_count":{},"enabled":{}}}"#,
        pat,
        morph.patches.len(),
        morph.global_extent,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_patch() -> VitilligoPatch {
        VitilligoPatch {
            position: [0.2, 0.5, 0.1],
            radius: 0.05,
            depigmentation: 1.0,
        }
    }

    #[test]
    fn test_default_pattern() {
        let m = new_vitiligo_morph(4);
        assert_eq!(
            m.pattern,
            VitiligoPaattern::Focal /* default pattern must be Focal */
        );
    }

    #[test]
    fn test_set_pattern() {
        let mut m = new_vitiligo_morph(4);
        vim_set_pattern(&mut m, VitiligoPaattern::Universal);
        assert_eq!(
            m.pattern,
            VitiligoPaattern::Universal /* pattern must be set */
        );
    }

    #[test]
    fn test_add_patch() {
        let mut m = new_vitiligo_morph(4);
        vim_add_patch(&mut m, make_patch());
        assert_eq!(vim_patch_count(&m), 1 /* one patch after add */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_vitiligo_morph(4);
        vim_add_patch(&mut m, make_patch());
        vim_clear(&mut m);
        assert_eq!(vim_patch_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_extent_clamp() {
        let mut m = new_vitiligo_morph(4);
        vim_set_extent(&mut m, 1.5);
        assert!((m.global_extent - 1.0).abs() < 1e-6 /* extent clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_vitiligo_morph(6);
        vim_set_extent(&mut m, 0.5);
        assert_eq!(
            vim_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_vitiligo_morph(4);
        vim_set_enabled(&mut m, false);
        assert!(vim_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_pattern() {
        let m = new_vitiligo_morph(4);
        let j = vim_to_json(&m);
        assert!(j.contains("\"pattern\"") /* JSON must have pattern */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_vitiligo_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_extent() {
        let mut m = new_vitiligo_morph(3);
        vim_set_extent(&mut m, 0.5);
        let out = vim_evaluate(&m);
        assert!((out[0] - 0.5).abs() < 1e-5 /* evaluate must match global_extent */);
    }
}
