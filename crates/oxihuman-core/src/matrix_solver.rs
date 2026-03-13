// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

//! Gaussian elimination for Ax=b systems (small dense matrices).

/// Solve a dense linear system Ax = b using Gaussian elimination with partial pivoting.
/// Returns None if the matrix is singular.
pub fn gaussian_solve(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = b.len();
    if a.len() != n || a.iter().any(|row| row.len() != n) {
        return None;
    }
    let mut mat: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = a[i].clone();
            row.push(b[i]);
            row
        })
        .collect();

    for col in 0..n {
        /* partial pivot */
        let mut max_row = col;
        let mut max_val = mat[col][col].abs();
        for row in (col + 1)..n {
            if mat[row][col].abs() > max_val {
                max_val = mat[row][col].abs();
                max_row = row;
            }
        }
        mat.swap(col, max_row);

        let pivot = mat[col][col];
        if pivot.abs() < 1e-15 {
            return None;
        }

        for row in (col + 1)..n {
            let factor = mat[row][col] / pivot;
            for k in col..=n {
                let sub = factor * mat[col][k];
                mat[row][k] -= sub;
            }
        }
    }

    /* back-substitution */
    let mut x = vec![0.0f64; n];
    for i in (0..n).rev() {
        let mut sum = mat[i][n];
        for j in (i + 1)..n {
            sum -= mat[i][j] * x[j];
        }
        x[i] = sum / mat[i][i];
    }
    Some(x)
}

/// Compute the residual ||Ax - b||_inf.
pub fn residual_norm(a: &[Vec<f64>], b: &[f64], x: &[f64]) -> f64 {
    let n = b.len();
    let mut max_err = 0.0f64;
    for i in 0..n {
        let ax_i: f64 = a[i].iter().zip(x).map(|(&a, &x)| a * x).sum();
        max_err = max_err.max((ax_i - b[i]).abs());
    }
    max_err
}

/// Identity matrix of size n.
pub fn identity_matrix(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect()
}

/// Matrix-vector product.
pub fn mat_vec_mul(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    a.iter()
        .map(|row| row.iter().zip(x).map(|(&a, &x)| a * x).sum())
        .collect()
}

/// Matrix determinant via Gaussian elimination.
pub fn determinant(a: &[Vec<f64>]) -> Option<f64> {
    let n = a.len();
    if a.iter().any(|r| r.len() != n) {
        return None;
    }
    let mut mat: Vec<Vec<f64>> = a.to_vec();
    let mut det = 1.0f64;
    for col in 0..n {
        let mut max_row = col;
        for row in (col + 1)..n {
            if mat[row][col].abs() > mat[max_row][col].abs() {
                max_row = row;
            }
        }
        if max_row != col {
            mat.swap(col, max_row);
            det = -det;
        }
        let pivot = mat[col][col];
        if pivot.abs() < 1e-15 {
            return Some(0.0);
        }
        det *= pivot;
        for row in (col + 1)..n {
            let factor = mat[row][col] / pivot;
            for k in col..n {
                let sub = factor * mat[col][k];
                mat[row][k] -= sub;
            }
        }
    }
    Some(det)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_2x2() {
        /* solve 2x2 system: [[2,1],[1,3]] x = [5,10] -> x=[1,3] */
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let b = vec![5.0, 10.0];
        let x = gaussian_solve(&a, &b).expect("should succeed");
        assert!((x[0] - 1.0).abs() < 1e-9);
        assert!((x[1] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_solve_identity() {
        /* Ix = b has solution x = b */
        let a = identity_matrix(3);
        let b = vec![1.0, 2.0, 3.0];
        let x = gaussian_solve(&a, &b).expect("should succeed");
        for i in 0..3 {
            assert!((x[i] - b[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn test_singular_returns_none() {
        /* singular matrix returns None */
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let b = vec![1.0, 2.0];
        assert!(gaussian_solve(&a, &b).is_none());
    }

    #[test]
    fn test_residual_norm() {
        /* residual of exact solution is near 0 */
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let b = vec![5.0, 10.0];
        let x = gaussian_solve(&a, &b).expect("should succeed");
        let r = residual_norm(&a, &b, &x);
        assert!(r < 1e-9, "r={r}");
    }

    #[test]
    fn test_mat_vec_mul() {
        /* mat_vec_mul computes product correctly */
        let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let x = vec![1.0, 1.0];
        let y = mat_vec_mul(&a, &x);
        assert!((y[0] - 3.0).abs() < 1e-9);
        assert!((y[1] - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_determinant_2x2() {
        /* determinant of [[1,2],[3,4]] = -2 */
        let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let d = determinant(&a).expect("should succeed");
        assert!((d - (-2.0)).abs() < 1e-9, "d={d}");
    }

    #[test]
    fn test_determinant_singular() {
        /* singular matrix has det = 0 */
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let d = determinant(&a).expect("should succeed");
        assert!(d.abs() < 1e-9, "d={d}");
    }

    #[test]
    fn test_solve_3x3() {
        /* solve 3x3 system */
        let a = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 2.0, 0.0],
            vec![0.0, 0.0, 3.0],
        ];
        let b = vec![1.0, 4.0, 9.0];
        let x = gaussian_solve(&a, &b).expect("should succeed");
        assert!((x[0] - 1.0).abs() < 1e-9);
        assert!((x[1] - 2.0).abs() < 1e-9);
        assert!((x[2] - 3.0).abs() < 1e-9);
    }
}
