// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Quaternion utility functions operating on plain `[f32; 4]` arrays (xyzw).

#![allow(dead_code)]

/// Returns the identity quaternion `[x=0, y=0, z=0, w=1]`.
#[allow(dead_code)]
pub fn quat_identity() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}

/// Multiplies two quaternions `a * b`.
#[allow(dead_code)]
pub fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// Normalizes a quaternion to unit length.
#[allow(dead_code)]
pub fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-9 {
        return quat_identity();
    }
    let inv = 1.0 / len;
    [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
}

/// Returns the conjugate (inverse for unit quaternions) of `q`.
#[allow(dead_code)]
pub fn quat_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Rotates a 3D vector `v` by quaternion `q`.
#[allow(dead_code)]
pub fn quat_rotate_vec(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let vq = [v[0], v[1], v[2], 0.0];
    let qc = quat_conjugate(q);
    let tmp = quat_mul(q, vq);
    let res = quat_mul(tmp, qc);
    [res[0], res[1], res[2]]
}

/// Constructs a quaternion from an axis and angle (radians).
/// The axis must be non-zero; a zero axis returns identity.
#[allow(dead_code)]
pub fn quat_from_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-9 {
        return quat_identity();
    }
    let inv = 1.0 / len;
    let half = angle * 0.5;
    let s = half.sin();
    [axis[0] * inv * s, axis[1] * inv * s, axis[2] * inv * s, half.cos()]
}

/// Dot product of two quaternions.
#[allow(dead_code)]
pub fn quat_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// Spherical linear interpolation between two quaternions.
#[allow(dead_code)]
pub fn quat_slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = quat_dot(a, b);
    let b_adj = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };
    let (sa, sb) = if dot > 0.9995 {
        (1.0 - t, t)
    } else {
        let theta = dot.acos();
        let sin_theta = theta.sin();
        (
            ((1.0 - t) * theta).sin() / sin_theta,
            (t * theta).sin() / sin_theta,
        )
    };
    quat_normalize([
        a[0] * sa + b_adj[0] * sb,
        a[1] * sa + b_adj[1] * sb,
        a[2] * sa + b_adj[2] * sb,
        a[3] * sa + b_adj[3] * sb,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPS: f32 = 1e-4;

    #[test]
    fn test_identity() {
        let q = quat_identity();
        assert_eq!(q, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_normalize_identity() {
        let q = quat_normalize([0.0, 0.0, 0.0, 2.0]);
        assert!((q[3] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_conjugate() {
        let q = [1.0f32, 2.0, 3.0, 4.0];
        let c = quat_conjugate(q);
        assert_eq!(c, [-1.0, -2.0, -3.0, 4.0]);
    }

    #[test]
    fn test_mul_identity() {
        let id = quat_identity();
        let q = quat_from_axis_angle([0.0, 1.0, 0.0], PI / 4.0);
        let r = quat_mul(id, q);
        for i in 0..4 {
            assert!((r[i] - q[i]).abs() < EPS);
        }
    }

    #[test]
    fn test_from_axis_angle_zero() {
        let q = quat_from_axis_angle([0.0, 0.0, 0.0], 1.0);
        assert!((q[3] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_from_axis_angle_90_y() {
        let q = quat_from_axis_angle([0.0, 1.0, 0.0], PI / 2.0);
        let expected_w = (PI / 4.0).cos();
        assert!((q[3] - expected_w).abs() < EPS);
    }

    #[test]
    fn test_dot_identity_self() {
        let id = quat_identity();
        assert!((quat_dot(id, id) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_slerp_t0() {
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], PI / 2.0);
        let r = quat_slerp(a, b, 0.0);
        for i in 0..4 {
            assert!((r[i] - a[i]).abs() < EPS);
        }
    }

    #[test]
    fn test_slerp_t1() {
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], PI / 2.0);
        let r = quat_slerp(a, b, 1.0);
        for i in 0..4 {
            assert!((r[i] - b[i]).abs() < EPS);
        }
    }

    #[test]
    fn test_rotate_vec_identity() {
        let q = quat_identity();
        let v = [1.0f32, 2.0, 3.0];
        let r = quat_rotate_vec(q, v);
        for i in 0..3 {
            assert!((r[i] - v[i]).abs() < EPS);
        }
    }
}
