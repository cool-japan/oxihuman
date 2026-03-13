// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::params::ParamState;

/// Life stage categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LifeStage {
    Infant,     // 0-2 years
    Child,      // 3-12 years
    Adolescent, // 13-17 years
    YoungAdult, // 18-35 years
    MiddleAge,  // 36-55 years
    Senior,     // 56-75 years
    Elderly,    // 76+ years
}

impl LifeStage {
    /// Infer life stage from age in years.
    pub fn from_age_years(age_years: f32) -> Self {
        if age_years < 3.0 {
            LifeStage::Infant
        } else if age_years < 13.0 {
            LifeStage::Child
        } else if age_years < 18.0 {
            LifeStage::Adolescent
        } else if age_years < 36.0 {
            LifeStage::YoungAdult
        } else if age_years < 56.0 {
            LifeStage::MiddleAge
        } else if age_years < 76.0 {
            LifeStage::Senior
        } else {
            LifeStage::Elderly
        }
    }

    /// Approximate age range midpoint (years).
    pub fn midpoint_years(&self) -> f32 {
        match self {
            LifeStage::Infant => 1.0,
            LifeStage::Child => 7.5,
            LifeStage::Adolescent => 15.0,
            LifeStage::YoungAdult => 26.5,
            LifeStage::MiddleAge => 45.5,
            LifeStage::Senior => 65.5,
            LifeStage::Elderly => 85.0,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            LifeStage::Infant => "Infant",
            LifeStage::Child => "Child",
            LifeStage::Adolescent => "Adolescent",
            LifeStage::YoungAdult => "Young Adult",
            LifeStage::MiddleAge => "Middle Age",
            LifeStage::Senior => "Senior",
            LifeStage::Elderly => "Elderly",
        }
    }

    /// All life stages in order.
    pub fn all() -> [LifeStage; 7] {
        [
            LifeStage::Infant,
            LifeStage::Child,
            LifeStage::Adolescent,
            LifeStage::YoungAdult,
            LifeStage::MiddleAge,
            LifeStage::Senior,
            LifeStage::Elderly,
        ]
    }
}

/// Compute height factor from age using a smooth piecewise function.
/// - 0→18 years: grows from 0.3 to 1.0 (sigmoid-ish)
/// - 18→60: stays near 1.0 (slight decrease to 0.97)
/// - 60→90: decreases to 0.93
fn compute_height_factor(age: f32) -> f32 {
    let age = age.clamp(0.0, 120.0);
    if age <= 18.0 {
        // Sigmoid-ish growth from 0.3 to 1.0 over 0..18
        let t = age / 18.0; // 0..1
                            // Use smoothstep-like: 0.3 + 0.7 * smoothstep
        let s = t * t * (3.0 - 2.0 * t);
        0.3 + 0.7 * s
    } else if age <= 60.0 {
        // Linear slight decrease from 1.0 to 0.97 over 18..60
        let t = (age - 18.0) / (60.0 - 18.0);
        1.0 - 0.03 * t
    } else {
        // Linear decrease from 0.97 to 0.93 over 60..90
        let t = ((age - 60.0) / (90.0 - 60.0)).min(1.0);
        0.97 - 0.04 * t
    }
}

/// Age-dependent body parameter adjustments.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AgeProfile {
    /// Age in years (0..=120).
    pub age_years: f32,
    pub life_stage: LifeStage,
    /// Height relative to peak adult height [0.1, 1.0].
    /// Grows to 1.0 around age 18-20, then decreases slightly with age.
    pub height_factor: f32,
    /// Weight tendency [0, 1]. Increases through adulthood.
    pub weight_tendency: f32,
    /// Muscle tone [0, 1]. Peaks in young adulthood, declines with age.
    pub muscle_factor: f32,
    /// Skin elasticity [0, 1]. Decreases with age.
    pub skin_elasticity: f32,
    /// Bone density [0, 1]. Peaks at ~30, decreases after.
    pub bone_density: f32,
    /// Posture score [0, 1]. 1=upright, decreases with age (stooping).
    pub posture: f32,
}

