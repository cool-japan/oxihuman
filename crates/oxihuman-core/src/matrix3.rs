// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 3×3 matrix with determinant, inverse, transpose, and multiply.

#![allow(dead_code)]

/// A 3×3 matrix stored in row-major order.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    /// Row-major: `m[row][col]`.
    pub m: [[f32; 3]; 3],
}

/// Create a Mat3 from 9 row-major values.
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn mat3(
    a00: f32,
    a01: f32,
    a02: f32,
    a10: f32,
    a11: f32,
    a12: f32,
    a20: f32,
    a21: f32,
    a22: f32,
) -> Mat3 {
    Mat3 {
        m: [[a00, a01, a02], [a10, a11, a12], [a20, a21, a22]],
    }
}

/// Identity matrix.
#[allow(dead_code)]
pub fn mat3_identity() -> Mat3 {
    mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
}

/// Zero matrix.
#[allow(dead_code)]
pub fn mat3_zero() -> Mat3 {
    mat3(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
}

/// Transpose a 3×3 matrix.
#[allow(dead_code)]
pub fn mat3_transpose(m: &Mat3) -> Mat3 {
    mat3(
        m.m[0][0], m.m[1][0], m.m[2][0], m.m[0][1], m.m[1][1], m.m[2][1], m.m[0][2], m.m[1][2],
        m.m[2][2],
    )
}

/// Determinant of a 3×3 matrix.
#[allow(dead_code)]
pub fn mat3_det(m: &Mat3) -> f32 {
    m.m[0][0] * (m.m[1][1] * m.m[2][2] - m.m[1][2] * m.m[2][1])
        - m.m[0][1] * (m.m[1][0] * m.m[2][2] - m.m[1][2] * m.m[2][0])
        + m.m[0][2] * (m.m[1][0] * m.m[2][1] - m.m[1][1] * m.m[2][0])
}

/// Inverse of a 3×3 matrix. Returns None if singular.
#[allow(dead_code)]
pub fn mat3_inverse(m: &Mat3) -> Option<Mat3> {
    let det = mat3_det(m);
    if det.abs() < 1e-10 {
        return None;
    }
    let inv = 1.0 / det;
    Some(mat3(
        (m.m[1][1] * m.m[2][2] - m.m[1][2] * m.m[2][1]) * inv,
        (m.m[0][2] * m.m[2][1] - m.m[0][1] * m.m[2][2]) * inv,
        (m.m[0][1] * m.m[1][2] - m.m[0][2] * m.m[1][1]) * inv,
        (m.m[1][2] * m.m[2][0] - m.m[1][0] * m.m[2][2]) * inv,
        (m.m[0][0] * m.m[2][2] - m.m[0][2] * m.m[2][0]) * inv,
        (m.m[0][2] * m.m[1][0] - m.m[0][0] * m.m[1][2]) * inv,
        (m.m[1][0] * m.m[2][1] - m.m[1][1] * m.m[2][0]) * inv,
        (m.m[0][1] * m.m[2][0] - m.m[0][0] * m.m[2][1]) * inv,
        (m.m[0][0] * m.m[1][1] - m.m[0][1] * m.m[1][0]) * inv,
    ))
}

/// Multiply two 3×3 matrices.
#[allow(dead_code)]
pub fn mat3_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut r = mat3_zero();
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                r.m[i][j] += a.m[i][k] * b.m[k][j];
            }
        }
    }
    r
}

/// Add two 3×3 matrices.
#[allow(dead_code)]
pub fn mat3_add(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut r = mat3_zero();
    for i in 0..3 {
        for j in 0..3 {
            r.m[i][j] = a.m[i][j] + b.m[i][j];
        }
    }
    r
}

/// Scale a 3×3 matrix by a scalar.
#[allow(dead_code)]
pub fn mat3_scale(m: &Mat3, s: f32) -> Mat3 {
    let mut r = mat3_zero();
    for i in 0..3 {
        for j in 0..3 {
            r.m[i][j] = m.m[i][j] * s;
        }
    }
    r
}

