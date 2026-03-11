// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Advanced aging pipeline: piecewise-linear age stages, morph deltas, skin/face/body params.

#[allow(dead_code)]
/// One point on the aging curve.
#[derive(Debug, Clone, PartialEq)]
pub struct AgeStage {
    pub years: f32,
    pub fat_pct: f32,
    pub muscle_pct: f32,
    pub bone_density: f32,
    pub skin_elasticity: f32,
}

#[allow(dead_code)]
/// Piecewise-linear aging model.
#[derive(Debug, Clone)]
pub struct AgingCurve {
    pub stages: Vec<AgeStage>,
}

#[allow(dead_code)]
/// Person base profile.
#[derive(Debug, Clone)]
pub struct AgeProfile {
    pub base_age: f32,
    /// 0 = male, 1 = female
    pub sex: u8,
    /// 0..=3 ethnicity index
    pub ethnicity: u8,
}

// ---------------------------------------------------------------------------
// Core interpolation
// ---------------------------------------------------------------------------

/// Linear interpolation of two AgeStages.
pub fn interpolate_age_stages(a: &AgeStage, b: &AgeStage, t: f32) -> AgeStage {
    let t = t.clamp(0.0, 1.0);
    AgeStage {
        years: a.years + (b.years - a.years) * t,
        fat_pct: a.fat_pct + (b.fat_pct - a.fat_pct) * t,
        muscle_pct: a.muscle_pct + (b.muscle_pct - a.muscle_pct) * t,
        bone_density: a.bone_density + (b.bone_density - a.bone_density) * t,
        skin_elasticity: a.skin_elasticity + (b.skin_elasticity - a.skin_elasticity) * t,
    }
}

/// Interpolate the aging curve at a given age in years.
pub fn compute_age_stage(curve: &AgingCurve, years: f32) -> AgeStage {
    let stages = &curve.stages;
    if stages.is_empty() {
        return AgeStage {
            years,
            fat_pct: 0.25,
            muscle_pct: 0.40,
            bone_density: 1.0,
            skin_elasticity: 0.8,
        };
    }
    if years <= stages[0].years {
        return stages[0].clone();
    }
    let last = &stages[stages.len() - 1];
    if years >= last.years {
        return last.clone();
    }
    for i in 0..stages.len() - 1 {
        let a = &stages[i];
        let b = &stages[i + 1];
        if years >= a.years && years <= b.years {
            let t = (years - a.years) / (b.years - a.years);
            return interpolate_age_stages(a, b, t);
        }
    }
    stages[0].clone()
}

// ---------------------------------------------------------------------------
// Default curves
// ---------------------------------------------------------------------------

/// Male aging curve — 10 stages from age 10 to 90.
pub fn default_aging_curve_male() -> AgingCurve {
    AgingCurve {
        stages: vec![
            AgeStage {
                years: 10.0,
                fat_pct: 0.15,
                muscle_pct: 0.35,
                bone_density: 0.70,
                skin_elasticity: 0.98,
            },
            AgeStage {
                years: 20.0,
                fat_pct: 0.18,
                muscle_pct: 0.45,
                bone_density: 0.95,
                skin_elasticity: 0.95,
            },
            AgeStage {
                years: 30.0,
                fat_pct: 0.22,
                muscle_pct: 0.44,
                bone_density: 1.00,
                skin_elasticity: 0.90,
            },
            AgeStage {
                years: 40.0,
                fat_pct: 0.26,
                muscle_pct: 0.42,
                bone_density: 0.98,
                skin_elasticity: 0.82,
            },
            AgeStage {
                years: 50.0,
                fat_pct: 0.30,
                muscle_pct: 0.38,
                bone_density: 0.93,
                skin_elasticity: 0.72,
            },
            AgeStage {
                years: 60.0,
                fat_pct: 0.32,
                muscle_pct: 0.33,
                bone_density: 0.86,
                skin_elasticity: 0.60,
            },
            AgeStage {
                years: 70.0,
                fat_pct: 0.33,
                muscle_pct: 0.27,
                bone_density: 0.76,
                skin_elasticity: 0.48,
            },
            AgeStage {
                years: 75.0,
                fat_pct: 0.32,
                muscle_pct: 0.23,
                bone_density: 0.68,
                skin_elasticity: 0.40,
            },
            AgeStage {
                years: 80.0,
                fat_pct: 0.30,
                muscle_pct: 0.19,
                bone_density: 0.60,
                skin_elasticity: 0.32,
            },
            AgeStage {
                years: 90.0,
                fat_pct: 0.27,
                muscle_pct: 0.14,
                bone_density: 0.50,
                skin_elasticity: 0.22,
            },
        ],
    }
}