impl AgeProfile {
    /// Compute an age profile for a given age in years.
    pub fn for_age(age_years: f32) -> Self {
        let age = age_years.clamp(0.0, 120.0);
        let life_stage = LifeStage::from_age_years(age);

        let height_factor = compute_height_factor(age);

        // Weight tendency: rises through adulthood, peaks ~50, slight decrease after
        let weight_tendency = if age <= 18.0 {
            let t = age / 18.0;
            0.2 + 0.3 * t
        } else if age <= 50.0 {
            let t = (age - 18.0) / (50.0 - 18.0);
            0.5 + 0.2 * t
        } else if age <= 80.0 {
            let t = (age - 50.0) / (80.0 - 50.0);
            0.7 - 0.1 * t
        } else {
            let t = ((age - 80.0) / 40.0).min(1.0);
            0.6 - 0.15 * t
        };

        // Muscle factor: grows until ~25, stays high until ~35, declines
        let muscle_factor = if age <= 25.0 {
            let t = age / 25.0;
            0.3 + 0.7 * t * t * (3.0 - 2.0 * t)
        } else if age <= 35.0 {
            1.0
        } else if age <= 80.0 {
            let t = (age - 35.0) / (80.0 - 35.0);
            1.0 - 0.6 * t
        } else {
            let t = ((age - 80.0) / 40.0).min(1.0);
            0.4 - 0.2 * t
        };

        // Skin elasticity: high in youth, linear decline with age
        let skin_elasticity = if age <= 20.0 {
            1.0
        } else {
            let t = ((age - 20.0) / 100.0).min(1.0);
            1.0 - 0.85 * t
        };

        // Bone density: grows until ~30, decreases after
        let bone_density = if age <= 30.0 {
            let t = age / 30.0;
            0.5 + 0.5 * t
        } else if age <= 90.0 {
            let t = (age - 30.0) / (90.0 - 30.0);
            1.0 - 0.5 * t
        } else {
            let t = ((age - 90.0) / 30.0).min(1.0);
            0.5 - 0.2 * t
        };

        // Posture: upright in youth, slight decline with age
        let posture = if age <= 40.0 {
            1.0
        } else if age <= 90.0 {
            let t = (age - 40.0) / (90.0 - 40.0);
            1.0 - 0.35 * t
        } else {
            let t = ((age - 90.0) / 30.0).min(1.0);
            0.65 - 0.2 * t
        };

        AgeProfile {
            age_years: age,
            life_stage,
            height_factor: height_factor.clamp(0.1, 1.0),
            weight_tendency: weight_tendency.clamp(0.0, 1.0),
            muscle_factor: muscle_factor.clamp(0.0, 1.0),
            skin_elasticity: skin_elasticity.clamp(0.0, 1.0),
            bone_density: bone_density.clamp(0.0, 1.0),
            posture: posture.clamp(0.0, 1.0),
        }
    }

    /// Apply this age profile to a ParamState, returning adjusted params.
    pub fn apply_to_params(&self, base: &ParamState) -> ParamState {
        let mut result = base.clone();

        // Scale height by height_factor
        result.height = (base.height * self.height_factor).clamp(0.0, 1.0);

        // Blend weight toward weight_tendency
        result.weight = (base.weight * 0.7 + self.weight_tendency * 0.3).clamp(0.0, 1.0);

        // Scale muscle by muscle_factor
        result.muscle = (base.muscle * self.muscle_factor).clamp(0.0, 1.0);

        // Set age parameter using normalized years
        result.age = years_to_param_age(self.age_years);

        // Store extra age-related parameters
        result
            .extra
            .insert("skin_elasticity".to_string(), self.skin_elasticity);
        result
            .extra
            .insert("bone_density".to_string(), self.bone_density);
        result.extra.insert("posture".to_string(), self.posture);

        result
    }