/// Multiply a 3×3 matrix by a 3D column vector.
#[allow(dead_code)]
pub fn mat3_mul_vec(m: &Mat3, v: [f32; 3]) -> [f32; 3] {
    [
        m.m[0][0] * v[0] + m.m[0][1] * v[1] + m.m[0][2] * v[2],
        m.m[1][0] * v[0] + m.m[1][1] * v[1] + m.m[1][2] * v[2],
        m.m[2][0] * v[0] + m.m[2][1] * v[1] + m.m[2][2] * v[2],
    ]
}

/// Trace of the matrix.
#[allow(dead_code)]
pub fn mat3_trace(m: &Mat3) -> f32 {
    m.m[0][0] + m.m[1][1] + m.m[2][2]
}

/// Check approximate equality.
#[allow(dead_code)]
pub fn mat3_approx_eq(a: &Mat3, b: &Mat3, eps: f32) -> bool {
    for i in 0..3 {
        for j in 0..3 {
            if (a.m[i][j] - b.m[i][j]).abs() >= eps {
                return false;
            }
        }
    }
    true
}

/// Create a 3D rotation matrix around the Z axis.
#[allow(dead_code)]
pub fn mat3_rot_z(theta: f32) -> Mat3 {
    let c = theta.cos();
    let s = theta.sin();
    mat3(c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0)
}

/// Outer product of two 3D vectors: result_ij = a_i * b_j.
#[allow(dead_code)]
pub fn mat3_outer(a: [f32; 3], b: [f32; 3]) -> Mat3 {
    mat3(
        a[0] * b[0],
        a[0] * b[1],
        a[0] * b[2],
        a[1] * b[0],
        a[1] * b[1],
        a[1] * b[2],
        a[2] * b[0],
        a[2] * b[1],
        a[2] * b[2],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity_det() {
        let id = mat3_identity();
        assert!((mat3_det(&id) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_inverse() {
        let m = mat3(1.0, 2.0, 0.0, 0.0, 1.0, 3.0, 0.0, 0.0, 1.0);
        let inv = mat3_inverse(&m).unwrap();
        let prod = mat3_mul(&m, &inv);
        assert!(mat3_approx_eq(&prod, &mat3_identity(), 1e-5));
    }

    #[test]
    fn test_singular_inverse() {
        let m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert!(mat3_inverse(&m).is_none());
    }

    #[test]
    fn test_transpose() {
        let m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let t = mat3_transpose(&m);
        assert!((t.m[0][1] - 4.0).abs() < 1e-5);
        assert!((t.m[1][0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_mul_identity() {
        let m = mat3(1.0, 2.0, 3.0, 0.0, 1.0, 4.0, 5.0, 6.0, 0.0);
        let id = mat3_identity();
        let result = mat3_mul(&m, &id);
        assert!(mat3_approx_eq(&result, &m, 1e-5));
    }

    #[test]
    fn test_mul_vec() {
        let m = mat3_identity();
        let v = mat3_mul_vec(&m, [1.0, 2.0, 3.0]);
        assert!((v[0] - 1.0).abs() < 1e-5);
        assert!((v[1] - 2.0).abs() < 1e-5);
        assert!((v[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotation_z() {
        let r = mat3_rot_z(PI / 2.0);
        let v = mat3_mul_vec(&r, [1.0, 0.0, 0.0]);
        assert!(v[0].abs() < 1e-5);
        assert!((v[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_trace() {
        let m = mat3(1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0);
        assert!((mat3_trace(&m) - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_outer_product() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let o = mat3_outer(a, b);
        assert!((o.m[0][1] - 1.0).abs() < 1e-5);
        assert!(o.m[0][0].abs() < 1e-5);
    }

    #[test]
    fn test_scale() {
        let m = mat3_identity();
        let s = mat3_scale(&m, 2.0);
        assert!((s.m[0][0] - 2.0).abs() < 1e-5);
    }
}
