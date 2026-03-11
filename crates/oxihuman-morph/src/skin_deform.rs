// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::params::ParamState;
use std::collections::HashMap;

/// Morph weight map: morph name → blend weight.
pub type MorphMap = HashMap<String, f32>;

// ---------------------------------------------------------------------------
// SkinDeformPattern
// ---------------------------------------------------------------------------

/// A skin deformation pattern: maps input parameters to morph weight contributions.
pub struct SkinDeformPattern {
    pub name: String,
    /// Body region, e.g. "forearm", "cheek", "belly".
    pub region: String,
    /// Which `ParamState` fields drive this pattern.
    pub driver_params: Vec<String>,
    /// Morph weights at zero deformation.
    pub base_weights: MorphMap,
    /// Morph weights at full deformation.
    pub max_weights: MorphMap,
}

impl SkinDeformPattern {
    /// Linearly interpolate between `base_weights` and `max_weights` by `t ∈ [0,1]`.
    pub fn evaluate(&self, t: f32) -> MorphMap {
        let t = t.clamp(0.0, 1.0);
        let mut out: MorphMap = self.base_weights.clone();
        for (k, v_max) in &self.max_weights {
            let v_base = self.base_weights.get(k).copied().unwrap_or(0.0);
            out.insert(k.clone(), v_base + (v_max - v_base) * t);
        }
        out
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn region(&self) -> &str {
        &self.region
    }
}

// ---------------------------------------------------------------------------
// SkinDeformSystem
// ---------------------------------------------------------------------------

/// Collection of `SkinDeformPattern`s with bulk evaluation.
pub struct SkinDeformSystem {
    patterns: Vec<SkinDeformPattern>,
}

impl SkinDeformSystem {
    pub fn new() -> Self {
        SkinDeformSystem {
            patterns: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, p: SkinDeformPattern) {
        self.patterns.push(p);
    }

    /// For each pattern, compute the average driver value from `params`,
    /// call `evaluate(t)`, and blend results additively (clamped to `[0,1]`).
    pub fn evaluate_all(&self, params: &ParamState) -> MorphMap {
        let mut out: MorphMap = HashMap::new();
        for pat in &self.patterns {
            let t = if pat.driver_params.is_empty() {
                0.0_f32
            } else {
                let sum: f32 = pat
                    .driver_params
                    .iter()
                    .map(|k| driver_value(params, k))
                    .sum();
                (sum / pat.driver_params.len() as f32).clamp(0.0, 1.0)
            };
            for (k, w) in pat.evaluate(t) {
                let entry = out.entry(k).or_insert(0.0);
                *entry = (*entry + w).clamp(0.0, 1.0);
            }
        }
        out
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn find_pattern(&self, name: &str) -> Option<&SkinDeformPattern> {
        self.patterns.iter().find(|p| p.name == name)
    }
}

impl Default for SkinDeformSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Driver extraction
// ---------------------------------------------------------------------------

fn driver_value(params: &ParamState, key: &str) -> f32 {
    match key {
        "muscle" => params.muscle,
        "weight" => params.weight,
        "age" => params.age,
        _ => params.extra.get(key).copied().unwrap_or(0.0),
    }
}

// ---------------------------------------------------------------------------
// Standalone helper functions
// ---------------------------------------------------------------------------

/// Generate wrinkle morph weights for a joint bend.
///
/// Produces entries like `"<morph_prefix>_inner"` and `"<morph_prefix>_outer"`,
/// with intensity proportional to `bend_angle / max_angle`.
pub fn wrinkle_weights(bend_angle: f32, max_angle: f32, morph_prefix: &str) -> MorphMap {
    let mut map = MorphMap::new();
    if max_angle <= 0.0 {
        return map;
    }
    let t = (bend_angle / max_angle).clamp(0.0, 1.0);
    // Smooth the wrinkle curve: more pronounced at higher bend angles.
    let inner = t * t;
    let outer = t * (1.0 - t * 0.5);
    map.insert(format!("{morph_prefix}_inner"), inner);
    map.insert(format!("{morph_prefix}_outer"), outer);
    map.insert(format!("{morph_prefix}_crease"), t);
    map
}

/// Generate muscle bulge morph weights for a given region and activation level.
///
/// `muscle_activation` ∈ [0, 1], `region` is a string like `"bicep"`, `"calf"`.
#[allow(clippy::too_many_arguments)]
pub fn bulge_weights(muscle_activation: f32, region: &str) -> MorphMap {
    let mut map = MorphMap::new();
    let t = muscle_activation.clamp(0.0, 1.0);
    // Peak bulge is strongest at ~0.75 activation.
    let bulge = (t * 1.333).clamp(0.0, 1.0);
    let stretch = t * 0.4;
    map.insert(format!("{region}_bulge"), bulge);
    map.insert(format!("{region}_stretch"), stretch);
    map.insert(format!("{region}_vein"), (t - 0.6).max(0.0) * 2.5);
    map
}

/// Generate fat/gravity sag morph weights.
///
/// `bmi` and `age` are normalised [0, 1] parameters.
/// `gravity_axis`: 0 = Y-down (standing), 1 = X (leaning), 2 = Z (prone).
pub fn sag_weights(bmi: f32, age: f32, gravity_axis: u8) -> MorphMap {
    let mut map = MorphMap::new();
    let b = bmi.clamp(0.0, 1.0);
    let a = age.clamp(0.0, 1.0);
    // Combined sag factor — fat mass × age-related skin laxity.
    let sag = b * 0.6 + a * 0.4;
    let axis_tag = match gravity_axis {
        0 => "down",
        1 => "side",
        _ => "prone",
    };
    map.insert(format!("belly_sag_{axis_tag}"), sag);
    map.insert(format!("chest_sag_{axis_tag}"), sag * 0.7);
    map.insert(
        format!("jowl_sag_{axis_tag}"),
        (a * 0.5 + b * 0.3).clamp(0.0, 1.0),
    );
    map.insert(format!("buttock_sag_{axis_tag}"), sag * 0.8);
    map
}

/// Blend two `MorphMap`s: `result[k] = lerp(a[k], b[k], t)`.
pub fn blend_skin_maps(a: &MorphMap, b: &MorphMap, t: f32) -> MorphMap {
    let t = t.clamp(0.0, 1.0);
    let mut out = MorphMap::new();
    // Keys from `a`
    for (k, va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        out.insert(k.clone(), va + (vb - va) * t);
    }
    // Keys only in `b`
    for (k, vb) in b {
        if !a.contains_key(k) {
            out.insert(k.clone(), vb * t);
        }
    }
    out
}

/// Clamp every weight in a `MorphMap` to `[lo, hi]`.
pub fn clamp_skin_map(map: &MorphMap, lo: f32, hi: f32) -> MorphMap {
    map.iter()
        .map(|(k, v)| (k.clone(), v.clamp(lo, hi)))
        .collect()
}

// ---------------------------------------------------------------------------
// Helpers for building patterns
// ---------------------------------------------------------------------------

fn make_pattern(
    name: &str,
    region: &str,
    drivers: &[&str],
    base: &[(&str, f32)],
    max: &[(&str, f32)],
) -> SkinDeformPattern {
    SkinDeformPattern {
        name: name.to_string(),
        region: region.to_string(),
        driver_params: drivers.iter().map(|s| s.to_string()).collect(),
        base_weights: base.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        max_weights: max.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    }
}

/// Build a `SkinDeformSystem` pre-loaded with 8 patterns covering the most
/// common skin-deformation scenarios.
pub fn default_skin_system() -> SkinDeformSystem {
    let mut sys = SkinDeformSystem::new();

    // 1. Elbow bend wrinkles
    sys.add_pattern(make_pattern(
        "elbow_bend",
        "forearm",
        &["elbow_flex"],
        &[("elbow_wrinkle_inner", 0.0), ("elbow_wrinkle_outer", 0.0)],
        &[("elbow_wrinkle_inner", 1.0), ("elbow_wrinkle_outer", 0.6)],
    ));

    // 2. Knee bend wrinkles
    sys.add_pattern(make_pattern(
        "knee_bend",
        "lower_leg",
        &["knee_flex"],
        &[("knee_wrinkle_inner", 0.0), ("knee_wrinkle_outer", 0.0)],
        &[("knee_wrinkle_inner", 1.0), ("knee_wrinkle_outer", 0.5)],
    ));

    // 3. Cheek squash (smile / puff)
    sys.add_pattern(make_pattern(
        "cheek_squash",
        "cheek",
        &["cheek_squash"],
        &[("cheek_bulge", 0.0), ("nasolabial_fold", 0.0)],
        &[("cheek_bulge", 0.9), ("nasolabial_fold", 0.7)],
    ));

    // 4. Belly sag (weight + age)
    sys.add_pattern(make_pattern(
        "belly_sag",
        "belly",
        &["weight", "age"],
        &[("belly_sag_down", 0.0), ("belly_overhang", 0.0)],
        &[("belly_sag_down", 1.0), ("belly_overhang", 0.8)],
    ));

    // 5. Bicep bulge (muscle activation)
    sys.add_pattern(make_pattern(
        "bicep_bulge",
        "upper_arm",
        &["muscle"],
        &[("bicep_bulge", 0.0), ("bicep_vein", 0.0)],
        &[("bicep_bulge", 1.0), ("bicep_vein", 0.6)],
    ));

    // 6. Shoulder wrinkle (arm raise)
    sys.add_pattern(make_pattern(
        "shoulder_wrinkle",
        "shoulder",
        &["shoulder_raise"],
        &[("shoulder_wrinkle_top", 0.0), ("deltoid_crease", 0.0)],
        &[("shoulder_wrinkle_top", 0.8), ("deltoid_crease", 0.5)],
    ));

    // 7. Neck wrinkle (age-driven)
    sys.add_pattern(make_pattern(
        "neck_wrinkle",
        "neck",
        &["age"],
        &[("neck_wrinkle_h", 0.0), ("neck_wrinkle_v", 0.0)],
        &[("neck_wrinkle_h", 0.9), ("neck_wrinkle_v", 0.5)],
    ));

    // 8. Brow compression (frown)
    sys.add_pattern(make_pattern(
        "brow_compression",
        "forehead",
        &["brow_compress"],
        &[("glabellar_crease", 0.0), ("brow_furrow", 0.0)],
        &[("glabellar_crease", 1.0), ("brow_furrow", 0.8)],
    ));

    sys
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_param(muscle: f32, weight: f32, age: f32) -> ParamState {
        ParamState::new(0.5, weight, muscle, age)
    }

    // --- SkinDeformPattern::evaluate ---

    #[test]
    fn evaluate_at_zero_returns_base() {
        let pat = make_pattern("test", "arm", &[], &[("a", 0.2)], &[("a", 0.8)]);
        let result = pat.evaluate(0.0);
        let v = *result.get("a").unwrap();
        assert!((v - 0.2).abs() < 1e-5, "expected 0.2, got {v}");
    }

    #[test]
    fn evaluate_at_one_returns_max() {
        let pat = make_pattern("test", "arm", &[], &[("a", 0.2)], &[("a", 0.8)]);
        let result = pat.evaluate(1.0);
        let v = *result.get("a").unwrap();
        assert!((v - 0.8).abs() < 1e-5, "expected 0.8, got {v}");
    }

    #[test]
    fn evaluate_at_half_is_midpoint() {
        let pat = make_pattern("test", "leg", &[], &[("x", 0.0)], &[("x", 1.0)]);
        let result = pat.evaluate(0.5);
        let v = *result.get("x").unwrap();
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn evaluate_clamps_t_above_one() {
        let pat = make_pattern("test", "leg", &[], &[("x", 0.0)], &[("x", 1.0)]);
        let result = pat.evaluate(2.0);
        let v = *result.get("x").unwrap();
        assert!((v - 1.0).abs() < 1e-5, "should clamp to 1.0");
    }

    #[test]
    fn evaluate_clamps_t_below_zero() {
        let pat = make_pattern("test", "leg", &[], &[("x", 0.0)], &[("x", 1.0)]);
        let result = pat.evaluate(-1.0);
        let v = *result.get("x").unwrap();
        assert!((v - 0.0).abs() < 1e-5, "should clamp to 0.0");
    }

    #[test]
    fn evaluate_includes_base_only_keys() {
        let pat = make_pattern("test", "face", &[], &[("base_only", 0.3)], &[]);
        let result = pat.evaluate(0.5);
        // base_only stays at 0.3 (no max entry → lerp(0.3, 0.3, 0.5))
        let v = *result.get("base_only").unwrap();
        assert!((v - 0.3).abs() < 1e-5);
    }

    // --- SkinDeformSystem ---

    #[test]
    fn system_starts_empty() {
        let sys = SkinDeformSystem::new();
        assert_eq!(sys.pattern_count(), 0);
    }

    #[test]
    fn add_pattern_increments_count() {
        let mut sys = SkinDeformSystem::new();
        let pat = make_pattern("p1", "arm", &[], &[], &[]);
        sys.add_pattern(pat);
        assert_eq!(sys.pattern_count(), 1);
    }

    #[test]
    fn find_pattern_returns_none_for_missing() {
        let sys = SkinDeformSystem::new();
        assert!(sys.find_pattern("ghost").is_none());
    }

    #[test]
    fn find_pattern_returns_correct_region() {
        let mut sys = SkinDeformSystem::new();
        sys.add_pattern(make_pattern("elbow", "forearm", &[], &[], &[]));
        let found = sys.find_pattern("elbow").unwrap();
        assert_eq!(found.region(), "forearm");
    }

    #[test]
    fn evaluate_all_zero_drivers_yields_base_weights() {
        let mut sys = SkinDeformSystem::new();
        sys.add_pattern(make_pattern(
            "neck",
            "neck",
            &["age"],
            &[("neck_crease", 0.1)],
            &[("neck_crease", 0.9)],
        ));
        let params = make_param(0.0, 0.0, 0.0); // age=0 → t=0
        let result = sys.evaluate_all(&params);
        let v = *result.get("neck_crease").unwrap();
        assert!((v - 0.1).abs() < 1e-5, "expected base=0.1, got {v}");
    }

    #[test]
    fn evaluate_all_full_muscle_gives_max_bicep() {
        let mut sys = SkinDeformSystem::new();
        sys.add_pattern(make_pattern(
            "bicep",
            "upper_arm",
            &["muscle"],
            &[("bicep_bulge", 0.0)],
            &[("bicep_bulge", 1.0)],
        ));
        let params = make_param(1.0, 0.5, 0.5);
        let result = sys.evaluate_all(&params);
        let v = *result.get("bicep_bulge").unwrap();
        assert!((v - 1.0).abs() < 1e-5);
    }

    // --- wrinkle_weights ---

    #[test]
    fn wrinkle_weights_zero_bend_is_zero() {
        let w = wrinkle_weights(0.0, 180.0, "elbow_wrinkle");
        let inner = *w.get("elbow_wrinkle_inner").unwrap();
        assert!(inner.abs() < 1e-6);
    }

    #[test]
    fn wrinkle_weights_full_bend_inner_is_one() {
        let w = wrinkle_weights(180.0, 180.0, "elbow_wrinkle");
        let inner = *w.get("elbow_wrinkle_inner").unwrap();
        assert!((inner - 1.0).abs() < 1e-5);
    }

    #[test]
    fn wrinkle_weights_max_angle_zero_returns_empty() {
        let w = wrinkle_weights(90.0, 0.0, "x");
        assert!(w.is_empty());
    }

    // --- bulge_weights ---

    #[test]
    fn bulge_weights_zero_activation() {
        let w = bulge_weights(0.0, "bicep");
        let b = *w.get("bicep_bulge").unwrap();
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn bulge_weights_full_activation_clamps_to_one() {
        let w = bulge_weights(1.0, "bicep");
        let b = *w.get("bicep_bulge").unwrap();
        assert!(b <= 1.0 + 1e-6);
    }

    // --- sag_weights ---

    #[test]
    fn sag_weights_zero_params_minimal_sag() {
        let w = sag_weights(0.0, 0.0, 0);
        let sag = *w.get("belly_sag_down").unwrap();
        assert!(sag.abs() < 1e-6);
    }

    #[test]
    fn sag_weights_keys_contain_axis_tag() {
        let w = sag_weights(0.5, 0.5, 1);
        assert!(w.contains_key("belly_sag_side"));
        assert!(w.contains_key("chest_sag_side"));
    }

    // --- blend_skin_maps ---

    #[test]
    fn blend_skin_maps_at_zero_returns_a() {
        let a: MorphMap = [("k".to_string(), 0.2)].into();
        let b: MorphMap = [("k".to_string(), 0.8)].into();
        let r = blend_skin_maps(&a, &b, 0.0);
        assert!((r["k"] - 0.2).abs() < 1e-5);
    }

    #[test]
    fn blend_skin_maps_at_one_returns_b() {
        let a: MorphMap = [("k".to_string(), 0.2)].into();
        let b: MorphMap = [("k".to_string(), 0.8)].into();
        let r = blend_skin_maps(&a, &b, 1.0);
        assert!((r["k"] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn blend_skin_maps_b_only_key_scales_by_t() {
        let a: MorphMap = HashMap::new();
        let b: MorphMap = [("only_b".to_string(), 1.0)].into();
        let r = blend_skin_maps(&a, &b, 0.5);
        assert!((r["only_b"] - 0.5).abs() < 1e-5);
    }

    // --- clamp_skin_map ---

    #[test]
    fn clamp_skin_map_clamps_above_hi() {
        let m: MorphMap = [("a".to_string(), 2.0)].into();
        let c = clamp_skin_map(&m, 0.0, 1.0);
        assert!((c["a"] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn clamp_skin_map_clamps_below_lo() {
        let m: MorphMap = [("a".to_string(), -1.0)].into();
        let c = clamp_skin_map(&m, 0.0, 1.0);
        assert!((c["a"] - 0.0).abs() < 1e-5);
    }

    // --- default_skin_system ---

    #[test]
    fn default_skin_system_has_eight_patterns() {
        let sys = default_skin_system();
        assert_eq!(sys.pattern_count(), 8);
    }

    #[test]
    fn default_skin_system_contains_belly_sag() {
        let sys = default_skin_system();
        assert!(sys.find_pattern("belly_sag").is_some());
    }

    #[test]
    fn default_skin_system_evaluate_all_no_panic() {
        let sys = default_skin_system();
        let params = ParamState::default();
        let result = sys.evaluate_all(&params);
        // All weights should be in [0,1]
        for v in result.values() {
            assert!(*v >= 0.0 && *v <= 1.0, "weight out of range: {v}");
        }
    }
}