    /// Lerp between two AgeProfiles.
    pub fn lerp(&self, other: &AgeProfile, t: f32) -> AgeProfile {
        let t = t.clamp(0.0, 1.0);
        let age_years = self.age_years + (other.age_years - self.age_years) * t;
        AgeProfile {
            age_years,
            life_stage: LifeStage::from_age_years(age_years),
            height_factor: self.height_factor + (other.height_factor - self.height_factor) * t,
            weight_tendency: self.weight_tendency
                + (other.weight_tendency - self.weight_tendency) * t,
            muscle_factor: self.muscle_factor + (other.muscle_factor - self.muscle_factor) * t,
            skin_elasticity: self.skin_elasticity
                + (other.skin_elasticity - self.skin_elasticity) * t,
            bone_density: self.bone_density + (other.bone_density - self.bone_density) * t,
            posture: self.posture + (other.posture - self.posture) * t,
        }
    }
}

/// Generate a sequence of AgeProfiles from age_start to age_end.
/// `steps` controls how many profiles are generated.
pub fn age_progression(age_start: f32, age_end: f32, steps: usize) -> Vec<AgeProfile> {
    if steps == 0 {
        return Vec::new();
    }
    if steps == 1 {
        return vec![AgeProfile::for_age(age_start)];
    }
    (0..steps)
        .map(|i| {
            let t = i as f32 / (steps - 1) as f32;
            let age = age_start + (age_end - age_start) * t;
            AgeProfile::for_age(age)
        })
        .collect()
}

/// Convert OxiHuman normalized age parameter `[0,1]` to approximate years.
/// 0.0 -> 0 years, 0.5 -> 35 years (adult), 1.0 -> 90 years
pub fn param_age_to_years(param_age: f32) -> f32 {
    let t = param_age.clamp(0.0, 1.0);
    // Piecewise: 0..0.5 maps to 0..35, 0.5..1.0 maps to 35..90
    if t <= 0.5 {
        t * 2.0 * 35.0
    } else {
        35.0 + (t - 0.5) * 2.0 * 55.0
    }
}

/// Convert years to normalized OxiHuman age parameter.
pub fn years_to_param_age(years: f32) -> f32 {
    let y = years.clamp(0.0, 90.0);
    if y <= 35.0 {
        y / 35.0 * 0.5
    } else {
        0.5 + (y - 35.0) / 55.0 * 0.5
    }
}

/// Compute aging delta: how much do parameters change between age A and age B?
/// Returns a description of the changes.
pub fn aging_delta(age_a: f32, age_b: f32) -> String {
    let profile_a = AgeProfile::for_age(age_a);
    let profile_b = AgeProfile::for_age(age_b);

    let dh = profile_b.height_factor - profile_a.height_factor;
    let dw = profile_b.weight_tendency - profile_a.weight_tendency;
    let dm = profile_b.muscle_factor - profile_a.muscle_factor;
    let ds = profile_b.skin_elasticity - profile_a.skin_elasticity;
    let db = profile_b.bone_density - profile_a.bone_density;
    let dp = profile_b.posture - profile_a.posture;

    format!(
        "Age {:.1}y ({}) -> {:.1}y ({}): \
         height_factor {:+.3}, \
         weight_tendency {:+.3}, \
         muscle {:+.3}, \
         skin_elasticity {:+.3}, \
         bone_density {:+.3}, \
         posture {:+.3}",
        age_a,
        profile_a.life_stage.label(),
        age_b,
        profile_b.life_stage.label(),
        dh,
        dw,
        dm,
        ds,
        db,
        dp,
    )
}

/// BMI category based on weight and height parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum BmiCategory {
    Underweight,
    Normal,
    Overweight,
    Obese,
}

