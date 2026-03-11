// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Align mesh to principal axes via PCA (3×3 covariance matrix).

/// Result of PCA axis alignment.
#[allow(dead_code)]
pub struct AlignAxesResult {
    pub positions: Vec<[f32; 3]>,
    pub centroid: [f32; 3],
    pub principal_axes: [[f32; 3]; 3],
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-9 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute 3×3 covariance matrix of point cloud (stored row-major).
#[allow(dead_code)]
pub fn covariance_3x3(positions: &[[f32; 3]]) -> [f32; 9] {
    if positions.is_empty() {
        return [0.0; 9];
    }
    let n = positions.len() as f32;
    let mut c = [0.0_f32; 3];
    for p in positions {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let centroid = [c[0] / n, c[1] / n, c[2] / n];
    let mut cov = [0.0_f32; 9];
    for p in positions {
        let d = sub3(*p, centroid);
        cov[0] += d[0] * d[0];
        cov[1] += d[0] * d[1];
        cov[2] += d[0] * d[2];
        cov[3] += d[1] * d[0];
        cov[4] += d[1] * d[1];
        cov[5] += d[1] * d[2];
        cov[6] += d[2] * d[0];
        cov[7] += d[2] * d[1];
        cov[8] += d[2] * d[2];
    }
    let inv = 1.0 / n;
    for v in &mut cov {
        *v *= inv;
    }
    cov
}

/// Power iteration to find dominant eigenvector of symmetric 3×3 matrix.
fn power_iterate(mat: [f32; 9], iters: usize) -> [f32; 3] {
    let mut v = [1.0_f32, 0.5, 0.25];
    for _ in 0..iters {
        let mv = [
            mat[0] * v[0] + mat[1] * v[1] + mat[2] * v[2],
            mat[3] * v[0] + mat[4] * v[1] + mat[5] * v[2],
            mat[6] * v[0] + mat[7] * v[1] + mat[8] * v[2],
        ];
        v = normalize3(mv);
    }
    v
}

/// Deflate matrix by removing contribution of eigenvector.
fn deflate(mat: [f32; 9], ev: [f32; 3], eigenval: f32) -> [f32; 9] {
    let mut out = mat;
    for i in 0..3 {
        for j in 0..3 {
            out[i * 3 + j] -= eigenval * ev[i] * ev[j];
        }
    }
    out
}

/// Compute Rayleigh quotient (eigenvalue estimate).
fn rayleigh(mat: [f32; 9], v: [f32; 3]) -> f32 {
    let mv = [
        mat[0] * v[0] + mat[1] * v[1] + mat[2] * v[2],
        mat[3] * v[0] + mat[4] * v[1] + mat[5] * v[2],
        mat[6] * v[0] + mat[7] * v[1] + mat[8] * v[2],
    ];
    dot3(v, mv)
}

/// Compute centroid of positions.
#[allow(dead_code)]
pub fn compute_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let n = positions.len() as f32;
    let mut c = [0.0_f32; 3];
    for p in positions {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    [c[0] / n, c[1] / n, c[2] / n]
}

/// Align mesh to principal axes via PCA.
#[allow(dead_code)]
pub fn align_to_principal_axes(positions: &[[f32; 3]]) -> AlignAxesResult {
    let centroid = compute_centroid(positions);
    let cov = covariance_3x3(positions);
    let e0 = power_iterate(cov, 32);
    let lam0 = rayleigh(cov, e0);
    let cov2 = deflate(cov, e0, lam0);
    let e1_raw = power_iterate(cov2, 32);
    let e1 = normalize3(sub3(
        e1_raw,
        [
            e0[0] * dot3(e0, e1_raw),
            e0[1] * dot3(e0, e1_raw),
            e0[2] * dot3(e0, e1_raw),
        ],
    ));
    let e2 = normalize3(cross3(e0, e1));
    let aligned: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| {
            let d = sub3(*p, centroid);
            [dot3(d, e0), dot3(d, e1), dot3(d, e2)]
        })
        .collect();
    AlignAxesResult {
        positions: aligned,
        centroid,
        principal_axes: [e0, e1, e2],
    }
}

/// Compute variance along the first principal axis.
#[allow(dead_code)]
pub fn variance_along_axis(positions: &[[f32; 3]], axis: [f32; 3]) -> f32 {
    let centroid = compute_centroid(positions);
    let mut sum = 0.0_f32;
    for p in positions {
        let d = dot3(sub3(*p, centroid), axis);
        sum += d * d;
    }
    if positions.is_empty() {
        0.0
    } else {
        sum / positions.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elongated_cloud() -> Vec<[f32; 3]> {
        (0..20).map(|i| [i as f32, i as f32 * 0.1, 0.0]).collect()
    }

    #[test]
    fn covariance_is_symmetric() {
        let pts = elongated_cloud();
        let cov = covariance_3x3(&pts);
        assert!((cov[1] - cov[3]).abs() < 1e-5);
        assert!((cov[2] - cov[6]).abs() < 1e-5);
        assert!((cov[5] - cov[7]).abs() < 1e-5);
    }

    #[test]
    fn covariance_empty_returns_zeros() {
        let cov = covariance_3x3(&[]);
        for v in cov {
            assert_eq!(v, 0.0);
        }
    }

    #[test]
    fn align_preserves_vertex_count() {
        let pts = elongated_cloud();
        let result = align_to_principal_axes(&pts);
        assert_eq!(result.positions.len(), pts.len());
    }

    #[test]
    fn aligned_centroid_near_origin() {
        let pts = elongated_cloud();
        let result = align_to_principal_axes(&pts);
        let c = compute_centroid(&result.positions);
        assert!(c[0].abs() < 1e-4 && c[1].abs() < 1e-4);
    }

    #[test]
    fn principal_axes_near_orthogonal() {
        let pts = elongated_cloud();
        let result = align_to_principal_axes(&pts);
        let d01 = dot3(result.principal_axes[0], result.principal_axes[1]).abs();
        assert!(d01 < 0.1, "axes not nearly orthogonal: {d01}");
    }

    #[test]
    fn variance_along_axis_nonneg() {
        let pts = elongated_cloud();
        let axis = [1.0, 0.0, 0.0];
        let v = variance_along_axis(&pts, axis);
        assert!(v >= 0.0);
    }

    #[test]
    fn compute_centroid_simple() {
        let pts = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let c = compute_centroid(&pts);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn compute_centroid_empty() {
        let c = compute_centroid(&[]);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn align_single_point() {
        let pts = vec![[3.0, 2.0, 1.0]];
        let result = align_to_principal_axes(&pts);
        assert_eq!(result.positions.len(), 1);
    }

    #[test]
    fn power_iterate_produces_unit_vec() {
        let mat = [1.0_f32, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.5];
        let v = power_iterate(mat, 64);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }
}
