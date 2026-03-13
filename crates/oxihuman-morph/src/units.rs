// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Unit conversions between real-world measurements and normalized parameters.
//!
//! MakeHuman uses a parametric space where all sliders are in [0.0, 1.0].
//! These functions help convert real-world measurements to/from that space.

/// Height range covered by the height parameter (in cm).
/// Approximate MakeHuman default: 0 → ~150 cm, 1 → ~200 cm.
const HEIGHT_MIN_CM: f32 = 150.0;
const HEIGHT_MAX_CM: f32 = 200.0;

/// BMI range covered by the weight parameter.
/// 0 → underweight (~15 BMI), 1 → obese (~40 BMI).
const BMI_MIN: f32 = 15.0;
const BMI_MAX: f32 = 40.0;

/// Age range covered by the age parameter (years).
const AGE_MIN_YR: f32 = 2.0;
const AGE_MAX_YR: f32 = 90.0;

/// Convert height in centimetres to a normalized height parameter [0.0, 1.0].
pub fn height_cm_to_param(cm: f32) -> f32 {
    ((cm - HEIGHT_MIN_CM) / (HEIGHT_MAX_CM - HEIGHT_MIN_CM)).clamp(0.0, 1.0)
}

/// Convert a normalized height parameter back to centimetres.
pub fn param_to_height_cm(param: f32) -> f32 {
    HEIGHT_MIN_CM + param.clamp(0.0, 1.0) * (HEIGHT_MAX_CM - HEIGHT_MIN_CM)
}

/// Compute Body Mass Index from weight (kg) and height (cm).
pub fn compute_bmi(weight_kg: f32, height_cm: f32) -> f32 {
    let h_m = height_cm / 100.0;
    if h_m < 0.01 {
        return 0.0;
    }
    weight_kg / (h_m * h_m)
}

/// Convert BMI to a normalized weight parameter [0.0, 1.0].
pub fn bmi_to_weight_param(bmi: f32) -> f32 {
    ((bmi - BMI_MIN) / (BMI_MAX - BMI_MIN)).clamp(0.0, 1.0)
}

/// Convert weight_kg + height_cm directly to a weight parameter.
pub fn weight_kg_to_param(weight_kg: f32, height_cm: f32) -> f32 {
    bmi_to_weight_param(compute_bmi(weight_kg, height_cm))
}

/// Convert age in years to a normalized age parameter [0.0, 1.0].
pub fn age_yr_to_param(years: f32) -> f32 {
    ((years - AGE_MIN_YR) / (AGE_MAX_YR - AGE_MIN_YR)).clamp(0.0, 1.0)
}

/// Convert a normalized age parameter back to approximate years.
pub fn param_to_age_yr(param: f32) -> f32 {
    AGE_MIN_YR + param.clamp(0.0, 1.0) * (AGE_MAX_YR - AGE_MIN_YR)
}

/// Convenience: build a ParamState from real-world measurements.
pub fn params_from_measurements(
    height_cm: f32,
    weight_kg: f32,
    age_years: f32,
    muscle_param: f32, // muscle has no direct real-world equivalent yet
) -> crate::params::ParamState {
    crate::params::ParamState::new(
        height_cm_to_param(height_cm),
        weight_kg_to_param(weight_kg, height_cm),
        muscle_param.clamp(0.0, 1.0),
        age_yr_to_param(age_years),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_height_maps_to_midpoint() {
        // 175 cm is between 150-200: (175-150)/(200-150) = 0.5
        let p = height_cm_to_param(175.0);
        assert!((p - 0.5).abs() < 1e-5);
    }

    #[test]
    fn height_roundtrip() {
        for cm in [150.0f32, 165.0, 175.0, 185.0, 200.0] {
            let p = height_cm_to_param(cm);
            let back = param_to_height_cm(p);
            assert!(
                (back - cm).abs() < 0.1,
                "height roundtrip failed for {}cm: got {}cm",
                cm,
                back
            );
        }
    }

    #[test]
    fn bmi_calculation() {
        // 70kg at 175cm → BMI = 70 / (1.75)^2 = 70/3.0625 ≈ 22.86
        let bmi = compute_bmi(70.0, 175.0);
        assert!((bmi - 22.86).abs() < 0.1);
    }

    #[test]
    fn normal_bmi_maps_to_midrange() {
        // BMI 22.86 on scale 15-40: (22.86-15)/(40-15) = 7.86/25 ≈ 0.314
        let p = bmi_to_weight_param(22.86);
        assert!(p > 0.2 && p < 0.5);
    }

    #[test]
    fn age_30_maps_to_midrange() {
        // (30 - 2) / (90 - 2) = 28/88 ≈ 0.318
        let p = age_yr_to_param(30.0);
        assert!(p > 0.2 && p < 0.5);
    }

    #[test]
    fn age_roundtrip() {
        for yr in [10.0f32, 25.0, 40.0, 60.0, 80.0] {
            let p = age_yr_to_param(yr);
            let back = param_to_age_yr(p);
            assert!(
                (back - yr).abs() < 0.1,
                "age roundtrip failed for {}yr: got {}yr",
                yr,
                back
            );
        }
    }

    #[test]
    fn params_from_real_measurements() {
        // 175cm, 70kg, 30yr, 0.6 muscle
        let p = params_from_measurements(175.0, 70.0, 30.0, 0.6);
        assert!((0.0..=1.0).contains(&p.height));
        assert!((0.0..=1.0).contains(&p.weight));
        assert!((0.0..=1.0).contains(&p.muscle));
        assert!((0.0..=1.0).contains(&p.age));
        // 175cm → param = 0.5
        assert!((p.height - 0.5).abs() < 1e-5);
    }

    #[test]
    fn extreme_values_clamped() {
        assert!((height_cm_to_param(50.0) - 0.0).abs() < 1e-5);
        assert!((height_cm_to_param(300.0) - 1.0).abs() < 1e-5);
        assert!((age_yr_to_param(0.0) - 0.0).abs() < 1e-5);
        assert!((age_yr_to_param(200.0) - 1.0).abs() < 1e-5);
    }
}
