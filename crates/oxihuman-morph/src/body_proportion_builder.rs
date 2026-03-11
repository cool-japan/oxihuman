// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Body proportion morphs – height, weight, BMI, and limb ratios.
//!
//! Provides [`BodyProportions`] driven by a [`BodyProportionConfig`], with
//! helpers to estimate BMI, blend two proportion states, and map the result
//! to a flat morph-weight vector ready for the deformation engine.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Flat morph-weight vector produced by `proportions_to_morph_weights`.
pub type MorphWeightVec = Vec<f32>;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Body-build archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BodyBuild {
    /// Lean / linear frame.
    Ectomorph,
    /// Athletic / muscular frame.
    Mesomorph,
    /// Rounder / heavier frame.
    Endomorph,
}

impl BodyBuild {
    /// Return the display name for this build.
    pub fn name(self) -> &'static str {
        match self {
            BodyBuild::Ectomorph => "Ectomorph",
            BodyBuild::Mesomorph => "Mesomorph",
            BodyBuild::Endomorph => "Endomorph",
        }
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Configuration knobs for proportion morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyProportionConfig {
    /// Minimum allowed height scale (multiplier, e.g. 0.7 = 70 % of base).
    pub min_height: f32,
    /// Maximum allowed height scale.
    pub max_height: f32,
    /// Minimum allowed weight scale.
    pub min_weight: f32,
    /// Maximum allowed weight scale.
    pub max_weight: f32,
    /// Whether to mirror limb edits symmetrically.
    pub symmetric_limbs: bool,
}

/// A complete set of body-proportion morph values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyProportions {
    /// Overall height scale (1.0 = neutral).
    pub height_scale: f32,
    /// Overall weight / volume scale (1.0 = neutral).
    pub weight_scale: f32,
    /// Per-limb ratio overrides keyed by limb name.
    pub limb_ratios: HashMap<String, f32>,
    /// Shoulder width scale (1.0 = neutral).
    pub shoulder_width: f32,
    /// Hip width scale (1.0 = neutral).
    pub hip_width: f32,
}

// ---------------------------------------------------------------------------
// BodyProportionConfig helpers
// ---------------------------------------------------------------------------

/// Return a [`BodyProportionConfig`] with sensible defaults.
pub fn default_proportion_config() -> BodyProportionConfig {
    BodyProportionConfig {
        min_height: 0.5,
        max_height: 1.5,
        min_weight: 0.5,
        max_weight: 2.0,
        symmetric_limbs: true,
    }
}

// ---------------------------------------------------------------------------
// BodyProportions construction / mutation
// ---------------------------------------------------------------------------

/// Create a new [`BodyProportions`] at neutral (1.0) values.
pub fn new_body_proportions() -> BodyProportions {
    BodyProportions {
        height_scale: 1.0,
        weight_scale: 1.0,
        limb_ratios: HashMap::new(),
        shoulder_width: 1.0,
        hip_width: 1.0,
    }
}

/// Set the height scale, clamped to the config range.
pub fn set_height_scale(bp: &mut BodyProportions, cfg: &BodyProportionConfig, scale: f32) {
    bp.height_scale = scale.clamp(cfg.min_height, cfg.max_height);
}

/// Set the weight scale, clamped to the config range.
pub fn set_weight_scale(bp: &mut BodyProportions, cfg: &BodyProportionConfig, scale: f32) {
    bp.weight_scale = scale.clamp(cfg.min_weight, cfg.max_weight);
}

/// Set a named limb ratio (e.g. `"upper_arm"`, `"thigh"`).
///
/// Values are clamped to `[0.1, 3.0]`.
pub fn set_limb_ratio(bp: &mut BodyProportions, limb: &str, ratio: f32) {
    bp.limb_ratios
        .insert(limb.to_string(), ratio.clamp(0.1, 3.0));
}

/// Set the shoulder width scale, clamped to `[0.5, 2.0]`.
pub fn set_shoulder_width(bp: &mut BodyProportions, scale: f32) {
    bp.shoulder_width = scale.clamp(0.5, 2.0);
}

