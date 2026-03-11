// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2×2 matrix with determinant, inverse, multiply, and add.

#![allow(dead_code)]

/// A 2×2 matrix stored in row-major order: [[row0col0, row0col1], [row1col0, row1col1]].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2 {
    pub m: [[f32; 2]; 2],
}

/// Create a Mat2 from row-major values.
#[allow(dead_code)]
pub fn mat2(a: f32, b: f32, c: f32, d: f32) -> Mat2 {
    Mat2 {
        m: [[a, b], [c, d]],
    }
}

/// Identity matrix.
#[allow(dead_code)]
pub fn mat2_identity() -> Mat2 {
    mat2(1.0, 0.0, 0.0, 1.0)
}

/// Zero matrix.
#[allow(dead_code)]
pub fn mat2_zero() -> Mat2 {
    mat2(0.0, 0.0, 0.0, 0.0)
}

/// Determinant of a 2×2 matrix.
#[allow(dead_code)]
pub fn mat2_det(m: &Mat2) -> f32 {
    m.m[0][0] * m.m[1][1] - m.m[0][1] * m.m[1][0]
}

/// Inverse of a 2×2 matrix. Returns None if singular.
#[allow(dead_code)]
pub fn mat2_inverse(m: &Mat2) -> Option<Mat2> {
    let det = mat2_det(m);
    if det.abs() < 1e-10 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some(mat2(
        m.m[1][1] * inv_det,
        -m.m[0][1] * inv_det,
        -m.m[1][0] * inv_det,
        m.m[0][0] * inv_det,
    ))
}

/// Transpose of a 2×2 matrix.
#[allow(dead_code)]
pub fn mat2_transpose(m: &Mat2) -> Mat2 {
    mat2(m.m[0][0], m.m[1][0], m.m[0][1], m.m[1][1])
}

/// Multiply two 2×2 matrices: result = a * b.
#[allow(dead_code)]
pub fn mat2_mul(a: &Mat2, b: &Mat2) -> Mat2 {
    mat2(
        a.m[0][0] * b.m[0][0] + a.m[0][1] * b.m[1][0],
        a.m[0][0] * b.m[0][1] + a.m[0][1] * b.m[1][1],
        a.m[1][0] * b.m[0][0] + a.m[1][1] * b.m[1][0],
        a.m[1][0] * b.m[0][1] + a.m[1][1] * b.m[1][1],
    )
}

/// Add two 2×2 matrices.
#[allow(dead_code)]
pub fn mat2_add(a: &Mat2, b: &Mat2) -> Mat2 {
    mat2(
        a.m[0][0] + b.m[0][0],
        a.m[0][1] + b.m[0][1],
        a.m[1][0] + b.m[1][0],
        a.m[1][1] + b.m[1][1],
    )
}

/// Scale a 2×2 matrix by a scalar.
#[allow(dead_code)]
pub fn mat2_scale(m: &Mat2, s: f32) -> Mat2 {
    mat2(m.m[0][0] * s, m.m[0][1] * s, m.m[1][0] * s, m.m[1][1] * s)
}

/// Multiply a 2×2 matrix by a 2D column vector.
#[allow(dead_code)]
pub fn mat2_mul_vec(m: &Mat2, v: [f32; 2]) -> [f32; 2] {
    [
        m.m[0][0] * v[0] + m.m[0][1] * v[1],
        m.m[1][0] * v[0] + m.m[1][1] * v[1],
    ]
}

/// Create a 2D rotation matrix for angle `theta` (radians).
#[allow(dead_code)]
pub fn mat2_rotation(theta: f32) -> Mat2 {
    let c = theta.cos();
    let s = theta.sin();
    mat2(c, -s, s, c)
}

/// Trace of the matrix (sum of diagonal elements).
#[allow(dead_code)]
pub fn mat2_trace(m: &Mat2) -> f32 {
    m.m[0][0] + m.m[1][1]
}

/// Check if two matrices are approximately equal within `eps`.
#[allow(dead_code)]
pub fn mat2_approx_eq(a: &Mat2, b: &Mat2, eps: f32) -> bool {
    (a.m[0][0] - b.m[0][0]).abs() < eps
        && (a.m[0][1] - b.m[0][1]).abs() < eps
        && (a.m[1][0] - b.m[1][0]).abs() < eps
        && (a.m[1][1] - b.m[1][1]).abs() < eps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity_det() {
        let id = mat2_identity();
        assert!((mat2_det(&id) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_zero_det() {
        let z = mat2_zero();
        assert_eq!(mat2_det(&z), 0.0);
    }

    #[test]
    fn test_inverse() {
        let m = mat2(2.0, 1.0, 5.0, 3.0);
        let inv = mat2_inverse(&m).unwrap();
        let prod = mat2_mul(&m, &inv);
        assert!(mat2_approx_eq(&prod, &mat2_identity(), 1e-5));
    }

    #[test]
    fn test_singular_inverse() {
        let m = mat2(1.0, 2.0, 2.0, 4.0);
        assert!(mat2_inverse(&m).is_none());
    }

    #[test]
    fn test_mul_identity() {
        let m = mat2(3.0, 7.0, 2.0, 5.0);
        let id = mat2_identity();
        let result = mat2_mul(&m, &id);
        assert!(mat2_approx_eq(&result, &m, 1e-5));
    }

    #[test]
    fn test_transpose() {
        let m = mat2(1.0, 2.0, 3.0, 4.0);
        let t = mat2_transpose(&m);
        assert!((t.m[0][1] - 3.0).abs() < 1e-5);
        assert!((t.m[1][0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_mul_vec() {
        let m = mat2(1.0, 0.0, 0.0, 2.0);
        let v = mat2_mul_vec(&m, [3.0, 4.0]);
        assert!((v[0] - 3.0).abs() < 1e-5);
        assert!((v[1] - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotation_matrix() {
        let r = mat2_rotation(PI / 2.0);
        let v = mat2_mul_vec(&r, [1.0, 0.0]);
        assert!(v[0].abs() < 1e-5);
        assert!((v[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_trace() {
        let m = mat2(3.0, 1.0, 2.0, 5.0);
        assert!((mat2_trace(&m) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale() {
        let m = mat2_identity();
        let s = mat2_scale(&m, 3.0);
        assert!((s.m[0][0] - 3.0).abs() < 1e-5);
        assert!((s.m[1][1] - 3.0).abs() < 1e-5);
    }
}
