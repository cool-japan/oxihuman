// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

/// A parameter map: name → value.
pub type ParamMap = HashMap<String, f32>;

/// Defines allowed ranges and mutation behavior for one parameter.
pub struct ParamSpec {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub step: f32,     // quantization step (0 = continuous)
    pub mutable: bool, // can this be mutated?
}

/// The mutation configuration.
pub struct MutationConfig {
    pub mutation_rate: f32,         // probability each param mutates [0,1]
    pub mutation_scale: f32,        // std dev of Gaussian mutation as fraction of range
    pub clamp_to_range: bool,       // clamp output to [min, max]
    pub preserve_proportions: bool, // scale all mutated params proportionally
}

/// Result of one generation step.
pub struct MutationResult {
    pub params: ParamMap,
    pub mutated_keys: Vec<String>,
    pub mutation_deltas: HashMap<String, f32>,
}

// ── LCG helpers (no rand crate) ─────────────────────────────────────────────

fn lcg_step(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

fn lcg_f32(state: &mut u64) -> f32 {
    *state = lcg_step(*state);
    (*state >> 33) as f32 / (u32::MAX as f32)
}

fn lcg_normal(state: &mut u64) -> f32 {
    // Box-Muller transform
    let u1 = lcg_f32(state).max(1e-10);
    let u2 = lcg_f32(state);
    (-2.0_f32 * u1.ln()).sqrt() * (2.0_f32 * std::f32::consts::PI * u2).cos()
}

// ── MutationEngine ───────────────────────────────────────────────────────────

pub struct MutationEngine {
    specs: Vec<ParamSpec>,
    config: MutationConfig,
}

impl MutationEngine {
    pub fn new(specs: Vec<ParamSpec>, config: MutationConfig) -> Self {
        Self { specs, config }
    }

    /// LCG-based deterministic mutation.
    pub fn mutate(&self, params: &ParamMap, seed: u64) -> MutationResult {
        let mut state = seed;
        let mut out = params.clone();
        let mut mutated_keys = Vec::new();
        let mut mutation_deltas: HashMap<String, f32> = HashMap::new();

        for spec in &self.specs {
            if !spec.mutable {
                continue;
            }
            let roll = lcg_f32(&mut state);
            if roll >= self.config.mutation_rate {
                continue;
            }
            let range = spec.max - spec.min;
            let noise = lcg_normal(&mut state) * self.config.mutation_scale * range;
            let base = *out.get(&spec.name).unwrap_or(&0.0);
            let mut new_val = base + noise;

            if self.config.clamp_to_range {
                new_val = new_val.clamp(spec.min, spec.max);
            }

            if spec.step > 0.0 {
                new_val = (new_val / spec.step).round() * spec.step;
                if self.config.clamp_to_range {
                    new_val = new_val.clamp(spec.min, spec.max);
                }
            }

            let delta = new_val - base;
            out.insert(spec.name.clone(), new_val);
            mutated_keys.push(spec.name.clone());
            mutation_deltas.insert(spec.name.clone(), delta);
        }

        // Preserve proportions: scale all mutated params so their sum stays the same.
        if self.config.preserve_proportions && !mutated_keys.is_empty() {
            let old_sum: f32 = mutated_keys
                .iter()
                .map(|k| params.get(k).copied().unwrap_or(0.0))
                .sum();
            let new_sum: f32 = mutated_keys
                .iter()
                .map(|k| out.get(k).copied().unwrap_or(0.0))
                .sum();
            if new_sum.abs() > 1e-9 {
                let scale = old_sum / new_sum;
                for key in &mutated_keys {
                    let v = out.get(key).copied().unwrap_or(0.0) * scale;
                    let spec = self.specs.iter().find(|s| &s.name == key);
                    let v = if self.config.clamp_to_range {
                        if let Some(s) = spec {
                            v.clamp(s.min, s.max)
                        } else {
                            v
                        }
                    } else {
                        v
                    };
                    out.insert(key.clone(), v);
                    // Update deltas after proportion adjustment
                    let base = params.get(key).copied().unwrap_or(0.0);
                    mutation_deltas.insert(key.clone(), v - base);
                }
            }
        }

        MutationResult {
            params: out,
            mutated_keys,
            mutation_deltas,
        }
    }

    /// Uniform crossover: for each param, pick from parent_a or parent_b with 50/50 probability.
    pub fn crossover(&self, parent_a: &ParamMap, parent_b: &ParamMap, seed: u64) -> ParamMap {
        let mut state = seed;
        let mut out = ParamMap::new();
        for spec in &self.specs {
            let roll = lcg_f32(&mut state);
            let val = if roll < 0.5 {
                parent_a.get(&spec.name).copied().unwrap_or(spec.min)
            } else {
                parent_b.get(&spec.name).copied().unwrap_or(spec.min)
            };
            out.insert(spec.name.clone(), val);
        }
        out
    }

    /// Blend crossover: interpolate each param as `a + t*(b-a)`, clamp to range.
    pub fn blend_crossover(&self, parent_a: &ParamMap, parent_b: &ParamMap, t: f32) -> ParamMap {
        let mut out = ParamMap::new();
        for spec in &self.specs {
            let a = parent_a.get(&spec.name).copied().unwrap_or(spec.min);
            let b = parent_b.get(&spec.name).copied().unwrap_or(spec.min);
            let v = (a + t * (b - a)).clamp(spec.min, spec.max);
            out.insert(spec.name.clone(), v);
        }
        out
    }

    /// Generate random params uniformly in [min, max] per spec.
    pub fn generate_random(&self, seed: u64) -> ParamMap {
        let mut state = seed;
        let mut out = ParamMap::new();
        for spec in &self.specs {
            let v = spec.min + lcg_f32(&mut state) * (spec.max - spec.min);
            let v = if spec.step > 0.0 {
                (v / spec.step).round() * spec.step
            } else {
                v
            };
            let v = v.clamp(spec.min, spec.max);
            out.insert(spec.name.clone(), v);
        }
        out
    }
}

// ── Free functions ───────────────────────────────────────────────────────────

/// Rank population by sum of squared normalized differences to `target`.
/// Returns indices sorted best (smallest error) first.
pub fn fitness_rank(population: &[ParamMap], target: &ParamMap, specs: &[ParamSpec]) -> Vec<usize> {
    let mut scores: Vec<(usize, f32)> = population
        .iter()
        .enumerate()
        .map(|(i, pm)| {
            let score: f32 = specs
                .iter()
                .map(|spec| {
                    let range = (spec.max - spec.min).max(1e-9);
                    let v = pm.get(&spec.name).copied().unwrap_or(0.0);
                    let t = target.get(&spec.name).copied().unwrap_or(0.0);
                    let diff = (v - t) / range;
                    diff * diff
                })
                .sum();
            (i, score)
        })
        .collect();

    scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    scores.into_iter().map(|(i, _)| i).collect()
}

/// Tournament selection: pick the best of k random candidates.
#[allow(clippy::too_many_arguments)]
pub fn tournament_select<'a>(
    population: &'a [ParamMap],
    fitness: &[f32],
    k: usize,
    seed: u64,
) -> &'a ParamMap {
    assert!(!population.is_empty(), "population must not be empty");
    assert_eq!(
        population.len(),
        fitness.len(),
        "fitness length must match population"
    );
    let k = k.min(population.len()).max(1);

    let mut state = seed;
    let n = population.len();

    let mut best_idx = {
        let r = lcg_f32(&mut state);
        (r * n as f32).floor() as usize % n
    };

    for _ in 1..k {
        let r = lcg_f32(&mut state);
        let idx = (r * n as f32).floor() as usize % n;
        // Lower fitness score = better (as used in fitness_rank)
        if fitness[idx] < fitness[best_idx] {
            best_idx = idx;
        }
    }

    &population[best_idx]
}

