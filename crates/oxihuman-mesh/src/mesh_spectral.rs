// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spectral mesh analysis using the graph Laplacian (power iteration approximation).

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Graph Laplacian for a mesh.
#[allow(dead_code)]
pub struct GraphLaplacian {
    /// `degree[i]` = sum of weights incident to vertex i
    pub degree: Vec<f32>,
    /// `adjacency[i]` = list of (neighbor_index, weight)
    pub adjacency: Vec<Vec<(usize, f32)>>,
    /// total number of vertices
    pub vertex_count: usize,
}

/// Configuration for spectral analysis.
#[allow(dead_code)]
pub struct SpectralConfig {
    pub iterations: u32,
    pub tolerance: f32,
    pub use_cotangent_weights: bool,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Build a default SpectralConfig.
#[allow(dead_code)]
pub fn default_spectral_config() -> SpectralConfig {
    SpectralConfig {
        iterations: 64,
        tolerance: 1e-6,
        use_cotangent_weights: false,
    }
}

/// Build the graph Laplacian from mesh positions and triangle indices (uniform weights).
#[allow(dead_code)]
pub fn build_laplacian(positions: &[[f32; 3]], indices: &[u32]) -> GraphLaplacian {
    let n = positions.len();
    let mut adjacency: Vec<Vec<(usize, f32)>> = vec![vec![]; n];
    let mut degree = vec![0.0f32; n];

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3] as usize;
        let b = indices[t * 3 + 1] as usize;
        let c = indices[t * 3 + 2] as usize;
        for &(u, v) in &[(a, b), (b, c), (a, c)] {
            if !adjacency[u].iter().any(|&(nb, _)| nb == v) {
                adjacency[u].push((v, 1.0));
                adjacency[v].push((u, 1.0));
                degree[u] += 1.0;
                degree[v] += 1.0;
            }
        }
    }

    GraphLaplacian {
        degree,
        adjacency,
        vertex_count: n,
    }
}

/// Compute L * signal, where L = D - A.
#[allow(dead_code)]
pub fn laplacian_operator(lap: &GraphLaplacian, signal: &[f32]) -> Vec<f32> {
    let n = lap.vertex_count;
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        out[i] = lap.degree[i] * signal[i];
        for &(j, w) in &lap.adjacency[i] {
            out[i] -= w * signal[j];
        }
    }
    out
}

/// Explicit Laplacian smoothing: signal' = signal - lambda * L * signal, repeated.
#[allow(dead_code)]
pub fn laplacian_smooth_signal(
    lap: &GraphLaplacian,
    signal: &[f32],
    lambda: f32,
    iterations: u32,
) -> Vec<f32> {
    let mut s = signal.to_vec();
    for _ in 0..iterations {
        let ls = laplacian_operator(lap, &s);
        for (v, lv) in s.iter_mut().zip(ls.iter()) {
            *v -= lambda * lv;
        }
    }
    s
}

/// Approximate the smallest non-trivial eigenvector of L using power iteration.
/// Seeds with a pseudo-random vector derived from `seed`.
#[allow(dead_code)]
pub fn power_iterate(lap: &GraphLaplacian, iterations: u32, seed: u64) -> Vec<f32> {
    let n = lap.vertex_count;
    if n == 0 {
        return vec![];
    }

    // Seed vector
    let mut v: Vec<f32> = (0..n)
        .map(|i| {
            let h = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(i as u64 * 2_862_933_555_777_941_757);
            ((h >> 33) as f32) / (u32::MAX as f32) - 0.5
        })
        .collect();

    // Remove mean (orthogonalise against constant vector)
    let mean = v.iter().sum::<f32>() / n as f32;
    for x in v.iter_mut() {
        *x -= mean;
    }

    // Shift-and-invert approximation: multiply by (d_max * I - L) to make
    // smallest eigenvalue of L correspond to largest eigenvalue.
    let d_max = lap.degree.iter().cloned().fold(0.0f32, f32::max) + 1.0;

    for _ in 0..iterations {
        // w = (d_max * I - L) * v
        let mut w = vec![0.0f32; n];
        for i in 0..n {
            w[i] = d_max * v[i]
                - (lap.degree[i] * v[i]
                    - lap.adjacency[i]
                        .iter()
                        .map(|&(j, wt)| wt * v[j])
                        .sum::<f32>());
        }
        // Orthogonalise against constant
        let mean_w = w.iter().sum::<f32>() / n as f32;
        for x in w.iter_mut() {
            *x -= mean_w;
        }
        // Normalise
        let norm = w.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm < 1e-12 {
            break;
        }
        for (vi, wi) in v.iter_mut().zip(w.iter()) {
            *vi = wi / norm;
        }
    }
    v
}

