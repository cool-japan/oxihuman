// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::params::ParamState;

/// Simple Linear Congruential Generator for deterministic randomness (no rand crate).
pub struct Lcg {
    state: u64,
}

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    /// Next pseudo-random u64.
    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters from Knuth/Numerical Recipes
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Uniform f32 in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 33) as f32 / (1u64 << 31) as f32
    }

    /// Box-Muller transform: N(0,1) sample.
    pub fn next_normal(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-10);
        let u2 = self.next_f32();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f32::consts::PI * u2;
        r * theta.cos()
    }

    /// Sample from N(mean, std), clamped to [0, 1].
    pub fn sample_normal(&mut self, mean: f32, std: f32) -> f32 {
        (mean + self.next_normal() * std).clamp(0.0, 1.0)
    }
}

/// Anthropometric distribution for a population group.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnthroDistribution {
    pub name: String,
    /// Mean and std for height parameter [0..1]
    pub height_mean: f32,
    pub height_std: f32,
    /// Mean and std for weight parameter [0..1]
    pub weight_mean: f32,
    pub weight_std: f32,
    /// Mean and std for muscle parameter [0..1]
    pub muscle_mean: f32,
    pub muscle_std: f32,
    /// Mean and std for age parameter [0..1]
    pub age_mean: f32,
    pub age_std: f32,
}

impl AnthroDistribution {
    /// Average adult (no demographic specifics).
    pub fn average_adult() -> Self {
        Self {
            name: "average_adult".into(),
            height_mean: 0.50,
            height_std: 0.15,
            weight_mean: 0.45,
            weight_std: 0.18,
            muscle_mean: 0.35,
            muscle_std: 0.15,
            age_mean: 0.40,
            age_std: 0.20,
        }
    }

    /// Young adult (20-35 years).
    pub fn young_adult() -> Self {
        Self {
            name: "young_adult".into(),
            height_mean: 0.52,
            height_std: 0.12,
            weight_mean: 0.42,
            weight_std: 0.15,
            muscle_mean: 0.40,
            muscle_std: 0.12,
            age_mean: 0.25,
            age_std: 0.08,
        }
    }

    /// Older adult (55+ years).
    pub fn older_adult() -> Self {
        Self {
            name: "older_adult".into(),
            height_mean: 0.46,
            height_std: 0.10,
            weight_mean: 0.52,
            weight_std: 0.15,
            muscle_mean: 0.28,
            muscle_std: 0.10,
            age_mean: 0.75,
            age_std: 0.10,
        }
    }

    /// Athletic build.
    pub fn athletic() -> Self {
        Self {
            name: "athletic".into(),
            height_mean: 0.55,
            height_std: 0.10,
            weight_mean: 0.48,
            weight_std: 0.10,
            muscle_mean: 0.70,
            muscle_std: 0.12,
            age_mean: 0.35,
            age_std: 0.10,
        }
    }

    /// Heavier build.
    pub fn heavy() -> Self {
        Self {
            name: "heavy".into(),
            height_mean: 0.45,
            height_std: 0.12,
            weight_mean: 0.75,
            weight_std: 0.12,
            muscle_mean: 0.25,
            muscle_std: 0.10,
            age_mean: 0.45,
            age_std: 0.15,
        }
    }

    /// Sample a `ParamState` from this distribution using the provided RNG.
    pub fn sample(&self, rng: &mut Lcg) -> ParamState {
        ParamState::new(
            rng.sample_normal(self.height_mean, self.height_std),
            rng.sample_normal(self.weight_mean, self.weight_std),
            rng.sample_normal(self.muscle_mean, self.muscle_std),
            rng.sample_normal(self.age_mean, self.age_std),
        )
    }

