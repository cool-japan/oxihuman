// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Genetic body parameter inheritance and trait blending for OxiHuman.
//!
//! Provides Mendelian-style discrete inheritance, continuous blending,
//! crossover masking, and population-level diversity scoring.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A set of body parameters representing one parent's genetic contribution.
#[derive(Debug, Clone)]
pub struct GeneticParams {
    pub height: f32,
    pub weight: f32,
    pub muscle: f32,
    pub age: f32,
    /// Arbitrary named extra parameters (e.g. "nose_width", "jaw_size").
    pub extra: HashMap<String, f32>,
}

impl GeneticParams {
    /// Create a zeroed `GeneticParams`.
    pub fn new() -> Self {
        Self {
            height: 0.0,
            weight: 0.0,
            muscle: 0.0,
            age: 0.0,
            extra: HashMap::new(),
        }
    }
}

impl Default for GeneticParams {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------

/// Named individual defined by two parents and dominance / seed settings.
#[derive(Debug, Clone)]
pub struct GeneticProfile {
    pub name: String,
    pub parent_a: GeneticParams,
    pub parent_b: GeneticParams,
    /// Blend weight for parent A (0.0 = all B, 1.0 = all A). Default 0.5.
    pub dominance: f32,
    /// Optional random seed for stochastic trait variation.
    pub seed: Option<u32>,
}

impl GeneticProfile {
    /// Construct a new profile with equal dominance and no seed.
    pub fn new(name: impl Into<String>, parent_a: GeneticParams, parent_b: GeneticParams) -> Self {
        Self {
            name: name.into(),
            parent_a,
            parent_b,
            dominance: 0.5,
            seed: None,
        }
    }
}

// ---------------------------------------------------------------------------

/// A collection of [`GeneticProfile`] instances representing a population.
#[derive(Debug, Clone, Default)]
pub struct GeneticPopulation {
    pub profiles: Vec<GeneticProfile>,
}

impl GeneticPopulation {
    /// Create an empty population.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    /// Add a profile to the population.
    pub fn add(&mut self, profile: GeneticProfile) {
        self.profiles.push(profile);
    }

    /// Number of profiles in the population.
    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    /// Compute the dominant blend for every profile and return the results.
    pub fn blend_all(&self) -> Vec<GeneticParams> {
        self.profiles.iter().map(dominant_blend).collect()
    }

