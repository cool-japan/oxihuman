// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Subcutaneous fat layer morph stub.

/// Fat distribution pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FatPattern {
    Android,
    Gynoid,
    Uniform,
}

/// Subcutaneous fat morph controller.
#[derive(Debug, Clone)]
pub struct SubcutaneousFatMorph {
    pub total_fat: f32,
    pub pattern: FatPattern,
    pub visceral_ratio: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl SubcutaneousFatMorph {
    pub fn new(morph_count: usize) -> Self {
        SubcutaneousFatMorph {
            total_fat: 0.3,
            pattern: FatPattern::Uniform,
            visceral_ratio: 0.2,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new subcutaneous fat morph.
pub fn new_subcutaneous_fat_morph(morph_count: usize) -> SubcutaneousFatMorph {
    SubcutaneousFatMorph::new(morph_count)
}

/// Set total fat level.
pub fn sfm_set_fat(morph: &mut SubcutaneousFatMorph, fat: f32) {
    morph.total_fat = fat.clamp(0.0, 1.0);
}

/// Set distribution pattern.
pub fn sfm_set_pattern(morph: &mut SubcutaneousFatMorph, pattern: FatPattern) {
    morph.pattern = pattern;
}

/// Set visceral-to-subcutaneous ratio.
pub fn sfm_set_visceral_ratio(morph: &mut SubcutaneousFatMorph, ratio: f32) {
    morph.visceral_ratio = ratio.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: fat × pattern_scale).
pub fn sfm_evaluate(morph: &SubcutaneousFatMorph) -> Vec<f32> {
    /* Stub: pattern modulates fat weight */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let scale = match morph.pattern {
        FatPattern::Android => 0.9,
        FatPattern::Gynoid => 0.85,
        FatPattern::Uniform => 1.0,
    };
    vec![morph.total_fat * scale; morph.morph_count]
}

/// Enable or disable.
pub fn sfm_set_enabled(morph: &mut SubcutaneousFatMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn sfm_to_json(morph: &SubcutaneousFatMorph) -> String {
    let pat = match morph.pattern {
        FatPattern::Android => "android",
        FatPattern::Gynoid => "gynoid",
        FatPattern::Uniform => "uniform",
    };
    format!(
        r#"{{"total_fat":{},"pattern":"{}","visceral_ratio":{},"enabled":{}}}"#,
        morph.total_fat, pat, morph.visceral_ratio, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_fat() {
        let m = new_subcutaneous_fat_morph(4);
        assert!((m.total_fat - 0.3).abs() < 1e-6 /* default fat must be 0.3 */);
    }

    #[test]
    fn test_set_fat_clamps() {
        let mut m = new_subcutaneous_fat_morph(4);
        sfm_set_fat(&mut m, 1.5);
        assert!((m.total_fat - 1.0).abs() < 1e-6 /* fat clamped to 1.0 */);
    }

    #[test]
    fn test_set_pattern() {
        let mut m = new_subcutaneous_fat_morph(4);
        sfm_set_pattern(&mut m, FatPattern::Gynoid);
        assert_eq!(m.pattern, FatPattern::Gynoid /* pattern must be set */);
    }

    #[test]
    fn test_visceral_ratio_clamped() {
        let mut m = new_subcutaneous_fat_morph(4);
        sfm_set_visceral_ratio(&mut m, -0.1);
        assert!((m.visceral_ratio).abs() < 1e-6 /* visceral ratio clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_subcutaneous_fat_morph(5);
        assert_eq!(
            sfm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_subcutaneous_fat_morph(4);
        sfm_set_enabled(&mut m, false);
        assert!(sfm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_pattern() {
        let m = new_subcutaneous_fat_morph(4);
        let j = sfm_to_json(&m);
        assert!(j.contains("\"pattern\"") /* JSON must have pattern field */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_subcutaneous_fat_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_android_scale() {
        let mut m = new_subcutaneous_fat_morph(2);
        sfm_set_fat(&mut m, 1.0);
        sfm_set_pattern(&mut m, FatPattern::Android);
        let out = sfm_evaluate(&m);
        assert!((out[0] - 0.9).abs() < 1e-5 /* android pattern applies 0.9 scale */);
    }
}
