// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BMI-driven body shape morph stub.

/// BMI classification category.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BmiCategory {
    Underweight,
    Normal,
    Overweight,
    Obese,
}

/// BMI-driven body shape morph controller.
#[derive(Debug, Clone)]
pub struct BodyMassIndexMorph {
    pub bmi: f32,
    pub category: BmiCategory,
    pub influence: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl BodyMassIndexMorph {
    pub fn new(morph_count: usize) -> Self {
        BodyMassIndexMorph {
            bmi: 22.0,
            category: BmiCategory::Normal,
            influence: 1.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new BMI morph controller.
pub fn new_bmi_morph(morph_count: usize) -> BodyMassIndexMorph {
    BodyMassIndexMorph::new(morph_count)
}

/// Set BMI value and update category.
pub fn bmi_set_value(morph: &mut BodyMassIndexMorph, bmi: f32) {
    morph.bmi = bmi.clamp(10.0, 60.0);
    morph.category = if bmi < 18.5 {
        BmiCategory::Underweight
    } else if bmi < 25.0 {
        BmiCategory::Normal
    } else if bmi < 30.0 {
        BmiCategory::Overweight
    } else {
        BmiCategory::Obese
    };
}

/// Set overall influence scale.
pub fn bmi_set_influence(morph: &mut BodyMassIndexMorph, influence: f32) {
    morph.influence = influence.clamp(0.0, 1.0);
}

/// Evaluate morph weights based on BMI (stub: normalized weights).
pub fn bmi_evaluate(morph: &BodyMassIndexMorph) -> Vec<f32> {
    /* Stub: distribute influence evenly across morph targets */
    if morph.morph_count == 0 || !morph.enabled {
        return vec![];
    }
    let w = morph.influence * ((morph.bmi - 10.0) / 50.0).clamp(0.0, 1.0);
    vec![w; morph.morph_count]
}

/// Enable or disable morph.
pub fn bmi_set_enabled(morph: &mut BodyMassIndexMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn bmi_to_json(morph: &BodyMassIndexMorph) -> String {
    let cat = match morph.category {
        BmiCategory::Underweight => "underweight",
        BmiCategory::Normal => "normal",
        BmiCategory::Overweight => "overweight",
        BmiCategory::Obese => "obese",
    };
    format!(
        r#"{{"bmi":{},"category":"{}","influence":{},"morph_count":{},"enabled":{}}}"#,
        morph.bmi, cat, morph.influence, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default_bmi() {
        let m = new_bmi_morph(8);
        assert!((m.bmi - 22.0).abs() < 1e-6 /* default BMI must be 22.0 */);
    }

    #[test]
    fn test_category_normal() {
        let m = new_bmi_morph(4);
        assert_eq!(
            m.category,
            BmiCategory::Normal /* default category must be Normal */
        );
    }

    #[test]
    fn test_set_underweight() {
        let mut m = new_bmi_morph(4);
        bmi_set_value(&mut m, 17.0);
        assert_eq!(
            m.category,
            BmiCategory::Underweight /* BMI 17 => Underweight */
        );
    }

    #[test]
    fn test_set_overweight() {
        let mut m = new_bmi_morph(4);
        bmi_set_value(&mut m, 27.0);
        assert_eq!(
            m.category,
            BmiCategory::Overweight /* BMI 27 => Overweight */
        );
    }

    #[test]
    fn test_set_obese() {
        let mut m = new_bmi_morph(4);
        bmi_set_value(&mut m, 35.0);
        assert_eq!(m.category, BmiCategory::Obese /* BMI 35 => Obese */);
    }

    #[test]
    fn test_influence_clamped() {
        let mut m = new_bmi_morph(4);
        bmi_set_influence(&mut m, 2.0);
        assert!((m.influence - 1.0).abs() < 1e-6 /* influence clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_bmi_morph(6);
        let out = bmi_evaluate(&m);
        assert_eq!(out.len(), 6 /* output length must match morph_count */);
    }

    #[test]
    fn test_evaluate_disabled_empty() {
        let mut m = new_bmi_morph(4);
        bmi_set_enabled(&mut m, false);
        let out = bmi_evaluate(&m);
        assert!(out.is_empty() /* disabled morph must return empty */);
    }

    #[test]
    fn test_to_json_contains_bmi() {
        let m = new_bmi_morph(4);
        let j = bmi_to_json(&m);
        assert!(j.contains("\"bmi\"") /* JSON must contain bmi field */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_bmi_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }
}
