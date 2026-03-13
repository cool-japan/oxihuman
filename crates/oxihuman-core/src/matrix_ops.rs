// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct Matrix {
    pub data: Vec<f64>,
    pub rows: usize,
    pub cols: usize,
}

#[allow(dead_code)]
pub fn new_matrix(rows: usize, cols: usize) -> Matrix {
    Matrix { data: vec![0.0; rows * cols], rows, cols }
}

#[allow(dead_code)]
pub fn mat_set(m: &mut Matrix, r: usize, c: usize, v: f64) {
    m.data[r * m.cols + c] = v;
}

#[allow(dead_code)]
pub fn mat_get(m: &Matrix, r: usize, c: usize) -> f64 {
    m.data[r * m.cols + c]
}

#[allow(dead_code)]
pub fn mat_add(a: &Matrix, b: &Matrix) -> Option<Matrix> {
    if a.rows != b.rows || a.cols != b.cols {
        return None;
    }
    let data: Vec<f64> = a.data.iter().zip(b.data.iter()).map(|(x, y)| x + y).collect();
    Some(Matrix { data, rows: a.rows, cols: a.cols })
}

#[allow(dead_code)]
pub fn mat_mul(a: &Matrix, b: &Matrix) -> Option<Matrix> {
    if a.cols != b.rows {
        return None;
    }
    let mut result = new_matrix(a.rows, b.cols);
    #[allow(clippy::needless_range_loop)]
    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut sum = 0.0f64;
            for k in 0..a.cols {
                sum += mat_get(a, i, k) * mat_get(b, k, j);
            }
            mat_set(&mut result, i, j, sum);
        }
    }
    Some(result)
}

#[allow(dead_code)]
pub fn mat_transpose(m: &Matrix) -> Matrix {
    let mut result = new_matrix(m.cols, m.rows);
    #[allow(clippy::needless_range_loop)]
    for i in 0..m.rows {
        for j in 0..m.cols {
            mat_set(&mut result, j, i, mat_get(m, i, j));
        }
    }
    result
}

#[allow(dead_code)]
pub fn mat_trace(m: &Matrix) -> f64 {
    let n = m.rows.min(m.cols);
    (0..n).map(|i| mat_get(m, i, i)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut m = new_matrix(2, 2);
        mat_set(&mut m, 0, 1, 2.71);
        assert!((mat_get(&m, 0, 1) - 2.71).abs() < 1e-10);
    }

    #[test]
    fn test_add() {
        let mut a = new_matrix(2, 2);
        let mut b = new_matrix(2, 2);
        mat_set(&mut a, 0, 0, 1.0);
        mat_set(&mut b, 0, 0, 2.0);
        let c = mat_add(&a, &b).expect("should succeed");
        assert!((mat_get(&c, 0, 0) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_add_dimension_mismatch() {
        let a = new_matrix(2, 3);
        let b = new_matrix(3, 2);
        assert!(mat_add(&a, &b).is_none());
    }

    #[test]
    fn test_mul() {
        let mut a = new_matrix(2, 2);
        let mut b = new_matrix(2, 2);
        mat_set(&mut a, 0, 0, 1.0);
        mat_set(&mut a, 0, 1, 2.0);
        mat_set(&mut a, 1, 0, 3.0);
        mat_set(&mut a, 1, 1, 4.0);
        mat_set(&mut b, 0, 0, 1.0);
        mat_set(&mut b, 1, 1, 1.0);
        let c = mat_mul(&a, &b).expect("should succeed");
        assert!((mat_get(&c, 0, 0) - 1.0).abs() < 1e-10);
        assert!((mat_get(&c, 1, 1) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_mul_dimension_mismatch() {
        let a = new_matrix(2, 3);
        let b = new_matrix(2, 2);
        assert!(mat_mul(&a, &b).is_none());
    }

    #[test]
    fn test_transpose() {
        let mut m = new_matrix(2, 3);
        mat_set(&mut m, 0, 2, 5.0);
        let t = mat_transpose(&m);
        assert_eq!(t.rows, 3);
        assert_eq!(t.cols, 2);
        assert!((mat_get(&t, 2, 0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_trace() {
        let mut m = new_matrix(3, 3);
        mat_set(&mut m, 0, 0, 1.0);
        mat_set(&mut m, 1, 1, 2.0);
        mat_set(&mut m, 2, 2, 3.0);
        assert!((mat_trace(&m) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_matrix() {
        let m = new_matrix(3, 3);
        assert!((mat_trace(&m)).abs() < 1e-10);
    }
}
