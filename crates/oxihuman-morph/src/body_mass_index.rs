// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Body Mass Index calculations and morph weight derivation.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BmiClassification {
    Underweight,
    Normal,
    Overweight,
    ObeseI,
    ObeseII,
    ObeseIII,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BmiProfile {
    pub height_m: f32,
    pub weight_kg: f32,
    pub bmi_value: f32,
    pub classification: BmiClassification,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BmiMorphWeights {
    pub thin: f32,
    pub average: f32,
    pub heavy: f32,
    pub obese: f32,
}

#[allow(dead_code)]
pub fn compute_bmi_value(height_m: f32, weight_kg: f32) -> f32 {
    if height_m <= 0.0 {
        return 0.0;
    }
    weight_kg / (height_m * height_m)
}

#[allow(dead_code)]
pub fn classify_bmi(bmi: f32) -> BmiClassification {
    if bmi < 18.5 {
        BmiClassification::Underweight
    } else if bmi < 25.0 {
        BmiClassification::Normal
    } else if bmi < 30.0 {
        BmiClassification::Overweight
    } else if bmi < 35.0 {
        BmiClassification::ObeseI
    } else if bmi < 40.0 {
        BmiClassification::ObeseII
    } else {
        BmiClassification::ObeseIII
    }
}

#[allow(dead_code)]
pub fn new_bmi_profile(height_m: f32, weight_kg: f32) -> BmiProfile {
    let bmi_value = compute_bmi_value(height_m, weight_kg);
    let classification = classify_bmi(bmi_value);
    BmiProfile {
        height_m,
        weight_kg,
        bmi_value,
        classification,
    }
}

#[allow(dead_code)]
pub fn bmi_to_morph_weights(bmi: f32) -> BmiMorphWeights {
    let norm = ((bmi - 15.0) / 30.0).clamp(0.0, 1.0);
    let thin = (1.0 - norm * 2.0).clamp(0.0, 1.0);
    let average = (1.0 - (norm - 0.35).abs() * 4.0).clamp(0.0, 1.0);
    let heavy = ((norm - 0.5) * 3.0).clamp(0.0, 1.0);
    let obese = ((norm - 0.75) * 4.0).clamp(0.0, 1.0);
    BmiMorphWeights { thin, average, heavy, obese }
}

#[allow(dead_code)]
pub fn weight_from_bmi(bmi: f32, height_m: f32) -> f32 {
    bmi * height_m * height_m
}

#[allow(dead_code)]
pub fn bmi_profile_to_json(profile: &BmiProfile) -> String {
    let class_str = match &profile.classification {
        BmiClassification::Underweight => "underweight",
        BmiClassification::Normal => "normal",
        BmiClassification::Overweight => "overweight",
        BmiClassification::ObeseI => "obese_i",
        BmiClassification::ObeseII => "obese_ii",
        BmiClassification::ObeseIII => "obese_iii",
    };
    format!(
        r#"{{"height_m":{},"weight_kg":{},"bmi":{},"classification":"{}"}}"#,
        profile.height_m, profile.weight_kg, profile.bmi_value, class_str
    )
}

#[allow(dead_code)]
pub fn blend_bmi_profiles(a: &BmiProfile, b: &BmiProfile, t: f32) -> BmiProfile {
    let t = t.clamp(0.0, 1.0);
    let h = a.height_m + (b.height_m - a.height_m) * t;
    let w = a.weight_kg + (b.weight_kg - a.weight_kg) * t;
    new_bmi_profile(h, w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_bmi() {
        let bmi = compute_bmi_value(1.75, 70.0);
        assert!((bmi - 22.857).abs() < 0.1);
    }

    #[test]
    fn test_classify_normal() {
        assert_eq!(classify_bmi(22.0), BmiClassification::Normal);
    }

    #[test]
    fn test_classify_underweight() {
        assert_eq!(classify_bmi(17.0), BmiClassification::Underweight);
    }

    #[test]
    fn test_classify_overweight() {
        assert_eq!(classify_bmi(27.0), BmiClassification::Overweight);
    }

    #[test]
    fn test_classify_obese() {
        assert_eq!(classify_bmi(32.0), BmiClassification::ObeseI);
    }

    #[test]
    fn test_morph_weights_range() {
        let w = bmi_to_morph_weights(22.0);
        assert!((0.0..=1.0).contains(&w.thin));
        assert!((0.0..=1.0).contains(&w.average));
        assert!((0.0..=1.0).contains(&w.heavy));
    }

    #[test]
    fn test_weight_from_bmi() {
        let w = weight_from_bmi(22.0, 1.75);
        assert!((w - 67.375).abs() < 0.1);
    }

    #[test]
    fn test_profile_to_json() {
        let p = new_bmi_profile(1.75, 70.0);
        let j = bmi_profile_to_json(&p);
        assert!(j.contains("normal"));
    }

    #[test]
    fn test_blend_profiles() {
        let a = new_bmi_profile(1.70, 60.0);
        let b = new_bmi_profile(1.80, 80.0);
        let mid = blend_bmi_profiles(&a, &b, 0.5);
        assert!((mid.height_m - 1.75).abs() < 1e-4);
    }

    #[test]
    fn test_zero_height() {
        let bmi = compute_bmi_value(0.0, 70.0);
        assert!(bmi.abs() < 1e-6);
    }
}
