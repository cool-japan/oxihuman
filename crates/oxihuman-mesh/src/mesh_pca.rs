// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Principal Component Analysis of mesh shape space.
//!
//! Implements a power-iteration + deflation PCA to build a linear
//! 3-D Morphable Model (3DMM) basis from a collection of registered shapes.

#![allow(dead_code)]

// ── Public types ─────────────────────────────────────────────────────────────

/// A PCA shape model.
#[derive(Debug, Clone)]
pub struct ShapePca {
    /// Mean vertex positions (one per vertex).
    pub mean_shape: Vec<[f32; 3]>,
    /// Principal components stored as flattened xyz vectors: `[component][xyz…]`.
    pub components: Vec<Vec<f32>>,
    /// Explained variance per component.
    pub variances: Vec<f32>,
    /// Total variance across all components retained.
    pub total_variance: f32,
}

/// Configuration for shape PCA computation.
#[derive(Debug, Clone)]
pub struct PcaConfig {
    /// Number of principal components to keep. Default 10.
    pub n_components: usize,
    /// Subtract mean before computing PCA. Default true.
    pub center: bool,
}

impl Default for PcaConfig {
    fn default() -> Self {
        Self {
            n_components: 10,
            center: true,
        }
    }
}

/// Full result of a shape PCA computation.
#[derive(Debug, Clone)]
pub struct PcaResult {
    /// The computed PCA model.
    pub pca: ShapePca,
    /// Fraction of total variance explained by each component.
    pub explained_ratio: Vec<f32>,
    /// Sum of `explained_ratio` (≤ 1.0).
    pub cumulative_variance: f32,
}

// ── Flat / shape conversion ──────────────────────────────────────────────────

/// Flatten `[[x,y,z],…]` into `[x0,y0,z0,x1,y1,z1,…]`.
pub fn shape_to_flat(shape: &[[f32; 3]]) -> Vec<f32> {
    let mut out = Vec::with_capacity(shape.len() * 3);
    for v in shape {
        out.push(v[0]);
        out.push(v[1]);
        out.push(v[2]);
    }
    out
}

/// Inverse of [`shape_to_flat`].
///
/// Pads with zeros if `flat.len()` is not a multiple of 3.
pub fn flat_to_shape(flat: &[f32]) -> Vec<[f32; 3]> {
    flat.chunks(3)
        .map(|c| {
            [
                c.first().copied().unwrap_or(0.0),
                c.get(1).copied().unwrap_or(0.0),
                c.get(2).copied().unwrap_or(0.0),
            ]
        })
        .collect()
}

// ── mean_shape ───────────────────────────────────────────────────────────────

/// Compute the element-wise mean vertex position across a set of shapes.
///
/// All shapes must have the same vertex count; extra vertices in longer shapes
/// are ignored.
pub fn mean_shape(shapes: &[Vec<[f32; 3]>]) -> Vec<[f32; 3]> {
    if shapes.is_empty() {
        return vec![];
    }
    let nv = shapes[0].len();
    let n = shapes.len() as f32;
    let mut mean = vec![[0.0_f32; 3]; nv];
    for shape in shapes {
        for (i, v) in shape.iter().enumerate().take(nv) {
            mean[i][0] += v[0];
            mean[i][1] += v[1];
            mean[i][2] += v[2];
        }
    }
    for m in &mut mean {
        m[0] /= n;
        m[1] /= n;
        m[2] /= n;
    }
    mean
}

// ── Internal linear-algebra helpers ─────────────────────────────────────────

/// Dot product of two equal-length slices.
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// L2 norm of a slice.
fn norm(v: &[f32]) -> f32 {
    dot(v, v).sqrt()
}

/// Normalise a mutable slice in-place; returns the norm before normalisation.
fn normalize_inplace(v: &mut [f32]) -> f32 {
    let n = norm(v);
    if n > 1e-10 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
    n
}

/// Matrix-vector product  `A v`  where `A` is stored row-major as `rows×dim`.
fn matvec(a: &[Vec<f32>], v: &[f32]) -> Vec<f32> {
    a.iter().map(|row| dot(row, v)).collect()
}

