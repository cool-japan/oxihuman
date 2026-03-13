// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Stretch mark surface morph stub.

/// Body region where stretch marks appear.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StretchMarkRegion {
    Abdomen,
    Hips,
    Thighs,
    UpperArms,
    Breasts,
}

/// A stretch mark entry.
#[derive(Debug, Clone)]
pub struct StretchMarkEntry {
    pub region: StretchMarkRegion,
    pub density: f32,
    pub age_factor: f32,
}

/// Stretch mark morph controller.
#[derive(Debug, Clone)]
pub struct StretchMarkMorph {
    pub entries: Vec<StretchMarkEntry>,
    pub global_intensity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl StretchMarkMorph {
    pub fn new(morph_count: usize) -> Self {
        StretchMarkMorph {
            entries: Vec::new(),
            global_intensity: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new stretch mark morph.
pub fn new_stretch_mark_morph(morph_count: usize) -> StretchMarkMorph {
    StretchMarkMorph::new(morph_count)
}

/// Add a stretch mark entry.
pub fn smm_add_entry(morph: &mut StretchMarkMorph, entry: StretchMarkEntry) {
    morph.entries.push(entry);
}

/// Set global intensity.
pub fn smm_set_intensity(morph: &mut StretchMarkMorph, intensity: f32) {
    morph.global_intensity = intensity.clamp(0.0, 1.0);
}

/// Clear all entries.
pub fn smm_clear(morph: &mut StretchMarkMorph) {
    morph.entries.clear();
}

/// Evaluate morph weights (stub: uniform from global_intensity).
pub fn smm_evaluate(morph: &StretchMarkMorph) -> Vec<f32> {
    /* Stub: uniform weight from global_intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_intensity; morph.morph_count]
}

/// Enable or disable.
pub fn smm_set_enabled(morph: &mut StretchMarkMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return entry count.
pub fn smm_entry_count(morph: &StretchMarkMorph) -> usize {
    morph.entries.len()
}

/// Serialize to JSON-like string.
pub fn smm_to_json(morph: &StretchMarkMorph) -> String {
    format!(
        r#"{{"entry_count":{},"global_intensity":{},"morph_count":{},"enabled":{}}}"#,
        morph.entries.len(),
        morph.global_intensity,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry() -> StretchMarkEntry {
        StretchMarkEntry {
            region: StretchMarkRegion::Abdomen,
            density: 0.5,
            age_factor: 0.3,
        }
    }

    #[test]
    fn test_initial_empty() {
        let m = new_stretch_mark_morph(4);
        assert_eq!(smm_entry_count(&m), 0 /* no entries initially */);
    }

    #[test]
    fn test_add_entry() {
        let mut m = new_stretch_mark_morph(4);
        smm_add_entry(&mut m, make_entry());
        assert_eq!(smm_entry_count(&m), 1 /* one entry after add */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_stretch_mark_morph(4);
        smm_add_entry(&mut m, make_entry());
        smm_clear(&mut m);
        assert_eq!(smm_entry_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_stretch_mark_morph(4);
        smm_set_intensity(&mut m, 2.0);
        assert!((m.global_intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_stretch_mark_morph(5);
        assert_eq!(
            smm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_stretch_mark_morph(4);
        smm_set_enabled(&mut m, false);
        assert!(smm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_entry_count() {
        let m = new_stretch_mark_morph(4);
        let j = smm_to_json(&m);
        assert!(j.contains("\"entry_count\"") /* JSON must have entry_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_stretch_mark_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_stretch_mark_morph(3);
        smm_set_intensity(&mut m, 0.6);
        let out = smm_evaluate(&m);
        assert!((out[0] - 0.6).abs() < 1e-5 /* evaluate must match global_intensity */);
    }

    #[test]
    fn test_region_variant() {
        let e = StretchMarkEntry {
            region: StretchMarkRegion::Thighs,
            density: 0.4,
            age_factor: 0.2,
        };
        assert_eq!(
            e.region,
            StretchMarkRegion::Thighs /* region must match */
        );
    }
}
