// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Expression randomizer — deterministic pseudo-random expression weight sampling from a seed.

/// Config for expression randomization.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionRandomizerConfig {
    /// Number of expression channels.
    pub channel_count: usize,
    /// Amplitude scale applied to each sample (0..=1).
    pub amplitude: f32,
    /// Sparsity: fraction of channels set to zero (0 = dense, 1 = all zero).
    pub sparsity: f32,
}

impl Default for ExpressionRandomizerConfig {
    fn default() -> Self {
        Self {
            channel_count: 8,
            amplitude: 0.6,
            sparsity: 0.3,
        }
    }
}

/// Lightweight PCG-style 32-bit pseudo-random generator (deterministic, no external deps).
fn pcg32(state: &mut u64) -> u32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let xor = ((*state >> 18) ^ *state) >> 27;
    let rot = (*state >> 59) as u32;
    ((xor.wrapping_shr(rot)) | (xor.wrapping_shl(u64::BITS.wrapping_sub(rot)))) as u32
}

fn pcg_f32(state: &mut u64) -> f32 {
    // Map u32 to [0, 1)
    (pcg32(state) >> 8) as f32 / 16_777_216.0
}

/// Sample a randomized expression weight vector.
#[allow(dead_code)]
pub fn sample_expression(seed: u64, cfg: &ExpressionRandomizerConfig) -> Vec<f32> {
    let mut rng = seed ^ 0xDEAD_BEEF_CAFE_1234;
    (0..cfg.channel_count)
        .map(|_| {
            let sparse_val = pcg_f32(&mut rng);
            if sparse_val < cfg.sparsity {
                0.0
            } else {
                let v = pcg_f32(&mut rng);
                // Signed: map [0,1) -> [-1,1)
                let signed = v * 2.0 - 1.0;
                (signed * cfg.amplitude).clamp(-1.0, 1.0)
            }
        })
        .collect()
}

/// Sample and return only the non-zero channels as (index, weight) pairs.
#[allow(dead_code)]
pub fn sample_sparse_expression(seed: u64, cfg: &ExpressionRandomizerConfig) -> Vec<(usize, f32)> {
    sample_expression(seed, cfg)
        .into_iter()
        .enumerate()
        .filter(|(_, w)| w.abs() > 1e-6)
        .collect()
}

/// Blend between two sampled expressions at interpolation factor t.
#[allow(dead_code)]
pub fn blend_sampled(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x * inv + y * t)
        .collect()
}

/// Returns the L1 norm of a sampled expression vector.
#[allow(dead_code)]
pub fn expression_energy(weights: &[f32]) -> f32 {
    weights.iter().map(|w| w.abs()).sum()
}

/// Returns the index of the channel with the largest absolute weight.
#[allow(dead_code)]
pub fn dominant_channel(weights: &[f32]) -> Option<usize> {
    weights
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.abs()
                .partial_cmp(&b.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// Normalize weights to unit L2 norm.  Returns zeros if input is zero.
#[allow(dead_code)]
pub fn normalize_expression(weights: &[f32]) -> Vec<f32> {
    let l2: f32 = weights.iter().map(|w| w * w).sum::<f32>().sqrt();
    if l2 < 1e-8 {
        weights.to_vec()
    } else {
        weights.iter().map(|w| w / l2).collect()
    }
}

/// Serialise weights to a compact JSON array string.
#[allow(dead_code)]
pub fn expression_to_json(weights: &[f32]) -> String {
    let inner: Vec<String> = weights.iter().map(|w| format!("{:.4}", w)).collect();
    format!("[{}]", inner.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ExpressionRandomizerConfig {
        ExpressionRandomizerConfig::default()
    }

    #[test]
    fn sample_length_matches_config() {
        let w = sample_expression(42, &cfg());
        assert_eq!(w.len(), cfg().channel_count);
    }

    #[test]
    fn deterministic_same_seed() {
        let a = sample_expression(99, &cfg());
        let b = sample_expression(99, &cfg());
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_differ() {
        let a = sample_expression(1, &cfg());
        let b = sample_expression(2, &cfg());
        assert_ne!(a, b);
    }

    #[test]
    fn weights_in_range() {
        let w = sample_expression(7, &cfg());
        assert!(w.iter().all(|v| (-1.0..=1.0).contains(v)));
    }

    #[test]
    fn sparse_expression_fewer_entries() {
        let full = sample_expression(55, &cfg());
        let sparse = sample_sparse_expression(55, &cfg());
        assert!(sparse.len() <= full.len());
    }

    #[test]
    fn blend_midpoint() {
        let a = vec![0.0f32; 4];
        let b = vec![1.0f32; 4];
        let m = blend_sampled(&a, &b, 0.5);
        assert!(m.iter().all(|v| (v - 0.5).abs() < 1e-6));
    }

    #[test]
    fn energy_zero_for_zeros() {
        let z = vec![0.0f32; 6];
        assert!(expression_energy(&z) < 1e-8);
    }

    #[test]
    fn dominant_channel_found() {
        let mut w = vec![0.1f32, 0.9, 0.3];
        w[1] = 0.9;
        assert_eq!(dominant_channel(&w), Some(1));
    }

    #[test]
    fn normalize_unit_length() {
        let w = vec![3.0f32, 4.0];
        let n = normalize_expression(&w);
        let l2: f32 = n.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((l2 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_array_format() {
        let w = vec![0.5f32, -0.5];
        let j = expression_to_json(&w);
        assert!(j.starts_with('[') && j.ends_with(']'));
    }
}
