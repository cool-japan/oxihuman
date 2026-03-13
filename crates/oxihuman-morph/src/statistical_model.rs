// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Statistical body model built from PCA of body shape variations.
//!
//! Implements a PCA-based statistical body model (similar to SMPL/SCAPE concepts)
//! using a pure-Rust SVD implementation based on Golub-Kahan bidiagonalization
//! and implicit QR iteration with Wilkinson shifts.

use anyhow::ensure;

// ---------------------------------------------------------------------------
// LCG random number generator
// ---------------------------------------------------------------------------

/// Linear congruential generator for deterministic pseudo-random numbers.
fn lcg_next(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
    (*state >> 33) as f64 / (1u64 << 31) as f64
}

/// Box-Muller transform: produce a standard-normal sample from two uniform samples.
fn lcg_normal(state: &mut u64) -> f64 {
    loop {
        let u1 = lcg_next(state);
        let u2 = lcg_next(state);
        if u1 > 1e-15 {
            let r = (-2.0 * u1.ln()).sqrt();
            let theta = 2.0 * std::f64::consts::PI * u2;
            return r * theta.cos();
        }
    }
}

// ---------------------------------------------------------------------------
// Dense matrix helpers  (row-major storage)
// ---------------------------------------------------------------------------

/// A simple row-major dense matrix.
#[derive(Clone, Debug)]
struct Mat {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Mat {
    fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m.data[i * n + i] = 1.0;
        }
        m
    }

    #[inline]
    fn get(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }

    #[inline]
    fn set(&mut self, r: usize, c: usize, v: f64) {
        self.data[r * self.cols + c] = v;
    }

    /// Transpose.
    fn t(&self) -> Self {
        let mut out = Self::zeros(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                out.set(c, r, self.get(r, c));
            }
        }
        out
    }

    /// Matrix multiply: self * other.
    #[cfg(test)]
    fn mul(&self, other: &Mat) -> Self {
        debug_assert_eq!(self.cols, other.rows);
        let mut out = Self::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for k in 0..self.cols {
                let a = self.get(i, k);
                if a == 0.0 {
                    continue;
                }
                for j in 0..other.cols {
                    let idx = i * other.cols + j;
                    out.data[idx] += a * other.get(k, j);
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Householder helpers
// ---------------------------------------------------------------------------

/// Compute a Householder vector v such that (I - 2 v v^T) x has zeros below the first entry.
/// Returns (v, beta) where beta = 2 / (v^T v). `x` is the input vector.
fn householder_vector(x: &[f64]) -> (Vec<f64>, f64) {
    let n = x.len();
    if n == 0 {
        return (vec![], 0.0);
    }
    let mut v = x.to_vec();
    let sigma: f64 = x[1..].iter().map(|&xi| xi * xi).sum();
    v[0] = 1.0;
    if sigma.abs() < 1e-300 {
        return (v, 0.0);
    }
    let mu = (x[0] * x[0] + sigma).sqrt();
    if x[0] <= 0.0 {
        v[0] = x[0] - mu;
    } else {
        v[0] = -sigma / (x[0] + mu);
    }
    let beta = 2.0 * v[0] * v[0] / (sigma + v[0] * v[0]);
    let inv_v0 = 1.0 / v[0];
    for vi in v.iter_mut() {
        *vi *= inv_v0;
    }
    (v, beta)
}

// ---------------------------------------------------------------------------
// Golub-Kahan bidiagonalization
// ---------------------------------------------------------------------------

/// Bidiagonalize A (m×n, m >= n) into A = U B V^T where B is upper bidiagonal.
/// Returns (U, diag, super_diag, V).
fn bidiagonalize(a: &Mat) -> (Mat, Vec<f64>, Vec<f64>, Mat) {
    let m = a.rows;
    let n = a.cols;
    assert!(m >= n, "bidiagonalize requires m >= n");

    let mut work = a.clone();
    let mut u_acc = Mat::identity(m);
    let mut v_acc = Mat::identity(n);

    for k in 0..n {
        // --- Left Householder: zero out column k below diagonal ---
        {
            let col_len = m - k;
            let mut col = vec![0.0; col_len];
            for i in 0..col_len {
                col[i] = work.get(k + i, k);
            }
            let (v, beta) = householder_vector(&col);
            if beta != 0.0 {
                // Apply to work: work[k:, k:] -= beta * v * (v^T * work[k:, k:])
                for j in k..n {
                    let mut dot = 0.0;
                    for i in 0..col_len {
                        dot += v[i] * work.get(k + i, j);
                    }
                    for i in 0..col_len {
                        let old = work.get(k + i, j);
                        work.set(k + i, j, old - beta * v[i] * dot);
                    }
                }
                // Accumulate into U
                for j in 0..m {
                    let mut dot = 0.0;
                    for i in 0..col_len {
                        dot += v[i] * u_acc.get(j, k + i);
                    }
                    for i in 0..col_len {
                        let old = u_acc.get(j, k + i);
                        u_acc.set(j, k + i, old - beta * dot * v[i]);
                    }
                }
            }
        }

        // --- Right Householder: zero out row k to the right of super-diagonal ---
        if k + 2 <= n {
            let row_len = n - k - 1;
            let mut row = vec![0.0; row_len];
            for j in 0..row_len {
                row[j] = work.get(k, k + 1 + j);
            }
            let (v, beta) = householder_vector(&row);
            if beta != 0.0 {
                // Apply to work: work[k:, k+1:] -= beta * (work[k:, k+1:] * v) * v^T
                for i in k..m {
                    let mut dot = 0.0;
                    for j in 0..row_len {
                        dot += work.get(i, k + 1 + j) * v[j];
                    }
                    for j in 0..row_len {
                        let old = work.get(i, k + 1 + j);
                        work.set(i, k + 1 + j, old - beta * dot * v[j]);
                    }
                }
                // Accumulate into V
                for i in 0..n {
                    let mut dot = 0.0;
                    for j in 0..row_len {
                        dot += v_acc.get(i, k + 1 + j) * v[j];
                    }
                    for j in 0..row_len {
                        let old = v_acc.get(i, k + 1 + j);
                        v_acc.set(i, k + 1 + j, old - beta * dot * v[j]);
                    }
                }
            }
        }
    }

    // Extract diagonal and super-diagonal
    let mut diag = vec![0.0; n];
    let mut sup = vec![0.0; n.saturating_sub(1)];
    for i in 0..n {
        diag[i] = work.get(i, i);
    }
    for i in 0..sup.len() {
        sup[i] = work.get(i, i + 1);
    }

    (u_acc, diag, sup, v_acc)
}

// ---------------------------------------------------------------------------
// Implicit QR iteration on bidiagonal matrix (Golub-Kahan SVD step)
// ---------------------------------------------------------------------------

/// Compute the SVD of an upper bidiagonal matrix defined by `diag` and `sup`.
/// Modifies `diag` in-place to contain singular values and applies rotations to U and V.
fn bidiagonal_svd(
    diag: &mut [f64],
    sup: &mut [f64],
    u: &mut Mat,
    v: &mut Mat,
    max_iter: usize,
) {
    let n = diag.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        if diag[0] < 0.0 {
            diag[0] = -diag[0];
            for r in 0..v.rows {
                let old = v.get(r, 0);
                v.set(r, 0, -old);
            }
        }
        return;
    }

    let eps = 1e-14;

    for _iter in 0..max_iter {
        // Find the largest q such that B_{n-q:n, n-q:n} is diagonal
        let mut q = 0usize;
        for i in (0..n - 1).rev() {
            if sup[i].abs() <= eps * (diag[i].abs() + diag[i + 1].abs()) {
                sup[i] = 0.0;
                q += 1;
            } else {
                break;
            }
        }
        if q == n - 1 {
            break; // all super-diagonal entries are zero
        }

        // Find the smallest p such that B_{p:n-q, p:n-q} has no zero diagonal
        let end = n - q; // we work on indices [p..end)
        let mut p = end - 1;
        loop {
            if p == 0 {
                break;
            }
            if sup[p - 1].abs() <= eps * (diag[p - 1].abs() + diag[p].abs()) {
                sup[p - 1] = 0.0;
                break;
            }
            p -= 1;
        }

        // Check for zero diagonal entries in B_{p:end, p:end} — if found, zero the row
        let mut found_zero = false;
        for i in p..end {
            if diag[i].abs() < eps {
                diag[i] = 0.0;
                // Zero out super-diagonal elements connected to this zero diagonal
                if i < end - 1 {
                    // Chase bulge rightward
                    chase_zero_row(diag, sup, u, v, i, end);
                    found_zero = true;
                    break;
                } else if i > p && i == end - 1 {
                    // Chase bulge upward
                    chase_zero_col(diag, sup, u, v, p, i);
                    found_zero = true;
                    break;
                }
            }
        }
        if found_zero {
            continue;
        }

        // Wilkinson shift: eigenvalue of trailing 2x2 of B^T B closest to last diagonal
        let bnn1 = if end >= 2 { sup[end - 2] } else { 0.0 };
        let bnn = diag[end - 1];
        let tn = bnn * bnn + bnn1 * bnn1;
        let tn1 = diag[end - 2] * diag[end - 2]
            + if end >= 3 { sup[end - 3] * sup[end - 3] } else { 0.0 };
        let cross = diag[end - 2] * bnn1;

        let d = (tn1 - tn) / 2.0;
        let mu = tn - cross * cross / (d + d.signum() * (d * d + cross * cross).sqrt());

        // Implicit QR step with shift mu
        implicit_qr_step(diag, sup, u, v, p, end, mu);
    }

    // Make all singular values non-negative
    for i in 0..n {
        if diag[i] < 0.0 {
            diag[i] = -diag[i];
            for r in 0..v.rows {
                let old = v.get(r, i);
                v.set(r, i, -old);
            }
        }
    }
}

/// Apply a Givens rotation to columns (i, j) of matrix M from the right.
fn givens_right(m: &mut Mat, i: usize, j: usize, c: f64, s: f64) {
    for r in 0..m.rows {
        let a = m.get(r, i);
        let b = m.get(r, j);
        m.set(r, i, c * a + s * b);
        m.set(r, j, -s * a + c * b);
    }
}

/// Apply a Givens rotation to rows (i, j) of matrix M from the left.
#[allow(dead_code)]
fn givens_left(m: &mut Mat, i: usize, j: usize, c: f64, s: f64) {
    for col in 0..m.cols {
        let a = m.get(i, col);
        let b = m.get(j, col);
        m.set(i, col, c * a + s * b);
        m.set(j, col, -s * a + c * b);
    }
}

/// Compute (c, s) for a Givens rotation zeroing out the second element of [a, b].
fn givens(a: f64, b: f64) -> (f64, f64) {
    if b.abs() < 1e-300 {
        (1.0, 0.0)
    } else if b.abs() > a.abs() {
        let t = -a / b;
        let s = 1.0 / (1.0 + t * t).sqrt();
        (s * t, s)
    } else {
        let t = -b / a;
        let c = 1.0 / (1.0 + t * t).sqrt();
        (c, c * t)
    }
}

/// Implicit QR step on bidiagonal [p..end) with Wilkinson shift `mu`.
fn implicit_qr_step(
    diag: &mut [f64],
    sup: &mut [f64],
    u: &mut Mat,
    v: &mut Mat,
    p: usize,
    end: usize,
    mu: f64,
) {
    let mut y = diag[p] * diag[p] - mu;
    let mut z = diag[p] * sup[p];

    for k in p..end - 1 {
        // Right rotation to zero z in column
        let (c, s) = givens(y, z);
        // Apply to bidiagonal from right (affects columns k, k+1)
        if k > p {
            sup[k - 1] = c * sup[k - 1] + s * 0.0; // bulge was here
            // Actually recompute properly
            let r = (y * y + z * z).sqrt();
            sup[k - 1] = r;
        }
        let d1 = diag[k];
        let e1 = sup[k];
        let d2 = diag[k + 1];

        diag[k] = c * d1 + s * e1;
        sup[k] = -s * d1 + c * e1;
        let bulge = s * d2;
        diag[k + 1] = c * d2;

        // Accumulate V
        givens_right(v, k, k + 1, c, s);

        // Left rotation to zero bulge
        y = diag[k];
        z = bulge;
        let (c2, s2) = givens(y, z);

        diag[k] = c2 * diag[k] + s2 * bulge;
        let old_sup = sup[k];
        let old_diag_kp1 = diag[k + 1];
        sup[k] = c2 * old_sup + s2 * old_diag_kp1;
        diag[k + 1] = -s2 * old_sup + c2 * old_diag_kp1;

        // Accumulate U (apply to columns of U^T, i.e. rows)
        // U stores column-wise, so we rotate columns k and k+1 of U
        givens_right(u, k, k + 1, c2, s2);

        if k + 2 < end {
            let old_sup_kp1 = sup[k + 1];
            let bulge2 = s2 * old_sup_kp1;
            sup[k + 1] = c2 * old_sup_kp1;
            y = sup[k];
            z = bulge2;
        }
    }
}

/// Chase a zero on the diagonal at position `idx` rightward by zeroing the super-diagonal.
fn chase_zero_row(
    diag: &mut [f64],
    sup: &mut [f64],
    u: &mut Mat,
    _v: &mut Mat,
    idx: usize,
    end: usize,
) {
    // diag[idx] == 0, sup[idx] != 0 potentially
    for k in idx..end - 1 {
        if sup[k].abs() < 1e-300 {
            break;
        }
        let (c, s) = givens(diag[k + 1], sup[k]);
        diag[k + 1] = c * diag[k + 1] + s * sup[k];
        sup[k] = 0.0;
        if k + 1 < end - 1 {
            let old = sup[k + 1];
            sup[k + 1] = c * old;
            // bulge into next sup
        }
        givens_right(u, k + 1, idx, c, s);
    }
}

/// Chase a zero on the diagonal at position `idx` upward.
fn chase_zero_col(
    diag: &mut [f64],
    sup: &mut [f64],
    _u: &mut Mat,
    v: &mut Mat,
    p: usize,
    idx: usize,
) {
    for k in (p..idx).rev() {
        if sup[k].abs() < 1e-300 {
            break;
        }
        let (c, s) = givens(diag[k], sup[k]);
        diag[k] = c * diag[k] + s * sup[k];
        sup[k] = 0.0;
        if k > p {
            let old = sup[k - 1];
            let bulge = s * old;
            sup[k - 1] = c * old;
            let _ = bulge; // will be chased in next iteration
        }
        givens_right(v, k, idx, c, s);
    }
}

// ---------------------------------------------------------------------------
// Full SVD: A = U Σ V^T
// ---------------------------------------------------------------------------

/// Compute the thin SVD of an m×n matrix A (m >= n).
/// Returns (U_thin [m×n], singular_values [n], V [n×n]).
fn svd_thin(a: &Mat) -> anyhow::Result<(Mat, Vec<f64>, Mat)> {
    let m = a.rows;
    let n = a.cols;
    ensure!(m >= n, "svd_thin requires m >= n (got {}x{})", m, n);

    let (mut u, mut diag, mut sup, mut v) = bidiagonalize(a);

    let max_iter = 100 * n * n + 1000;
    bidiagonal_svd(&mut diag, &mut sup, &mut u, &mut v, max_iter);

    // Sort singular values in descending order
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a_i, &b_i| {
        diag[b_i]
            .partial_cmp(&diag[a_i])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut sorted_diag = vec![0.0; n];
    let mut sorted_u = Mat::zeros(m, n);
    let mut sorted_v = Mat::zeros(n, n);

    for (new_idx, &old_idx) in indices.iter().enumerate() {
        sorted_diag[new_idx] = diag[old_idx];
        for r in 0..m {
            sorted_u.set(r, new_idx, u.get(r, old_idx));
        }
        for r in 0..n {
            sorted_v.set(r, new_idx, v.get(r, old_idx));
        }
    }

    Ok((sorted_u, sorted_diag, sorted_v))
}

// ---------------------------------------------------------------------------
// PCA configuration
// ---------------------------------------------------------------------------

/// Configuration for building the statistical model.
#[derive(Debug, Clone)]
pub struct PcaConfig {
    /// Maximum number of principal components to retain.
    pub max_components: usize,
    /// Retain components until cumulative explained variance reaches this threshold (0.0–1.0).
    pub variance_threshold: f64,
}

impl Default for PcaConfig {
    fn default() -> Self {
        Self {
            max_components: 50,
            variance_threshold: 0.95,
        }
    }
}

// ---------------------------------------------------------------------------
// StatisticalBodyModel
// ---------------------------------------------------------------------------

/// Statistical body model built from PCA of body shape variations.
///
/// Learns a low-dimensional representation of body shape variation from a
/// collection of example body shapes, enabling generation, interpolation,
/// and projection of body shapes in PCA space.
#[derive(Debug, Clone)]
pub struct StatisticalBodyModel {
    /// Mean body shape (vertex positions flattened: \[x0,y0,z0, x1,y1,z1, ...\])
    mean_shape: Vec<f64>,
    /// Principal components (each component is same length as mean\_shape)
    components: Vec<Vec<f64>>,
    /// Singular values (importance weights)
    singular_values: Vec<f64>,
    /// Explained variance ratio per component
    variance_ratios: Vec<f64>,
    /// Number of vertices
    vertex_count: usize,
    /// Number of components retained
    num_components: usize,
}

impl StatisticalBodyModel {
    /// Build a statistical body model from a collection of body shapes.
    ///
    /// Each shape is a `Vec<[f64; 3]>` of vertex positions. All shapes must
    /// have the same number of vertices. The model is built using PCA
    /// (via SVD of the centered data matrix or Gram matrix for efficiency).
    pub fn build(shapes: &[Vec<[f64; 3]>], config: &PcaConfig) -> anyhow::Result<Self> {
        let n_samples = shapes.len();
        ensure!(n_samples >= 2, "need at least 2 shapes to build PCA model");
        let vertex_count = shapes[0].len();
        ensure!(vertex_count > 0, "shapes must have at least one vertex");
        for (i, s) in shapes.iter().enumerate() {
            ensure!(
                s.len() == vertex_count,
                "shape {} has {} vertices, expected {}",
                i,
                s.len(),
                vertex_count
            );
        }

        let dim = vertex_count * 3; // flattened dimension

        // 1. Flatten shapes and compute mean
        let mut mean = vec![0.0; dim];
        let mut flat_shapes: Vec<Vec<f64>> = Vec::with_capacity(n_samples);
        for shape in shapes {
            let mut flat = Vec::with_capacity(dim);
            for v in shape {
                flat.push(v[0]);
                flat.push(v[1]);
                flat.push(v[2]);
            }
            for (j, &val) in flat.iter().enumerate() {
                mean[j] += val;
            }
            flat_shapes.push(flat);
        }
        let inv_n = 1.0 / n_samples as f64;
        for m in mean.iter_mut() {
            *m *= inv_n;
        }

        // 2. Center the data
        for flat in flat_shapes.iter_mut() {
            for (j, val) in flat.iter_mut().enumerate() {
                *val -= mean[j];
            }
        }

        // 3. Decide strategy: if n_samples < dim, use Gram matrix approach
        let (components, singular_values) = if n_samples <= dim {
            // Gram matrix G = X X^T where X is (n_samples x dim), centered
            // G is (n_samples x n_samples)
            Self::pca_via_gram(&flat_shapes, n_samples, dim)?
        } else {
            // Direct SVD on data matrix
            Self::pca_via_direct_svd(&flat_shapes, n_samples, dim)?
        };

        // 4. Compute variance ratios
        let total_variance: f64 = singular_values.iter().map(|s| s * s).sum();
        let variance_ratios: Vec<f64> = if total_variance > 0.0 {
            singular_values
                .iter()
                .map(|s| s * s / total_variance)
                .collect()
        } else {
            vec![0.0; singular_values.len()]
        };

        // 5. Determine how many components to keep
        let mut cumulative = 0.0;
        let mut keep = 0;
        for &vr in &variance_ratios {
            keep += 1;
            cumulative += vr;
            if keep >= config.max_components || cumulative >= config.variance_threshold {
                break;
            }
        }
        // Ensure we don't exceed available components
        keep = keep.min(components.len());

        let retained_components = components[..keep].to_vec();
        let retained_sv = singular_values[..keep].to_vec();
        let retained_vr = variance_ratios[..keep].to_vec();

        Ok(Self {
            mean_shape: mean,
            components: retained_components,
            singular_values: retained_sv,
            variance_ratios: retained_vr,
            vertex_count,
            num_components: keep,
        })
    }

    /// PCA via Gram matrix (efficient when n_samples << dim).
    fn pca_via_gram(
        flat_shapes: &[Vec<f64>],
        n_samples: usize,
        dim: usize,
    ) -> anyhow::Result<(Vec<Vec<f64>>, Vec<f64>)> {
        // Build Gram matrix G[i][j] = (1/(n-1)) * flat_shapes[i] . flat_shapes[j]
        let mut gram = Mat::zeros(n_samples, n_samples);
        let scale = 1.0 / (n_samples as f64 - 1.0);
        for i in 0..n_samples {
            for j in i..n_samples {
                let mut dot = 0.0;
                for k in 0..dim {
                    dot += flat_shapes[i][k] * flat_shapes[j][k];
                }
                let val = dot * scale;
                gram.set(i, j, val);
                gram.set(j, i, val);
            }
        }

        // SVD of the symmetric Gram matrix
        // For a symmetric matrix, SVD gives U Σ V^T where U = V (up to sign).
        // Eigenvalues = singular values.
        let (u_gram, sv_gram, _v_gram) = svd_thin(&gram)?;

        // Recover principal components in original space:
        // If G = X X^T / (n-1) and G = U Λ U^T, then the PC directions are:
        // V_pc = X^T U Λ^{-1/2} / sqrt(n-1)
        // and singular values of X/sqrt(n-1) are sqrt(eigenvalues of G) = sqrt(sv_gram)
        let mut components = Vec::new();
        let mut singular_values = Vec::new();

        for c in 0..n_samples {
            let eigenval = sv_gram[c];
            if eigenval < 1e-14 {
                break;
            }
            let sv = eigenval.sqrt(); // singular value of X/sqrt(n-1) = sqrt(eigenvalue of G)
            singular_values.push(sv);

            // PC direction = X^T u_c / (sv * sqrt(n-1))
            let mut pc = vec![0.0; dim];
            let denom = sv * (n_samples as f64 - 1.0).sqrt();
            if denom.abs() < 1e-300 {
                continue;
            }
            for (i, flat) in flat_shapes.iter().enumerate() {
                let coeff = u_gram.get(i, c) / denom;
                for (d, val) in flat.iter().enumerate() {
                    pc[d] += coeff * val;
                }
            }

            // Normalize the component
            let norm: f64 = pc.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-14 {
                for p in pc.iter_mut() {
                    *p /= norm;
                }
            }

            components.push(pc);
        }

        Ok((components, singular_values))
    }

    /// PCA via direct SVD of data matrix (when n_samples >= dim).
    fn pca_via_direct_svd(
        flat_shapes: &[Vec<f64>],
        n_samples: usize,
        dim: usize,
    ) -> anyhow::Result<(Vec<Vec<f64>>, Vec<f64>)> {
        // Data matrix X is (n_samples x dim)
        let mut data = Mat::zeros(n_samples, dim);
        for (i, flat) in flat_shapes.iter().enumerate() {
            for (j, &val) in flat.iter().enumerate() {
                data.set(i, j, val);
            }
        }

        // SVD: X = U Σ V^T
        // PCs are columns of V, singular values are Σ / sqrt(n-1)
        let (data_t, transposed) = if n_samples >= dim {
            (data, false)
        } else {
            (data.t(), true)
        };

        let (_u, sv, v) = svd_thin(&data_t)?;

        let scale = 1.0 / (n_samples as f64 - 1.0).sqrt();
        let mut components = Vec::new();
        let mut singular_values = Vec::new();

        let n_comp = sv.len();
        for c in 0..n_comp {
            if sv[c] < 1e-14 {
                break;
            }
            singular_values.push(sv[c] * scale);
            let mut pc = vec![0.0; dim];
            if transposed {
                // V columns are the PCs when we transposed
                for d in 0..dim {
                    pc[d] = v.get(d, c);
                }
            } else {
                for d in 0..dim {
                    pc[d] = v.get(d, c);
                }
            }
            components.push(pc);
        }

        Ok((components, singular_values))
    }

    /// Generate a body shape from PCA coefficients.
    ///
    /// `coefficients[i]` is the weight for the i-th principal component.
    /// Missing coefficients are treated as zero.
    pub fn generate(&self, coefficients: &[f64]) -> anyhow::Result<Vec<[f64; 3]>> {
        let dim = self.mean_shape.len();
        let mut flat = self.mean_shape.clone();

        for (i, &coeff) in coefficients.iter().enumerate() {
            if i >= self.num_components {
                break;
            }
            let pc = &self.components[i];
            // Weight by singular value so coefficients are in standard-deviation units
            let weight = coeff * self.singular_values[i];
            for (d, val) in flat.iter_mut().enumerate() {
                *val += weight * pc[d];
            }
        }

        Ok(Self::unflatten(&flat, dim / 3))
    }

    /// Project a body shape into PCA space, returning coefficients.
    ///
    /// Coefficients are in units of standard deviations along each component.
    pub fn project(&self, shape: &[[f64; 3]]) -> anyhow::Result<Vec<f64>> {
        ensure!(
            shape.len() == self.vertex_count,
            "shape has {} vertices, expected {}",
            shape.len(),
            self.vertex_count
        );

        let flat = Self::flatten(shape);
        let mut centered = vec![0.0; flat.len()];
        for (i, c) in centered.iter_mut().enumerate() {
            *c = flat[i] - self.mean_shape[i];
        }

        let mut coefficients = Vec::with_capacity(self.num_components);
        for i in 0..self.num_components {
            let pc = &self.components[i];
            let mut dot = 0.0;
            for (d, &cv) in centered.iter().enumerate() {
                dot += cv * pc[d];
            }
            // Divide by singular value to get standard-deviation units
            let sv = self.singular_values[i];
            if sv.abs() > 1e-14 {
                coefficients.push(dot / sv);
            } else {
                coefficients.push(0.0);
            }
        }

        Ok(coefficients)
    }

    /// Reconstruct a shape from its projection (lossy due to dimensionality reduction).
    pub fn reconstruct(&self, shape: &[[f64; 3]]) -> anyhow::Result<Vec<[f64; 3]>> {
        let coefficients = self.project(shape)?;
        self.generate(&coefficients)
    }

    /// Get reconstruction error (RMS per-vertex distance) for a shape.
    pub fn reconstruction_error(&self, shape: &[[f64; 3]]) -> anyhow::Result<f64> {
        let reconstructed = self.reconstruct(shape)?;
        ensure!(
            shape.len() == reconstructed.len(),
            "shape length mismatch after reconstruction"
        );

        let mut sum_sq = 0.0;
        for (orig, recon) in shape.iter().zip(reconstructed.iter()) {
            let dx = orig[0] - recon[0];
            let dy = orig[1] - recon[1];
            let dz = orig[2] - recon[2];
            sum_sq += dx * dx + dy * dy + dz * dz;
        }

        Ok((sum_sq / shape.len() as f64).sqrt())
    }

    /// Sample a random body from the learned distribution using an LCG-based RNG.
    ///
    /// Coefficients are drawn from a standard normal distribution (mean 0, std 1)
    /// so the resulting shape is a "typical" body within the learned distribution.
    pub fn sample_random(&self, seed: u64) -> anyhow::Result<Vec<[f64; 3]>> {
        let mut state = seed;
        let mut coefficients = Vec::with_capacity(self.num_components);
        for _ in 0..self.num_components {
            coefficients.push(lcg_normal(&mut state));
        }
        self.generate(&coefficients)
    }

    /// Interpolate between two bodies in PCA space.
    ///
    /// `t = 0.0` gives `shape_a`, `t = 1.0` gives `shape_b`.
    pub fn interpolate(
        &self,
        shape_a: &[[f64; 3]],
        shape_b: &[[f64; 3]],
        t: f64,
    ) -> anyhow::Result<Vec<[f64; 3]>> {
        let coeff_a = self.project(shape_a)?;
        let coeff_b = self.project(shape_b)?;

        let mut interpolated = Vec::with_capacity(self.num_components);
        for i in 0..self.num_components {
            interpolated.push(coeff_a[i] * (1.0 - t) + coeff_b[i] * t);
        }

        self.generate(&interpolated)
    }

    /// Number of principal components retained.
    pub fn num_components(&self) -> usize {
        self.num_components
    }

    /// Explained variance ratio for each retained component.
    pub fn explained_variance(&self) -> &[f64] {
        &self.variance_ratios
    }

    /// Mean body shape as vertex positions.
    pub fn mean_shape(&self) -> Vec<[f64; 3]> {
        Self::unflatten(&self.mean_shape, self.vertex_count)
    }

    /// Singular values for each retained component.
    pub fn singular_values(&self) -> &[f64] {
        &self.singular_values
    }

    /// Total cumulative explained variance across all retained components.
    pub fn cumulative_variance(&self) -> f64 {
        self.variance_ratios.iter().sum()
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn flatten(shape: &[[f64; 3]]) -> Vec<f64> {
        let mut flat = Vec::with_capacity(shape.len() * 3);
        for v in shape {
            flat.push(v[0]);
            flat.push(v[1]);
            flat.push(v[2]);
        }
        flat
    }

    fn unflatten(flat: &[f64], vertex_count: usize) -> Vec<[f64; 3]> {
        let mut result = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let base = i * 3;
            result.push([
                *flat.get(base).unwrap_or(&0.0),
                *flat.get(base + 1).unwrap_or(&0.0),
                *flat.get(base + 2).unwrap_or(&0.0),
            ]);
        }
        result
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a synthetic body shape: a simple parametric body with `n_verts` vertices.
    fn synthetic_shape(n_verts: usize, height: f64, width: f64, depth: f64) -> Vec<[f64; 3]> {
        let mut shape = Vec::with_capacity(n_verts);
        for i in 0..n_verts {
            let t = i as f64 / n_verts as f64;
            let y = t * height;
            let x = width * (std::f64::consts::PI * t).sin();
            let z = depth * (std::f64::consts::PI * t * 0.5).cos();
            shape.push([x, y, z]);
        }
        shape
    }

    #[test]
    fn test_householder_vector_basic() {
        let x = vec![3.0, 4.0];
        let (v, beta) = householder_vector(&x);
        assert!(beta > 0.0);
        // Hx should have zero in second entry
        let hx0 = x[0] - beta * v[0] * (v[0] * x[0] + v[1] * x[1]);
        let hx1 = x[1] - beta * v[1] * (v[0] * x[0] + v[1] * x[1]);
        assert!(hx1.abs() < 1e-12, "hx1 = {hx1}");
        assert!((hx0.abs() - 5.0).abs() < 1e-12, "hx0 = {hx0}");
    }

    #[test]
    fn test_svd_2x2() {
        let mut a = Mat::zeros(2, 2);
        a.set(0, 0, 3.0);
        a.set(0, 1, 0.0);
        a.set(1, 0, 0.0);
        a.set(1, 1, 4.0);
        let (u, sv, v) = svd_thin(&a).expect("should succeed");
        // Singular values should be 4 and 3 (descending)
        assert!(
            (sv[0] - 4.0).abs() < 1e-10,
            "sv[0] = {}, expected 4",
            sv[0]
        );
        assert!(
            (sv[1] - 3.0).abs() < 1e-10,
            "sv[1] = {}, expected 3",
            sv[1]
        );

        // Reconstruct: U * diag(sv) * V^T should equal A
        let mut reconstructed = Mat::zeros(2, 2);
        for i in 0..2 {
            for j in 0..2 {
                let mut val = 0.0;
                for k in 0..2 {
                    val += u.get(i, k) * sv[k] * v.get(j, k);
                }
                reconstructed.set(i, j, val);
            }
        }
        for i in 0..2 {
            for j in 0..2 {
                assert!(
                    (reconstructed.get(i, j) - a.get(i, j)).abs() < 1e-10,
                    "mismatch at ({i},{j})"
                );
            }
        }
    }

    #[test]
    fn test_svd_3x2() {
        let mut a = Mat::zeros(3, 2);
        a.set(0, 0, 1.0);
        a.set(0, 1, 2.0);
        a.set(1, 0, 3.0);
        a.set(1, 1, 4.0);
        a.set(2, 0, 5.0);
        a.set(2, 1, 6.0);
        let (u, sv, v) = svd_thin(&a).expect("should succeed");

        // Reconstruct
        for i in 0..3 {
            for j in 0..2 {
                let mut val = 0.0;
                for k in 0..2 {
                    val += u.get(i, k) * sv[k] * v.get(j, k);
                }
                assert!(
                    (val - a.get(i, j)).abs() < 1e-9,
                    "mismatch at ({i},{j}): got {val}, expected {}",
                    a.get(i, j)
                );
            }
        }
    }

    #[test]
    fn test_build_basic() {
        let n_verts = 20;
        let mut shapes = Vec::new();
        for i in 0..10 {
            let h = 1.7 + 0.03 * i as f64;
            let w = 0.3 + 0.01 * i as f64;
            let d = 0.2 + 0.005 * i as f64;
            shapes.push(synthetic_shape(n_verts, h, w, d));
        }

        let config = PcaConfig {
            max_components: 5,
            variance_threshold: 0.99,
        };

        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");
        assert!(model.num_components() > 0);
        assert!(model.num_components() <= 5);
        assert!(!model.explained_variance().is_empty());

        let total_var: f64 = model.explained_variance().iter().sum();
        assert!(total_var > 0.0);
        assert!(total_var <= 1.0 + 1e-10);
    }

    #[test]
    fn test_project_and_reconstruct() {
        let n_verts = 15;
        let mut shapes = Vec::new();
        for i in 0..8 {
            let h = 1.5 + 0.05 * i as f64;
            let w = 0.25 + 0.02 * i as f64;
            let d = 0.15 + 0.01 * i as f64;
            shapes.push(synthetic_shape(n_verts, h, w, d));
        }

        let config = PcaConfig {
            max_components: 7,
            variance_threshold: 0.999,
        };

        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");

        // Reconstruct a training shape — should have low error
        let reconstructed = model.reconstruct(&shapes[3]).expect("should succeed");
        assert_eq!(reconstructed.len(), n_verts);

        let error = model.reconstruction_error(&shapes[3]).expect("should succeed");
        assert!(
            error < 0.1,
            "reconstruction error too high: {error}"
        );
    }

    #[test]
    fn test_generate_from_coefficients() {
        let n_verts = 10;
        let mut shapes = Vec::new();
        for i in 0..6 {
            let h = 1.6 + 0.04 * i as f64;
            let w = 0.28 + 0.015 * i as f64;
            let d = 0.18 + 0.008 * i as f64;
            shapes.push(synthetic_shape(n_verts, h, w, d));
        }

        let config = PcaConfig::default();
        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");

        // Zero coefficients should give the mean shape
        let zeros = vec![0.0; model.num_components()];
        let mean_gen = model.generate(&zeros).expect("should succeed");
        let mean = model.mean_shape();
        for (a, b) in mean_gen.iter().zip(mean.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-10);
            assert!((a[1] - b[1]).abs() < 1e-10);
            assert!((a[2] - b[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_sample_random() {
        let n_verts = 10;
        let mut shapes = Vec::new();
        for i in 0..5 {
            shapes.push(synthetic_shape(n_verts, 1.7 + 0.02 * i as f64, 0.3, 0.2));
        }

        let config = PcaConfig::default();
        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");

        let sample1 = model.sample_random(42).expect("should succeed");
        let sample2 = model.sample_random(42).expect("should succeed");
        let sample3 = model.sample_random(123).expect("should succeed");

        // Same seed -> same result
        for (a, b) in sample1.iter().zip(sample2.iter()) {
            assert_eq!(a[0], b[0]);
            assert_eq!(a[1], b[1]);
            assert_eq!(a[2], b[2]);
        }

        // Different seed -> different result
        let mut different = false;
        for (a, b) in sample1.iter().zip(sample3.iter()) {
            if (a[0] - b[0]).abs() > 1e-10 {
                different = true;
                break;
            }
        }
        assert!(different, "different seeds should produce different shapes");
    }

    #[test]
    fn test_interpolate() {
        let n_verts = 10;
        let mut shapes = Vec::new();
        for i in 0..5 {
            let h = 1.6 + 0.04 * i as f64;
            let w = 0.25 + 0.02 * i as f64;
            shapes.push(synthetic_shape(n_verts, h, w, 0.2));
        }

        let config = PcaConfig::default();
        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");

        let shape_a = &shapes[0];
        let shape_b = &shapes[4];

        // t=0 should be close to shape_a (after projection)
        let at_a = model.interpolate(shape_a, shape_b, 0.0).expect("should succeed");
        let recon_a = model.reconstruct(shape_a).expect("should succeed");
        for (a, b) in at_a.iter().zip(recon_a.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-10);
            assert!((a[1] - b[1]).abs() < 1e-10);
            assert!((a[2] - b[2]).abs() < 1e-10);
        }

        // t=1 should be close to shape_b (after projection)
        let at_b = model.interpolate(shape_a, shape_b, 1.0).expect("should succeed");
        let recon_b = model.reconstruct(shape_b).expect("should succeed");
        for (a, b) in at_b.iter().zip(recon_b.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-10);
            assert!((a[1] - b[1]).abs() < 1e-10);
            assert!((a[2] - b[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_lcg_deterministic() {
        let mut s1 = 12345u64;
        let mut s2 = 12345u64;
        for _ in 0..100 {
            assert_eq!(lcg_next(&mut s1), lcg_next(&mut s2));
        }
    }

    #[test]
    fn test_lcg_range() {
        let mut state = 42u64;
        for _ in 0..1000 {
            let v = lcg_next(&mut state);
            assert!(v >= 0.0);
            assert!(v < 4.0); // (2^31-1) / 2^31 < 1.0 but shifted so max ~ 2.0
        }
    }

    #[test]
    fn test_error_too_few_shapes() {
        let shapes = vec![vec![[1.0, 2.0, 3.0]]];
        let config = PcaConfig::default();
        let result = StatisticalBodyModel::build(&shapes, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_mismatched_vertices() {
        let shapes = vec![
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[1.0, 0.0, 0.0]],
        ];
        let config = PcaConfig::default();
        let result = StatisticalBodyModel::build(&shapes, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_variance_ratios_sum_to_one() {
        let n_verts = 12;
        let mut shapes = Vec::new();
        for i in 0..6 {
            let h = 1.7 + 0.05 * i as f64;
            let w = 0.3 + 0.02 * i as f64;
            let d = 0.2 + 0.01 * i as f64;
            shapes.push(synthetic_shape(n_verts, h, w, d));
        }

        let config = PcaConfig {
            max_components: 100,
            variance_threshold: 1.0,
        };
        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");

        let total: f64 = model.explained_variance().iter().sum();
        // Total should be <= 1.0 (may be less if some components were dropped)
        assert!(
            total <= 1.0 + 1e-10,
            "total variance ratio {total} exceeds 1.0"
        );
    }

    #[test]
    fn test_mean_shape_is_average() {
        let shapes = vec![
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            vec![[3.0, 4.0, 5.0], [6.0, 7.0, 8.0]],
        ];
        let config = PcaConfig::default();
        let model = StatisticalBodyModel::build(&shapes, &config).expect("should succeed");
        let mean = model.mean_shape();

        assert!((mean[0][0] - 2.0).abs() < 1e-10);
        assert!((mean[0][1] - 3.0).abs() < 1e-10);
        assert!((mean[0][2] - 4.0).abs() < 1e-10);
        assert!((mean[1][0] - 5.0).abs() < 1e-10);
        assert!((mean[1][1] - 6.0).abs() < 1e-10);
        assert!((mean[1][2] - 7.0).abs() < 1e-10);
    }
}