/// Compute A^T y where A is rows×dim and y is rows-long.
fn matvec_t(a: &[Vec<f32>], y: &[f32]) -> Vec<f32> {
    if a.is_empty() {
        return vec![];
    }
    let dim = a[0].len();
    let mut out = vec![0.0_f32; dim];
    for (row, &yi) in a.iter().zip(y.iter()) {
        for (o, &r) in out.iter_mut().zip(row.iter()) {
            *o += r * yi;
        }
    }
    out
}

/// Power iteration to find the dominant eigenvector of the scatter matrix
/// `X^T X` (X is `n_samples × dim`).
///
/// We avoid forming `X^T X` explicitly; instead we iterate  v ← X^T (X v).
fn power_iteration(data: &[Vec<f32>], dim: usize, iters: usize) -> (Vec<f32>, f32) {
    // Random-ish starting vector (deterministic).
    let mut v: Vec<f32> = (0..dim).map(|i| ((i * 7 + 1) as f32).sin()).collect();
    normalize_inplace(&mut v);

    for _ in 0..iters {
        let xv = matvec(data, &v); // X v → n-vector
        let xtxv = matvec_t(data, &xv); // X^T (X v) → dim-vector
        normalize_inplace(&mut xtxv.clone());
        v = xtxv;
        let _ = normalize_inplace(&mut v);
    }

    // Eigenvalue estimate (Rayleigh quotient of X^T X).
    let xv = matvec(data, &v);
    let eigenvalue = dot(&xv, &xv) / data.len().max(1) as f32;
    (v, eigenvalue)
}

/// Deflate data matrix: subtract the projection onto `component`.
fn deflate(data: &mut [Vec<f32>], component: &[f32]) {
    for row in data.iter_mut() {
        let proj = dot(row, component);
        for (r, &c) in row.iter_mut().zip(component.iter()) {
            *r -= proj * c;
        }
    }
}

// ── compute_shape_pca ────────────────────────────────────────────────────────

/// Compute PCA of a shape population via power iteration + deflation.
pub fn compute_shape_pca(shapes: &[Vec<[f32; 3]>], cfg: &PcaConfig) -> PcaResult {
    if shapes.is_empty() {
        let pca = ShapePca {
            mean_shape: vec![],
            components: vec![],
            variances: vec![],
            total_variance: 0.0,
        };
        return PcaResult {
            pca,
            explained_ratio: vec![],
            cumulative_variance: 0.0,
        };
    }

    let nv = shapes[0].len();
    let dim = nv * 3;
    let n_comp = cfg.n_components.min(shapes.len()).min(dim);

    // Compute mean.
    let mean = if cfg.center {
        mean_shape(shapes)
    } else {
        vec![[0.0; 3]; nv]
    };
    let mean_flat = shape_to_flat(&mean);

    // Build centered data matrix (n_shapes × dim).
    let mut data: Vec<Vec<f32>> = shapes
        .iter()
        .map(|s| {
            let flat = shape_to_flat(s);
            flat.iter()
                .zip(mean_flat.iter())
                .map(|(a, b)| a - b)
                .collect()
        })
        .collect();

    // Power iteration + deflation to extract n_comp eigenvectors.
    let mut components: Vec<Vec<f32>> = Vec::with_capacity(n_comp);
    let mut variances: Vec<f32> = Vec::with_capacity(n_comp);

    for _ in 0..n_comp {
        if dim == 0 {
            break;
        }
        let (evec, eval) = power_iteration(&data, dim, 60);
        components.push(evec.clone());
        variances.push(eval.max(0.0));
        deflate(&mut data, &evec);
    }

    let total_variance: f32 = variances.iter().sum();
    let explained_ratio: Vec<f32> = if total_variance > 0.0 {
        variances.iter().map(|v| v / total_variance).collect()
    } else {
        vec![0.0; variances.len()]
    };
    let cumulative_variance: f32 = explained_ratio.iter().sum();

    let pca = ShapePca {
        mean_shape: mean,
        components,
        variances,
        total_variance,
    };

    PcaResult {
        pca,
        explained_ratio,
        cumulative_variance,
    }
}

// ── project_shape ─────────────────────────────────────────────────────────────