/// Return 10 human body parameter specs, all in [0, 1].
pub fn default_human_specs() -> Vec<ParamSpec> {
    let names = [
        "height",
        "weight",
        "muscle",
        "age",
        "head_size",
        "neck_length",
        "shoulder_width",
        "hip_width",
        "leg_length",
        "arm_length",
    ];
    names
        .iter()
        .map(|&name| ParamSpec {
            name: name.to_string(),
            min: 0.0,
            max: 1.0,
            step: 0.0,
            mutable: true,
        })
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_specs() -> Vec<ParamSpec> {
        vec![
            ParamSpec {
                name: "a".into(),
                min: 0.0,
                max: 1.0,
                step: 0.0,
                mutable: true,
            },
            ParamSpec {
                name: "b".into(),
                min: 0.0,
                max: 2.0,
                step: 0.0,
                mutable: true,
            },
            ParamSpec {
                name: "c".into(),
                min: -1.0,
                max: 1.0,
                step: 0.0,
                mutable: false,
            },
        ]
    }

    fn simple_config(rate: f32) -> MutationConfig {
        MutationConfig {
            mutation_rate: rate,
            mutation_scale: 0.1,
            clamp_to_range: true,
            preserve_proportions: false,
        }
    }

    fn simple_params() -> ParamMap {
        let mut m = ParamMap::new();
        m.insert("a".into(), 0.5);
        m.insert("b".into(), 1.0);
        m.insert("c".into(), 0.0);
        m
    }

    // ── 1. new ───────────────────────────────────────────────────────────────
    #[test]
    fn test_new_stores_specs_and_config() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.5));
        assert_eq!(engine.specs.len(), 3);
        assert!((engine.config.mutation_rate - 0.5).abs() < 1e-6);
    }

    // ── 2. mutate: rate=0 produces no mutations ──────────────────────────────
    #[test]
    fn test_mutate_rate_zero_no_mutations() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.0));
        let params = simple_params();
        let result = engine.mutate(&params, 42);
        assert!(result.mutated_keys.is_empty());
        assert!(result.mutation_deltas.is_empty());
        assert_eq!(result.params["a"], 0.5);
        assert_eq!(result.params["b"], 1.0);
    }

    // ── 3. mutate: rate=1 mutates all mutable params ─────────────────────────
    #[test]
    fn test_mutate_rate_one_mutates_all_mutable() {
        let engine = MutationEngine::new(simple_specs(), simple_config(1.0));
        let params = simple_params();
        let result = engine.mutate(&params, 7);
        // "c" is not mutable, so only "a" and "b" can be mutated
        for key in &result.mutated_keys {
            assert_ne!(key, "c", "immutable param 'c' must not be mutated");
        }
        // With rate=1, both mutable params should appear in mutated_keys
        assert!(result.mutated_keys.contains(&"a".to_string()));
        assert!(result.mutated_keys.contains(&"b".to_string()));
    }

    // ── 4. mutate: deterministic with same seed ───────────────────────────────
    #[test]
    fn test_mutate_deterministic() {
        let engine = MutationEngine::new(simple_specs(), simple_config(1.0));
        let params = simple_params();
        let r1 = engine.mutate(&params, 12345);
        let r2 = engine.mutate(&params, 12345);
        assert_eq!(r1.params["a"], r2.params["a"]);
        assert_eq!(r1.params["b"], r2.params["b"]);
        assert_eq!(r1.mutated_keys, r2.mutated_keys);
    }

    // ── 5. mutate: different seeds give different results ────────────────────
    #[test]
    fn test_mutate_different_seeds() {
        let engine = MutationEngine::new(simple_specs(), simple_config(1.0));
        let params = simple_params();
        let r1 = engine.mutate(&params, 1);
        let r2 = engine.mutate(&params, 9999999);
        // Very unlikely to be identical
        let same = r1.params["a"] == r2.params["a"] && r1.params["b"] == r2.params["b"];
        assert!(!same, "different seeds should produce different mutations");
    }

    // ── 6. mutate: clamping keeps values in range ────────────────────────────
    #[test]
    fn test_mutate_clamp_to_range() {
        let specs = vec![ParamSpec {
            name: "x".into(),
            min: 0.0,
            max: 1.0,
            step: 0.0,
            mutable: true,
        }];
        let config = MutationConfig {
            mutation_rate: 1.0,
            mutation_scale: 10.0, // extreme mutation
            clamp_to_range: true,
            preserve_proportions: false,
        };
        let engine = MutationEngine::new(specs, config);
        let mut params = ParamMap::new();
        params.insert("x".into(), 0.5);
        for seed in 0u64..50 {
            let result = engine.mutate(&params, seed);
            let v = result.params["x"];
            assert!((0.0..=1.0).contains(&v), "value {v} out of [0,1]");
        }
    }

    // ── 7. mutate: quantization step ─────────────────────────────────────────
    #[test]
    fn test_mutate_step_quantization() {
        let specs = vec![ParamSpec {
            name: "q".into(),
            min: 0.0,
            max: 1.0,
            step: 0.25,
            mutable: true,
        }];
        let config = MutationConfig {
            mutation_rate: 1.0,
            mutation_scale: 0.3,
            clamp_to_range: true,
            preserve_proportions: false,
        };
        let engine = MutationEngine::new(specs, config);
        let mut params = ParamMap::new();
        params.insert("q".into(), 0.5);
        for seed in 0u64..30 {
            let result = engine.mutate(&params, seed);
            let v = result.params["q"];
            let quantized = (v / 0.25).round() * 0.25;
            assert!(
                (v - quantized).abs() < 1e-5,
                "value {v} not quantized to 0.25"
            );
        }
    }

    // ── 8. mutate: mutation_deltas match actual change ────────────────────────
    #[test]
    fn test_mutate_deltas_correct() {
        let engine = MutationEngine::new(simple_specs(), simple_config(1.0));
        let params = simple_params();
        let result = engine.mutate(&params, 42);
        for (key, delta) in &result.mutation_deltas {
            let original = params.get(key).copied().unwrap_or(0.0);
            let new_val = result.params.get(key).copied().unwrap_or(0.0);
            assert!((delta - (new_val - original)).abs() < 1e-5);
        }
    }

    // ── 9. crossover: output contains values from one of the parents ─────────
    #[test]
    fn test_crossover_values_from_parents() {
        let specs = simple_specs();
        let engine = MutationEngine::new(specs, simple_config(0.5));
        let mut pa = ParamMap::new();
        let mut pb = ParamMap::new();
        pa.insert("a".into(), 0.1);
        pa.insert("b".into(), 0.2);
        pa.insert("c".into(), -0.5);
        pb.insert("a".into(), 0.9);
        pb.insert("b".into(), 1.8);
        pb.insert("c".into(), 0.5);

        let child = engine.crossover(&pa, &pb, 99);
        for key in ["a", "b", "c"] {
            let v = child[key];
            let a_val = pa[key];
            let b_val = pb[key];
            assert!(
                (v - a_val).abs() < 1e-6 || (v - b_val).abs() < 1e-6,
                "key '{key}': value {v} is neither from parent_a ({a_val}) nor parent_b ({b_val})"
            );
        }
    }

    // ── 10. crossover: deterministic ─────────────────────────────────────────
    #[test]
    fn test_crossover_deterministic() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.5));
        let pa = simple_params();
        let mut pb = simple_params();
        pb.insert("a".into(), 0.9);
        let c1 = engine.crossover(&pa, &pb, 42);
        let c2 = engine.crossover(&pa, &pb, 42);
        assert_eq!(c1["a"], c2["a"]);
        assert_eq!(c1["b"], c2["b"]);
    }

    // ── 11. blend_crossover: t=0 returns parent_a ────────────────────────────
    #[test]
    fn test_blend_crossover_t0_is_a() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.5));
        let pa = simple_params();
        let mut pb = simple_params();
        pb.insert("a".into(), 0.9);
        pb.insert("b".into(), 1.5);
        let child = engine.blend_crossover(&pa, &pb, 0.0);
        assert!((child["a"] - pa["a"]).abs() < 1e-5);
        assert!((child["b"] - pa["b"]).abs() < 1e-5);
    }

    // ── 12. blend_crossover: t=1 returns parent_b ────────────────────────────
    #[test]
    fn test_blend_crossover_t1_is_b() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.5));
        let pa = simple_params();
        let mut pb = simple_params();
        pb.insert("a".into(), 0.9);
        pb.insert("b".into(), 1.5);
        let child = engine.blend_crossover(&pa, &pb, 1.0);
        assert!((child["a"] - pb["a"]).abs() < 1e-5);
        assert!((child["b"] - pb["b"]).abs() < 1e-5);
    }

    // ── 13. blend_crossover: t=0.5 is midpoint ───────────────────────────────
    #[test]
    fn test_blend_crossover_midpoint() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.5));
        let mut pa = ParamMap::new();
        let mut pb = ParamMap::new();
        pa.insert("a".into(), 0.0);
        pa.insert("b".into(), 0.0);
        pa.insert("c".into(), -1.0);
        pb.insert("a".into(), 1.0);
        pb.insert("b".into(), 2.0);
        pb.insert("c".into(), 1.0);
        let child = engine.blend_crossover(&pa, &pb, 0.5);
        assert!((child["a"] - 0.5).abs() < 1e-5);
        assert!((child["b"] - 1.0).abs() < 1e-5);
    }

    // ── 14. blend_crossover: clamps to range ─────────────────────────────────
    #[test]
    fn test_blend_crossover_clamps() {
        let engine = MutationEngine::new(simple_specs(), simple_config(0.5));
        let mut pa = ParamMap::new();
        let mut pb = ParamMap::new();
        pa.insert("a".into(), 0.8);
        pa.insert("b".into(), 1.0);
        pa.insert("c".into(), 0.0);
        pb.insert("a".into(), 1.0);
        pb.insert("b".into(), 2.0);
        pb.insert("c".into(), 0.0);
        let child = engine.blend_crossover(&pa, &pb, 2.0); // t=2 would exceed range
        assert!(child["a"] <= 1.0);
        assert!(child["b"] <= 2.0);
    }

    // ── 15. generate_random: values in [min, max] ────────────────────────────
    #[test]
    fn test_generate_random_in_range() {
        let engine = MutationEngine::new(default_human_specs(), simple_config(0.5));
        for seed in 0u64..20 {
            let pm = engine.generate_random(seed);
            for spec in default_human_specs() {
                let v = pm[&spec.name];
                assert!(
                    v >= spec.min && v <= spec.max,
                    "param '{}' = {} out of range",
                    spec.name,
                    v
                );
            }
        }
    }

    // ── 16. generate_random: all spec keys present ───────────────────────────
    #[test]
    fn test_generate_random_all_keys_present() {
        let specs = default_human_specs();
        let engine = MutationEngine::new(specs, simple_config(0.5));
        let pm = engine.generate_random(123);
        for spec in default_human_specs() {
            assert!(pm.contains_key(&spec.name), "missing key '{}'", spec.name);
        }
    }

    // ── 17. fitness_rank: perfect match ranks first ───────────────────────────
    #[test]
    fn test_fitness_rank_perfect_match_first() {
        let specs = default_human_specs();
        let mut target = ParamMap::new();
        for s in &specs {
            target.insert(s.name.clone(), 0.5);
        }

        let mut exact = ParamMap::new();
        for s in &specs {
            exact.insert(s.name.clone(), 0.5);
        }
        let mut far = ParamMap::new();
        for s in &specs {
            far.insert(s.name.clone(), 1.0);
        }

        let population = vec![far, exact];
        let ranked = fitness_rank(&population, &target, &specs);
        // index 1 (exact match) should rank first
        assert_eq!(ranked[0], 1);
    }

    // ── 18. tournament_select: returns element from population ───────────────
    #[test]
    fn test_tournament_select_returns_population_member() {
        let specs = default_human_specs();
        let engine = MutationEngine::new(specs, simple_config(0.5));
        let pop: Vec<ParamMap> = (0..5).map(|s| engine.generate_random(s)).collect();
        let fitness: Vec<f32> = (0..5).map(|i| i as f32).collect();
        let selected = tournament_select(&pop, &fitness, 3, 42);
        // The result must be one of the population members
        let found = pop.iter().any(|pm| pm == selected);
        assert!(found, "selected member not found in population");
    }

    // ── 19. tournament_select: picks lowest fitness ───────────────────────────
    #[test]
    fn test_tournament_select_prefers_best_fitness() {
        let specs = default_human_specs();
        let engine = MutationEngine::new(specs, simple_config(0.5));
        let pop: Vec<ParamMap> = (0..10).map(|s| engine.generate_random(s)).collect();
        // fitness[0] is the absolute best
        let fitness: Vec<f32> = (0..10).map(|i| i as f32 * 10.0).collect();
        // With k = population size, best must always win
        let selected = tournament_select(&pop, &fitness, 10, 77);
        assert_eq!(selected, &pop[0]);
    }

    // ── 20. default_human_specs: 10 specs, all [0,1] ─────────────────────────
    #[test]
    fn test_default_human_specs_count_and_range() {
        let specs = default_human_specs();
        assert_eq!(specs.len(), 10);
        for spec in &specs {
            assert!((spec.min - 0.0).abs() < 1e-6);
            assert!((spec.max - 1.0).abs() < 1e-6);
            assert!(spec.mutable);
        }
    }

    // ── 21. lcg_f32: values in [0, 1) ─────────────────────────────────────────
    #[test]
    fn test_lcg_f32_range() {
        let mut state = 123456789u64;
        for _ in 0..1000 {
            let v = lcg_f32(&mut state);
            assert!((0.0..1.0 + 1e-5).contains(&v), "lcg_f32 out of range: {v}");
        }
    }

    // ── 22. lcg_normal: reasonable distribution ───────────────────────────────
    #[test]
    fn test_lcg_normal_distribution() {
        let mut state = 42u64;
        let samples: Vec<f32> = (0..1000).map(|_| lcg_normal(&mut state)).collect();
        let mean = samples.iter().sum::<f32>() / samples.len() as f32;
        let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        assert!(mean.abs() < 0.2, "mean {mean} too far from 0");
        assert!((var - 1.0).abs() < 1.0, "variance {var} too far from 1");
    }
}
