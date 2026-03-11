// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fat / muscle / bone composition model with formula-based conversions.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Body composition as fractions (should sum to ~1.0).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BodyComposition {
    /// Fraction of body fat (0.0–1.0).
    pub fat_pct: f32,
    /// Fraction of muscle mass.
    pub muscle_pct: f32,
    /// Fraction of bone mass.
    pub bone_pct: f32,
    /// Fraction of water.
    pub water_pct: f32,
}

/// Extended profile including biometric context.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompositionProfile {
    /// 0 = male, 1 = female.
    pub sex: u8,
    pub age: f32,
    pub height_m: f32,
    pub weight_kg: f32,
    pub composition: BodyComposition,
}

// ── Validation & basic formulas ───────────────────────────────────────────────

/// Returns true if all fractions are in `[0,1]` and sum is within 1% of 1.0.
#[allow(dead_code)]
pub fn validate_composition(comp: &BodyComposition) -> bool {
    let parts = [comp.fat_pct, comp.muscle_pct, comp.bone_pct, comp.water_pct];
    if parts.iter().any(|v| !(0.0..=1.0).contains(v)) {
        return false;
    }
    let sum: f32 = parts.iter().sum();
    (sum - 1.0).abs() < 0.01
}

/// Body Mass Index.
#[allow(dead_code)]
pub fn bmi(height_m: f32, weight_kg: f32) -> f32 {
    if height_m < 0.01 {
        return 0.0;
    }
    weight_kg / (height_m * height_m)
}

/// Deurenberg formula: estimate body fat % from BMI, sex, and age.
/// sex: 0 = male, 1 = female.
#[allow(dead_code)]
pub fn body_fat_from_bmi_sex_age(bmi_val: f32, sex: u8, age_years: f32) -> f32 {
    // BF% = 1.20 * BMI + 0.23 * age - 10.8 * sex_factor - 5.4
    // sex_factor: 1 for male, 0 for female
    let sex_factor = if sex == 0 { 1.0_f32 } else { 0.0_f32 };
    let bf = 1.20 * bmi_val + 0.23 * age_years - 10.8 * sex_factor - 5.4;
    (bf / 100.0).clamp(0.02, 0.65)
}

/// Lean mass in kilograms.
#[allow(dead_code)]
pub fn lean_mass_kg(weight_kg: f32, fat_pct: f32) -> f32 {
    weight_kg * (1.0 - fat_pct.clamp(0.0, 1.0))
}

/// Fat mass in kilograms.
#[allow(dead_code)]
pub fn fat_mass_kg(weight_kg: f32, fat_pct: f32) -> f32 {
    weight_kg * fat_pct.clamp(0.0, 1.0)
}

/// Classify body fat percentage.
/// sex: 0 = male, 1 = female.
#[allow(dead_code)]
pub fn classify_body_fat(fat_pct: f32, sex: u8) -> &'static str {
    if sex == 0 {
        // Male thresholds (ACE guidelines)
        if fat_pct < 0.05 {
            "essential"
        } else if fat_pct < 0.14 {
            "athletic"
        } else if fat_pct < 0.18 {
            "fitness"
        } else if fat_pct < 0.25 {
            "average"
        } else {
            "obese"
        }
    } else {
        // Female thresholds
        if fat_pct < 0.10 {
            "essential"
        } else if fat_pct < 0.21 {
            "athletic"
        } else if fat_pct < 0.25 {
            "fitness"
        } else if fat_pct < 0.32 {
            "average"
        } else {
            "obese"
        }
    }
}

/// Devine formula: ideal body weight in kg.
/// sex: 0 = male, 1 = female.
#[allow(dead_code)]
pub fn ideal_weight_devine(height_m: f32, sex: u8) -> f32 {
    let height_cm = height_m * 100.0;
    let inches_over_5ft = ((height_cm / 2.54) - 60.0).max(0.0);
    if sex == 0 {
        50.0 + 2.3 * inches_over_5ft
    } else {
        45.5 + 2.3 * inches_over_5ft
    }
}

/// Fat-Free Mass Index = lean_mass_kg / height_m^2.
#[allow(dead_code)]
pub fn ffmi(lean_mass_kg_val: f32, height_m: f32) -> f32 {
    if height_m < 0.01 {
        return 0.0;
    }
    lean_mass_kg_val / (height_m * height_m)
}

// ── Morph parameter mapping ───────────────────────────────────────────────────

/// Map body composition to a list of morph parameter names and values.
#[allow(dead_code)]
pub fn morph_params_from_composition(comp: &BodyComposition) -> Vec<(String, f32)> {
    vec![
        ("fat-torso".to_string(), comp.fat_pct),
        ("fat-arms".to_string(), comp.fat_pct * 0.7),
        ("fat-legs".to_string(), comp.fat_pct * 0.8),
        ("muscle-torso".to_string(), comp.muscle_pct),
        (
            "muscle-arms".to_string(),
            comp.muscle_pct * 1.1_f32.min(1.0),
        ),
        ("muscle-legs".to_string(), comp.muscle_pct * 0.9),
        ("bone-mass".to_string(), comp.bone_pct),
    ]
}