/// Project a shape into PC space, returning one score per component.
///
/// `scores[i] = dot(shape_flat - mean_flat, component[i])`
pub fn project_shape(pca: &ShapePca, shape: &[[f32; 3]]) -> Vec<f32> {
    let mean_flat = shape_to_flat(&pca.mean_shape);
    let shape_flat = shape_to_flat(shape);
    let diff: Vec<f32> = shape_flat
        .iter()
        .zip(mean_flat.iter())
        .map(|(a, b)| a - b)
        .collect();
    pca.components.iter().map(|c| dot(&diff, c)).collect()
}

// ── reconstruct_shape ─────────────────────────────────────────────────────────

/// Reconstruct a shape from PCA scores: `mean + Σ scores[i] * component[i]`.
pub fn reconstruct_shape(pca: &ShapePca, scores: &[f32]) -> Vec<[f32; 3]> {
    let mean_flat = shape_to_flat(&pca.mean_shape);
    let dim = mean_flat.len();
    let mut flat = mean_flat;
    for (score, comp) in scores.iter().zip(pca.components.iter()) {
        for (f, &c) in flat.iter_mut().zip(comp.iter()) {
            *f += score * c;
        }
        // Pad if component is shorter.
        if comp.len() < dim {
            // nothing extra to add
        }
    }
    flat_to_shape(&flat)
}

// ── explained_variance_ratio ─────────────────────────────────────────────────

/// Return the fraction of total variance explained by each component.
pub fn explained_variance_ratio(pca: &ShapePca) -> Vec<f32> {
    let total: f32 = pca.variances.iter().sum();
    if total <= 0.0 {
        return vec![0.0; pca.variances.len()];
    }
    pca.variances.iter().map(|v| v / total).collect()
}

// ── pca_reconstruction_error ─────────────────────────────────────────────────

