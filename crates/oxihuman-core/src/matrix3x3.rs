#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 3x3 matrix math.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Mat3 {
    pub m: [[f32; 3]; 3],
}

#[allow(dead_code)]
pub fn mat3_identity() -> Mat3 {
    Mat3 {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    }
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn mat3_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                out[i][j] += a.m[i][k] * b.m[k][j];
            }
        }
    }
    Mat3 { m: out }
}

#[allow(dead_code)]
pub fn mat3_det(m: &Mat3) -> f32 {
    let a = &m.m;
    a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
}

#[allow(dead_code)]
pub fn mat3_transpose(m: &Mat3) -> Mat3 {
    let a = &m.m;
    Mat3 {
        m: [
            [a[0][0], a[1][0], a[2][0]],
            [a[0][1], a[1][1], a[2][1]],
            [a[0][2], a[1][2], a[2][2]],
        ],
    }
}

#[allow(dead_code)]
pub fn mat3_transform(m: &Mat3, v: [f32; 3]) -> [f32; 3] {
    let a = &m.m;
    [
        a[0][0] * v[0] + a[0][1] * v[1] + a[0][2] * v[2],
        a[1][0] * v[0] + a[1][1] * v[1] + a[1][2] * v[2],
        a[2][0] * v[0] + a[2][1] * v[1] + a[2][2] * v[2],
    ]
}

#[allow(dead_code)]
pub fn mat3_from_scale(sx: f32, sy: f32, sz: f32) -> Mat3 {
    Mat3 {
        m: [[sx, 0.0, 0.0], [0.0, sy, 0.0], [0.0, 0.0, sz]],
    }
}

