#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! BMI-based body morph.

/// BMI-derived morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BmiMorph {
    /// BMI value (kg/m²).
    pub bmi: f32,
    /// Fat distribution blend [0, 1].
    pub fat_distribution: f32,
    /// Muscle-mass blend [0, 1].
    pub muscle_mass: f32,
}

/// Returns a default `BmiMorph` for a normal-weight person (BMI 22).
#[allow(dead_code)]
pub fn default_bmi_morph() -> BmiMorph {
    BmiMorph {
        bmi: 22.0,
        fat_distribution: 0.3,
        muscle_mass: 0.4,
    }
}

/// Classify a BMI value into a category.
///
/// Returns:
/// * `0` – underweight (BMI < 18.5)
/// * `1` – normal (18.5 ≤ BMI < 25)
/// * `2` – overweight (25 ≤ BMI < 30)
/// * `3` – obese (BMI ≥ 30)
#[allow(dead_code)]
pub fn bmi_category(bmi: f32) -> u8 {
    if bmi < 18.5 {
        0
    } else if bmi < 25.0 {
        1
    } else if bmi < 30.0 {
        2
    } else {
        3
    }
}

/// Apply BMI morph weights to a mutable slice.
///
/// Expects at least 2 elements: `[fat_distribution, muscle_mass]`.
#[allow(dead_code)]
pub fn apply_bmi_morph(weights: &mut [f32], bm: &BmiMorph) {
    if !weights.is_empty() {
        weights[0] = bm.fat_distribution;
    }
    if weights.len() >= 2 {
        weights[1] = bm.muscle_mass;
    }
}

/// Linearly blend two `BmiMorph` structs.
#[allow(dead_code)]
pub fn bmi_blend(a: &BmiMorph, b: &BmiMorph, t: f32) -> BmiMorph {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    BmiMorph {
        bmi: lerp(a.bmi, b.bmi),
        fat_distribution: lerp(a.fat_distribution, b.fat_distribution),
        muscle_mass: lerp(a.muscle_mass, b.muscle_mass),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bmi_morph_normal() {
        let bm = default_bmi_morph();
        assert!((bm.bmi - 22.0).abs() < 1e-6);
        assert_eq!(bmi_category(bm.bmi), 1);
    }

    #[test]
    fn bmi_category_underweight() {
        assert_eq!(bmi_category(17.0), 0);
    }

    #[test]
    fn bmi_category_normal() {
        assert_eq!(bmi_category(22.0), 1);
    }

    #[test]
    fn bmi_category_overweight() {
        assert_eq!(bmi_category(27.0), 2);
    }

    #[test]
    fn bmi_category_obese() {
        assert_eq!(bmi_category(35.0), 3);
    }

    #[test]
    fn apply_bmi_morph_fills_weights() {
        let bm = BmiMorph { bmi: 25.0, fat_distribution: 0.6, muscle_mass: 0.3 };
        let mut w = vec![0.0_f32; 2];
        apply_bmi_morph(&mut w, &bm);
        assert!((w[0] - 0.6).abs() < 1e-6);
        assert!((w[1] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn apply_bmi_morph_short_slice() {
        let bm = default_bmi_morph();
        let mut w: Vec<f32> = Vec::new();
        apply_bmi_morph(&mut w, &bm); // must not panic
    }

    #[test]
    fn bmi_blend_at_zero() {
        let a = default_bmi_morph();
        let b = BmiMorph { bmi: 30.0, fat_distribution: 0.8, muscle_mass: 0.2 };
        let r = bmi_blend(&a, &b, 0.0);
        assert!((r.bmi - a.bmi).abs() < 1e-6);
    }

    #[test]
    fn bmi_blend_at_one() {
        let a = default_bmi_morph();
        let b = BmiMorph { bmi: 30.0, fat_distribution: 0.8, muscle_mass: 0.2 };
        let r = bmi_blend(&a, &b, 1.0);
        assert!((r.bmi - b.bmi).abs() < 1e-6);
    }

    #[test]
    fn bmi_blend_midpoint() {
        let a = BmiMorph { bmi: 20.0, fat_distribution: 0.0, muscle_mass: 0.0 };
        let b = BmiMorph { bmi: 30.0, fat_distribution: 1.0, muscle_mass: 1.0 };
        let r = bmi_blend(&a, &b, 0.5);
        assert!((r.bmi - 25.0).abs() < 1e-5);
    }

    #[test]
    fn bmi_category_boundary_18_5() {
        assert_eq!(bmi_category(18.5), 1);
    }
}