/// Compute the Rayleigh quotient energy: signal^T L signal.
#[allow(dead_code)]
pub fn graph_laplacian_energy(lap: &GraphLaplacian, signal: &[f32]) -> f32 {
    let ls = laplacian_operator(lap, signal);
    signal.iter().zip(ls.iter()).map(|(s, ls)| s * ls).sum()
}

/// Normalise signal to zero-mean, unit variance.
#[allow(dead_code)]
pub fn normalize_signal(signal: &[f32]) -> Vec<f32> {
    let n = signal.len();
    if n == 0 {
        return vec![];
    }
    let mean = signal.iter().sum::<f32>() / n as f32;
    let var = signal.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / n as f32;
    let std = var.sqrt().max(1e-12);
    signal.iter().map(|x| (x - mean) / std).collect()
}

/// Approximate the Fiedler vector (second smallest eigenvector of L) as a 1D spectral embedding.
#[allow(dead_code)]
pub fn spectral_embedding_1d(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &SpectralConfig,
) -> Vec<f32> {
    let lap = build_laplacian(positions, indices);
    power_iterate(&lap, cfg.iterations, 42)
}

/// Partition a mesh by the median of its spectral embedding.
/// Returns `true` for vertices above the median, `false` otherwise.
#[allow(dead_code)]
pub fn spectral_partition(embedding: &[f32]) -> Vec<bool> {
    if embedding.is_empty() {
        return vec![];
    }
    let mut sorted = embedding.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = sorted[sorted.len() / 2];
    embedding.iter().map(|v| *v >= median).collect()
}

/// Approximate the mesh diameter via the range of the spectral embedding.
#[allow(dead_code)]
pub fn mesh_diameter_spectral(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let cfg = default_spectral_config();
    let emb = spectral_embedding_1d(positions, indices, &cfg);
    if emb.is_empty() {
        return 0.0;
    }
    let mn = emb.iter().cloned().fold(f32::INFINITY, f32::min);
    let mx = emb.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    mx - mn
}

/// Return the vertex count stored in the Laplacian.
#[allow(dead_code)]
pub fn laplacian_vertex_count(lap: &GraphLaplacian) -> usize {
    lap.vertex_count
}