/// Estimate BMI category from OxiHuman weight parameter (rough approximation).
/// weight_param in [0, 1]: 0=very thin, 0.3=normal low, 0.5=normal high, 0.7=overweight, 1.0=obese
pub fn estimate_bmi_category(weight_param: f32) -> BmiCategory {
    let w = weight_param.clamp(0.0, 1.0);
    if w < 0.25 {
        BmiCategory::Underweight
    } else if w < 0.55 {
        BmiCategory::Normal
    } else if w < 0.75 {
        BmiCategory::Overweight
    } else {
        BmiCategory::Obese
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn life_stage_infant_from_age_1() {
        assert_eq!(LifeStage::from_age_years(1.0), LifeStage::Infant);
    }

    #[test]
    fn life_stage_child_from_age_8() {
        assert_eq!(LifeStage::from_age_years(8.0), LifeStage::Child);
    }

    #[test]
    fn life_stage_young_adult_from_age_25() {
        assert_eq!(LifeStage::from_age_years(25.0), LifeStage::YoungAdult);
    }

    #[test]
    fn life_stage_senior_from_age_65() {
        assert_eq!(LifeStage::from_age_years(65.0), LifeStage::Senior);
    }

    #[test]
    fn life_stage_all_has_seven() {
        assert_eq!(LifeStage::all().len(), 7);
    }

    #[test]
    fn age_profile_infant_short_height() {
        let p = AgeProfile::for_age(1.0);
        // At age 1, height factor should be well below 1.0 (near 0.3 range)
        assert!(
            p.height_factor < 0.5,
            "infant height_factor={}",
            p.height_factor
        );
    }

    #[test]
    fn age_profile_adult_full_height() {
        let p = AgeProfile::for_age(25.0);
        // Young adult should be near peak height
        assert!(
            p.height_factor > 0.95,
            "adult height_factor={}",
            p.height_factor
        );
    }

    #[test]
    fn age_profile_elderly_reduced_height() {
        let p = AgeProfile::for_age(80.0);
        // Elderly should have reduced height vs peak
        assert!(
            p.height_factor < 0.97,
            "elderly height_factor={}",
            p.height_factor
        );
    }

    #[test]
    fn age_profile_muscle_peaks_young_adult() {
        let young = AgeProfile::for_age(30.0);
        let old = AgeProfile::for_age(75.0);
        assert!(
            young.muscle_factor > old.muscle_factor,
            "young={} old={}",
            young.muscle_factor,
            old.muscle_factor
        );
    }

    #[test]
    fn age_profile_lerp_midpoint() {
        let a = AgeProfile::for_age(20.0);
        let b = AgeProfile::for_age(60.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.age_years - 40.0).abs() < 1e-4);
        // Mid height factor should be between a and b
        let hf = mid.height_factor;
        let lo = a.height_factor.min(b.height_factor);
        let hi = a.height_factor.max(b.height_factor);
        assert!(hf >= lo - 1e-5 && hf <= hi + 1e-5);
    }

    #[test]
    fn age_progression_correct_length() {
        let profiles = age_progression(0.0, 90.0, 10);
        assert_eq!(profiles.len(), 10);
        assert!((profiles[0].age_years - 0.0).abs() < 1e-4);
        assert!((profiles[9].age_years - 90.0).abs() < 1e-4);
    }

    #[test]
    fn param_age_to_years_zero_is_zero() {
        let years = param_age_to_years(0.0);
        assert!((years - 0.0).abs() < 1e-5, "years={}", years);
    }

    #[test]
    fn param_age_to_years_half_is_midlife() {
        let years = param_age_to_years(0.5);
        assert!((years - 35.0).abs() < 1e-5, "years={}", years);
    }

    #[test]
    fn years_to_param_age_roundtrip() {
        for y in [0.0f32, 10.0, 25.0, 35.0, 50.0, 70.0, 90.0] {
            let p = years_to_param_age(y);
            let back = param_age_to_years(p);
            assert!((back - y).abs() < 1e-3, "y={} p={} back={}", y, p, back);
        }
    }

    #[test]
    fn estimate_bmi_normal_range() {
        assert_eq!(estimate_bmi_category(0.4), BmiCategory::Normal);
        assert_eq!(estimate_bmi_category(0.1), BmiCategory::Underweight);
        assert_eq!(estimate_bmi_category(0.65), BmiCategory::Overweight);
        assert_eq!(estimate_bmi_category(0.9), BmiCategory::Obese);
    }

    #[test]
    fn aging_delta_string_not_empty() {
        let s = aging_delta(20.0, 70.0);
        assert!(!s.is_empty());
        assert!(s.contains("->"), "missing arrow in: {}", s);
    }
}
