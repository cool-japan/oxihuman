// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Quaternion arithmetic: multiply, conjugate, slerp, normalize.

#![allow(dead_code)]

/// A quaternion [x, y, z, w] where w is the scalar part.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Create a quaternion.
#[allow(dead_code)]
pub fn quat(x: f32, y: f32, z: f32, w: f32) -> Quat {
    Quat { x, y, z, w }
}

/// Identity quaternion (no rotation).
#[allow(dead_code)]
pub fn quat_identity() -> Quat {
    quat(0.0, 0.0, 0.0, 1.0)
}

/// Norm (magnitude) of a quaternion.
#[allow(dead_code)]
pub fn quat_norm(q: &Quat) -> f32 {
    (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt()
}

/// Normalize a quaternion to unit length.
#[allow(dead_code)]
pub fn quat_normalize(q: &Quat) -> Quat {
    let n = quat_norm(q);
    if n < 1e-10 {
        return quat_identity();
    }
    quat(q.x / n, q.y / n, q.z / n, q.w / n)
}

/// Conjugate of a quaternion (negate vector part).
#[allow(dead_code)]
pub fn quat_conjugate(q: &Quat) -> Quat {
    quat(-q.x, -q.y, -q.z, q.w)
}

/// Inverse of a unit quaternion (same as conjugate for unit quats).
#[allow(dead_code)]
pub fn quat_inverse(q: &Quat) -> Quat {
    let n2 = q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w;
    if n2 < 1e-10 {
        return quat_identity();
    }
    quat(-q.x / n2, -q.y / n2, -q.z / n2, q.w / n2)
}

/// Multiply two quaternions: result = a * b.
#[allow(dead_code)]
pub fn quat_mul(a: &Quat, b: &Quat) -> Quat {
    quat(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    )
}

/// Dot product of two quaternions.
#[allow(dead_code)]
pub fn quat_dot(a: &Quat, b: &Quat) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}

/// Spherical linear interpolation between two unit quaternions.
#[allow(dead_code)]
pub fn quat_slerp(a: &Quat, b: &Quat, t: f32) -> Quat {
    let t = t.clamp(0.0, 1.0);
    let mut dot = quat_dot(a, b).clamp(-1.0, 1.0);

    // Ensure shortest path.
    let mut b2 = *b;
    if dot < 0.0 {
        b2 = quat(-b.x, -b.y, -b.z, -b.w);
        dot = -dot;
    }

    if dot > 0.9995 {
        // Linear interpolation for nearly parallel quaternions.
        return quat_normalize(&quat(
            a.x + t * (b2.x - a.x),
            a.y + t * (b2.y - a.y),
            a.z + t * (b2.z - a.z),
            a.w + t * (b2.w - a.w),
        ));
    }

    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();
    let s0 = (theta_0 - theta).sin() / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;

    quat(
        s0 * a.x + s1 * b2.x,
        s0 * a.y + s1 * b2.y,
        s0 * a.z + s1 * b2.z,
        s0 * a.w + s1 * b2.w,
    )
}

/// Create a rotation quaternion from axis-angle (axis must be unit).
#[allow(dead_code)]
pub fn quat_from_axis_angle(axis: [f32; 3], angle: f32) -> Quat {
    let half = angle * 0.5;
    let s = half.sin();
    quat(axis[0] * s, axis[1] * s, axis[2] * s, half.cos())
}

/// Rotate a 3D vector by a unit quaternion.
#[allow(dead_code)]
pub fn quat_rotate_vec(q: &Quat, v: [f32; 3]) -> [f32; 3] {
    let vq = quat(v[0], v[1], v[2], 0.0);
    let qc = quat_conjugate(q);
    let r = quat_mul(&quat_mul(q, &vq), &qc);
    [r.x, r.y, r.z]
}

/// Check if two quaternions are approximately equal within `eps`.
#[allow(dead_code)]
pub fn quat_approx_eq(a: &Quat, b: &Quat, eps: f32) -> bool {
    (a.x - b.x).abs() < eps
        && (a.y - b.y).abs() < eps
        && (a.z - b.z).abs() < eps
        && (a.w - b.w).abs() < eps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity_norm() {
        let id = quat_identity();
        assert!((quat_norm(&id) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize() {
        let q = quat(1.0, 1.0, 1.0, 1.0);
        let n = quat_normalize(&q);
        assert!((quat_norm(&n) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_conjugate() {
        let q = quat(1.0, 2.0, 3.0, 4.0);
        let c = quat_conjugate(&q);
        assert_eq!(c.x, -1.0);
        assert_eq!(c.y, -2.0);
        assert_eq!(c.z, -3.0);
        assert_eq!(c.w, 4.0);
    }

    #[test]
    fn test_mul_identity() {
        let id = quat_identity();
        let q = quat_normalize(&quat(1.0, 0.0, 0.0, 1.0));
        let result = quat_mul(&q, &id);
        assert!(quat_approx_eq(&result, &q, 1e-5));
    }

    #[test]
    fn test_rotate_x_axis_90deg() {
        let axis = [0.0f32, 0.0, 1.0];
        let q = quat_from_axis_angle(axis, PI / 2.0);
        let v = quat_rotate_vec(&q, [1.0, 0.0, 0.0]);
        assert!(v[0].abs() < 1e-5);
        assert!((v[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_slerp_t0() {
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], PI / 2.0);
        let r = quat_slerp(&a, &b, 0.0);
        assert!(quat_approx_eq(&r, &a, 1e-5));
    }

    #[test]
    fn test_slerp_t1() {
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], PI / 2.0);
        let r = quat_slerp(&a, &b, 1.0);
        assert!(quat_approx_eq(&r, &b, 1e-5));
    }

    #[test]
    fn test_inverse_times_self_is_identity() {
        let q = quat_normalize(&quat(1.0, 2.0, 3.0, 4.0));
        let inv = quat_inverse(&q);
        let result = quat_mul(&q, &inv);
        assert!(quat_approx_eq(&result, &quat_identity(), 1e-5));
    }

    #[test]
    fn test_dot_product() {
        let id = quat_identity();
        let d = quat_dot(&id, &id);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_axis_angle_roundtrip_norm() {
        let q = quat_from_axis_angle([1.0, 0.0, 0.0], PI / 3.0);
        assert!((quat_norm(&q) - 1.0).abs() < 1e-5);
    }
}