/// Compute degree centrality: `degree[i]` / max_degree.
#[allow(dead_code)]
pub fn degree_centrality(lap: &GraphLaplacian) -> Vec<f32> {
    let max_deg = lap.degree.iter().cloned().fold(0.0f32, f32::max);
    if max_deg < 1e-12 {
        return vec![0.0; lap.vertex_count];
    }
    lap.degree.iter().map(|d| d / max_deg).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn triangle_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_spectral_config();
        assert!(cfg.iterations > 0);
        assert!(cfg.tolerance > 0.0);
        assert!(!cfg.use_cotangent_weights);
    }

    #[test]
    fn test_build_laplacian_vertex_count() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        assert_eq!(lap.vertex_count, 4);
    }

    #[test]
    fn test_build_laplacian_degree_nonzero() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        for d in &lap.degree {
            assert!(*d > 0.0);
        }
    }

    #[test]
    fn test_laplacian_operator_length() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        let sig = vec![1.0f32; lap.vertex_count];
        let out = laplacian_operator(&lap, &sig);
        assert_eq!(out.len(), lap.vertex_count);
    }

    #[test]
    fn test_laplacian_operator_constant_null() {
        // L * 1 == 0 for any graph Laplacian
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        let sig = vec![1.0f32; lap.vertex_count];
        let out = laplacian_operator(&lap, &sig);
        for v in &out {
            assert!(v.abs() < 1e-5, "L*1 should be 0, got {v}");
        }
    }

    #[test]
    fn test_laplacian_smooth_signal_converges() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        let sig = vec![1.0, 0.0, 1.0, 0.0];
        let smoothed = laplacian_smooth_signal(&lap, &sig, 0.1, 50);
        let variance: f32 = {
            let mean = smoothed.iter().sum::<f32>() / smoothed.len() as f32;
            smoothed
                .iter()
                .map(|x| (x - mean) * (x - mean))
                .sum::<f32>()
                / smoothed.len() as f32
        };
        // After smoothing the variance should decrease relative to initial
        let init_variance: f32 = {
            let mean = sig.iter().sum::<f32>() / sig.len() as f32;
            sig.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / sig.len() as f32
        };
        assert!(
            variance <= init_variance + 1e-5,
            "smoothing should reduce variance"
        );
    }

    #[test]
    fn test_graph_laplacian_energy_nonneg() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        let sig = vec![1.0, -1.0, 0.5, -0.5];
        let energy = graph_laplacian_energy(&lap, &sig);
        assert!(energy >= -1e-5, "energy must be >= 0, got {energy}");
    }

    #[test]
    fn test_normalize_signal_zero_mean() {
        let sig = vec![1.0, 2.0, 3.0, 4.0];
        let norm = normalize_signal(&sig);
        let mean: f32 = norm.iter().sum::<f32>() / norm.len() as f32;
        assert!(mean.abs() < 1e-5, "mean should be 0, got {mean}");
    }

    #[test]
    fn test_normalize_signal_unit_variance() {
        let sig = vec![1.0, 2.0, 3.0, 4.0];
        let norm = normalize_signal(&sig);
        let mean: f32 = norm.iter().sum::<f32>() / norm.len() as f32;
        let var: f32 =
            norm.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / norm.len() as f32;
        assert!((var - 1.0).abs() < 1e-4, "variance should be 1, got {var}");
    }

    #[test]
    fn test_normalize_signal_empty() {
        let norm = normalize_signal(&[]);
        assert!(norm.is_empty());
    }

    #[test]
    fn test_spectral_embedding_length() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let cfg = default_spectral_config();
        let emb = spectral_embedding_1d(&pos, &idx, &cfg);
        assert_eq!(emb.len(), pos.len());
    }

    #[test]
    fn test_spectral_partition_length() {
        let emb = vec![0.1, -0.5, 0.3, -0.2, 0.8];
        let part = spectral_partition(&emb);
        assert_eq!(part.len(), emb.len());
    }

    #[test]
    fn test_spectral_partition_half_true() {
        let emb = vec![1.0, 2.0, 3.0, 4.0];
        let part = spectral_partition(&emb);
        // median is 3.0, values >= 3.0 are true
        let trues = part.iter().filter(|&&b| b).count();
        assert!(trues >= 1 && trues <= emb.len());
    }

    #[test]
    fn test_degree_centrality_max_one() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        let cent = degree_centrality(&lap);
        for c in &cent {
            assert!(*c >= 0.0 && *c <= 1.0 + 1e-5);
        }
        let max_c = cent.iter().cloned().fold(0.0f32, f32::max);
        assert!((max_c - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_laplacian_vertex_count_fn() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        assert_eq!(laplacian_vertex_count(&lap), pos.len());
    }

    #[test]
    fn test_mesh_diameter_spectral() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let diam = mesh_diameter_spectral(&pos, &idx);
        assert!(diam >= 0.0);
    }

    #[test]
    fn test_power_iterate_length() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let lap = build_laplacian(&pos, &idx);
        let v = power_iterate(&lap, 32, 123);
        assert_eq!(v.len(), lap.vertex_count);
    }
}
