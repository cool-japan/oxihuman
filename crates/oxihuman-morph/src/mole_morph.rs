// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mole/nevus placement morph stub.

/// Mole type classification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoleType {
    CommonNevus,
    DysplasticNevus,
    CongenitalNevus,
    BluNevus,
}

/// A mole placement entry.
#[derive(Debug, Clone)]
pub struct MoleEntry {
    pub mole_type: MoleType,
    pub position: [f32; 3],
    pub radius: f32,
    pub elevation: f32,
}

/// Mole morph controller.
#[derive(Debug, Clone)]
pub struct MoleMorph {
    pub moles: Vec<MoleEntry>,
    pub global_opacity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl MoleMorph {
    pub fn new(morph_count: usize) -> Self {
        MoleMorph {
            moles: Vec::new(),
            global_opacity: 1.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new mole morph.
pub fn new_mole_morph(morph_count: usize) -> MoleMorph {
    MoleMorph::new(morph_count)
}

/// Add a mole entry.
pub fn mom_add_mole(morph: &mut MoleMorph, entry: MoleEntry) {
    morph.moles.push(entry);
}

/// Set global opacity.
pub fn mom_set_opacity(morph: &mut MoleMorph, opacity: f32) {
    morph.global_opacity = opacity.clamp(0.0, 1.0);
}

/// Clear all moles.
pub fn mom_clear(morph: &mut MoleMorph) {
    morph.moles.clear();
}

/// Evaluate morph weights (stub: uniform from global_opacity).
pub fn mom_evaluate(morph: &MoleMorph) -> Vec<f32> {
    /* Stub: uniform weight from global_opacity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.global_opacity; morph.morph_count]
}

/// Enable or disable.
pub fn mom_set_enabled(morph: &mut MoleMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return mole count.
pub fn mom_mole_count(morph: &MoleMorph) -> usize {
    morph.moles.len()
}

/// Serialize to JSON-like string.
pub fn mom_to_json(morph: &MoleMorph) -> String {
    format!(
        r#"{{"mole_count":{},"global_opacity":{},"morph_count":{},"enabled":{}}}"#,
        morph.moles.len(),
        morph.global_opacity,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mole() -> MoleEntry {
        MoleEntry {
            mole_type: MoleType::CommonNevus,
            position: [0.1, 0.2, 0.3],
            radius: 0.02,
            elevation: 0.01,
        }
    }

    #[test]
    fn test_initial_empty() {
        let m = new_mole_morph(4);
        assert_eq!(mom_mole_count(&m), 0 /* no moles initially */);
    }

    #[test]
    fn test_add_mole() {
        let mut m = new_mole_morph(4);
        mom_add_mole(&mut m, make_mole());
        assert_eq!(mom_mole_count(&m), 1 /* one mole after add */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_mole_morph(4);
        mom_add_mole(&mut m, make_mole());
        mom_clear(&mut m);
        assert_eq!(mom_mole_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut m = new_mole_morph(4);
        mom_set_opacity(&mut m, 2.0);
        assert!((m.global_opacity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_mole_morph(5);
        assert_eq!(
            mom_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_mole_morph(4);
        mom_set_enabled(&mut m, false);
        assert!(mom_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_mole_count() {
        let m = new_mole_morph(4);
        let j = mom_to_json(&m);
        assert!(j.contains("\"mole_count\"") /* JSON must have mole_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_mole_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_opacity() {
        let mut m = new_mole_morph(3);
        mom_set_opacity(&mut m, 0.7);
        let out = mom_evaluate(&m);
        assert!((out[0] - 0.7).abs() < 1e-5 /* evaluate must match global_opacity */);
    }
}