/// Female aging curve — 10 stages from age 10 to 90.
pub fn default_aging_curve_female() -> AgingCurve {
    AgingCurve {
        stages: vec![
            AgeStage {
                years: 10.0,
                fat_pct: 0.20,
                muscle_pct: 0.30,
                bone_density: 0.68,
                skin_elasticity: 0.98,
            },
            AgeStage {
                years: 20.0,
                fat_pct: 0.24,
                muscle_pct: 0.36,
                bone_density: 0.92,
                skin_elasticity: 0.96,
            },
            AgeStage {
                years: 30.0,
                fat_pct: 0.27,
                muscle_pct: 0.35,
                bone_density: 0.97,
                skin_elasticity: 0.91,
            },
            AgeStage {
                years: 40.0,
                fat_pct: 0.30,
                muscle_pct: 0.33,
                bone_density: 0.94,
                skin_elasticity: 0.82,
            },
            AgeStage {
                years: 50.0,
                fat_pct: 0.34,
                muscle_pct: 0.29,
                bone_density: 0.84,
                skin_elasticity: 0.68,
            },
            AgeStage {
                years: 60.0,
                fat_pct: 0.36,
                muscle_pct: 0.24,
                bone_density: 0.74,
                skin_elasticity: 0.54,
            },
            AgeStage {
                years: 70.0,
                fat_pct: 0.35,
                muscle_pct: 0.19,
                bone_density: 0.64,
                skin_elasticity: 0.42,
            },
            AgeStage {
                years: 75.0,
                fat_pct: 0.33,
                muscle_pct: 0.16,
                bone_density: 0.57,
                skin_elasticity: 0.35,
            },
            AgeStage {
                years: 80.0,
                fat_pct: 0.30,
                muscle_pct: 0.13,
                bone_density: 0.49,
                skin_elasticity: 0.27,
            },
            AgeStage {
                years: 90.0,
                fat_pct: 0.26,
                muscle_pct: 0.09,
                bone_density: 0.38,
                skin_elasticity: 0.18,
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Delta & param functions
// ---------------------------------------------------------------------------

/// Morph parameter changes between two age stages.
pub fn age_progression_deltas(from: &AgeStage, to: &AgeStage) -> Vec<(String, f32)> {
    vec![
        ("delta_fat_pct".into(), to.fat_pct - from.fat_pct),
        ("delta_muscle_pct".into(), to.muscle_pct - from.muscle_pct),
        (
            "delta_bone_density".into(),
            to.bone_density - from.bone_density,
        ),
        (
            "delta_skin_elasticity".into(),
            to.skin_elasticity - from.skin_elasticity,
        ),
    ]
}

/// Skin aging parameters: wrinkles, sagging, age spots.
pub fn skin_aging_params(years: f32, sex: u8) -> Vec<(String, f32)> {
    let age_factor = ((years - 20.0) / 70.0).clamp(0.0, 1.0);
    let sex_mul = if sex == 1 { 1.05f32 } else { 1.0f32 }; // females age skin slightly faster
    vec![
        ("wrinkle_forehead".into(), age_factor * 0.8 * sex_mul),
        ("wrinkle_eyes".into(), age_factor * 0.9 * sex_mul),
        ("wrinkle_mouth".into(), age_factor * 0.7 * sex_mul),
        ("sag_cheeks".into(), age_factor * 0.6),
        ("sag_jowl".into(), age_factor * 0.5),
        ("sag_neck".into(), age_factor * 0.55),
        ("age_spots".into(), (age_factor - 0.3).max(0.0) * 0.6),
        ("pore_size".into(), age_factor * 0.4),
    ]
}

/// Face aging parameters: brow, jaw, nose changes.
pub fn face_aging_params(years: f32) -> Vec<(String, f32)> {
    let af = ((years - 20.0) / 70.0).clamp(0.0, 1.0);
    vec![
        ("brow_droop".into(), af * 0.5),
        ("jaw_resorption".into(), af * 0.35),
        ("nose_tip_droop".into(), af * 0.3),
        ("ear_growth".into(), af * 0.2),
        ("lip_thinning".into(), af * 0.4),
        ("nasolabial_depth".into(), af * 0.6),
        ("eye_hollow".into(), af * 0.45),
    ]
}

/// Body aging parameters: posture, fat redistribution, muscle loss.
pub fn body_aging_params(years: f32, sex: u8) -> Vec<(String, f32)> {
    let af = ((years - 20.0) / 70.0).clamp(0.0, 1.0);
    let belly_mul = if sex == 0 { 1.2f32 } else { 1.0f32 }; // males more belly fat
    vec![
        ("posture_kyphosis".into(), af * 0.4),
        ("belly_fat".into(), af * 0.6 * belly_mul),
        ("muscle_loss_arms".into(), af * 0.5),
        ("muscle_loss_legs".into(), af * 0.45),
        ("height_loss".into(), af * 0.03), // up to ~3% height loss
        ("hip_fat".into(), af * 0.35),
        ("skin_looseness_body".into(), af * 0.5),
    ]
}

/// Simulate full aging from base_age to target_years.
pub fn simulate_aging(
    profile: &AgeProfile,
    target_years: f32,
    curve: &AgingCurve,
) -> Vec<(String, f32)> {
    let from = compute_age_stage(curve, profile.base_age);
    let to = compute_age_stage(curve, target_years);
    let mut params = age_progression_deltas(&from, &to);
    params.extend(skin_aging_params(target_years, profile.sex));
    params.extend(face_aging_params(target_years));
    params.extend(body_aging_params(target_years, profile.sex));
    params
}

/// Negate all aging deltas (de-aging / reverse aging).
pub fn reverse_aging(params: &[(String, f32)]) -> Vec<(String, f32)> {
    params.iter().map(|(k, v)| (k.clone(), -v)).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_midpoint() {
        let a = AgeStage {
            years: 20.0,
            fat_pct: 0.20,
            muscle_pct: 0.40,
            bone_density: 1.0,
            skin_elasticity: 0.95,
        };
        let b = AgeStage {
            years: 40.0,
            fat_pct: 0.30,
            muscle_pct: 0.35,
            bone_density: 0.90,
            skin_elasticity: 0.75,
        };
        let mid = interpolate_age_stages(&a, &b, 0.5);
        assert!((mid.fat_pct - 0.25).abs() < 1e-5);
        assert!((mid.muscle_pct - 0.375).abs() < 1e-5);
    }

    #[test]
    fn test_interpolate_clamp_t() {
        let a = AgeStage {
            years: 0.0,
            fat_pct: 0.1,
            muscle_pct: 0.5,
            bone_density: 1.0,
            skin_elasticity: 1.0,
        };
        let b = AgeStage {
            years: 100.0,
            fat_pct: 0.4,
            muscle_pct: 0.1,
            bone_density: 0.4,
            skin_elasticity: 0.2,
        };
        let clamped = interpolate_age_stages(&a, &b, 2.0);
        assert!((clamped.fat_pct - b.fat_pct).abs() < 1e-5);
    }

    #[test]
    fn test_compute_age_stage_clamps_low() {
        let curve = default_aging_curve_male();
        let stage = compute_age_stage(&curve, 5.0);
        assert!((stage.years - 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_compute_age_stage_clamps_high() {
        let curve = default_aging_curve_male();
        let stage = compute_age_stage(&curve, 100.0);
        assert!((stage.years - 90.0).abs() < 1e-3);
    }

    #[test]
    fn test_bone_density_decreases_with_age() {
        let curve = default_aging_curve_male();
        let young = compute_age_stage(&curve, 30.0);
        let old = compute_age_stage(&curve, 70.0);
        assert!(old.bone_density < young.bone_density);
    }

    #[test]
    fn test_skin_elasticity_decreases_with_age() {
        let curve = default_aging_curve_female();
        let young = compute_age_stage(&curve, 20.0);
        let old = compute_age_stage(&curve, 80.0);
        assert!(old.skin_elasticity < young.skin_elasticity);
    }

    #[test]
    fn test_male_vs_female_differ() {
        let male = default_aging_curve_male();
        let female = default_aging_curve_female();
        let m50 = compute_age_stage(&male, 50.0);
        let f50 = compute_age_stage(&female, 50.0);
        // females have higher fat_pct at 50
        assert!(f50.fat_pct > m50.fat_pct);
    }

    #[test]
    fn test_skin_aging_params_non_empty() {
        let params = skin_aging_params(60.0, 0);
        assert!(!params.is_empty());
    }

    #[test]
    fn test_skin_aging_params_young_all_near_zero() {
        let params = skin_aging_params(20.0, 0);
        for (_, v) in &params {
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn test_face_aging_params_non_empty() {
        let params = face_aging_params(50.0);
        assert!(!params.is_empty());
        for (_, v) in &params {
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn test_body_aging_params_non_empty() {
        let params = body_aging_params(65.0, 0);
        assert!(!params.is_empty());
    }

    #[test]
    fn test_simulate_aging_non_empty() {
        let curve = default_aging_curve_male();
        let profile = AgeProfile {
            base_age: 25.0,
            sex: 0,
            ethnicity: 0,
        };
        let params = simulate_aging(&profile, 65.0, &curve);
        assert!(!params.is_empty());
    }

    #[test]
    fn test_reverse_aging_negates() {
        let params = vec![
            ("wrinkle_forehead".into(), 0.5f32),
            ("sag_cheeks".into(), 0.3f32),
        ];
        let reversed = reverse_aging(&params);
        for (orig, rev) in params.iter().zip(reversed.iter()) {
            assert!((orig.1 + rev.1).abs() < 1e-6);
        }
    }

    #[test]
    fn test_age_progression_deltas_count() {
        let curve = default_aging_curve_male();
        let from = compute_age_stage(&curve, 30.0);
        let to = compute_age_stage(&curve, 70.0);
        let deltas = age_progression_deltas(&from, &to);
        assert_eq!(deltas.len(), 4);
    }

    #[test]
    fn test_default_curves_have_10_stages() {
        assert_eq!(default_aging_curve_male().stages.len(), 10);
        assert_eq!(default_aging_curve_female().stages.len(), 10);
    }

    #[test]
    fn test_simulate_aging_backward_gives_negatives() {
        let curve = default_aging_curve_male();
        let profile = AgeProfile {
            base_age: 60.0,
            sex: 0,
            ethnicity: 0,
        };
        // aging backward (target < base) — delta_fat_pct should be negative
        let params = simulate_aging(&profile, 30.0, &curve);
        let fat_delta = params.iter().find(|(k, _)| k == "delta_fat_pct").unwrap();
        assert!(fat_delta.1 < 0.0);
    }
}