/// Inverse mapping: reconstruct approximate composition from morph params.
#[allow(dead_code)]
pub fn composition_from_morph_params(params: &[(String, f32)]) -> BodyComposition {
    let get = |key: &str| -> f32 {
        params
            .iter()
            .find(|(k, _)| k == key)
            .map_or(0.0, |(_, v)| *v)
    };
    let fat_pct = get("fat-torso").clamp(0.0, 1.0);
    let muscle_pct = get("muscle-torso").clamp(0.0, 1.0);
    let bone_pct = get("bone-mass").clamp(0.0, 1.0);
    let rest = (1.0_f32 - fat_pct - muscle_pct - bone_pct).max(0.0);
    BodyComposition {
        fat_pct,
        muscle_pct,
        bone_pct,
        water_pct: rest,
    }
}

/// Linear interpolation between two compositions.
#[allow(dead_code)]
pub fn interpolate_compositions(
    a: &BodyComposition,
    b: &BodyComposition,
    t: f32,
) -> BodyComposition {
    let lerp = |x: f32, y: f32| x + t * (y - x);
    BodyComposition {
        fat_pct: lerp(a.fat_pct, b.fat_pct),
        muscle_pct: lerp(a.muscle_pct, b.muscle_pct),
        bone_pct: lerp(a.bone_pct, b.bone_pct),
        water_pct: lerp(a.water_pct, b.water_pct),
    }
}

/// L2 distance between two composition vectors.
#[allow(dead_code)]
pub fn composition_distance(a: &BodyComposition, b: &BodyComposition) -> f32 {
    let df = a.fat_pct - b.fat_pct;
    let dm = a.muscle_pct - b.muscle_pct;
    let db = a.bone_pct - b.bone_pct;
    let dw = a.water_pct - b.water_pct;
    (df * df + dm * dm + db * db + dw * dw).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_comp() -> BodyComposition {
        BodyComposition {
            fat_pct: 0.20,
            muscle_pct: 0.45,
            bone_pct: 0.15,
            water_pct: 0.20,
        }
    }

    #[test]
    fn test_validate_composition_valid() {
        assert!(validate_composition(&valid_comp()));
    }

    #[test]
    fn test_validate_composition_over_1() {
        let c = BodyComposition {
            fat_pct: 0.5,
            muscle_pct: 0.5,
            bone_pct: 0.5,
            water_pct: 0.5,
        };
        assert!(!validate_composition(&c));
    }

    #[test]
    fn test_validate_composition_negative() {
        let c = BodyComposition {
            fat_pct: -0.1,
            muscle_pct: 0.5,
            bone_pct: 0.2,
            water_pct: 0.4,
        };
        assert!(!validate_composition(&c));
    }

    #[test]
    fn test_bmi_formula() {
        let b = bmi(1.8, 81.0);
        assert!((b - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_bmi_zero_height() {
        assert_eq!(bmi(0.0, 80.0), 0.0);
    }

    #[test]
    fn test_body_fat_no_nan() {
        let bf = body_fat_from_bmi_sex_age(25.0, 0, 30.0);
        assert!(!bf.is_nan());
    }

    #[test]
    fn test_body_fat_clamped() {
        let bf = body_fat_from_bmi_sex_age(25.0, 0, 30.0);
        assert!((0.02..=0.65).contains(&bf));
    }

    #[test]
    fn test_classify_fat_male_athletic() {
        assert_eq!(classify_body_fat(0.10, 0), "athletic");
    }

    #[test]
    fn test_classify_fat_female_obese() {
        assert_eq!(classify_body_fat(0.38, 1), "obese");
    }

    #[test]
    fn test_classify_fat_male_essential() {
        assert_eq!(classify_body_fat(0.03, 0), "essential");
    }

    #[test]
    fn test_ideal_weight_sex_difference() {
        let male = ideal_weight_devine(1.75, 0);
        let female = ideal_weight_devine(1.75, 1);
        assert!(male > female);
    }

    #[test]
    fn test_ideal_weight_devine_known() {
        // 5'9" = 175.26 cm. inches over 5ft = 9
        let w = ideal_weight_devine(1.7526, 0);
        assert!((w - 70.7).abs() < 1.0);
    }

    #[test]
    fn test_ffmi_range() {
        let f = ffmi(70.0, 1.8);
        assert!(f > 10.0 && f < 30.0);
    }

    #[test]
    fn test_interpolate_t0() {
        let a = valid_comp();
        let b = BodyComposition {
            fat_pct: 0.30,
            muscle_pct: 0.40,
            bone_pct: 0.10,
            water_pct: 0.20,
        };
        let out = interpolate_compositions(&a, &b, 0.0);
        assert!((out.fat_pct - a.fat_pct).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_t1() {
        let a = valid_comp();
        let b = BodyComposition {
            fat_pct: 0.30,
            muscle_pct: 0.40,
            bone_pct: 0.10,
            water_pct: 0.20,
        };
        let out = interpolate_compositions(&a, &b, 1.0);
        assert!((out.fat_pct - b.fat_pct).abs() < 1e-6);
    }

    #[test]
    fn test_composition_distance_zero_same() {
        let a = valid_comp();
        let d = composition_distance(&a, &a);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_composition_distance_positive_different() {
        let a = valid_comp();
        let b = BodyComposition {
            fat_pct: 0.30,
            muscle_pct: 0.40,
            bone_pct: 0.10,
            water_pct: 0.20,
        };
        assert!(composition_distance(&a, &b) > 0.0);
    }

    #[test]
    fn test_lean_mass_kg() {
        assert!((lean_mass_kg(80.0, 0.20) - 64.0).abs() < 1e-4);
    }

    #[test]
    fn test_fat_mass_kg() {
        assert!((fat_mass_kg(80.0, 0.20) - 16.0).abs() < 1e-4);
    }
}
