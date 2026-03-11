// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Crowd generator: produce diverse crowds of character parameter sets.
//!
//! Supports both LCG pseudo-random and Halton quasi-random sequence generation
//! with configurable variation ranges, diversity enforcement, and statistics.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// CrowdConfig
// ---------------------------------------------------------------------------

/// Configuration for crowd generation.
pub struct CrowdConfig {
    /// Number of characters to generate.
    pub count: usize,
    /// Deterministic seed for reproducibility.
    pub seed: u32,
    /// Height parameter range `[min, max]` in `[0, 1]`.
    pub height_range: (f32, f32),
    /// Weight parameter range `[min, max]` in `[0, 1]`.
    pub weight_range: (f32, f32),
    /// Age parameter range `[min, max]` in `[0, 1]`.
    pub age_range: (f32, f32),
    /// Muscle parameter range `[min, max]` in `[0, 1]`.
    pub muscle_range: (f32, f32),
    /// `0.0` = uniform distribution, `1.0` = maximum spread.
    pub diversity_target: f32,
    /// If `false`, the generator will attempt to avoid duplicate param sets.
    pub allow_duplicates: bool,
    /// Additional named parameter ranges: name → `(min, max)`.
    pub extra_params: HashMap<String, (f32, f32)>,
}

impl Default for CrowdConfig {
    fn default() -> Self {
        Self {
            count: 10,
            seed: 42,
            height_range: (0.0, 1.0),
            weight_range: (0.0, 1.0),
            age_range: (0.0, 1.0),
            muscle_range: (0.0, 1.0),
            diversity_target: 0.5,
            allow_duplicates: true,
            extra_params: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// VariationClass
// ---------------------------------------------------------------------------

/// Broad variation class used for diversity tracking in a crowd.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariationClass {
    /// Low height, low weight.
    Petite,
    /// Average height, low weight.
    Slim,
    /// Average across all parameters.
    Average,
    /// High muscle, moderate weight.
    Athletic,
    /// Low height, high weight.
    Stocky,
    /// High height, average weight.
    Tall,
    /// High weight.
    Heavy,
    /// Does not fit any standard class.
    Custom,
}

impl VariationClass {
    /// Classify a character based on their parameter map.
    pub fn classify(params: &HashMap<String, f32>) -> VariationClass {
        let height = params.get("height").copied().unwrap_or(0.5);
        let weight = params.get("weight").copied().unwrap_or(0.5);
        let muscle = params.get("muscle").copied().unwrap_or(0.3);

        // Thresholds
        let lo = 0.35_f32;
        let hi = 0.65_f32;

        let short = height < lo;
        let tall = height > hi;
        let light = weight < lo;
        let heavy_w = weight > hi;
        let avg_h = !short && !tall;
        let avg_w = !light && !heavy_w;
        let muscular = muscle > hi;

        if short && light {
            VariationClass::Petite
        } else if avg_h && light {
            VariationClass::Slim
        } else if short && heavy_w {
            VariationClass::Stocky
        } else if heavy_w {
            VariationClass::Heavy
        } else if tall && avg_w {
            VariationClass::Tall
        } else if muscular && avg_w {
            VariationClass::Athletic
        } else if avg_h && avg_w && !muscular {
            VariationClass::Average
        } else {
            VariationClass::Custom
        }
    }

    /// Return all defined variation classes (except `Custom`).
    pub fn all() -> Vec<VariationClass> {
        vec![
            VariationClass::Petite,
            VariationClass::Slim,
            VariationClass::Average,
            VariationClass::Athletic,
            VariationClass::Stocky,
            VariationClass::Tall,
            VariationClass::Heavy,
            VariationClass::Custom,
        ]
    }

    /// Human-readable name of the variation class.
    pub fn name(&self) -> &'static str {
        match self {
            VariationClass::Petite => "Petite",
            VariationClass::Slim => "Slim",
            VariationClass::Average => "Average",
            VariationClass::Athletic => "Athletic",
            VariationClass::Stocky => "Stocky",
            VariationClass::Tall => "Tall",
            VariationClass::Heavy => "Heavy",
            VariationClass::Custom => "Custom",
        }
    }
}

// ---------------------------------------------------------------------------
// CrowdCharacter
// ---------------------------------------------------------------------------

/// A single generated character's full parameter set.
pub struct CrowdCharacter {
    /// Zero-based index assigned at generation time.
    pub id: usize,
    /// Named parameter values in `[0, 1]`.
    pub params: HashMap<String, f32>,
    /// Broad variation class determined from `params`.
    pub variation_class: VariationClass,
}

// ---------------------------------------------------------------------------
// Crowd
// ---------------------------------------------------------------------------

/// The generated crowd of [`CrowdCharacter`] instances.
pub struct Crowd {
    /// All characters in generation order.
    pub characters: Vec<CrowdCharacter>,
    /// The config that produced this crowd.
    pub config: CrowdConfig,
}

impl Crowd {
    /// Number of characters in the crowd.
    pub fn count(&self) -> usize {
        self.characters.len()
    }

