// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2x2 matrix math utilities.

/// A 2x2 matrix stored in row-major order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2 {
    pub m: [[f32; 2]; 2],
}

/// Returns the 2x2 identity matrix.
pub fn mat2_identity() -> Mat2 {
    Mat2 {
        m: [[1.0, 0.0], [0.0, 1.0]],
    }
}

/// Multiplies two 2x2 matrices: `a * b`.
pub fn mat2_mul(a: &Mat2, b: &Mat2) -> Mat2 {
    Mat2 {
        m: [
            [
                a.m[0][0] * b.m[0][0] + a.m[0][1] * b.m[1][0],
                a.m[0][0] * b.m[0][1] + a.m[0][1] * b.m[1][1],
            ],
            [
                a.m[1][0] * b.m[0][0] + a.m[1][1] * b.m[1][0],
                a.m[1][0] * b.m[0][1] + a.m[1][1] * b.m[1][1],
            ],
        ],
    }
}

/// Computes the determinant of a 2x2 matrix.
pub fn mat2_det(m: &Mat2) -> f32 {
    m.m[0][0] * m.m[1][1] - m.m[0][1] * m.m[1][0]
}

/// Returns the inverse of a 2x2 matrix, or `None` if singular.
pub fn mat2_inverse(m: &Mat2) -> Option<Mat2> {
    let det = mat2_det(m);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some(Mat2 {
        m: [
            [m.m[1][1] * inv_det, -m.m[0][1] * inv_det],
            [-m.m[1][0] * inv_det, m.m[0][0] * inv_det],
        ],
    })
}

/// Alias for mat2_inverse (legacy name).
pub fn mat2_inv(m: &Mat2) -> Option<Mat2> {
    mat2_inverse(m)
}

/// Transforms a 2D vector by a 2x2 matrix.
pub fn mat2_transform(m: &Mat2, v: [f32; 2]) -> [f32; 2] {
    [
        m.m[0][0] * v[0] + m.m[0][1] * v[1],
        m.m[1][0] * v[0] + m.m[1][1] * v[1],
    ]
}

/// Alias using spec name.
pub fn mat2_mul_vec2(a: &Mat2, v: [f32; 2]) -> [f32; 2] {
    mat2_transform(a, v)
}

/// Returns the transpose of a 2x2 matrix.
pub fn mat2_transpose(m: &Mat2) -> Mat2 {
    Mat2 {
        m: [[m.m[0][0], m.m[1][0]], [m.m[0][1], m.m[1][1]]],
    }
}

/// Returns a 2x2 rotation matrix for the given angle in radians.
pub fn mat2_from_angle(angle_rad: f32) -> Mat2 {
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    Mat2 {
        m: [[c, -s], [s, c]],
    }
}

/// Returns a 2x2 scale matrix.
pub fn mat2_scale(sx: f32, sy: f32) -> Mat2 {
    Mat2 {
        m: [[sx, 0.0], [0.0, sy]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_identity() {
        /* identity matrix has 1s on diagonal */
        let id = mat2_identity();
        assert_eq!(id.m, [[1.0, 0.0], [0.0, 1.0]]);
    }

    #[test]
    fn test_mul_identity() {
        /* identity * A = A */
        let id = mat2_identity();
        let a = Mat2 {
            m: [[2.0, 3.0], [4.0, 5.0]],
        };
        let r = mat2_mul(&id, &a);
        assert!((r.m[0][0] - 2.0).abs() < EPS);
        assert!((r.m[1][1] - 5.0).abs() < EPS);
    }

    #[test]
    fn test_mul() {
        /* matrix multiplication */
        let a = Mat2 {
            m: [[1.0, 2.0], [3.0, 4.0]],
        };
        let b = Mat2 {
            m: [[5.0, 6.0], [7.0, 8.0]],
        };
        let r = mat2_mul(&a, &b);
        assert!((r.m[0][0] - 19.0).abs() < EPS);
        assert!((r.m[0][1] - 22.0).abs() < EPS);
        assert!((r.m[1][0] - 43.0).abs() < EPS);
        assert!((r.m[1][1] - 50.0).abs() < EPS);
    }

    #[test]
    fn test_det_identity() {
        /* det of identity is 1 */
        let id = mat2_identity();
        assert!((mat2_det(&id) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_det_zero() {
        /* singular matrix has zero determinant */
        let m = Mat2 {
            m: [[1.0, 2.0], [2.0, 4.0]],
        };
        assert!(mat2_det(&m).abs() < EPS);
    }

    #[test]
    fn test_inv_identity() {
        /* inverse of identity is identity */
        let id = mat2_identity();
        let inv = mat2_inverse(&id).expect("should succeed");
        assert!((inv.m[0][0] - 1.0).abs() < EPS);
        assert!((inv.m[1][1] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_inv_singular() {
        /* singular matrix returns None */
        let m = Mat2 {
            m: [[1.0, 2.0], [2.0, 4.0]],
        };
        assert!(mat2_inverse(&m).is_none());
    }

    #[test]
    fn test_from_angle_90() {
        /* 90° rotation maps [1,0] to [0,1] */
        let r = mat2_from_angle(FRAC_PI_2);
        let v = mat2_mul_vec2(&r, [1.0, 0.0]);
        assert!((v[0] - 0.0).abs() < 1e-5);
        assert!((v[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale() {
        /* scale matrix */
        let s = mat2_scale(2.0, 3.0);
        let v = mat2_mul_vec2(&s, [1.0, 1.0]);
        assert!((v[0] - 2.0).abs() < EPS);
        assert!((v[1] - 3.0).abs() < EPS);
    }
}
