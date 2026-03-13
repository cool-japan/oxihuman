// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tattoo deform with skin stretch stub.

/// Tattoo deformation entry.
#[derive(Debug, Clone)]
pub struct TattooEntry {
    pub id: u32,
    pub anchor: [f32; 3],
    pub stretch_factor: f32,
    pub opacity: f32,
}

/// Tattoo morph controller.
#[derive(Debug, Clone)]
pub struct TattooMorph {
    pub entries: Vec<TattooEntry>,
    pub skin_stretch_influence: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl TattooMorph {
    pub fn new(morph_count: usize) -> Self {
        TattooMorph {
            entries: Vec::new(),
            skin_stretch_influence: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new tattoo morph controller.
pub fn new_tattoo_morph(morph_count: usize) -> TattooMorph {
    TattooMorph::new(morph_count)
}

/// Add a tattoo entry.
pub fn tm_add_tattoo(morph: &mut TattooMorph, entry: TattooEntry) {
    morph.entries.push(entry);
}

/// Set skin stretch influence on tattoo deformation.
pub fn tm_set_stretch_influence(morph: &mut TattooMorph, influence: f32) {
    morph.skin_stretch_influence = influence.clamp(0.0, 1.0);
}

/// Remove a tattoo by id.
pub fn tm_remove_tattoo(morph: &mut TattooMorph, id: u32) {
    morph.entries.retain(|e| e.id != id);
}

/// Evaluate morph weights (stub: aggregate stretch effect).
pub fn tm_evaluate(morph: &TattooMorph) -> Vec<f32> {
    /* Stub: proportional to number of tattoos * stretch_influence */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = (morph.entries.len() as f32 * morph.skin_stretch_influence).min(1.0);
    vec![w; morph.morph_count]
}

/// Enable or disable.
pub fn tm_set_enabled(morph: &mut TattooMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return tattoo count.
pub fn tm_tattoo_count(morph: &TattooMorph) -> usize {
    morph.entries.len()
}

/// Serialize to JSON-like string.
pub fn tm_to_json(morph: &TattooMorph) -> String {
    format!(
        r#"{{"tattoo_count":{},"stretch_influence":{},"morph_count":{},"enabled":{}}}"#,
        morph.entries.len(),
        morph.skin_stretch_influence,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: u32) -> TattooEntry {
        TattooEntry {
            id,
            anchor: [0.0, 0.0, 0.0],
            stretch_factor: 1.0,
            opacity: 1.0,
        }
    }

    #[test]
    fn test_initial_count() {
        let m = new_tattoo_morph(4);
        assert_eq!(tm_tattoo_count(&m), 0 /* no tattoos initially */);
    }

    #[test]
    fn test_add_tattoo() {
        let mut m = new_tattoo_morph(4);
        tm_add_tattoo(&mut m, make_entry(1));
        assert_eq!(tm_tattoo_count(&m), 1 /* one tattoo after add */);
    }

    #[test]
    fn test_remove_tattoo() {
        let mut m = new_tattoo_morph(4);
        tm_add_tattoo(&mut m, make_entry(1));
        tm_remove_tattoo(&mut m, 1);
        assert_eq!(tm_tattoo_count(&m), 0 /* tattoo removed */);
    }

    #[test]
    fn test_stretch_influence_clamped() {
        let mut m = new_tattoo_morph(4);
        tm_set_stretch_influence(&mut m, 2.0);
        assert!((m.skin_stretch_influence - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_tattoo_morph(5);
        assert_eq!(
            tm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_tattoo_morph(4);
        tm_set_enabled(&mut m, false);
        assert!(tm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_tattoo_count() {
        let m = new_tattoo_morph(4);
        let j = tm_to_json(&m);
        assert!(j.contains("\"tattoo_count\"") /* JSON must have tattoo_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_tattoo_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_capped_at_one() {
        let mut m = new_tattoo_morph(2);
        tm_set_stretch_influence(&mut m, 1.0);
        for i in 0..5 {
            tm_add_tattoo(&mut m, make_entry(i));
        }
        let out = tm_evaluate(&m);
        assert!(out[0] <= 1.0 /* weight must not exceed 1.0 */);
    }
}
