// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Simple LCG pseudo-random number generator (no rand crate needed)
// ---------------------------------------------------------------------------

/// Simple Linear Congruential Generator for deterministic randomness.
pub struct Lcg {
    state: u64,
}

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    /// Next f32 in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as f32 / (u32::MAX as f32)
    }

    /// Next f32 in [min, max).
    pub fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Box-Muller transform: N(mean, std_dev).
    pub fn next_gaussian(&mut self, mean: f32, std_dev: f32) -> f32 {
        let u1 = self.next_f32() + 1e-10;
        let u2 = self.next_f32();
        let z = (-2.0_f32 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        mean + std_dev * z
    }
}

// ---------------------------------------------------------------------------
// Van der Corput low-discrepancy sequence
// ---------------------------------------------------------------------------

/// Van der Corput sequence value for index `n` in base `base`.
/// Reflects n's base-b digits about the decimal point.
pub fn van_der_corput(n: usize, base: usize) -> f32 {
    let mut result = 0.0_f64;
    let mut denominator = 1.0_f64;
    let mut n_remaining = n;
    while n_remaining > 0 {
        denominator *= base as f64;
        result += (n_remaining % base) as f64 / denominator;
        n_remaining /= base;
    }
    result as f32
}

// ---------------------------------------------------------------------------
// Sampling strategy
// ---------------------------------------------------------------------------

/// Strategy for generating varied parameter sets.
pub enum SamplingStrategy {
    /// Pure uniform random in `[0,1]`.
    Uniform,
    /// Normal distribution centered at base params, clamped to `[0,1]`.
    Gaussian { std_dev: f32 },
    /// Latin hypercube sampling for uniform coverage.
    LatinHypercube,
    /// Sobol-like low-discrepancy (simple van der Corput sequence).
    LowDiscrepancy,
}

// ---------------------------------------------------------------------------
// Parameter specification
// ---------------------------------------------------------------------------

/// A parameter specification with name, range, and distribution hint.
pub struct ParamSpec {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub weight: f32,
}

impl ParamSpec {
    pub fn new(name: impl Into<String>, min: f32, max: f32, default: f32) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            default,
            weight: 1.0,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

// ---------------------------------------------------------------------------
// Diversity sampler
// ---------------------------------------------------------------------------

/// Diversity sampler that generates varied body parameter sets.
pub struct DiversitySampler {
    params: Vec<ParamSpec>,
    strategy: SamplingStrategy,
    seed: u64,
}

/// First 6 primes for low-discrepancy sequence (one per dimension).
const LD_PRIMES: [usize; 6] = [2, 3, 5, 7, 11, 13];

impl DiversitySampler {
    pub fn new(strategy: SamplingStrategy) -> Self {
        Self {
            params: Vec::new(),
            strategy,
            seed: 42,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn add_param(&mut self, spec: ParamSpec) {
        self.params.push(spec);
    }

    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// Generate N diverse parameter sets.
    pub fn sample(&self, n: usize) -> Vec<HashMap<String, f32>> {
        if n == 0 || self.params.is_empty() {
            return Vec::new();
        }

        let mut rng = Lcg::new(self.seed);

        match &self.strategy {
            SamplingStrategy::Uniform => self.sample_uniform(&mut rng, n),
            SamplingStrategy::Gaussian { std_dev } => {
                // Use defaults as the base
                let base: HashMap<String, f32> = self
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.default))
                    .collect();
                self.sample_gaussian(&mut rng, &base, *std_dev, n)
            }
            SamplingStrategy::LatinHypercube => self.sample_lhs(&mut rng, n),
            SamplingStrategy::LowDiscrepancy => self.sample_ld(n),
        }
    }

    /// Generate one sample near given base parameters.
    pub fn sample_near(&self, base: &HashMap<String, f32>, n: usize) -> Vec<HashMap<String, f32>> {
        if n == 0 || self.params.is_empty() {
            return Vec::new();
        }
        let mut rng = Lcg::new(self.seed);
        let std_dev = match &self.strategy {
            SamplingStrategy::Gaussian { std_dev } => *std_dev,
            _ => 0.1,
        };
        self.sample_gaussian(&mut rng, base, std_dev, n)
    }

    /// Generate population with guaranteed coverage of extremes.
    pub fn sample_with_extremes(&self, n: usize) -> Vec<HashMap<String, f32>> {
        if self.params.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(n);

        // First sample: all minimums
        let min_sample: HashMap<String, f32> = self
            .params
            .iter()
            .map(|p| (p.name.clone(), p.min))
            .collect();
        result.push(min_sample);

        // Second sample: all maximums
        if n >= 2 {
            let max_sample: HashMap<String, f32> = self
                .params
                .iter()
                .map(|p| (p.name.clone(), p.max))
                .collect();
            result.push(max_sample);
        }

        // Fill the rest with normal sampling
        if n > 2 {
            let remaining = self.sample(n - 2);
            result.extend(remaining);
        }

        result.truncate(n);
        result
    }