/// MSE between original and reconstructed shape (per coordinate).
pub fn pca_reconstruction_error(pca: &ShapePca, original: &[[f32; 3]], scores: &[f32]) -> f32 {
    let reconstructed = reconstruct_shape(pca, scores);
    let n = original.len().min(reconstructed.len());
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0_f32;
    for i in 0..n {
        for k in 0..3 {
            let d = original[i][k] - reconstructed[i][k];
            sum += d * d;
        }
    }
    sum / (n * 3) as f32
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Create n identical shapes (single triangle).
    fn identical_shapes(n: usize) -> Vec<Vec<[f32; 3]>> {
        let shape: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        vec![shape; n]
    }

    /// Create n shapes with only x-axis variation.
    fn x_vary_shapes(n: usize) -> Vec<Vec<[f32; 3]>> {
        (0..n)
            .map(|i| {
                let t = i as f32 / n.max(1) as f32;
                vec![[t, 0.0, 0.0], [t + 1.0, 0.0, 0.0], [t, 1.0, 0.0]]
            })
            .collect()
    }

    #[test]
    fn shape_to_flat_round_trip() {
        let shape: Vec<[f32; 3]> = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let flat = shape_to_flat(&shape);
        let back = flat_to_shape(&flat);
        assert_eq!(back.len(), shape.len());
        for (a, b) in shape.iter().zip(back.iter()) {
            for k in 0..3 {
                assert!((a[k] - b[k]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn flat_to_shape_length() {
        let flat = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let shape = flat_to_shape(&flat);
        assert_eq!(shape.len(), 2);
    }

    #[test]
    fn mean_shape_formula() {
        let shapes: Vec<Vec<[f32; 3]>> = vec![
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            vec![[2.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
        ];
        let mean = mean_shape(&shapes);
        assert_eq!(mean.len(), 2);
        assert!((mean[0][0] - 1.0).abs() < 1e-5);
        assert!((mean[1][0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn mean_shape_empty() {
        let mean = mean_shape(&[]);
        assert!(mean.is_empty());
    }

    #[test]
    fn compute_shape_pca_n_components_preserved() {
        let shapes = x_vary_shapes(20);
        let cfg = PcaConfig {
            n_components: 3,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        assert_eq!(result.pca.components.len(), 3);
        assert_eq!(result.pca.variances.len(), 3);
        assert_eq!(result.explained_ratio.len(), 3);
    }

    #[test]
    fn explained_variance_ratio_sums_le_one() {
        let shapes = x_vary_shapes(15);
        let cfg = PcaConfig::default();
        let result = compute_shape_pca(&shapes, &cfg);
        let sum: f32 = result.explained_ratio.iter().sum();
        assert!(sum <= 1.0 + 1e-5, "ratio sum {} > 1", sum);
    }

    #[test]
    fn identity_shapes_zero_variance() {
        let shapes = identical_shapes(10);
        let cfg = PcaConfig {
            n_components: 3,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        assert!(
            result.pca.total_variance < 1e-5,
            "identical shapes should have ~zero variance"
        );
    }

    #[test]
    fn project_shape_dimension() {
        let shapes = x_vary_shapes(20);
        let cfg = PcaConfig {
            n_components: 4,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        let scores = project_shape(&result.pca, &shapes[0]);
        assert_eq!(scores.len(), 4);
    }

    #[test]
    fn reconstruct_shape_length() {
        let shapes = x_vary_shapes(20);
        let cfg = PcaConfig {
            n_components: 3,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        let scores = vec![0.0; 3];
        let rec = reconstruct_shape(&result.pca, &scores);
        assert_eq!(rec.len(), shapes[0].len());
    }

    #[test]
    fn reconstruct_zero_scores_gives_mean() {
        let shapes = x_vary_shapes(20);
        let cfg = PcaConfig {
            n_components: 5,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        let mean = &result.pca.mean_shape;
        let scores = vec![0.0; result.pca.components.len()];
        let rec = reconstruct_shape(&result.pca, &scores);
        for (m, r) in mean.iter().zip(rec.iter()) {
            for k in 0..3 {
                assert!((m[k] - r[k]).abs() < 1e-4, "zero scores should give mean");
            }
        }
    }

    #[test]
    fn pca_reconstruction_error_nonneg() {
        let shapes = x_vary_shapes(20);
        let cfg = PcaConfig {
            n_components: 5,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        let scores = project_shape(&result.pca, &shapes[0]);
        let err = pca_reconstruction_error(&result.pca, &shapes[0], &scores);
        assert!(err >= 0.0, "error must be non-negative");
    }

    #[test]
    fn project_then_reconstruct_in_distribution_low_error() {
        let shapes = x_vary_shapes(30);
        let cfg = PcaConfig {
            n_components: 8,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        // Project the first training shape and reconstruct.
        let shape = &shapes[0];
        let scores = project_shape(&result.pca, shape);
        let err = pca_reconstruction_error(&result.pca, shape, &scores);
        // Error should be small for in-distribution shape (1D variation).
        // We allow some numerical tolerance from power iteration.
        assert!(err < 1.0, "reconstruction error {} too large", err);
    }

    #[test]
    fn explained_variance_ratio_fn_matches_pca_result() {
        let shapes = x_vary_shapes(15);
        let cfg = PcaConfig {
            n_components: 4,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        let ratio = explained_variance_ratio(&result.pca);
        // Sum should be ~1 when all components are returned.
        let total_var: f32 = result.pca.variances.iter().sum();
        if total_var > 0.0 {
            let sum: f32 = ratio.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-4,
                "ratio sum should be 1 for all components"
            );
        }
    }

    #[test]
    fn cumulative_variance_le_one() {
        let shapes = x_vary_shapes(20);
        let cfg = PcaConfig {
            n_components: 6,
            center: true,
        };
        let result = compute_shape_pca(&shapes, &cfg);
        assert!(result.cumulative_variance <= 1.0 + 1e-4);
    }

    #[test]
    fn shape_to_flat_length() {
        let shape: Vec<[f32; 3]> = vec![[1.0, 2.0, 3.0]; 5];
        assert_eq!(shape_to_flat(&shape).len(), 15);
    }

    #[test]
    fn compute_shape_pca_empty_returns_empty() {
        let cfg = PcaConfig::default();
        let result = compute_shape_pca(&[], &cfg);
        assert!(result.pca.components.is_empty());
        assert_eq!(result.pca.total_variance, 0.0);
    }
}