    /// Generate `count` diverse `ParamState` values (evenly spaced through the distribution).
    ///
    /// Uses systematic sampling: height is spread across evenly-spaced quantiles for
    /// better coverage, while other parameters are sampled randomly.
    pub fn sample_diverse(&self, count: usize, seed: u64) -> Vec<ParamState> {
        if count == 0 {
            return Vec::new();
        }
        let mut rng = Lcg::new(seed);
        (0..count)
            .map(|i| {
                // Evenly space height across the range for diversity.
                let t = if count == 1 {
                    0.5_f32
                } else {
                    i as f32 / (count - 1) as f32
                };
                let height = (self.height_mean - 2.0 * self.height_std + t * 4.0 * self.height_std)
                    .clamp(0.0, 1.0);
                ParamState::new(
                    height,
                    rng.sample_normal(self.weight_mean, self.weight_std),
                    rng.sample_normal(self.muscle_mean, self.muscle_std),
                    rng.sample_normal(self.age_mean, self.age_std),
                )
            })
            .collect()
    }
}

/// Randomizer that generates varied body configurations.
pub struct BodyRandomizer {
    pub distribution: AnthroDistribution,
    rng: Lcg,
}

impl BodyRandomizer {
    pub fn new(distribution: AnthroDistribution, seed: u64) -> Self {
        Self {
            distribution,
            rng: Lcg::new(seed),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = Lcg::new(seed);
        self
    }

    /// Generate the next random `ParamState`.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> ParamState {
        self.distribution.sample(&mut self.rng)
    }

    /// Generate `count` random `ParamState` values.
    pub fn generate(&mut self, count: usize) -> Vec<ParamState> {
        (0..count).map(|_| self.next()).collect()
    }

    /// Standard randomizer with `average_adult` distribution.
    pub fn standard(seed: u64) -> Self {
        Self::new(AnthroDistribution::average_adult(), seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_deterministic() {
        let mut a = Lcg::new(42);
        let mut b = Lcg::new(42);
        for _ in 0..20 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn lcg_next_f32_in_range() {
        let mut rng = Lcg::new(99);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!(v >= 0.0, "value {v} below 0");
            assert!(v < 1.0, "value {v} not below 1");
        }
    }

    #[test]
    fn sample_clamps_to_unit() {
        let dist = AnthroDistribution::average_adult();
        let mut rng = Lcg::new(7);
        for _ in 0..500 {
            let p = dist.sample(&mut rng);
            assert!(
                (0.0..=1.0).contains(&p.height),
                "height out of range: {}",
                p.height
            );
            assert!(
                (0.0..=1.0).contains(&p.weight),
                "weight out of range: {}",
                p.weight
            );
            assert!(
                (0.0..=1.0).contains(&p.muscle),
                "muscle out of range: {}",
                p.muscle
            );
            assert!((0.0..=1.0).contains(&p.age), "age out of range: {}", p.age);
        }
    }

    #[test]
    fn generate_count_correct() {
        let mut randomizer = BodyRandomizer::standard(1);
        assert_eq!(randomizer.generate(10).len(), 10);
    }

    #[test]
    fn different_seeds_different_results() {
        let mut r1 = BodyRandomizer::standard(1);
        let mut r2 = BodyRandomizer::standard(2);
        let p1 = r1.next();
        let p2 = r2.next();
        // It is astronomically unlikely that two different seeds yield identical results.
        assert!(
            (p1.height - p2.height).abs() > 1e-6
                || (p1.weight - p2.weight).abs() > 1e-6
                || (p1.muscle - p2.muscle).abs() > 1e-6
                || (p1.age - p2.age).abs() > 1e-6,
            "Different seeds produced identical ParamState"
        );
    }

    #[test]
    fn sample_diverse_count_correct() {
        let dist = AnthroDistribution::average_adult();
        assert_eq!(dist.sample_diverse(5, 42).len(), 5);
    }

    #[test]
    fn athletic_has_high_muscle() {
        assert!(AnthroDistribution::athletic().muscle_mean > 0.5);
    }

    #[test]
    fn older_adult_has_high_age() {
        assert!(AnthroDistribution::older_adult().age_mean > 0.6);
    }

    #[test]
    fn standard_randomizer_works() {
        // Should not panic.
        let _ = BodyRandomizer::standard(42).next();
    }
}