    /// Compute diversity score: average pairwise L2 distance between samples.
    pub fn diversity_score(samples: &[HashMap<String, f32>]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0_f32;
        let mut count = 0usize;

        for i in 0..samples.len() {
            for j in (i + 1)..samples.len() {
                let sq_dist: f32 = samples[i]
                    .iter()
                    .filter_map(|(k, v)| samples[j].get(k).map(|w| (v - w).powi(2)))
                    .sum();
                total += sq_dist.sqrt();
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
    // Internal helpers
    // -----------------------------------------------------------------------

    fn sample_uniform(&self, rng: &mut Lcg, n: usize) -> Vec<HashMap<String, f32>> {
        (0..n)
            .map(|_| {
                self.params
                    .iter()
                    .map(|p| (p.name.clone(), rng.next_range(p.min, p.max)))
                    .collect()
            })
            .collect()
    }

    fn sample_gaussian(
        &self,
        rng: &mut Lcg,
        base: &HashMap<String, f32>,
        std_dev: f32,
        n: usize,
    ) -> Vec<HashMap<String, f32>> {
        (0..n)
            .map(|_| {
                self.params
                    .iter()
                    .map(|p| {
                        let center = base.get(&p.name).copied().unwrap_or(p.default);
                        let range = p.max - p.min;
                        let val = rng.next_gaussian(center, std_dev * range * p.weight);
                        (p.name.clone(), val.clamp(p.min, p.max))
                    })
                    .collect()
            })
            .collect()
    }

    fn sample_lhs(&self, rng: &mut Lcg, n: usize) -> Vec<HashMap<String, f32>> {
        // For each parameter, create a permutation of strata [0..n)
        let param_strata: Vec<Vec<usize>> = self
            .params
            .iter()
            .map(|_| {
                let mut strata: Vec<usize> = (0..n).collect();
                // Fisher-Yates shuffle using our LCG
                for i in (1..strata.len()).rev() {
                    let j = (rng.next_f32() * (i + 1) as f32) as usize;
                    let j = j.min(i);
                    strata.swap(i, j);
                }
                strata
            })
            .collect();

        (0..n)
            .map(|i| {
                self.params
                    .iter()
                    .enumerate()
                    .map(|(dim, p)| {
                        let stratum = param_strata[dim][i];
                        // Sample uniformly within stratum
                        let lo = stratum as f32 / n as f32;
                        let hi = (stratum + 1) as f32 / n as f32;
                        let t = lo + rng.next_f32() * (hi - lo);
                        let val = p.min + t * (p.max - p.min);
                        (p.name.clone(), val)
                    })
                    .collect()
            })
            .collect()
    }

    fn sample_ld(&self, n: usize) -> Vec<HashMap<String, f32>> {
        (0..n)
            .map(|i| {
                self.params
                    .iter()
                    .enumerate()
                    .map(|(dim, p)| {
                        let t = if dim < LD_PRIMES.len() {
                            // Use 1-indexed to avoid the trivial 0 value
                            van_der_corput(i + 1, LD_PRIMES[dim])
                        } else {
                            // Fall back to uniform via van_der_corput base 2 offset
                            let mut rng =
                                Lcg::new(self.seed.wrapping_add(dim as u64).wrapping_add(i as u64));
                            rng.next_f32()
                        };
                        let val = p.min + t.clamp(0.0, 1.0) * (p.max - p.min);
                        (p.name.clone(), val)
                    })
                    .collect()
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Default human body parameter specs
// ---------------------------------------------------------------------------

/// Human body parameter specs (height, weight, muscle, age, etc.).
pub fn default_body_params() -> Vec<ParamSpec> {
    vec![
        ParamSpec::new("height", 0.0, 1.0, 0.5),
        ParamSpec::new("weight", 0.0, 1.0, 0.5),
        ParamSpec::new("muscle", 0.0, 1.0, 0.3),
        ParamSpec::new("age", 0.0, 1.0, 0.35),
        ParamSpec::new("bmi_factor", 0.0, 1.0, 0.4),
        ParamSpec::new("shoulder_width", 0.0, 1.0, 0.5),
        ParamSpec::new("hip_width", 0.0, 1.0, 0.5),
    ]
}

/// Quick-generate N random body profiles using LatinHypercube strategy.
pub fn generate_population(n: usize, seed: u64) -> Vec<HashMap<String, f32>> {
    let mut sampler = DiversitySampler::new(SamplingStrategy::LatinHypercube).with_seed(seed);
    for spec in default_body_params() {
        sampler.add_param(spec);
    }
    sampler.sample(n)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcg_new() {
        let lcg = Lcg::new(0);
        // State should be seed + 1 = 1 initially
        assert_eq!(lcg.state, 1);

        let lcg2 = Lcg::new(42);
        assert_eq!(lcg2.state, 43);
    }

    #[test]
    fn test_lcg_next_f32_range() {
        let mut lcg = Lcg::new(12345);
        for _ in 0..100 {
            let v = lcg.next_f32();
            assert!((0.0..1.0).contains(&v), "Expected [0,1), got {v}");
        }
    }

    #[test]
    fn test_lcg_next_range() {
        let mut lcg = Lcg::new(99);
        for _ in 0..100 {
            let v = lcg.next_range(2.0, 5.0);
            assert!((2.0..5.0).contains(&v), "Expected [2,5), got {v}");
        }
    }

    #[test]
    fn test_lcg_next_gaussian() {
        let mut lcg = Lcg::new(777);
        let mut sum = 0.0_f32;
        let n = 1000;
        for _ in 0..n {
            sum += lcg.next_gaussian(0.5, 0.1);
        }
        let mean = sum / n as f32;
        // Mean should be close to 0.5
        assert!((mean - 0.5).abs() < 0.05, "Mean {mean} not near 0.5");
    }

    #[test]
    fn test_van_der_corput_base2() {
        // n=1 in base 2: 1 -> 0.1 in binary = 0.5
        assert!((van_der_corput(1, 2) - 0.5).abs() < 1e-6);
        // n=2: 10 -> 0.01 = 0.25
        assert!((van_der_corput(2, 2) - 0.25).abs() < 1e-6);
        // n=3: 11 -> 0.11 = 0.75
        assert!((van_der_corput(3, 2) - 0.75).abs() < 1e-6);
        // n=4: 100 -> 0.001 = 0.125
        assert!((van_der_corput(4, 2) - 0.125).abs() < 1e-6);
        // n=0 should give 0
        assert_eq!(van_der_corput(0, 2), 0.0);
    }

    #[test]
    fn test_param_spec_new() {
        let spec = ParamSpec::new("height", 0.0, 1.0, 0.5);
        assert_eq!(spec.name, "height");
        assert_eq!(spec.min, 0.0);
        assert_eq!(spec.max, 1.0);
        assert_eq!(spec.default, 0.5);
        assert_eq!(spec.weight, 1.0);

        let spec2 = spec.with_weight(2.5);
        assert_eq!(spec2.weight, 2.5);
    }

    fn make_sampler(strategy: SamplingStrategy) -> DiversitySampler {
        let mut s = DiversitySampler::new(strategy).with_seed(42);
        s.add_param(ParamSpec::new("height", 0.0, 1.0, 0.5));
        s.add_param(ParamSpec::new("weight", 0.0, 1.0, 0.5));
        s.add_param(ParamSpec::new("age", 0.0, 1.0, 0.35));
        s
    }

    #[test]
    fn test_sampler_uniform() {
        let s = make_sampler(SamplingStrategy::Uniform);
        let samples = s.sample(20);
        assert_eq!(samples.len(), 20);
        for sample in &samples {
            assert_eq!(sample.len(), 3);
            for v in sample.values() {
                assert!(*v >= 0.0 && *v <= 1.0, "Out of range: {v}");
            }
        }
    }

    #[test]
    fn test_sampler_gaussian() {
        let s = make_sampler(SamplingStrategy::Gaussian { std_dev: 0.1 });
        let samples = s.sample(50);
        assert_eq!(samples.len(), 50);
        for sample in &samples {
            for v in sample.values() {
                assert!(*v >= 0.0 && *v <= 1.0, "Out of [0,1]: {v}");
            }
        }
    }

    #[test]
    fn test_sampler_latin_hypercube() {
        let s = make_sampler(SamplingStrategy::LatinHypercube);
        let samples = s.sample(10);
        assert_eq!(samples.len(), 10);
        // All values in range
        for sample in &samples {
            for v in sample.values() {
                assert!(*v >= 0.0 && *v <= 1.0, "Out of range: {v}");
            }
        }
        // LHS: each stratum [k/n, (k+1)/n] for each param should be covered
        // Check that no two samples have identical height values (very unlikely to collide)
        let heights: Vec<f32> = samples.iter().map(|m| *m.get("height").unwrap()).collect();
        // All values should be distinct (LHS guarantee)
        for i in 0..heights.len() {
            for j in (i + 1)..heights.len() {
                assert!(
                    (heights[i] - heights[j]).abs() > 1e-6,
                    "LHS produced duplicate heights at {i},{j}"
                );
            }
        }
    }

    #[test]
    fn test_sampler_low_discrepancy() {
        let s = make_sampler(SamplingStrategy::LowDiscrepancy);
        let samples = s.sample(16);
        assert_eq!(samples.len(), 16);
        for sample in &samples {
            for v in sample.values() {
                assert!(*v >= 0.0 && *v <= 1.0, "Out of range: {v}");
            }
        }
        // Check that height values follow van_der_corput(1..17, 2) pattern
        for (i, sample) in samples.iter().enumerate() {
            let expected = van_der_corput(i + 1, 2);
            let actual = *sample.get("height").unwrap();
            assert!(
                (actual - expected).abs() < 1e-5,
                "LD mismatch at i={i}: expected {expected}, got {actual}"
            );
        }
    }

    #[test]
    fn test_sample_near() {
        let mut s =
            DiversitySampler::new(SamplingStrategy::Gaussian { std_dev: 0.05 }).with_seed(7);
        s.add_param(ParamSpec::new("height", 0.0, 1.0, 0.5));
        s.add_param(ParamSpec::new("weight", 0.0, 1.0, 0.5));

        let base: HashMap<String, f32> =
            [("height".to_string(), 0.8), ("weight".to_string(), 0.2)].into();

        let samples = s.sample_near(&base, 30);
        assert_eq!(samples.len(), 30);

        // Most samples should be near the base values
        let mut near_count = 0;
        for sample in &samples {
            let h = sample["height"];
            let w = sample["weight"];
            if (h - 0.8).abs() < 0.3 && (w - 0.2).abs() < 0.3 {
                near_count += 1;
            }
        }
        assert!(
            near_count >= 20,
            "Expected most samples near base, got {near_count}/30"
        );
    }

    #[test]
    fn test_diversity_score() {
        // Identical samples -> score 0
        let s1: HashMap<String, f32> = [("a".to_string(), 0.5)].into();
        let identical = vec![s1.clone(), s1.clone()];
        assert_eq!(DiversitySampler::diversity_score(&identical), 0.0);

        // Two maximally spread samples
        let lo: HashMap<String, f32> = [("x".to_string(), 0.0), ("y".to_string(), 0.0)].into();
        let hi: HashMap<String, f32> = [("x".to_string(), 1.0), ("y".to_string(), 1.0)].into();
        let spread = vec![lo, hi];
        let score = DiversitySampler::diversity_score(&spread);
        // L2 distance = sqrt(1^2 + 1^2) = sqrt(2)
        assert!(
            (score - 2.0_f32.sqrt()).abs() < 1e-5,
            "Expected sqrt(2), got {score}"
        );

        // Single sample -> score 0
        let single = vec![s1];
        assert_eq!(DiversitySampler::diversity_score(&single), 0.0);
    }

    #[test]
    fn test_default_body_params() {
        let params = default_body_params();
        assert_eq!(params.len(), 7);

        let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"height"));
        assert!(names.contains(&"weight"));
        assert!(names.contains(&"muscle"));
        assert!(names.contains(&"age"));
        assert!(names.contains(&"bmi_factor"));
        assert!(names.contains(&"shoulder_width"));
        assert!(names.contains(&"hip_width"));

        for p in &params {
            assert_eq!(p.min, 0.0);
            assert_eq!(p.max, 1.0);
            assert!(p.default >= 0.0 && p.default <= 1.0);
        }
    }

    #[test]
    fn test_generate_population() {
        let pop = generate_population(20, 42);
        assert_eq!(pop.len(), 20);
        for individual in &pop {
            assert_eq!(individual.len(), 7);
            for v in individual.values() {
                assert!(*v >= 0.0 && *v <= 1.0, "Out of range: {v}");
            }
        }
        // Deterministic: same seed should give same result
        let pop2 = generate_population(20, 42);
        assert_eq!(pop.len(), pop2.len());
        for (a, b) in pop.iter().zip(pop2.iter()) {
            for (k, v) in a {
                assert_eq!(*v, *b.get(k).unwrap());
            }
        }
    }

    #[test]
    fn test_sample_with_extremes() {
        let s = make_sampler(SamplingStrategy::Uniform);
        let samples = s.sample_with_extremes(10);
        assert_eq!(samples.len(), 10);

        // First sample should be all minimums
        let first = &samples[0];
        for v in first.values() {
            assert_eq!(*v, 0.0, "First sample should be all mins");
        }

        // Second sample should be all maximums
        let second = &samples[1];
        for v in second.values() {
            assert_eq!(*v, 1.0, "Second sample should be all maxes");
        }

        // All values in range
        for sample in &samples {
            for v in sample.values() {
                assert!(*v >= 0.0 && *v <= 1.0);
            }
        }

        // Empty case
        let empty = DiversitySampler::new(SamplingStrategy::Uniform).sample_with_extremes(5);
        assert!(empty.is_empty());
    }
}
