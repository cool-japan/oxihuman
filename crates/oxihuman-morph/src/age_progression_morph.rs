// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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

/// Evaluate morph weights for current age via piecewise-linear interpolation between AgeStage
/// entries sorted by `age_years`.
///
/// - If no stages: returns zeroed weights of length `morph_count`.
/// - If `current_age` <= first stage: returns first stage weights (clamped to `morph_count`).
/// - If `current_age` >= last stage: returns last stage weights (clamped to `morph_count`).
/// - Otherwise: linearly interpolates between the two bracketing stages.
pub fn apm_evaluate(apm: &AgeProgressionMorph) -> Vec<f32> {
    if apm.stages.is_empty() {
        return vec![0.0; apm.morph_count];
    }

    // Work on a sorted view by age_years without mutating the original.
    let mut sorted_indices: Vec<usize> = (0..apm.stages.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        apm.stages[a]
            .age_years
            .partial_cmp(&apm.stages[b].age_years)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let first = &apm.stages[sorted_indices[0]];
    let last = &apm.stages[*sorted_indices.last().unwrap_or(&0)];

    if apm.current_age <= first.age_years {
        let mc = apm.morph_count;
        let mut out = vec![0.0f32; mc];
        let copy_len = mc.min(first.morph_weights.len());
        out[..copy_len].copy_from_slice(&first.morph_weights[..copy_len]);
        return out;
    }

    if apm.current_age >= last.age_years {
        let mc = apm.morph_count;
        let mut out = vec![0.0f32; mc];
        let copy_len = mc.min(last.morph_weights.len());
        out[..copy_len].copy_from_slice(&last.morph_weights[..copy_len]);
        return out;
    }

    // Find bracketing interval [i, i+1] in sorted order.
    let mut bracket_lo = 0usize;
    for k in 0..sorted_indices.len().saturating_sub(1) {
        let age_lo = apm.stages[sorted_indices[k]].age_years;
        let age_hi = apm.stages[sorted_indices[k + 1]].age_years;
        if age_lo <= apm.current_age && apm.current_age < age_hi {
            bracket_lo = k;
            break;
        }
    }

    let stage_lo = &apm.stages[sorted_indices[bracket_lo]];
    let stage_hi = &apm.stages[sorted_indices[bracket_lo + 1]];

    let span = stage_hi.age_years - stage_lo.age_years;
    let t = if span.abs() < 1e-9 {
        0.0f32
    } else {
        (apm.current_age - stage_lo.age_years) / span
    };
    let t = t.clamp(0.0, 1.0);

    let mc = apm.morph_count;
    let mut out = vec![0.0f32; mc];
    for (j, o) in out.iter_mut().enumerate() {
        let w_lo = stage_lo.morph_weights.get(j).copied().unwrap_or(0.0);
        let w_hi = stage_hi.morph_weights.get(j).copied().unwrap_or(0.0);
        *o = (1.0 - t) * w_lo + t * w_hi;
    }
    out
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

    #[test]
    fn test_interpolation_midpoint() {
        // Two stages: age 20 with weights [0.0, 1.0], age 40 with weights [1.0, 0.0].
        // At age 30 (t=0.5) expected output is [0.5, 0.5].
        let mut a = new_age_progression_morph(2);
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 20.0,
                morph_weights: vec![0.0, 1.0],
                label: "young".into(),
            },
        );
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 40.0,
                morph_weights: vec![1.0, 0.0],
                label: "middle".into(),
            },
        );
        apm_set_age(&mut a, 30.0);
        let out = apm_evaluate(&a);
        assert_eq!(out.len(), 2, "output length must equal morph_count");
        assert!(
            (out[0] - 0.5).abs() < 1e-5,
            "interpolated weight[0] must be 0.5, got {}",
            out[0]
        );
        assert!(
            (out[1] - 0.5).abs() < 1e-5,
            "interpolated weight[1] must be 0.5, got {}",
            out[1]
        );
    }

    #[test]
    fn test_clamp_below_first_stage() {
        let mut a = new_age_progression_morph(2);
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 20.0,
                morph_weights: vec![0.3, 0.7],
                label: "young".into(),
            },
        );
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 40.0,
                morph_weights: vec![0.8, 0.2],
                label: "middle".into(),
            },
        );
        apm_set_age(&mut a, 5.0);
        let out = apm_evaluate(&a);
        assert!(
            (out[0] - 0.3).abs() < 1e-5,
            "below-range age must return first stage weights"
        );
        assert!(
            (out[1] - 0.7).abs() < 1e-5,
            "below-range age must return first stage weights"
        );
    }

    #[test]
    fn test_clamp_above_last_stage() {
        let mut a = new_age_progression_morph(2);
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 20.0,
                morph_weights: vec![0.3, 0.7],
                label: "young".into(),
            },
        );
        apm_add_stage(
            &mut a,
            AgeStage {
                age_years: 40.0,
                morph_weights: vec![0.8, 0.2],
                label: "middle".into(),
            },
        );
        apm_set_age(&mut a, 80.0);
        let out = apm_evaluate(&a);
        assert!(
            (out[0] - 0.8).abs() < 1e-5,
            "above-range age must return last stage weights"
        );
        assert!(
            (out[1] - 0.2).abs() < 1e-5,
            "above-range age must return last stage weights"
        );
    }
}
