// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Muscle tone/definition morph stub.

/// Muscle group identifier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MuscleGroup {
    Arms,
    Shoulders,
    Chest,
    Abdomen,
    Back,
    Legs,
    Glutes,
    Neck,
}

/// Muscle tone morph controller.
#[derive(Debug, Clone)]
pub struct MuscleToneMorph {
    pub global_tone: f32,
    pub group_overrides: Vec<(MuscleGroup, f32)>,
    pub definition: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl MuscleToneMorph {
    pub fn new(morph_count: usize) -> Self {
        MuscleToneMorph {
            global_tone: 0.5,
            group_overrides: Vec::new(),
            definition: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new muscle tone morph controller.
pub fn new_muscle_tone_morph(morph_count: usize) -> MuscleToneMorph {
    MuscleToneMorph::new(morph_count)
}

/// Set global muscle tone.
pub fn mtm_set_tone(morph: &mut MuscleToneMorph, tone: f32) {
    morph.global_tone = tone.clamp(0.0, 1.0);
}

/// Set definition (muscle separation visibility).
pub fn mtm_set_definition(morph: &mut MuscleToneMorph, definition: f32) {
    morph.definition = definition.clamp(0.0, 1.0);
}

/// Override tone for a specific muscle group.
pub fn mtm_set_group_override(morph: &mut MuscleToneMorph, group: MuscleGroup, tone: f32) {
    let clamped = tone.clamp(0.0, 1.0);
    if let Some(entry) = morph.group_overrides.iter_mut().find(|(g, _)| *g == group) {
        entry.1 = clamped;
    } else {
        morph.group_overrides.push((group, clamped));
    }
}

/// Evaluate morph weights (stub: blend global tone with definition).
pub fn mtm_evaluate(morph: &MuscleToneMorph) -> Vec<f32> {
    /* Stub: weight = global_tone * definition */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = morph.global_tone * morph.definition;
    vec![w; morph.morph_count]
}

/// Enable or disable.
pub fn mtm_set_enabled(morph: &mut MuscleToneMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return number of group overrides.
pub fn mtm_override_count(morph: &MuscleToneMorph) -> usize {
    morph.group_overrides.len()
}

/// Serialize to JSON-like string.
pub fn mtm_to_json(morph: &MuscleToneMorph) -> String {
    format!(
        r#"{{"global_tone":{},"definition":{},"morph_count":{},"enabled":{}}}"#,
        morph.global_tone, morph.definition, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tone() {
        let m = new_muscle_tone_morph(8);
        assert!((m.global_tone - 0.5).abs() < 1e-6 /* default tone must be 0.5 */);
    }

    #[test]
    fn test_set_tone_clamps() {
        let mut m = new_muscle_tone_morph(4);
        mtm_set_tone(&mut m, 1.5);
        assert!((m.global_tone - 1.0).abs() < 1e-6 /* tone clamped to 1.0 */);
    }

    #[test]
    fn test_set_definition() {
        let mut m = new_muscle_tone_morph(4);
        mtm_set_definition(&mut m, 0.8);
        assert!((m.definition - 0.8).abs() < 1e-5 /* definition must be set */);
    }

    #[test]
    fn test_group_override_added() {
        let mut m = new_muscle_tone_morph(4);
        mtm_set_group_override(&mut m, MuscleGroup::Arms, 0.9);
        assert_eq!(
            mtm_override_count(&m),
            1 /* one override must be added */
        );
    }

    #[test]
    fn test_group_override_updated() {
        let mut m = new_muscle_tone_morph(4);
        mtm_set_group_override(&mut m, MuscleGroup::Arms, 0.5);
        mtm_set_group_override(&mut m, MuscleGroup::Arms, 0.9);
        assert_eq!(
            mtm_override_count(&m),
            1 /* duplicate group must not add new entry */
        );
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_muscle_tone_morph(5);
        let out = mtm_evaluate(&m);
        assert_eq!(out.len(), 5 /* output must match morph_count */);
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_muscle_tone_morph(4);
        mtm_set_enabled(&mut m, false);
        assert!(mtm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_fields() {
        let m = new_muscle_tone_morph(4);
        let j = mtm_to_json(&m);
        assert!(j.contains("\"global_tone\"") /* JSON must have global_tone */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_muscle_tone_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_product() {
        let mut m = new_muscle_tone_morph(2);
        mtm_set_tone(&mut m, 0.4);
        mtm_set_definition(&mut m, 0.5);
        let out = mtm_evaluate(&m);
        assert!((out[0] - 0.2).abs() < 1e-5 /* weight must be tone * definition */);
    }
}
