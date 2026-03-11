// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Smooth weight maps across the mesh surface using adjacency-based diffusion.

/// Result of a weight smoothing operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SmoothWeightResult {
    pub weights: Vec<f32>,
    pub iterations: usize,
    pub max_delta: f32,
}

/// Configuration for weight smoothing.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SmoothWeightConfig {
    pub iterations: usize,
    pub factor: f32,
    pub pin_boundary: bool,
}

impl Default for SmoothWeightConfig {
    fn default() -> Self {
        Self {
            iterations: 5,
            factor: 0.5,
            pin_boundary: false,
        }
    }
}

/// Build a simple adjacency list from a triangle index buffer.
#[allow(dead_code)]
pub fn build_adjacency_sw(positions_len: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![vec![]; positions_len];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            if !adj[u].contains(&v) {
                adj[u].push(v);
            }
            if !adj[v].contains(&u) {
                adj[v].push(u);
            }
        }
    }
    adj
}

/// Detect boundary vertices (vertices with fewer-than-ring neighbours in a
/// non-closed mesh).  Here we use a simple edge-count heuristic.
#[allow(dead_code)]
pub fn detect_boundary_sw(indices: &[u32], vertex_count: usize) -> Vec<bool> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        for &(u, v) in &[
            (a.min(b), a.max(b)),
            (b.min(c), b.max(c)),
            (a.min(c), a.max(c)),
        ] {
            *edge_count.entry((u, v)).or_insert(0) += 1;
        }
    }
    let mut boundary = vec![false; vertex_count];
    for (&(u, v), &cnt) in &edge_count {
        if cnt == 1 {
            boundary[u as usize] = true;
            boundary[v as usize] = true;
        }
    }
    boundary
}

/// Smooth a weight array using Laplacian diffusion over the mesh adjacency.
#[allow(dead_code)]
pub fn smooth_weights(
    weights: &[f32],
    indices: &[u32],
    config: &SmoothWeightConfig,
) -> SmoothWeightResult {
    let n = weights.len();
    let adj = build_adjacency_sw(n, indices);
    let boundary = detect_boundary_sw(indices, n);
    let mut w = weights.to_vec();
    let mut max_delta = 0.0_f32;

    for _iter in 0..config.iterations {
        let prev = w.clone();
        max_delta = 0.0;
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            if config.pin_boundary && boundary[i] {
                continue;
            }
            if adj[i].is_empty() {
                continue;
            }
            let avg: f32 = adj[i].iter().map(|&j| prev[j]).sum::<f32>() / adj[i].len() as f32;
            let new_w = prev[i] + config.factor * (avg - prev[i]);
            let delta = (new_w - prev[i]).abs();
            if delta > max_delta {
                max_delta = delta;
            }
            w[i] = new_w.clamp(0.0, 1.0);
        }
    }

    SmoothWeightResult {
        weights: w,
        iterations: config.iterations,
        max_delta,
    }
}

/// Normalize a weight array so values lie in `[0, 1]`.
#[allow(dead_code)]
pub fn normalize_smooth_weights(weights: &mut [f32]) {
    let max = weights.iter().cloned().fold(0.0_f32, f32::max);
    if max > 0.0 {
        for w in weights.iter_mut() {
            *w /= max;
        }
    }
}

/// Returns the number of vertices whose weight exceeds `threshold`.
#[allow(dead_code)]
pub fn count_above_threshold(weights: &[f32], threshold: f32) -> usize {
    weights.iter().filter(|&&w| w > threshold).count()
}

/// Clamp all weights to `[lo, hi]`.
#[allow(dead_code)]
pub fn clamp_weights(weights: &mut [f32], lo: f32, hi: f32) {
    for w in weights.iter_mut() {
        *w = w.clamp(lo, hi);
    }
}

/// Average weight value.
#[allow(dead_code)]
pub fn average_weight(weights: &[f32]) -> f32 {
    if weights.is_empty() {
        return 0.0;
    }
    weights.iter().sum::<f32>() / weights.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }
    fn flat_weights(n: usize, val: f32) -> Vec<f32> {
        vec![val; n]
    }

    #[test]
    fn smooth_keeps_uniform_unchanged() {
        let w = flat_weights(4, 0.5);
        let cfg = SmoothWeightConfig::default();
        let res = smooth_weights(&w, &tri_indices(), &cfg);
        for v in &res.weights {
            assert!((v - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn smooth_reduces_peak() {
        let mut w = flat_weights(4, 0.0);
        w[0] = 1.0;
        let cfg = SmoothWeightConfig {
            iterations: 10,
            factor: 0.5,
            pin_boundary: false,
        };
        let res = smooth_weights(&w, &tri_indices(), &cfg);
        assert!(res.weights[0] < 1.0);
    }

    #[test]
    fn normalize_smoke() {
        let mut w = vec![0.0_f32, 0.5, 1.0, 2.0];
        normalize_smooth_weights(&mut w);
        assert!((w[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn count_above_threshold_basic() {
        let w = vec![0.1, 0.5, 0.9, 0.4];
        assert_eq!(count_above_threshold(&w, 0.45), 2);
    }

    #[test]
    fn clamp_weights_test() {
        let mut w = vec![-0.5, 0.5, 1.5];
        clamp_weights(&mut w, 0.0, 1.0);
        assert!((0.0..=1.0).contains(&w[0]));
        assert!((0.0..=1.0).contains(&w[2]));
    }

    #[test]
    fn average_weight_empty() {
        assert_eq!(average_weight(&[]), 0.0);
    }

    #[test]
    fn average_weight_basic() {
        let w = vec![0.0, 1.0];
        assert!((average_weight(&w) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn detect_boundary_finds_boundary() {
        let boundary = detect_boundary_sw(&tri_indices(), 4);
        // at least one boundary vertex expected
        assert!(boundary.iter().any(|&b| b));
    }

    #[test]
    fn build_adjacency_symmetry() {
        let adj = build_adjacency_sw(4, &tri_indices());
        for (i, neighbours) in adj.iter().enumerate() {
            for &j in neighbours {
                assert!(adj[j].contains(&i));
            }
        }
    }

    #[test]
    fn result_fields() {
        let w = flat_weights(4, 0.3);
        let cfg = SmoothWeightConfig {
            iterations: 3,
            ..Default::default()
        };
        let res = smooth_weights(&w, &tri_indices(), &cfg);
        assert_eq!(res.iterations, 3);
        assert!(!res.weights.is_empty());
    }
}
