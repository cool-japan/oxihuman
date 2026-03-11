// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Anthropometric constraint enforcement for realistic body proportions.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// A single named anthropometric ratio constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnthroConstraint {
    pub name: String,
    pub description: String,
    pub min_ratio: f32,
    pub max_ratio: f32,
}

/// A set of anthropometric constraints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnthroConstraintSet {
    pub constraints: Vec<AnthroConstraint>,
}

/// A violation of a single constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnthroViolation {
    pub constraint_name: String,
    pub actual_ratio: f32,
    pub min_ratio: f32,
    pub max_ratio: f32,
    /// 0..1 – how far outside the bounds (0 = just at boundary, 1 = one full range-width outside).
    pub severity: f32,
}

/// Full result of checking a parameter set against a constraint set.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnthroCheckResult {
    pub violations: Vec<AnthroViolation>,
    pub is_realistic: bool,
    pub realism_score: f32,
}

// ---------------------------------------------------------------------------
// Standard constraint set
// ---------------------------------------------------------------------------

/// Return a set of ≥8 realistic anthropometric constraints.
#[allow(dead_code)]
pub fn standard_anthropometric_constraints() -> AnthroConstraintSet {
    let constraints = vec![
        AnthroConstraint {
            name: "head_height_to_body".into(),
            description: "Head height as fraction of total body height (1/6 to 1/8)".into(),
            min_ratio: 0.11,
            max_ratio: 0.17,
        },
        AnthroConstraint {
            name: "shoulder_to_hip_width".into(),
            description: "Shoulder width relative to hip width".into(),
            min_ratio: 0.8,
            max_ratio: 1.5,
        },
        AnthroConstraint {
            name: "arm_span_to_height".into(),
            description: "Arm span approximately equal to height".into(),
            min_ratio: 0.95,
            max_ratio: 1.05,
        },
        AnthroConstraint {
            name: "leg_to_torso".into(),
            description: "Leg length relative to torso length".into(),
            min_ratio: 0.9,
            max_ratio: 1.2,
        },
        AnthroConstraint {
            name: "bmi_realistic".into(),
            description: "Body mass index realistic range".into(),
            min_ratio: 15.0,
            max_ratio: 45.0,
        },
        AnthroConstraint {
            name: "foot_to_height".into(),
            description: "Foot length as fraction of total height".into(),
            min_ratio: 0.13,
            max_ratio: 0.17,
        },
        AnthroConstraint {
            name: "hand_to_forearm".into(),
            description: "Hand length relative to forearm length".into(),
            min_ratio: 0.6,
            max_ratio: 0.8,
        },
        AnthroConstraint {
            name: "neck_to_head".into(),
            description: "Neck circumference relative to head circumference".into(),
            min_ratio: 0.3,
            max_ratio: 0.5,
        },
    ];
    AnthroConstraintSet { constraints }
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Compute BMI from height (metres) and weight (kg).
#[allow(dead_code)]
pub fn bmi_from_params(height_m: f32, weight_kg: f32) -> f32 {
    if height_m <= 0.0 {
        return 0.0;
    }
    weight_kg / (height_m * height_m)
}

/// Compute severity: 0 if within [min, max], otherwise normalised overshoot.
#[allow(dead_code)]
pub fn violation_severity(actual: f32, min: f32, max: f32) -> f32 {
    let range = (max - min).max(f32::EPSILON);
    if actual < min {
        ((min - actual) / range).clamp(0.0, 1.0)
    } else if actual > max {
        ((actual - max) / range).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// 1.0 − mean(severity), clamped to 0..1.
#[allow(dead_code)]
pub fn realism_score(violations: &[AnthroViolation]) -> f32 {
    if violations.is_empty() {
        return 1.0;
    }
    let mean_sev = violations.iter().map(|v| v.severity).sum::<f32>() / violations.len() as f32;
    (1.0 - mean_sev).clamp(0.0, 1.0)
}

/// Compute all body ratios that can be derived from the named params.
///
/// Expected param names (all in consistent SI / normalised units):
/// `height`, `weight`, `shoulder_width`, `hip_width`, `head_height`,
/// `arm_span`, `leg_length`, `torso_length`, `foot_length`,
/// `hand_length`, `forearm_length`, `neck_circ`, `head_circ`.
#[allow(dead_code)]
pub fn params_to_body_ratios(params: &HashMap<String, f32>) -> HashMap<String, f32> {
    let get = |k: &str| -> Option<f32> { params.get(k).copied().filter(|&v| v > 0.0) };

    let mut ratios = HashMap::new();

    if let (Some(head_h), Some(height)) = (get("head_height"), get("height")) {
        ratios.insert("head_height_to_body".into(), head_h / height);
    }
    if let (Some(sw), Some(hw)) = (get("shoulder_width"), get("hip_width")) {
        ratios.insert("shoulder_to_hip_width".into(), sw / hw);
    }
    if let (Some(span), Some(height)) = (get("arm_span"), get("height")) {
        ratios.insert("arm_span_to_height".into(), span / height);
    }
    if let (Some(leg), Some(torso)) = (get("leg_length"), get("torso_length")) {
        ratios.insert("leg_to_torso".into(), leg / torso);
    }
    if let (Some(h), Some(w)) = (get("height"), get("weight")) {
        ratios.insert("bmi_realistic".into(), bmi_from_params(h, w));
    }
    if let (Some(foot), Some(height)) = (get("foot_length"), get("height")) {
        ratios.insert("foot_to_height".into(), foot / height);
    }
    if let (Some(hand), Some(fore)) = (get("hand_length"), get("forearm_length")) {
        ratios.insert("hand_to_forearm".into(), hand / fore);
    }
    if let (Some(neck), Some(head_c)) = (get("neck_circ"), get("head_circ")) {
        ratios.insert("neck_to_head".into(), neck / head_c);
    }

    ratios
}

/// Check body ratios derived from `params` against every constraint.
#[allow(dead_code)]
pub fn check_params_against_constraints(
    params: &HashMap<String, f32>,
    constraints: &AnthroConstraintSet,
) -> AnthroCheckResult {
    let ratios = params_to_body_ratios(params);
    let mut violations = Vec::new();

    for c in &constraints.constraints {
        let actual = match ratios.get(&c.name) {
            Some(&v) => v,
            None => continue,
        };
        let sev = violation_severity(actual, c.min_ratio, c.max_ratio);
        if sev > 0.0 {
            violations.push(AnthroViolation {
                constraint_name: c.name.clone(),
                actual_ratio: actual,
                min_ratio: c.min_ratio,
                max_ratio: c.max_ratio,
                severity: sev,
            });
        }
    }

    let score = realism_score(&violations);
    AnthroCheckResult {
        is_realistic: violations.is_empty(),
        violations,
        realism_score: score,
    }
}

/// Clamp params to satisfy constraints; returns count of params clamped.
///
/// For ratio-based constraints the function adjusts the numerator param to
/// bring the ratio within bounds (if both numerator and denominator exist).
#[allow(dead_code)]
pub fn enforce_constraints(
    params: &mut HashMap<String, f32>,
    constraints: &AnthroConstraintSet,
) -> usize {
    let mut clamped = 0usize;

    for c in &constraints.constraints {
        match c.name.as_str() {
            "bmi_realistic" => {
                let height = params.get("height").copied().unwrap_or(0.0);
                if height <= 0.0 {
                    continue;
                }
                if let Some(weight) = params.get_mut("weight") {
                    let bmi = *weight / (height * height);
                    if bmi < c.min_ratio {
                        *weight = c.min_ratio * height * height;
                        clamped += 1;
                    } else if bmi > c.max_ratio {
                        *weight = c.max_ratio * height * height;
                        clamped += 1;
                    }
                }
            }
            "head_height_to_body" => {
                clamp_ratio_numerator(params, "head_height", "height", c, &mut clamped);
            }
            "shoulder_to_hip_width" => {
                clamp_ratio_numerator(params, "shoulder_width", "hip_width", c, &mut clamped);
            }
            "arm_span_to_height" => {
                clamp_ratio_numerator(params, "arm_span", "height", c, &mut clamped);
            }
            "leg_to_torso" => {
                clamp_ratio_numerator(params, "leg_length", "torso_length", c, &mut clamped);
            }
            "foot_to_height" => {
                clamp_ratio_numerator(params, "foot_length", "height", c, &mut clamped);
            }
            "hand_to_forearm" => {
                clamp_ratio_numerator(params, "hand_length", "forearm_length", c, &mut clamped);
            }
            "neck_to_head" => {
                clamp_ratio_numerator(params, "neck_circ", "head_circ", c, &mut clamped);
            }
            _ => {}
        }
    }

    clamped
}

// Helper: clamp `numerator` so that numerator/denominator ∈ [min, max].
fn clamp_ratio_numerator(
    params: &mut HashMap<String, f32>,
    num_key: &str,
    den_key: &str,
    c: &AnthroConstraint,
    clamped: &mut usize,
) {
    let denom = params.get(den_key).copied().unwrap_or(0.0);
    if denom <= 0.0 {
        return;
    }
    if let Some(num) = params.get_mut(num_key) {
        let ratio = *num / denom;
        if ratio < c.min_ratio {
            *num = c.min_ratio * denom;
            *clamped += 1;
        } else if ratio > c.max_ratio {
            *num = c.max_ratio * denom;
            *clamped += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn typical_human() -> HashMap<String, f32> {
        let mut m = HashMap::new();
        m.insert("height".into(), 1.75);
        m.insert("weight".into(), 70.0);
        m.insert("head_height".into(), 0.23); // ~0.131 of 1.75
        m.insert("shoulder_width".into(), 0.46);
        m.insert("hip_width".into(), 0.38);
        m.insert("arm_span".into(), 1.76);
        m.insert("leg_length".into(), 0.95);
        m.insert("torso_length".into(), 0.85);
        m.insert("foot_length".into(), 0.26); // ~0.149 of 1.75
        m.insert("hand_length".into(), 0.14);
        m.insert("forearm_length".into(), 0.20);
        m.insert("neck_circ".into(), 0.13);
        m.insert("head_circ".into(), 0.38);
        m
    }

    #[test]
    fn test_bmi_from_params_normal() {
        let bmi = bmi_from_params(1.75, 70.0);
        assert!((bmi - 22.857).abs() < 0.01, "bmi={bmi}");
    }

    #[test]
    fn test_bmi_from_params_zero_height() {
        assert_eq!(bmi_from_params(0.0, 70.0), 0.0);
    }

    #[test]
    fn test_realism_score_no_violations() {
        let score = realism_score(&[]);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_realism_score_all_severe() {
        let vs = vec![
            AnthroViolation {
                constraint_name: "a".into(),
                actual_ratio: 0.0,
                min_ratio: 0.0,
                max_ratio: 0.0,
                severity: 1.0,
            },
            AnthroViolation {
                constraint_name: "b".into(),
                actual_ratio: 0.0,
                min_ratio: 0.0,
                max_ratio: 0.0,
                severity: 1.0,
            },
        ];
        assert_eq!(realism_score(&vs), 0.0);
    }

    #[test]
    fn test_violation_severity_in_bounds() {
        assert_eq!(violation_severity(0.5, 0.3, 0.7), 0.0);
    }

    #[test]
    fn test_violation_severity_below_min() {
        let sev = violation_severity(0.1, 0.3, 0.7);
        assert!(sev > 0.0);
        assert!(sev <= 1.0);
    }

    #[test]
    fn test_violation_severity_above_max() {
        let sev = violation_severity(0.9, 0.3, 0.7);
        assert!(sev > 0.0);
        assert!(sev <= 1.0);
    }

    #[test]
    fn test_violation_severity_at_min_boundary() {
        assert_eq!(violation_severity(0.3, 0.3, 0.7), 0.0);
    }

    #[test]
    fn test_violation_severity_at_max_boundary() {
        assert_eq!(violation_severity(0.7, 0.3, 0.7), 0.0);
    }

    #[test]
    fn test_standard_constraints_at_least_8() {
        let cs = standard_anthropometric_constraints();
        assert!(cs.constraints.len() >= 8, "len={}", cs.constraints.len());
    }

    #[test]
    fn test_check_valid_human_no_violations() {
        let params = typical_human();
        let cs = standard_anthropometric_constraints();
        let result = check_params_against_constraints(&params, &cs);
        assert!(
            result.is_realistic,
            "Expected no violations, got: {:?}",
            result.violations
        );
        assert!(result.realism_score > 0.9);
    }

    #[test]
    fn test_check_extreme_params_have_violations() {
        let mut params = HashMap::new();
        params.insert("height".into(), 1.0);
        params.insert("weight".into(), 200.0); // BMI = 200, way out of range
        params.insert("head_height".into(), 0.5); // head_height_to_body = 0.5, >0.17
        params.insert("hip_width".into(), 0.3);
        params.insert("shoulder_width".into(), 0.06); // shoulder/hip = 0.2, <0.8
        let cs = standard_anthropometric_constraints();
        let result = check_params_against_constraints(&params, &cs);
        assert!(!result.is_realistic);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_enforce_constraints_clamps_bmi() {
        let mut params = HashMap::new();
        params.insert("height".into(), 1.75);
        params.insert("weight".into(), 300.0); // BMI ~98
        let cs = standard_anthropometric_constraints();
        let count = enforce_constraints(&mut params, &cs);
        assert!(count >= 1);
        let bmi = bmi_from_params(1.75, *params.get("weight").unwrap());
        assert!(bmi <= 45.0 + 0.001);
    }

    #[test]
    fn test_params_to_body_ratios_returns_map() {
        let params = typical_human();
        let ratios = params_to_body_ratios(&params);
        assert!(!ratios.is_empty());
        assert!(ratios.contains_key("bmi_realistic"));
        assert!(ratios.contains_key("head_height_to_body"));
    }

    #[test]
    fn test_params_to_body_ratios_empty_params() {
        let params = HashMap::new();
        let ratios = params_to_body_ratios(&params);
        assert!(ratios.is_empty());
    }

    #[test]
    fn test_realism_score_partial() {
        let vs = vec![AnthroViolation {
            constraint_name: "x".into(),
            actual_ratio: 0.0,
            min_ratio: 0.0,
            max_ratio: 0.0,
            severity: 0.5,
        }];
        let s = realism_score(&vs);
        assert!((s - 0.5).abs() < 1e-5);
    }
}