    /// Mean pairwise L2 distance of all blended results.
    ///
    /// Returns `0.0` if the population has fewer than two members.
    pub fn diversity_score(&self) -> f32 {
        let blended = self.blend_all();
        let n = blended.len();
        if n < 2 {
            return 0.0;
        }
        let mut total = 0.0_f32;
        let mut count = 0u32;
        for i in 0..n {
            for j in (i + 1)..n {
                total += params_distance(&blended[i], &blended[j]);
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            total / count as f32
        }
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Simple Linear Congruential Generator producing values in `[0, 1)`.
///
/// Parameters: multiplier 1664525, increment 1013904223 (Numerical Recipes).
pub fn lcg_f32(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    // Use the upper 23 bits for the mantissa of a float in [0, 1).
    (*seed >> 9) as f32 / (1u32 << 23) as f32
}

/// L2 distance over the four core fields (height, weight, muscle, age).
pub fn params_distance(a: &GeneticParams, b: &GeneticParams) -> f32 {
    let dh = a.height - b.height;
    let dw = a.weight - b.weight;
    let dm = a.muscle - b.muscle;
    let da = a.age - b.age;
    (dh * dh + dw * dw + dm * dm + da * da).sqrt()
}

/// Clamp all core fields and every extra value to `[0, 1]`.
pub fn clamp_params(p: &mut GeneticParams) {
    p.height = p.height.clamp(0.0, 1.0);
    p.weight = p.weight.clamp(0.0, 1.0);
    p.muscle = p.muscle.clamp(0.0, 1.0);
    p.age = p.age.clamp(0.0, 1.0);
    for v in p.extra.values_mut() {
        *v = v.clamp(0.0, 1.0);
    }
}

/// Arithmetic mean of a slice of params.
///
/// Returns `None` if the slice is empty.
pub fn average_params(params: &[GeneticParams]) -> Option<GeneticParams> {
    if params.is_empty() {
        return None;
    }
    let n = params.len() as f32;
    let mut acc = GeneticParams::new();
    for p in params {
        acc.height += p.height;
        acc.weight += p.weight;
        acc.muscle += p.muscle;
        acc.age += p.age;
        for (k, v) in &p.extra {
            *acc.extra.entry(k.clone()).or_insert(0.0) += v;
        }
    }
    acc.height /= n;
    acc.weight /= n;
    acc.muscle /= n;
    acc.age /= n;
    for v in acc.extra.values_mut() {
        *v /= n;
    }
    Some(acc)
}

// ---------------------------------------------------------------------------
// Core blending functions
// ---------------------------------------------------------------------------

/// Linear interpolation between two param sets.
///
/// `t = 0.0` returns B; `t = 1.0` returns A.  
/// Extra keys present in both parents are blended; keys present in only one
/// parent are carried over unchanged at the appropriate weight boundary.
pub fn blend_params(a: &GeneticParams, b: &GeneticParams, t: f32) -> GeneticParams {
    let lerp = |va: f32, vb: f32| va * t + vb * (1.0 - t);

    let mut extra: HashMap<String, f32> = HashMap::new();

    // Keys from A
    for (k, va) in &a.extra {
        let vb = b.extra.get(k).copied().unwrap_or(0.0);
        extra.insert(k.clone(), lerp(*va, vb));
    }
    // Keys only in B
    for (k, vb) in &b.extra {
        if !a.extra.contains_key(k) {
            extra.insert(k.clone(), lerp(0.0, *vb));
        }
    }

    GeneticParams {
        height: lerp(a.height, b.height),
        weight: lerp(a.weight, b.weight),
        muscle: lerp(a.muscle, b.muscle),
        age: lerp(a.age, b.age),
        extra,
    }
}

/// Blend using the profile's `dominance` weight, with optional noise.
///
/// When `profile.seed` is `Some(s)`, ±2.5 % noise (up to ±5 % range) is
/// added to each of the four core fields, and the result is clamped to
/// `[0, 1]`.
pub fn dominant_blend(profile: &GeneticProfile) -> GeneticParams {
    let mut result = blend_params(&profile.parent_a, &profile.parent_b, profile.dominance);

    if let Some(s) = profile.seed {
        let mut s_local = s;
        let noise_scale = 0.05_f32;
        result.height += (lcg_f32(&mut s_local) - 0.5) * noise_scale;
        result.weight += (lcg_f32(&mut s_local) - 0.5) * noise_scale;
        result.muscle += (lcg_f32(&mut s_local) - 0.5) * noise_scale;
        result.age += (lcg_f32(&mut s_local) - 0.5) * noise_scale;
        clamp_params(&mut result);
    }

    result
}

/// Discrete Mendelian inheritance: for each field, flip an LCG coin and pick
/// either parent A's or parent B's value.
pub fn inherit_random(profile: &GeneticProfile, seed: u32) -> GeneticParams {
    let mut s = seed;

    let pick = |va: f32, vb: f32, s: &mut u32| -> f32 {
        if lcg_f32(s) >= 0.5 {
            va
        } else {
            vb
        }
    };

    let height = pick(profile.parent_a.height, profile.parent_b.height, &mut s);
    let weight = pick(profile.parent_a.weight, profile.parent_b.weight, &mut s);
    let muscle = pick(profile.parent_a.muscle, profile.parent_b.muscle, &mut s);
    let age = pick(profile.parent_a.age, profile.parent_b.age, &mut s);

    // For extra keys: union of both parents; coin flip per key.
    let mut extra: HashMap<String, f32> = HashMap::new();
    let mut all_keys: Vec<String> = profile.parent_a.extra.keys().cloned().collect();
    for k in profile.parent_b.extra.keys() {
        if !profile.parent_a.extra.contains_key(k) {
            all_keys.push(k.clone());
        }
    }
    for k in all_keys {
        let va = profile.parent_a.extra.get(&k).copied().unwrap_or(0.0);
        let vb = profile.parent_b.extra.get(&k).copied().unwrap_or(0.0);
        extra.insert(k, pick(va, vb, &mut s));
    }

    GeneticParams {
        height,
        weight,
        muscle,
        age,
        extra,
    }
}

/// Bitmask-driven inheritance.
///
/// | Bit | Field  |
/// |-----|--------|
/// | 0   | height |
/// | 1   | weight |
/// | 2   | muscle |
/// | 3   | age    |
///
/// If a bit is **set** the value comes from `a`; otherwise from `b`.
/// `extra` keys follow bit 0 (height) as a tie-breaker for simplicity.
pub fn crossover_blend(a: &GeneticParams, b: &GeneticParams, crossover_mask: u64) -> GeneticParams {
    let pick = |va: f32, vb: f32, bit: u64| -> f32 {
        if (crossover_mask >> bit) & 1 == 1 {
            va
        } else {
            vb
        }
    };

    let height = pick(a.height, b.height, 0);
    let weight = pick(a.weight, b.weight, 1);
    let muscle = pick(a.muscle, b.muscle, 2);
    let age = pick(a.age, b.age, 3);

    let mut extra: HashMap<String, f32> = HashMap::new();
    for k in a.extra.keys().chain(b.extra.keys()) {
        if extra.contains_key(k) {
            continue;
        }
        let va = a.extra.get(k).copied().unwrap_or(0.0);
        let vb = b.extra.get(k).copied().unwrap_or(0.0);
        // Extra keys inherit from whichever side bit 0 selects.
        extra.insert(k.clone(), pick(va, vb, 0));
    }

    GeneticParams {
        height,
        weight,
        muscle,
        age,
        extra,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_a() -> GeneticParams {
        let mut a = GeneticParams::new();
        a.height = 1.0;
        a.weight = 0.8;
        a.muscle = 0.6;
        a.age = 0.4;
        a.extra.insert("nose".to_string(), 0.9);
        a
    }

    fn make_b() -> GeneticParams {
        let mut b = GeneticParams::new();
        b.height = 0.0;
        b.weight = 0.2;
        b.muscle = 0.4;
        b.age = 0.6;
        b.extra.insert("nose".to_string(), 0.1);
        b
    }

    fn make_profile(dominance: f32, seed: Option<u32>) -> GeneticProfile {
        GeneticProfile {
            name: "test".to_string(),
            parent_a: make_a(),
            parent_b: make_b(),
            dominance,
            seed,
        }
    }

    #[test]
    fn test_genetic_params_default() {
        let p = GeneticParams::default();
        assert_eq!(p.height, 0.0);
        assert_eq!(p.weight, 0.0);
        assert_eq!(p.muscle, 0.0);
        assert_eq!(p.age, 0.0);
        assert!(p.extra.is_empty());
    }

    #[test]
    fn test_blend_params_midpoint() {
        let a = make_a();
        let b = make_b();
        let mid = blend_params(&a, &b, 0.5);
        assert!((mid.height - 0.5).abs() < 1e-5);
        assert!((mid.weight - 0.5).abs() < 1e-5);
        assert!((mid.muscle - 0.5).abs() < 1e-5);
        assert!((mid.age - 0.5).abs() < 1e-5);
        assert!((mid.extra["nose"] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_params_full_a() {
        let a = make_a();
        let b = make_b();
        let result = blend_params(&a, &b, 1.0);
        assert!((result.height - a.height).abs() < 1e-5);
        assert!((result.weight - a.weight).abs() < 1e-5);
        assert!((result.muscle - a.muscle).abs() < 1e-5);
        assert!((result.age - a.age).abs() < 1e-5);
    }

    #[test]
    fn test_blend_params_full_b() {
        let a = make_a();
        let b = make_b();
        let result = blend_params(&a, &b, 0.0);
        assert!((result.height - b.height).abs() < 1e-5);
        assert!((result.weight - b.weight).abs() < 1e-5);
        assert!((result.muscle - b.muscle).abs() < 1e-5);
        assert!((result.age - b.age).abs() < 1e-5);
    }

    #[test]
    fn test_dominant_blend_no_seed() {
        let profile = make_profile(1.0, None);
        let result = dominant_blend(&profile);
        // dominance = 1.0 means pure A
        assert!((result.height - 1.0).abs() < 1e-5);
        assert!((result.weight - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_dominant_blend_with_seed() {
        let profile = make_profile(0.5, Some(42));
        let result = dominant_blend(&profile);
        // All fields should be clamped to [0, 1]
        assert!(result.height >= 0.0 && result.height <= 1.0);
        assert!(result.weight >= 0.0 && result.weight <= 1.0);
        assert!(result.muscle >= 0.0 && result.muscle <= 1.0);
        assert!(result.age >= 0.0 && result.age <= 1.0);
        // Should differ from the no-seed version (noise applied)
        let profile_no_seed = make_profile(0.5, None);
        let no_seed = dominant_blend(&profile_no_seed);
        // At least one field should differ (with high probability for seed 42)
        let differs = (result.height - no_seed.height).abs() > 1e-6
            || (result.weight - no_seed.weight).abs() > 1e-6
            || (result.muscle - no_seed.muscle).abs() > 1e-6
            || (result.age - no_seed.age).abs() > 1e-6;
        assert!(differs, "noise should affect at least one field");
    }

    #[test]
    fn test_inherit_random_valid_range() {
        let profile = make_profile(0.5, None);
        let result = inherit_random(&profile, 1234);
        // Each field must be exactly one of the parent values
        let valid_h = result.height == 1.0 || result.height == 0.0;
        let valid_w = result.weight == 0.8 || result.weight == 0.2;
        let valid_m = result.muscle == 0.6 || result.muscle == 0.4;
        let valid_a = result.age == 0.4 || result.age == 0.6;
        assert!(valid_h, "height must be from one of the parents");
        assert!(valid_w, "weight must be from one of the parents");
        assert!(valid_m, "muscle must be from one of the parents");
        assert!(valid_a, "age must be from one of the parents");
    }

    #[test]
    fn test_crossover_blend_all_a() {
        let a = make_a();
        let b = make_b();
        // Bits 0-3 all set → all from A
        let result = crossover_blend(&a, &b, 0b1111);
        assert!((result.height - a.height).abs() < 1e-5);
        assert!((result.weight - a.weight).abs() < 1e-5);
        assert!((result.muscle - a.muscle).abs() < 1e-5);
        assert!((result.age - a.age).abs() < 1e-5);
    }

    #[test]
    fn test_crossover_blend_all_b() {
        let a = make_a();
        let b = make_b();
        // No bits set → all from B
        let result = crossover_blend(&a, &b, 0b0000);
        assert!((result.height - b.height).abs() < 1e-5);
        assert!((result.weight - b.weight).abs() < 1e-5);
        assert!((result.muscle - b.muscle).abs() < 1e-5);
        assert!((result.age - b.age).abs() < 1e-5);
    }

    #[test]
    fn test_crossover_blend_mixed() {
        let a = make_a();
        let b = make_b();
        // bit0=height from A, bit1=weight from B, bit2=muscle from A, bit3=age from B
        // mask = 0b0101 = 5
        let result = crossover_blend(&a, &b, 0b0101);
        assert!(
            (result.height - a.height).abs() < 1e-5,
            "bit0 set → height from A"
        );
        assert!(
            (result.weight - b.weight).abs() < 1e-5,
            "bit1 clear → weight from B"
        );
        assert!(
            (result.muscle - a.muscle).abs() < 1e-5,
            "bit2 set → muscle from A"
        );
        assert!((result.age - b.age).abs() < 1e-5, "bit3 clear → age from B");
    }

    #[test]
    fn test_genetic_population() {
        let mut pop = GeneticPopulation::new();
        assert_eq!(pop.count(), 0);

        pop.add(make_profile(0.3, None));
        pop.add(make_profile(0.7, None));
        pop.add(make_profile(0.5, Some(99)));
        assert_eq!(pop.count(), 3);

        let blended = pop.blend_all();
        assert_eq!(blended.len(), 3);

        // All blended results should have valid height values
        for bp in &blended {
            assert!(bp.height >= 0.0 && bp.height <= 1.0);
        }
    }

    #[test]
    fn test_diversity_score_identical() {
        let mut pop = GeneticPopulation::new();
        // Two identical profiles → distance = 0
        pop.add(make_profile(0.5, None));
        pop.add(make_profile(0.5, None));
        let score = pop.diversity_score();
        assert!(score.abs() < 1e-5, "identical profiles → diversity = 0");
    }

    #[test]
    fn test_params_distance() {
        let a = make_a();
        let b = make_b();
        let d = params_distance(&a, &b);
        // height diff = 1, weight diff = 0.6, muscle diff = 0.2, age diff = 0.2
        let expected = (1.0_f32 * 1.0 + 0.6 * 0.6 + 0.2 * 0.2 + 0.2 * 0.2_f32).sqrt();
        assert!(
            (d - expected).abs() < 1e-4,
            "L2 distance mismatch: got {d}, expected {expected}"
        );

        // Distance from a param to itself is 0
        assert!(params_distance(&a, &a).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_params() {
        let mut p = GeneticParams {
            height: 1.5,
            weight: -0.3,
            muscle: 0.5,
            age: 2.0,
            extra: {
                let mut m = HashMap::new();
                m.insert("x".to_string(), -1.0);
                m.insert("y".to_string(), 3.0);
                m
            },
        };
        clamp_params(&mut p);
        assert_eq!(p.height, 1.0);
        assert_eq!(p.weight, 0.0);
        assert_eq!(p.muscle, 0.5);
        assert_eq!(p.age, 1.0);
        assert_eq!(p.extra["x"], 0.0);
        assert_eq!(p.extra["y"], 1.0);
    }

    #[test]
    fn test_average_params() {
        // Empty slice → None
        assert!(average_params(&[]).is_none());

        let a = make_a();
        let b = make_b();
        let avg = average_params(&[a.clone(), b.clone()]).unwrap();
        assert!((avg.height - 0.5).abs() < 1e-5);
        assert!((avg.weight - 0.5).abs() < 1e-5);
        assert!((avg.muscle - 0.5).abs() < 1e-5);
        assert!((avg.age - 0.5).abs() < 1e-5);

        // Single element → itself
        let single = average_params(std::slice::from_ref(&a)).unwrap();
        assert!((single.height - a.height).abs() < 1e-5);
    }
}
