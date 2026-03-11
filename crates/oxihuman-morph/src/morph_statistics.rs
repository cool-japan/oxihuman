// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Statistics computed over a slice of morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MorphStats {
    pub mean: f32,
    pub std_dev: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub nonzero_count: usize,
}

/// Compute descriptive statistics for a weight slice.
#[allow(dead_code)]
pub fn compute_morph_stats(weights: &[f32]) -> MorphStats {
    if weights.is_empty() {
        return MorphStats::default();
    }
    let n = weights.len() as f32;
    let mean = weights.iter().copied().sum::<f32>() / n;
    let variance = weights.iter().map(|&w| (w - mean) * (w - mean)).sum::<f32>() / n;
    let std_dev = variance.sqrt();
    let min_val = weights.iter().copied().fold(f32::INFINITY, f32::min);
    let max_val = weights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let nonzero_count = weights.iter().filter(|&&w| w.abs() > 1e-8).count();
    MorphStats { mean, std_dev, min_val, max_val, nonzero_count }
}

/// Compute entropy of a weight distribution (normalized to [0,1] beforehand).
#[allow(dead_code)]
pub fn weights_entropy(weights: &[f32]) -> f32 {
    let sum: f32 = weights.iter().map(|&w| w.abs()).sum();
    if sum < 1e-9 {
        return 0.0;
    }
    weights
        .iter()
        .map(|&w| {
            let p = w.abs() / sum;
            if p < 1e-9 { 0.0 } else { -p * p.ln() }
        })
        .sum()
}

/// Compute L1 norm (sum of absolute values).
#[allow(dead_code)]
pub fn weights_l1_norm(weights: &[f32]) -> f32 {
    weights.iter().map(|&w| w.abs()).sum()
}

/// Compute L2 norm (Euclidean length).
#[allow(dead_code)]
pub fn weights_l2_norm(weights: &[f32]) -> f32 {
    weights.iter().map(|&w| w * w).sum::<f32>().sqrt()
}

/// Count weights whose absolute value exceeds the threshold.
#[allow(dead_code)]
pub fn active_morph_count(weights: &[f32], threshold: f32) -> usize {
    weights.iter().filter(|&&w| w.abs() > threshold).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_weights_returns_default() {
        let s = compute_morph_stats(&[]);
        assert!((s.mean).abs() < 1e-9);
    }

    #[test]
    fn mean_correct() {
        let s = compute_morph_stats(&[0.0, 0.5, 1.0]);
        assert!((s.mean - 0.5).abs() < 1e-5);
    }

    #[test]
    fn min_max_correct() {
        let s = compute_morph_stats(&[0.1, 0.9, 0.5]);
        assert!((s.min_val - 0.1).abs() < 1e-5);
        assert!((s.max_val - 0.9).abs() < 1e-5);
    }

    #[test]
    fn nonzero_count_correct() {
        let s = compute_morph_stats(&[0.0, 0.5, 0.0, 0.3]);
        assert_eq!(s.nonzero_count, 2);
    }

    #[test]
    fn weights_l1_norm_sum() {
        let norm = weights_l1_norm(&[0.25, 0.25, 0.25, 0.25]);
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_l2_norm_unit_vector() {
        let v = std::f32::consts::FRAC_1_SQRT_2;
        let norm = weights_l2_norm(&[v, v]);
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn active_morph_count_threshold() {
        let count = active_morph_count(&[0.0, 0.1, 0.05, 0.9], 0.09);
        assert_eq!(count, 2); // 0.1 and 0.9
    }

    #[test]
    fn weights_entropy_uniform() {
        // Uniform distribution maximizes entropy
        let e = weights_entropy(&[0.25, 0.25, 0.25, 0.25]);
        assert!(e > 0.0);
    }

    #[test]
    fn weights_entropy_single_nonzero() {
        let e = weights_entropy(&[1.0, 0.0, 0.0]);
        assert!((e).abs() < 1e-5);
    }

    #[test]
    fn std_dev_constant_weights() {
        let s = compute_morph_stats(&[0.5, 0.5, 0.5]);
        assert!(s.std_dev.abs() < 1e-5);
    }
}
