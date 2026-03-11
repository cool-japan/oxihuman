#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 4x4 matrix math (column-major).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

#[allow(dead_code)]
pub fn mat4_identity() -> Mat4 {
    Mat4 {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn mat4_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                out[i][j] += a.m[i][k] * b.m[k][j];
            }
        }
    }
    Mat4 { m: out }
}

#[allow(dead_code)]
pub fn mat4_translate(tx: f32, ty: f32, tz: f32) -> Mat4 {
    Mat4 {
        m: [
            [1.0, 0.0, 0.0, tx],
            [0.0, 1.0, 0.0, ty],
            [0.0, 0.0, 1.0, tz],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

#[allow(dead_code)]
pub fn mat4_scale(sx: f32, sy: f32, sz: f32) -> Mat4 {
    Mat4 {
        m: [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, sz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

#[allow(dead_code)]
pub fn mat4_perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y * 0.5).tan();
    let nf = 1.0 / (near - far);
    Mat4 {
        m: [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) * nf, 2.0 * far * near * nf],
            [0.0, 0.0, -1.0, 0.0],
        ],
    }
}

#[allow(dead_code)]
pub fn mat4_transform_point(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    let w = m.m[3][0] * p[0] + m.m[3][1] * p[1] + m.m[3][2] * p[2] + m.m[3][3];
    let inv_w = if w.abs() > 1e-9 { 1.0 / w } else { 1.0 };
    [
        (m.m[0][0] * p[0] + m.m[0][1] * p[1] + m.m[0][2] * p[2] + m.m[0][3]) * inv_w,
        (m.m[1][0] * p[0] + m.m[1][1] * p[1] + m.m[1][2] * p[2] + m.m[1][3]) * inv_w,
        (m.m[2][0] * p[0] + m.m[2][1] * p[1] + m.m[2][2] * p[2] + m.m[2][3]) * inv_w,
    ]
}

#[allow(dead_code)]
pub fn mat4_transpose(m: &Mat4) -> Mat4 {
    let a = &m.m;
    Mat4 {
        m: [
            [a[0][0], a[1][0], a[2][0], a[3][0]],
            [a[0][1], a[1][1], a[2][1], a[3][1]],
            [a[0][2], a[1][2], a[2][2], a[3][2]],
            [a[0][3], a[1][3], a[2][3], a[3][3]],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity_transform_point() {
        let id = mat4_identity();
        let p = [1.0f32, 2.0, 3.0];
        let r = mat4_transform_point(&id, p);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 2.0).abs() < 1e-6);
        assert!((r[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_translate() {
        let t = mat4_translate(1.0, 2.0, 3.0);
        let p = [0.0f32, 0.0, 0.0];
        let r = mat4_transform_point(&t, p);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 2.0).abs() < 1e-6);
        assert!((r[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale() {
        let s = mat4_scale(2.0, 3.0, 4.0);
        let p = [1.0f32, 1.0, 1.0];
        let r = mat4_transform_point(&s, p);
        assert!((r[0] - 2.0).abs() < 1e-6);
        assert!((r[1] - 3.0).abs() < 1e-6);
        assert!((r[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_mul_identity() {
        let id = mat4_identity();
        let r = mat4_mul(&id, &id);
        assert_eq!(r, id);
    }

    #[test]
    fn test_transpose_identity() {
        let id = mat4_identity();
        let t = mat4_transpose(&id);
        assert_eq!(t, id);
    }

    #[test]
    fn test_perspective_non_zero() {
        let p = mat4_perspective(PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
        assert!(p.m[0][0] > 0.0);
        assert!(p.m[1][1] > 0.0);
    }

    #[test]
    fn test_mul_scale() {
        let s = mat4_scale(2.0, 2.0, 2.0);
        let r = mat4_mul(&s, &s);
        let p = [1.0f32, 1.0, 1.0];
        let rv = mat4_transform_point(&r, p);
        assert!((rv[0] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_translate_then_scale() {
        let t = mat4_translate(1.0, 0.0, 0.0);
        let s = mat4_scale(2.0, 1.0, 1.0);
        let r = mat4_mul(&s, &t);
        let p = [0.0f32, 0.0, 0.0];
        let rv = mat4_transform_point(&r, p);
        assert!((rv[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_transpose_swap() {
        let t = mat4_translate(1.0, 2.0, 3.0);
        let tr = mat4_transpose(&t);
        assert!((tr.m[0][3] - 0.0).abs() < 1e-6);
        assert!((tr.m[3][0] - 1.0).abs() < 1e-6);
    }
}