#[allow(dead_code)]
pub fn mat3_from_rotation_z(angle: f32) -> Mat3 {
    let c = angle.cos();
    let s = angle.sin();
    Mat3 {
        m: [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
    }
}

/// Alias for mat3_transform using spec naming convention.
#[allow(dead_code)]
pub fn mat3_mul_vec3(a: &Mat3, v: [f32; 3]) -> [f32; 3] {
    mat3_transform(a, v)
}

/// Alias for mat3_from_scale using spec naming convention.
#[allow(dead_code)]
pub fn mat3_scale(s: [f32; 3]) -> Mat3 {
    mat3_from_scale(s[0], s[1], s[2])
}

/// Returns a rotation matrix around an arbitrary axis using Rodrigues' formula.
#[allow(dead_code)]
pub fn mat3_from_axis_angle(axis: [f32; 3], angle_rad: f32) -> Mat3 {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-9 {
        return mat3_identity();
    }
    let ux = axis[0] / len;
    let uy = axis[1] / len;
    let uz = axis[2] / len;
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let t = 1.0 - c;
    Mat3 {
        m: [
            [t * ux * ux + c, t * ux * uy - s * uz, t * ux * uz + s * uy],
            [t * ux * uy + s * uz, t * uy * uy + c, t * uy * uz - s * ux],
            [t * ux * uz - s * uy, t * uy * uz + s * ux, t * uz * uz + c],
        ],
    }
}

/// Returns the inverse of a 3x3 matrix, or `None` if singular.
#[allow(dead_code)]
pub fn mat3_inverse(a: &Mat3) -> Option<Mat3> {
    let det = mat3_det(a);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let m = &a.m;
    Some(Mat3 {
        m: [
            [
                (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv,
                (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv,
                (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv,
            ],
            [
                (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv,
                (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv,
                (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv,
            ],
            [
                (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv,
                (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv,
                (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv,
            ],
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity_det() {
        let id = mat3_identity();
        assert!((mat3_det(&id) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_identity_transform() {
        let id = mat3_identity();
        let v = [1.0f32, 2.0, 3.0];
        let r = mat3_transform(&id, v);
        assert_eq!(r, v);
    }

    #[test]
    fn test_transpose_identity() {
        let id = mat3_identity();
        let t = mat3_transpose(&id);
        assert_eq!(t, id);
    }

    #[test]
    fn test_mul_identity() {
        let id = mat3_identity();
        let result = mat3_mul(&id, &id);
        assert_eq!(result, id);
    }

    #[test]
    fn test_scale_matrix() {
        let s = mat3_from_scale(2.0, 3.0, 4.0);
        let v = [1.0f32, 1.0, 1.0];
        let r = mat3_transform(&s, v);
        assert!((r[0] - 2.0).abs() < 1e-6);
        assert!((r[1] - 3.0).abs() < 1e-6);
        assert!((r[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_rotation_z_zero() {
        let r = mat3_from_rotation_z(0.0);
        let v = [1.0f32, 0.0, 0.0];
        let rv = mat3_transform(&r, v);
        assert!((rv[0] - 1.0).abs() < 1e-6);
        assert!(rv[1].abs() < 1e-6);
    }

    #[test]
    fn test_rotation_z_pi() {
        let r = mat3_from_rotation_z(PI);
        let v = [1.0f32, 0.0, 0.0];
        let rv = mat3_transform(&r, v);
        assert!((rv[0] + 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_det_scale() {
        let s = mat3_from_scale(2.0, 3.0, 4.0);
        assert!((mat3_det(&s) - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_transpose_swap() {
        let s = mat3_from_scale(1.0, 2.0, 3.0);
        let t = mat3_transpose(&s);
        assert!((t.m[0][0] - 1.0).abs() < 1e-6);
        assert!((t.m[1][1] - 2.0).abs() < 1e-6);
        assert!((t.m[2][2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_mul_scale() {
        /* mat3_mul of two scale-2 matrices yields scale-4 */
        let s2 = mat3_from_scale(2.0, 2.0, 2.0);
        let result = mat3_mul(&s2, &s2);
        let v = [1.0f32, 1.0, 1.0];
        let rv = mat3_transform(&result, v);
        assert!((rv[0] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_mul_vec3_alias() {
        /* mat3_mul_vec3 is alias for mat3_transform */
        let id = mat3_identity();
        let v = [3.0f32, 4.0, 5.0];
        let r = mat3_mul_vec3(&id, v);
        assert_eq!(r, v);
    }

    #[test]
    fn test_scale_alias() {
        /* mat3_scale is alias for mat3_from_scale */
        let s = mat3_scale([2.0, 3.0, 4.0]);
        let v = [1.0f32, 1.0, 1.0];
        let r = mat3_transform(&s, v);
        assert!((r[0] - 2.0).abs() < 1e-6);
        assert!((r[1] - 3.0).abs() < 1e-6);
        assert!((r[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_axis_angle_identity_angle() {
        /* zero angle rotation returns identity */
        let r = mat3_from_axis_angle([0.0, 0.0, 1.0], 0.0);
        let id = mat3_identity();
        for i in 0..3 {
            for j in 0..3 {
                assert!((r.m[i][j] - id.m[i][j]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_from_axis_angle_z_90() {
        /* 90° around Z maps [1,0,0] to [0,1,0] */
        use std::f32::consts::FRAC_PI_2;
        let r = mat3_from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2);
        let v = mat3_mul_vec3(&r, [1.0, 0.0, 0.0]);
        assert!(v[0].abs() < 1e-5);
        assert!((v[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_inverse_identity() {
        /* inverse of identity is identity */
        let id = mat3_identity();
        let inv = mat3_inverse(&id).expect("should succeed");
        for i in 0..3 {
            for j in 0..3 {
                assert!((inv.m[i][j] - id.m[i][j]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_inverse_singular() {
        /* singular matrix returns None */
        let m = Mat3 {
            m: [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
        };
        assert!(mat3_inverse(&m).is_none());
    }

    #[test]
    fn test_inverse_roundtrip() {
        /* A * A^-1 = I for non-singular matrix */
        let s = mat3_from_scale(2.0, 3.0, 4.0);
        let inv = mat3_inverse(&s).expect("should succeed");
        let prod = mat3_mul(&s, &inv);
        let id = mat3_identity();
        for i in 0..3 {
            for j in 0..3 {
                assert!((prod.m[i][j] - id.m[i][j]).abs() < 1e-5);
            }
        }
    }
}