/// Set the hip width scale, clamped to `[0.5, 2.0]`.
pub fn set_hip_width(bp: &mut BodyProportions, scale: f32) {
    bp.hip_width = scale.clamp(0.5, 2.0);
}

/// Reset all proportion values to neutral (1.0).
pub fn reset_proportions(bp: &mut BodyProportions) {
    bp.height_scale = 1.0;
    bp.weight_scale = 1.0;
    bp.shoulder_width = 1.0;
    bp.hip_width = 1.0;
    bp.limb_ratios.clear();
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

/// Estimate BMI category given a height-scale and weight-scale.
///
/// Uses a simplified formula:
/// `bmi ≈ 22.0 * (weight_scale / height_scale²)`
pub fn bmi_estimate(height_scale: f32, weight_scale: f32) -> f32 {
    if height_scale <= 0.0 {
        return 0.0;
    }
    22.0 * (weight_scale / (height_scale * height_scale))
}

/// Infer a [`BodyBuild`] from height and weight scales.
pub fn body_build_from_params(height_scale: f32, weight_scale: f32) -> BodyBuild {
    let bmi = bmi_estimate(height_scale, weight_scale);
    if bmi < 20.0 {
        BodyBuild::Ectomorph
    } else if bmi < 27.0 {
        BodyBuild::Mesomorph
    } else {
        BodyBuild::Endomorph
    }
}

/// Linear blend between two proportion states.
///
/// `t = 0.0` → `a`, `t = 1.0` → `b`.
pub fn blend_proportions(a: &BodyProportions, b: &BodyProportions, t: f32) -> BodyProportions {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;

    // Merge limb ratios (union of keys)
    let mut limb_ratios = a.limb_ratios.clone();
    for (k, v_b) in &b.limb_ratios {
        let v_a = a.limb_ratios.get(k).copied().unwrap_or(1.0);
        limb_ratios.insert(k.clone(), lerp(v_a, *v_b));
    }

    BodyProportions {
        height_scale: lerp(a.height_scale, b.height_scale),
        weight_scale: lerp(a.weight_scale, b.weight_scale),
        shoulder_width: lerp(a.shoulder_width, b.shoulder_width),
        hip_width: lerp(a.hip_width, b.hip_width),
        limb_ratios,
    }
}

/// Convert proportion state to a flat morph-weight vector.
///
/// Order: `[height_delta, weight_delta, shoulder_delta, hip_delta,
///          …limb_deltas sorted by key]`
///
/// Each value is `scale - 1.0` so that 0.0 means "no change".
pub fn proportions_to_morph_weights(bp: &BodyProportions) -> MorphWeightVec {
    let mut out = vec![
        bp.height_scale - 1.0,
        bp.weight_scale - 1.0,
        bp.shoulder_width - 1.0,
        bp.hip_width - 1.0,
    ];
    let mut keys: Vec<&String> = bp.limb_ratios.keys().collect();
    keys.sort();
    for k in keys {
        out.push(bp.limb_ratios[k] - 1.0);
    }
    out
}

/// L2 norm of the morph-weight vector (magnitude of deviation from neutral).
pub fn proportion_vector_length(bp: &BodyProportions) -> f32 {
    let weights = proportions_to_morph_weights(bp);
    let sum_sq: f32 = weights.iter().map(|w| w * w).sum();
    sum_sq.sqrt()
}

/// Measure left/right symmetry of limb ratios.
///
/// Returns a value in `[0.0, 1.0]` where `1.0` is perfectly symmetric.
/// Pairs are matched by stripping `_l` / `_r` suffixes.
pub fn proportion_symmetry(bp: &BodyProportions) -> f32 {
    let mut pairs: Vec<(f32, f32)> = Vec::new();
    for (k, &v) in &bp.limb_ratios {
        if k.ends_with("_l") {
            let mirror = format!("{}_r", &k[..k.len() - 2]);
            if let Some(&vr) = bp.limb_ratios.get(&mirror) {
                pairs.push((v, vr));
            }
        }
    }
    if pairs.is_empty() {
        return 1.0;
    }
    let total_diff: f32 = pairs.iter().map(|(a, b)| (a - b).abs()).sum();
    let avg_diff = total_diff / pairs.len() as f32;
    (1.0 - avg_diff).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral() -> BodyProportions {
        new_body_proportions()
    }

    fn cfg() -> BodyProportionConfig {
        default_proportion_config()
    }

    // 1
    #[test]
    fn default_config_range_is_valid() {
        let c = cfg();
        assert!(c.min_height < c.max_height);
        assert!(c.min_weight < c.max_weight);
    }

    // 2
    #[test]
    fn new_proportions_are_neutral() {
        let bp = neutral();
        assert!((bp.height_scale - 1.0).abs() < 1e-6);
        assert!((bp.weight_scale - 1.0).abs() < 1e-6);
        assert!((bp.shoulder_width - 1.0).abs() < 1e-6);
        assert!((bp.hip_width - 1.0).abs() < 1e-6);
        assert!(bp.limb_ratios.is_empty());
    }

    // 3
    #[test]
    fn set_height_scale_clamps_to_config() {
        let mut bp = neutral();
        set_height_scale(&mut bp, &cfg(), 999.0);
        assert!((bp.height_scale - cfg().max_height).abs() < 1e-6);
        set_height_scale(&mut bp, &cfg(), -1.0);
        assert!((bp.height_scale - cfg().min_height).abs() < 1e-6);
    }

    // 4
    #[test]
    fn set_weight_scale_clamps_to_config() {
        let mut bp = neutral();
        set_weight_scale(&mut bp, &cfg(), 0.0);
        assert!((bp.weight_scale - cfg().min_weight).abs() < 1e-6);
    }

    // 5
    #[test]
    fn set_limb_ratio_stored_correctly() {
        let mut bp = neutral();
        set_limb_ratio(&mut bp, "upper_arm", 1.2);
        assert!((bp.limb_ratios["upper_arm"] - 1.2).abs() < 1e-6);
    }

    // 6
    #[test]
    fn set_limb_ratio_clamps_extremes() {
        let mut bp = neutral();
        set_limb_ratio(&mut bp, "leg", 100.0);
        assert!((bp.limb_ratios["leg"] - 3.0).abs() < 1e-6);
        set_limb_ratio(&mut bp, "leg", 0.0);
        assert!((bp.limb_ratios["leg"] - 0.1).abs() < 1e-6);
    }

    // 7
    #[test]
    fn set_shoulder_and_hip_width() {
        let mut bp = neutral();
        set_shoulder_width(&mut bp, 1.5);
        set_hip_width(&mut bp, 0.9);
        assert!((bp.shoulder_width - 1.5).abs() < 1e-6);
        assert!((bp.hip_width - 0.9).abs() < 1e-6);
    }

    // 8
    #[test]
    fn reset_proportions_restores_neutral() {
        let mut bp = neutral();
        set_height_scale(&mut bp, &cfg(), 1.3);
        set_limb_ratio(&mut bp, "arm", 1.1);
        reset_proportions(&mut bp);
        assert!((bp.height_scale - 1.0).abs() < 1e-6);
        assert!(bp.limb_ratios.is_empty());
    }

    // 9
    #[test]
    fn bmi_estimate_neutral_is_22() {
        let bmi = bmi_estimate(1.0, 1.0);
        assert!((bmi - 22.0).abs() < 1e-5);
    }

    // 10
    #[test]
    fn bmi_estimate_zero_height_is_zero() {
        assert!((bmi_estimate(0.0, 1.0) - 0.0).abs() < 1e-6);
    }

    // 11
    #[test]
    fn body_build_ectomorph_low_weight() {
        // bmi ≈ 22*(0.6/1.0) = 13.2 → Ectomorph
        assert_eq!(body_build_from_params(1.0, 0.6), BodyBuild::Ectomorph);
    }

    // 12
    #[test]
    fn body_build_mesomorph_neutral() {
        assert_eq!(body_build_from_params(1.0, 1.0), BodyBuild::Mesomorph);
    }

    // 13
    #[test]
    fn body_build_endomorph_high_weight() {
        // bmi ≈ 22*(1.5/1.0) = 33 → Endomorph
        assert_eq!(body_build_from_params(1.0, 1.5), BodyBuild::Endomorph);
    }

    // 14
    #[test]
    fn blend_proportions_at_zero_is_a() {
        let mut a = neutral();
        set_height_scale(&mut a, &cfg(), 1.2);
        let b = neutral();
        let blended = blend_proportions(&a, &b, 0.0);
        assert!((blended.height_scale - 1.2).abs() < 1e-5);
    }

    // 15
    #[test]
    fn blend_proportions_at_one_is_b() {
        let a = neutral();
        let mut b = neutral();
        set_height_scale(&mut b, &cfg(), 0.8);
        let blended = blend_proportions(&a, &b, 1.0);
        assert!((blended.height_scale - 0.8).abs() < 1e-5);
    }

    // 16
    #[test]
    fn blend_proportions_midpoint() {
        let mut a = neutral();
        let mut b = neutral();
        set_height_scale(&mut a, &cfg(), 1.0);
        set_height_scale(&mut b, &cfg(), 1.2);
        let blended = blend_proportions(&a, &b, 0.5);
        assert!((blended.height_scale - 1.1).abs() < 1e-5);
    }

    // 17
    #[test]
    fn proportions_to_morph_weights_neutral_is_zeros() {
        let bp = neutral();
        let w = proportions_to_morph_weights(&bp);
        assert_eq!(w.len(), 4);
        for v in &w {
            assert!(v.abs() < 1e-6);
        }
    }

    // 18
    #[test]
    fn proportions_to_morph_weights_includes_limbs() {
        let mut bp = neutral();
        set_limb_ratio(&mut bp, "arm", 1.2);
        let w = proportions_to_morph_weights(&bp);
        // 4 base + 1 limb
        assert_eq!(w.len(), 5);
        assert!((w[4] - 0.2).abs() < 1e-5);
    }

    // 19
    #[test]
    fn proportion_vector_length_neutral_is_zero() {
        let bp = neutral();
        assert!(proportion_vector_length(&bp) < 1e-6);
    }

    // 20
    #[test]
    fn proportion_symmetry_no_limb_pairs_is_one() {
        let bp = neutral();
        assert!((proportion_symmetry(&bp) - 1.0).abs() < 1e-6);
    }

    // 21
    #[test]
    fn proportion_symmetry_symmetric_pair_is_one() {
        let mut bp = neutral();
        set_limb_ratio(&mut bp, "arm_l", 1.2);
        set_limb_ratio(&mut bp, "arm_r", 1.2);
        assert!((proportion_symmetry(&bp) - 1.0).abs() < 1e-6);
    }

    // 22
    #[test]
    fn proportion_symmetry_asymmetric_pair_less_than_one() {
        let mut bp = neutral();
        set_limb_ratio(&mut bp, "arm_l", 1.0);
        set_limb_ratio(&mut bp, "arm_r", 1.5);
        let sym = proportion_symmetry(&bp);
        assert!(sym < 1.0);
    }

    // 23
    #[test]
    fn body_build_name_strings() {
        assert_eq!(BodyBuild::Ectomorph.name(), "Ectomorph");
        assert_eq!(BodyBuild::Mesomorph.name(), "Mesomorph");
        assert_eq!(BodyBuild::Endomorph.name(), "Endomorph");
    }

    // 24
    #[test]
    fn shoulder_width_clamped_at_boundaries() {
        let mut bp = neutral();
        set_shoulder_width(&mut bp, 10.0);
        assert!((bp.shoulder_width - 2.0).abs() < 1e-6);
        set_shoulder_width(&mut bp, 0.1);
        assert!((bp.shoulder_width - 0.5).abs() < 1e-6);
    }

    // 25
    #[test]
    fn hip_width_clamped_at_boundaries() {
        let mut bp = neutral();
        set_hip_width(&mut bp, 0.0);
        assert!((bp.hip_width - 0.5).abs() < 1e-6);
    }
}
