// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Age progression morph set stub.

/// An age stage entry in the progression sequence.
#[derive(Debug, Clone)]
pub struct AgeStage {
    pub age_years: f32,
    pub morph_weights: Vec<f32>,
    pub label: String,
}

/// Age progression morph controller.
#[derive(Debug, Clone)]
pub struct AgeProgressionMorph {
    pub stages: Vec<AgeStage>,
    pub current_age: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl AgeProgressionMorph {
    pub fn new(morph_count: usize) -> Self {
        AgeProgressionMorph {
            stages: Vec::new(),
            current_age: 25.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new age progression morph controller.
pub fn new_age_progression_morph(morph_count: usize) -> AgeProgressionMorph {
    AgeProgressionMorph::new(morph_count)
}

/// Add an age stage.
pub fn apm_add_stage(apm: &mut AgeProgressionMorph, stage: AgeStage) {
    apm.stages.push(stage);
}

/// Set current age for interpolation.
pub fn apm_set_age(apm: &mut AgeProgressionMorph, age: f32) {
    apm.current_age = age.max(0.0);
}

/// Evaluate morph weights for current age (stub: zeroed).
pub fn apm_evaluate(apm: &AgeProgressionMorph) -> Vec<f32> {
    /* Stub: returns zeroed weights */
    vec![0.0; apm.morph_count]
}

/// Return stage count.
pub fn apm_stage_count(apm: &AgeProgressionMorph) -> usize {
    apm.stages.len()
}

/// Enable or disable.
pub fn apm_set_enabled(apm: &mut AgeProgressionMorph, enabled: bool) {
    apm.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn apm_to_json(apm: &AgeProgressionMorph) -> String {
    format!(
        r#"{{"morph_count":{},"stage_count":{},"current_age":{:.1},"enabled":{}}}"#,
        apm.morph_count,
        apm.stages.len(),
        apm.current_age,
        apm.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_count() {
        let a = new_age_progression_morph(12);
        assert_eq!(a.morph_count, 12 /* morph count must match */,);
    }

    #[test]
    fn test_default_age() {
        let a = new_age_progression_morph(4);
        assert!((a.current_age - 25.0).abs() < 1e-5, /* default age must be 25.0 */);
    }

    #[test]
    fn test_no_stages_initially() {
        let a = new_age_progression_morph(4);
        assert_eq!(apm_stage_count(&a), 0 /* no stages initially */,);
    }

    #[test]
    fn test_add_stage() {
        let mut a = new_age_progression_morph(4);
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 30.0,
                morph_weights: vec![0.0; 4],
                label: "adult".into(),
            },
        );
        assert_eq!(apm_stage_count(&a), 1 /* one stage after add */,);
    }

    #[test]
    fn test_set_age() {
        let mut a = new_age_progression_morph(4);
        apm_set_age(&mut a, 60.0);
        assert!((a.current_age - 60.0).abs() < 1e-5, /* age must be set */);
    }

    #[test]
    fn test_age_clamped_negative() {
        let mut a = new_age_progression_morph(4);
        apm_set_age(&mut a, -5.0);
        assert!((a.current_age).abs() < 1e-6, /* negative age clamped to 0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let a = new_age_progression_morph(8);
        let out = apm_evaluate(&a);
        assert_eq!(out.len(), 8 /* output length must match morph_count */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut a = new_age_progression_morph(2);
        apm_set_enabled(&mut a, false);
        assert!(!a.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_current_age() {
        let a = new_age_progression_morph(4);
        let j = apm_to_json(&a);
        assert!(j.contains("\"current_age\""), /* json must contain current_age */);
    }

    #[test]
    fn test_enabled_default() {
        let a = new_age_progression_morph(1);
        assert!(a.enabled /* must be enabled by default */,);
    }
}
