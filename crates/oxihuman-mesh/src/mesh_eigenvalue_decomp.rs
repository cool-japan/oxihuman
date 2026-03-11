// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple eigenvalue decomposition for 3x3 symmetric matrices (covariance of point clouds).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Eigen3 {
    pub eigenvalues: [f32; 3],
    pub eigenvectors: [[f32; 3]; 3],
}

#[allow(dead_code)]
pub fn covariance_matrix(positions: &[[f32; 3]]) -> [[f32; 3]; 3] {
    if positions.is_empty() { return [[0.0; 3]; 3]; }
    let n = positions.len() as f64;
    let mean = [
        positions.iter().map(|p| p[0] as f64).sum::<f64>() / n,
        positions.iter().map(|p| p[1] as f64).sum::<f64>() / n,
        positions.iter().map(|p| p[2] as f64).sum::<f64>() / n,
    ];
    let mut cov = [[0.0f64; 3]; 3];
    for p in positions {
        let d = [p[0] as f64 - mean[0], p[1] as f64 - mean[1], p[2] as f64 - mean[2]];
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 { for j in 0..3 { cov[i][j] += d[i] * d[j]; } }
    }
    let mut result = [[0.0f32; 3]; 3];
    #[allow(clippy::needless_range_loop)]
    for i in 0..3 { for j in 0..3 { result[i][j] = (cov[i][j] / n) as f32; } }
    result
}

/// Jacobi iteration for 3x3 symmetric matrix eigenvalue decomposition.
#[allow(dead_code)]
pub fn eigen_decompose_3x3(mat: &[[f32; 3]; 3]) -> Eigen3 {
    let mut a = [[mat[0][0] as f64, mat[0][1] as f64, mat[0][2] as f64],
                 [mat[1][0] as f64, mat[1][1] as f64, mat[1][2] as f64],
                 [mat[2][0] as f64, mat[2][1] as f64, mat[2][2] as f64]];
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..50 {
        let mut p = 0usize; let mut q = 1usize;
        let mut mx = a[0][1].abs();
        if a[0][2].abs() > mx { p = 0; q = 2; mx = a[0][2].abs(); }
        if a[1][2].abs() > mx { p = 1; q = 2; }
        let _ = mx;
        let apq = a[p][q];
        if apq.abs() < 1e-15 { break; }
        let tau = (a[q][q] - a[p][p]) / (2.0 * apq);
        let t = if tau >= 0.0 { 1.0 / (tau + (1.0 + tau * tau).sqrt()) } else { -1.0 / (-tau + (1.0 + tau * tau).sqrt()) };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        let mut ap = [0.0f64; 3]; let mut aq = [0.0f64; 3];
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 { ap[i] = c * a[i][p] - s * a[i][q]; aq[i] = s * a[i][p] + c * a[i][q]; }
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 { a[i][p] = ap[i]; a[i][q] = aq[i]; a[p][i] = ap[i]; a[q][i] = aq[i]; }
        a[p][p] = c * ap[p] - s * ap[q];
        a[q][q] = s * aq[p] + c * aq[q];
        a[p][q] = 0.0; a[q][p] = 0.0;
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 { let vp = c*v[i][p]-s*v[i][q]; let vq = s*v[i][p]+c*v[i][q]; v[i][p]=vp; v[i][q]=vq; }
    }
    let eigenvalues = [a[0][0] as f32, a[1][1] as f32, a[2][2] as f32];
    let eigenvectors = [[v[0][0] as f32, v[1][0] as f32, v[2][0] as f32],
                         [v[0][1] as f32, v[1][1] as f32, v[2][1] as f32],
                         [v[0][2] as f32, v[1][2] as f32, v[2][2] as f32]];
    Eigen3 { eigenvalues, eigenvectors }
}

#[allow(dead_code)]
pub fn principal_axis(eigen: &Eigen3) -> usize {
    let mut idx = 0;
    if eigen.eigenvalues[1] > eigen.eigenvalues[idx] { idx = 1; }
    if eigen.eigenvalues[2] > eigen.eigenvalues[idx] { idx = 2; }
    idx
}

#[allow(dead_code)]
pub fn eigen_to_json(e: &Eigen3) -> String {
    format!("{{\"ev\":[{:.4},{:.4},{:.4}]}}", e.eigenvalues[0], e.eigenvalues[1], e.eigenvalues[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts() -> Vec<[f32; 3]> {
        vec![[1.0,0.0,0.0],[-1.0,0.0,0.0],[0.0,0.5,0.0],[0.0,-0.5,0.0],[0.0,0.0,0.1],[0.0,0.0,-0.1]]
    }

    #[test] fn test_cov_empty() { let c = covariance_matrix(&[]); assert!((c[0][0]).abs() < 1e-6); }
    #[test] fn test_cov_basic() { let c = covariance_matrix(&pts()); assert!(c[0][0] > 0.0); }
    #[test] fn test_eigen_basic() { let c = covariance_matrix(&pts()); let e = eigen_decompose_3x3(&c); assert!(e.eigenvalues[0].abs() + e.eigenvalues[1].abs() + e.eigenvalues[2].abs() > 0.0); }
    #[test] fn test_principal() { let c = covariance_matrix(&pts()); let e = eigen_decompose_3x3(&c); let p = principal_axis(&e); assert!(p < 3); }
    #[test] fn test_identity() {
        let m = [[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]];
        let e = eigen_decompose_3x3(&m);
        let mut vals = e.eigenvalues.to_vec();
        vals.sort_by(|a,b| a.partial_cmp(b).unwrap());
        assert!((vals[0] - 1.0).abs() < 0.01);
    }
    #[test] fn test_symmetric() { let c = covariance_matrix(&pts()); assert!((c[0][1] - c[1][0]).abs() < 1e-6); }
    #[test] fn test_eigen_positive() { let c = covariance_matrix(&pts()); let e = eigen_decompose_3x3(&c); assert!(e.eigenvalues.iter().all(|&v| v >= -1e-4)); }
    #[test] fn test_to_json() { let c = covariance_matrix(&pts()); let e = eigen_decompose_3x3(&c); assert!(eigen_to_json(&e).contains("ev")); }
    #[test] fn test_single_point() { let c = covariance_matrix(&[[1.0,2.0,3.0]]); assert!(c[0][0].abs() < 1e-6); }
    #[test] fn test_two_points() { let c = covariance_matrix(&[[0.0,0.0,0.0],[2.0,0.0,0.0]]); assert!(c[0][0] > 0.0); }
}