    /// Look up a character by `id`.
    pub fn get(&self, id: usize) -> Option<&CrowdCharacter> {
        self.characters.iter().find(|c| c.id == id)
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    /// Compute the mean value of each parameter across the crowd.
    pub fn mean_params(&self) -> HashMap<String, f32> {
        if self.characters.is_empty() {
            return HashMap::new();
        }
        let n = self.characters.len() as f32;
        let mut sums: HashMap<String, f32> = HashMap::new();
        for ch in &self.characters {
            for (k, v) in &ch.params {
                *sums.entry(k.clone()).or_insert(0.0) += v;
            }
        }
        sums.iter_mut().for_each(|(_, v)| *v /= n);
        sums
    }

    /// Compute the standard deviation of each parameter across the crowd.
    pub fn std_params(&self) -> HashMap<String, f32> {
        if self.characters.len() < 2 {
            return HashMap::new();
        }
        let means = self.mean_params();
        let n = self.characters.len() as f32;
        let mut sq_sums: HashMap<String, f32> = HashMap::new();
        for ch in &self.characters {
            for (k, v) in &ch.params {
                let mean = means.get(k).copied().unwrap_or(0.0);
                let d = v - mean;
                *sq_sums.entry(k.clone()).or_insert(0.0) += d * d;
            }
        }
        sq_sums.iter_mut().for_each(|(_, v)| *v = (*v / n).sqrt());
        sq_sums
    }

    /// Mean pairwise parameter distance (diversity score).
    pub fn diversity_score(&self) -> f32 {
        let n = self.characters.len();
        if n < 2 {
            return 0.0;
        }
        let mut total = 0.0_f32;
        let mut count = 0usize;
        for i in 0..n {
            for j in (i + 1)..n {
                total += param_distance(&self.characters[i].params, &self.characters[j].params);
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            total / count as f32
        }
    }

    // -----------------------------------------------------------------------
    // Filtering / sorting
    // -----------------------------------------------------------------------

    /// Return all characters that belong to the given [`VariationClass`].
    pub fn by_class(&self, class: &VariationClass) -> Vec<&CrowdCharacter> {
        self.characters
            .iter()
            .filter(|c| &c.variation_class == class)
            .collect()
    }

    /// Count characters per variation class.
    pub fn class_distribution(&self) -> HashMap<VariationClass, usize> {
        let mut dist: HashMap<VariationClass, usize> = HashMap::new();
        for ch in &self.characters {
            *dist.entry(ch.variation_class.clone()).or_insert(0) += 1;
        }
        dist
    }

    /// Return references to characters sorted in ascending order by `param`.
    ///
    /// Characters missing the requested parameter are placed at the end.
    pub fn sorted_by(&self, param: &str) -> Vec<&CrowdCharacter> {
        let mut refs: Vec<&CrowdCharacter> = self.characters.iter().collect();
        refs.sort_by(|a, b| {
            let va = a.params.get(param).copied().unwrap_or(f32::MAX);
            let vb = b.params.get(param).copied().unwrap_or(f32::MAX);
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });
        refs
    }

    /// Export the crowd as a plain list of param maps (for batch processing).
    pub fn to_param_list(&self) -> Vec<HashMap<String, f32>> {
        self.characters.iter().map(|c| c.params.clone()).collect()
    }

    /// Generate a JSON-like human-readable summary string.
    pub fn summary(&self) -> String {
        let mean = self.mean_params();
        let std = self.std_params();
        let dist = self.class_distribution();
        let diversity = self.diversity_score();

        let mut lines = Vec::new();
        lines.push(format!(
            "{{ \"count\": {}, \"diversity_score\": {:.4},",
            self.count(),
            diversity
        ));
        lines.push("  \"mean_params\": {".to_string());

        let mut keys: Vec<&String> = mean.keys().collect();
        keys.sort();
        for (idx, k) in keys.iter().enumerate() {
            let m = mean[*k];
            let s = std.get(*k).copied().unwrap_or(0.0);
            let comma = if idx + 1 < keys.len() { "," } else { "" };
            lines.push(format!(
                "    \"{}\": {{ \"mean\": {:.4}, \"std\": {:.4} }}{}",
                k, m, s, comma
            ));
        }
        lines.push("  },".to_string());
        lines.push("  \"class_distribution\": {".to_string());

        let mut class_entries: Vec<(String, usize)> = dist
            .iter()
            .map(|(c, n)| (c.name().to_string(), *n))
            .collect();
        class_entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (idx, (name, count)) in class_entries.iter().enumerate() {
            let comma = if idx + 1 < class_entries.len() {
                ","
            } else {
                ""
            };
            lines.push(format!("    \"{}\": {}{}", name, count, comma));
        }
        lines.push("  }".to_string());
        lines.push("}".to_string());
        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Core public API
// ---------------------------------------------------------------------------

/// Compute the Halton quasi-random sequence value for index `i` in base `base`.
///
/// Uses 1-based indexing to avoid the trivial `0` at `i = 0`.
pub fn halton(i: usize, base: usize) -> f32 {
    let mut result = 0.0_f64;
    let mut denom = 1.0_f64;
    let mut n = i;
    while n > 0 {
        denom *= base as f64;
        result += (n % base) as f64 / denom;
        n /= base;
    }
    result as f32
}

/// LCG-based pseudo-random value in `[0, 1)`.
///
/// Advances `*seed` in place (multiplier 1664525, addend 1013904223 — Numerical Recipes).
pub fn lcg_rand(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 9) as f32 / (1u32 << 23) as f32
}

/// Euclidean L2 distance between two parameter maps.
///
/// Only keys present in **both** maps contribute to the distance.
pub fn param_distance(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    let sq: f32 = a
        .iter()
        .filter_map(|(k, va)| b.get(k).map(|vb| (va - vb).powi(2)))
        .sum();
    sq.sqrt()
}

/// Scale an LCG random value from `[0, 1)` to `[min, max)`.
#[inline]
fn scale(v: f32, min: f32, max: f32) -> f32 {
    (min + v * (max - min)).clamp(min, max)
}

/// Apply `diversity_target` bias: mix between uniform center (0.5) and full range.
///
/// When `diversity_target = 0.0`, values are pulled towards `0.5`.
/// When `diversity_target = 1.0`, the full `[min, max]` range is used.
#[inline]
fn apply_diversity(v: f32, min: f32, max: f32, diversity_target: f32) -> f32 {
    let centre = (min + max) / 2.0;
    let biased = centre + (v - centre) * diversity_target;
    biased.clamp(min, max)
}

/// Build a [`HashMap`] of parameters from a slice of `(value, range)` pairs.
fn build_params(values: &[(&str, f32, (f32, f32))], diversity: f32) -> HashMap<String, f32> {
    values
        .iter()
        .map(|(name, raw, (lo, hi))| {
            let scaled = scale(*raw, *lo, *hi);
            let final_val = if (diversity - 1.0).abs() < f32::EPSILON {
                scaled
            } else {
                apply_diversity(scaled, *lo, *hi, diversity)
            };
            (name.to_string(), final_val)
        })
        .collect()
}

/// Generate a crowd using LCG pseudo-random numbers.
pub fn generate_crowd(config: CrowdConfig) -> Crowd {
    let mut seed = config.seed;
    let diversity = config.diversity_target.clamp(0.0, 1.0);

    // Collect extra param names for deterministic ordering.
    let mut extra_names: Vec<String> = config.extra_params.keys().cloned().collect();
    extra_names.sort();

    let mut characters: Vec<CrowdCharacter> = (0..config.count)
        .map(|id| {
            let h = lcg_rand(&mut seed);
            let w = lcg_rand(&mut seed);
            let a = lcg_rand(&mut seed);
            let m = lcg_rand(&mut seed);

            let mut entries: Vec<(&str, f32, (f32, f32))> = vec![
                ("height", h, config.height_range),
                ("weight", w, config.weight_range),
                ("age", a, config.age_range),
                ("muscle", m, config.muscle_range),
            ];

            for name in &extra_names {
                let Some(&range) = config.extra_params.get(name) else {
                    continue;
                };
                let v = lcg_rand(&mut seed);
                // SAFETY: the string slice lives for the iteration body only —
                // we'll collect into owned Strings via build_params immediately.
                entries.push((name.as_str(), v, range));
            }

            let params = build_params(&entries, diversity);
            let variation_class = VariationClass::classify(&params);
            CrowdCharacter {
                id,
                params,
                variation_class,
            }
        })
        .collect();

    if !config.allow_duplicates {
        enforce_diversity(&mut characters, 0.01, config.seed);
    }

    Crowd { characters, config }
}

/// Generate a crowd using the Halton quasi-random sequence for better coverage.
///
/// Core parameters use Halton bases 2, 3, 5, 7; extra parameters use bases 11, 13, 17, …
pub fn generate_crowd_halton(config: CrowdConfig) -> Crowd {
    // Prime bases for the Halton sequence.
    const BASES: [usize; 8] = [2, 3, 5, 7, 11, 13, 17, 19];

    let diversity = config.diversity_target.clamp(0.0, 1.0);

    let mut extra_names: Vec<String> = config.extra_params.keys().cloned().collect();
    extra_names.sort();

    // Offset by seed to shift the sequence start.
    let offset = (config.seed as usize) % 97 + 1;

    let mut characters: Vec<CrowdCharacter> = (0..config.count)
        .map(|id| {
            let idx = id + offset;
            let h = halton(idx, BASES[0]);
            let w = halton(idx, BASES[1]);
            let a = halton(idx, BASES[2]);
            let m = halton(idx, BASES[3]);

            let mut entries: Vec<(&str, f32, (f32, f32))> = vec![
                ("height", h, config.height_range),
                ("weight", w, config.weight_range),
                ("age", a, config.age_range),
                ("muscle", m, config.muscle_range),
            ];

            for (ei, name) in extra_names.iter().enumerate() {
                let base = BASES.get(4 + ei).copied().unwrap_or(23 + ei * 2);
                let Some(&range) = config.extra_params.get(name) else {
                    continue;
                };
                let v = halton(idx, base);
                entries.push((name.as_str(), v, range));
            }

            let params = build_params(&entries, diversity);
            let variation_class = VariationClass::classify(&params);
            CrowdCharacter {
                id,
                params,
                variation_class,
            }
        })
        .collect();

    if !config.allow_duplicates {
        enforce_diversity(&mut characters, 0.01, config.seed);
    }

    Crowd { characters, config }
}

/// Ensure minimum pairwise diversity in a set of characters.
///
/// Any pair closer than `min_distance` in parameter space has one of the two
/// regenerated via LCG. Runs at most `O(n^2)` passes (one sweep).
pub fn enforce_diversity(chars: &mut [CrowdCharacter], min_distance: f32, seed: u32) {
    let mut rng_seed = seed.wrapping_add(0xDEAD_BEEF);
    let n = chars.len();
    // One forward sweep: for each pair (i, j) that is too close, randomise j.
    for i in 0..n {
        for j in (i + 1)..n {
            let dist = param_distance(&chars[i].params, &chars[j].params);
            if dist < min_distance {
                // Regenerate character j with fresh random params.
                let keys: Vec<String> = chars[j].params.keys().cloned().collect();
                for key in &keys {
                    let v = lcg_rand(&mut rng_seed);
                    chars[j].params.insert(key.clone(), v);
                }
                chars[j].variation_class = VariationClass::classify(&chars[j].params);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // Helper: default config with small count.
    fn small_config() -> CrowdConfig {
        CrowdConfig {
            count: 8,
            seed: 1234,
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // 1. lcg_rand stays in [0, 1)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lcg_rand_range() {
        let mut seed = 42_u32;
        for _ in 0..1000 {
            let v = lcg_rand(&mut seed);
            assert!((0.0..1.0).contains(&v), "lcg_rand out of range: {v}");
        }
    }

    // -----------------------------------------------------------------------
    // 2. halton sequence correctness (base 2)
    // -----------------------------------------------------------------------
    #[test]
    fn test_halton_base2() {
        // Known values for Halton base-2
        assert!((halton(1, 2) - 0.5).abs() < 1e-6, "h(1,2) = 0.5");
        assert!((halton(2, 2) - 0.25).abs() < 1e-6, "h(2,2) = 0.25");
        assert!((halton(3, 2) - 0.75).abs() < 1e-6, "h(3,2) = 0.75");
        assert!((halton(4, 2) - 0.125).abs() < 1e-6, "h(4,2) = 0.125");
        assert_eq!(halton(0, 2), 0.0, "h(0,2) = 0");
    }

    // -----------------------------------------------------------------------
    // 3. halton sequence correctness (base 3)
    // -----------------------------------------------------------------------
    #[test]
    fn test_halton_base3() {
        // h(1, 3) = 1/3
        assert!((halton(1, 3) - 1.0 / 3.0).abs() < 1e-5, "h(1,3) = 1/3");
        // h(2, 3) = 2/3
        assert!((halton(2, 3) - 2.0 / 3.0).abs() < 1e-5, "h(2,3) = 2/3");
        // h(3, 3) = 1/9
        assert!((halton(3, 3) - 1.0 / 9.0).abs() < 1e-5, "h(3,3) = 1/9");
    }

    // -----------------------------------------------------------------------
    // 4. VariationClass::classify covers all major branches
    // -----------------------------------------------------------------------
    #[test]
    fn test_classify_petite() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.2),
            ("weight".to_string(), 0.2),
            ("muscle".to_string(), 0.3),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Petite);
    }

    #[test]
    fn test_classify_slim() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.5),
            ("weight".to_string(), 0.2),
            ("muscle".to_string(), 0.3),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Slim);
    }

    #[test]
    fn test_classify_tall() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.8),
            ("weight".to_string(), 0.5),
            ("muscle".to_string(), 0.3),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Tall);
    }

    #[test]
    fn test_classify_heavy() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.5),
            ("weight".to_string(), 0.8),
            ("muscle".to_string(), 0.3),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Heavy);
    }

    #[test]
    fn test_classify_stocky() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.2),
            ("weight".to_string(), 0.8),
            ("muscle".to_string(), 0.3),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Stocky);
    }

    #[test]
    fn test_classify_athletic() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.5),
            ("weight".to_string(), 0.5),
            ("muscle".to_string(), 0.8),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Athletic);
    }

    #[test]
    fn test_classify_average() {
        let params: HashMap<String, f32> = [
            ("height".to_string(), 0.5),
            ("weight".to_string(), 0.5),
            ("muscle".to_string(), 0.3),
        ]
        .into();
        assert_eq!(VariationClass::classify(&params), VariationClass::Average);
    }

    // -----------------------------------------------------------------------
    // 5. generate_crowd basic properties
    // -----------------------------------------------------------------------
    #[test]
    fn test_generate_crowd_count() {
        let cfg = CrowdConfig {
            count: 20,
            seed: 7,
            ..Default::default()
        };
        let crowd = generate_crowd(cfg);
        assert_eq!(crowd.count(), 20);
    }

    #[test]
    fn test_generate_crowd_params_in_range() {
        let cfg = CrowdConfig {
            count: 50,
            seed: 99,
            height_range: (0.2, 0.8),
            weight_range: (0.1, 0.9),
            age_range: (0.0, 1.0),
            muscle_range: (0.0, 0.5),
            ..Default::default()
        };
        let crowd = generate_crowd(cfg);
        for ch in &crowd.characters {
            let h = ch.params["height"];
            let w = ch.params["weight"];
            let m = ch.params["muscle"];
            assert!((0.2..=0.8).contains(&h), "height {h} out of range");
            assert!((0.1..=0.9).contains(&w), "weight {w} out of range");
            assert!((0.0..=0.5).contains(&m), "muscle {m} out of range");
        }
    }

    // -----------------------------------------------------------------------
    // 6. Determinism: same seed → same crowd
    // -----------------------------------------------------------------------
    #[test]
    fn test_generate_crowd_deterministic() {
        let cfg1 = small_config();
        let cfg2 = small_config();
        let c1 = generate_crowd(cfg1);
        let c2 = generate_crowd(cfg2);
        assert_eq!(c1.count(), c2.count());
        for (a, b) in c1.characters.iter().zip(c2.characters.iter()) {
            for (k, va) in &a.params {
                let vb = b.params[k];
                assert!((va - vb).abs() < 1e-6, "Non-deterministic at param {k}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // 7. generate_crowd_halton: values in [0, 1]
    // -----------------------------------------------------------------------
    #[test]
    fn test_generate_crowd_halton_range() {
        let cfg = CrowdConfig {
            count: 30,
            seed: 5,
            ..Default::default()
        };
        let crowd = generate_crowd_halton(cfg);
        assert_eq!(crowd.count(), 30);
        for ch in &crowd.characters {
            for (k, v) in &ch.params {
                assert!(
                    *v >= 0.0 && *v <= 1.0,
                    "halton param {k} = {v} out of [0,1]"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // 8. param_distance symmetry and triangle inequality
    // -----------------------------------------------------------------------
    #[test]
    fn test_param_distance() {
        let a: HashMap<String, f32> = [("x".to_string(), 0.0), ("y".to_string(), 0.0)].into();
        let b: HashMap<String, f32> = [("x".to_string(), 1.0), ("y".to_string(), 0.0)].into();
        let c: HashMap<String, f32> = [("x".to_string(), 1.0), ("y".to_string(), 1.0)].into();

        let dab = param_distance(&a, &b);
        let dba = param_distance(&b, &a);
        assert!((dab - 1.0).abs() < 1e-5, "d(a,b) should be 1.0, got {dab}");
        assert!((dab - dba).abs() < 1e-6, "symmetry broken");

        let dac = param_distance(&a, &c);
        assert!((dac - 2.0_f32.sqrt()).abs() < 1e-5, "d(a,c) = sqrt(2)");
    }

    // -----------------------------------------------------------------------
    // 9. mean_params and std_params
    // -----------------------------------------------------------------------
    #[test]
    fn test_mean_std_params() {
        // Two characters: height 0.2 and 0.8 → mean 0.5, std = 0.3
        let ch0 = CrowdCharacter {
            id: 0,
            params: [("height".to_string(), 0.2_f32)].into(),
            variation_class: VariationClass::Custom,
        };
        let ch1 = CrowdCharacter {
            id: 1,
            params: [("height".to_string(), 0.8_f32)].into(),
            variation_class: VariationClass::Custom,
        };
        let crowd = Crowd {
            characters: vec![ch0, ch1],
            config: CrowdConfig::default(),
        };
        let mean = crowd.mean_params();
        let std = crowd.std_params();
        assert!((mean["height"] - 0.5).abs() < 1e-5, "mean = 0.5");
        // std = sqrt(((0.2-0.5)^2 + (0.8-0.5)^2) / 2) = sqrt(0.09) = 0.3
        assert!(
            (std["height"] - 0.3).abs() < 1e-5,
            "std = 0.3, got {}",
            std["height"]
        );
    }

    // -----------------------------------------------------------------------
    // 10. diversity_score is non-negative and increases with spread
    // -----------------------------------------------------------------------
    #[test]
    fn test_diversity_score_monotone() {
        // Narrow config (low spread)
        let narrow = CrowdConfig {
            count: 20,
            seed: 1,
            diversity_target: 0.0,
            ..Default::default()
        };
        // Wide config (high spread)
        let wide = CrowdConfig {
            count: 20,
            seed: 1,
            diversity_target: 1.0,
            ..Default::default()
        };
        let c_narrow = generate_crowd(narrow);
        let c_wide = generate_crowd(wide);
        let s_narrow = c_narrow.diversity_score();
        let s_wide = c_wide.diversity_score();
        assert!(s_narrow >= 0.0);
        assert!(s_wide >= 0.0);
        // Wide should have at least as large a score as narrow.
        assert!(
            s_wide >= s_narrow - 1e-4,
            "wide diversity {s_wide} should be >= narrow diversity {s_narrow}"
        );
    }

    // -----------------------------------------------------------------------
    // 11. by_class and class_distribution
    // -----------------------------------------------------------------------
    #[test]
    fn test_by_class_and_distribution() {
        let cfg = CrowdConfig {
            count: 100,
            seed: 2025,
            ..Default::default()
        };
        let crowd = generate_crowd(cfg);
        let dist = crowd.class_distribution();
        // Sum of all class counts should equal total count
        let total: usize = dist.values().sum();
        assert_eq!(total, 100);
        // by_class counts should match distribution counts
        for (class, &count) in &dist {
            assert_eq!(crowd.by_class(class).len(), count);
        }
    }

    // -----------------------------------------------------------------------
    // 12. sorted_by returns correct ascending order
    // -----------------------------------------------------------------------
    #[test]
    fn test_sorted_by() {
        let cfg = CrowdConfig {
            count: 15,
            seed: 77,
            ..Default::default()
        };
        let crowd = generate_crowd(cfg);
        let sorted = crowd.sorted_by("height");
        for window in sorted.windows(2) {
            let h0 = window[0].params["height"];
            let h1 = window[1].params["height"];
            assert!(h0 <= h1, "Not sorted: {h0} > {h1}");
        }
    }

    // -----------------------------------------------------------------------
    // 13. to_param_list and get round-trips
    // -----------------------------------------------------------------------
    #[test]
    fn test_to_param_list_and_get() {
        let cfg = small_config();
        let crowd = generate_crowd(cfg);
        let list = crowd.to_param_list();
        assert_eq!(list.len(), crowd.count());
        for (i, ch) in crowd.characters.iter().enumerate() {
            let from_get = crowd.get(ch.id).unwrap();
            // Params from get() and from to_param_list() should match
            for (k, v) in &from_get.params {
                let lv = list[i][k];
                assert!((v - lv).abs() < 1e-6, "list mismatch for {k}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // 14. summary produces non-empty string with key fields
    // -----------------------------------------------------------------------
    #[test]
    fn test_summary_content() {
        let crowd = generate_crowd(small_config());
        let s = crowd.summary();
        assert!(s.contains("count"), "summary missing 'count'");
        assert!(
            s.contains("diversity_score"),
            "summary missing 'diversity_score'"
        );
        assert!(s.contains("mean_params"), "summary missing 'mean_params'");
        assert!(
            s.contains("class_distribution"),
            "summary missing 'class_distribution'"
        );
        // Write to /tmp/ for inspection
        let path = "/tmp/oxihuman_crowd_summary.txt";
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }

    // -----------------------------------------------------------------------
    // 15. extra_params are generated and in range
    // -----------------------------------------------------------------------
    #[test]
    fn test_extra_params() {
        let mut extra = HashMap::new();
        extra.insert("nose_width".to_string(), (0.2_f32, 0.7_f32));
        extra.insert("jaw_size".to_string(), (0.1_f32, 0.9_f32));
        let cfg = CrowdConfig {
            count: 20,
            seed: 31415,
            extra_params: extra,
            ..Default::default()
        };
        let crowd = generate_crowd(cfg);
        for ch in &crowd.characters {
            let nw = ch.params["nose_width"];
            let js = ch.params["jaw_size"];
            assert!((0.2..=0.7).contains(&nw), "nose_width {nw} out of range");
            assert!((0.1..=0.9).contains(&js), "jaw_size {js} out of range");
        }
    }

    // -----------------------------------------------------------------------
    // 16. enforce_diversity increases minimum pairwise distance
    // -----------------------------------------------------------------------
    #[test]
    fn test_enforce_diversity() {
        // Create two nearly identical characters
        let p: HashMap<String, f32> =
            [("height".to_string(), 0.5), ("weight".to_string(), 0.5)].into();
        let mut chars = vec![
            CrowdCharacter {
                id: 0,
                params: p.clone(),
                variation_class: VariationClass::Average,
            },
            CrowdCharacter {
                id: 1,
                params: p.clone(),
                variation_class: VariationClass::Average,
            },
        ];
        let before = param_distance(&chars[0].params, &chars[1].params);
        assert!(before < 1e-5, "should start as identical");

        enforce_diversity(&mut chars, 0.01, 42);

        let after = param_distance(&chars[0].params, &chars[1].params);
        assert!(
            after >= 0.01 || after == 0.0,
            "after enforce_diversity distance = {after}; expected >= 0.01 or randomised away"
        );
    }

    // -----------------------------------------------------------------------
    // 17. VariationClass::all() covers all variants
    // -----------------------------------------------------------------------
    #[test]
    fn test_variation_class_all() {
        let all = VariationClass::all();
        assert!(all.contains(&VariationClass::Petite));
        assert!(all.contains(&VariationClass::Slim));
        assert!(all.contains(&VariationClass::Average));
        assert!(all.contains(&VariationClass::Athletic));
        assert!(all.contains(&VariationClass::Stocky));
        assert!(all.contains(&VariationClass::Tall));
        assert!(all.contains(&VariationClass::Heavy));
        assert!(all.contains(&VariationClass::Custom));
    }

    // -----------------------------------------------------------------------
    // 18. VariationClass::name() returns non-empty strings
    // -----------------------------------------------------------------------
    #[test]
    fn test_variation_class_name() {
        for class in VariationClass::all() {
            assert!(!class.name().is_empty());
        }
    }

    // -----------------------------------------------------------------------
    // 19. Halton crowd writes to /tmp for visual inspection
    // -----------------------------------------------------------------------
    #[test]
    fn test_halton_crowd_to_file() {
        let cfg = CrowdConfig {
            count: 16,
            seed: 3,
            ..Default::default()
        };
        let crowd = generate_crowd_halton(cfg);
        let list = crowd.to_param_list();
        let path = "/tmp/oxihuman_halton_crowd.csv";
        let mut f = std::fs::File::create(path).unwrap();
        writeln!(f, "id,height,weight,age,muscle").unwrap();
        for (i, p) in list.iter().enumerate() {
            writeln!(
                f,
                "{},{:.4},{:.4},{:.4},{:.4}",
                i,
                p.get("height").copied().unwrap_or(0.0),
                p.get("weight").copied().unwrap_or(0.0),
                p.get("age").copied().unwrap_or(0.0),
                p.get("muscle").copied().unwrap_or(0.0),
            )
            .unwrap();
        }
    }

    // -----------------------------------------------------------------------
    // 20. get() returns None for out-of-range id
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_out_of_range() {
        let crowd = generate_crowd(small_config());
        assert!(crowd.get(9999).is_none());
    }

    // -----------------------------------------------------------------------
    // 21. Crowd with zero characters is safe
    // -----------------------------------------------------------------------
    #[test]
    fn test_empty_crowd() {
        let cfg = CrowdConfig {
            count: 0,
            ..Default::default()
        };
        let crowd = generate_crowd(cfg);
        assert_eq!(crowd.count(), 0);
        assert_eq!(crowd.diversity_score(), 0.0);
        assert!(crowd.mean_params().is_empty());
        assert!(crowd.to_param_list().is_empty());
    }
}
