// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Quaternion math utilities. Quaternion layout: [x, y, z, w].

use std::f32::consts::PI;

/// A quaternion [x, y, z, w].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuatMath {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Identity quaternion [0, 0, 0, 1].
pub fn quat_identity() -> QuatMath {
    QuatMath {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    }
}

/// Multiply two quaternions.
pub fn quat_mul(a: QuatMath, b: QuatMath) -> QuatMath {
    QuatMath {
        x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    }
}

/// Conjugate of a quaternion.
pub fn quat_conjugate(q: QuatMath) -> QuatMath {
    QuatMath {
        x: -q.x,
        y: -q.y,
        z: -q.z,
        w: q.w,
    }
}

/// Squared norm of a quaternion.
pub fn quat_norm_sq(q: QuatMath) -> f32 {
    q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w
}

/// Normalize a quaternion to unit length.
pub fn quat_normalize(q: QuatMath) -> QuatMath {
    let len = quat_norm_sq(q).sqrt();
    if len < 1e-12 {
        return quat_identity();
    }
    QuatMath {
        x: q.x / len,
        y: q.y / len,
        z: q.z / len,
        w: q.w / len,
    }
}

/// Build a rotation quaternion from axis (normalized) and angle in radians.
pub fn quat_from_axis_angle(axis: [f32; 3], angle_rad: f32) -> QuatMath {
    let half = angle_rad * 0.5;
    let s = half.sin();
    QuatMath {
        x: axis[0] * s,
        y: axis[1] * s,
        z: axis[2] * s,
        w: half.cos(),
    }
}

/// Rotate a 3D vector by a quaternion (q must be normalized).
pub fn quat_rotate_vec3(q: QuatMath, v: [f32; 3]) -> [f32; 3] {
    let vq = QuatMath {
        x: v[0],
        y: v[1],
        z: v[2],
        w: 0.0,
    };
    let res = quat_mul(quat_mul(q, vq), quat_conjugate(q));
    [res.x, res.y, res.z]
}

/// Spherical linear interpolation between two quaternions.
pub fn quat_slerp(a: QuatMath, b: QuatMath, t: f32) -> QuatMath {
    let mut dot = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
    let b = if dot < 0.0 {
        dot = -dot;
        QuatMath {
            x: -b.x,
            y: -b.y,
            z: -b.z,
            w: -b.w,
        }
    } else {
        b
    };
    if dot > 0.9995 {
        return quat_normalize(QuatMath {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
            z: a.z + t * (b.z - a.z),
            w: a.w + t * (b.w - a.w),
        });
    }
    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();
    let s0 = (theta_0 - theta).sin() / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;
    QuatMath {
        x: s0 * a.x + s1 * b.x,
        y: s0 * a.y + s1 * b.y,
        z: s0 * a.z + s1 * b.z,
        w: s0 * a.w + s1 * b.w,
    }
}

/// Convert quaternion to Euler angles [roll, pitch, yaw] in radians.
pub fn quat_to_euler(q: QuatMath) -> [f32; 3] {
    let sinr_cosp = 2.0 * (q.w * q.x + q.y * q.z);
    let cosr_cosp = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
    let roll = sinr_cosp.atan2(cosr_cosp);
    let sinp = 2.0 * (q.w * q.y - q.z * q.x);
    let pitch = if sinp.abs() >= 1.0 {
        sinp.signum() * PI / 2.0
    } else {
        sinp.asin()
    };
    let siny_cosp = 2.0 * (q.w * q.z + q.x * q.y);
    let cosy_cosp = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
    let yaw = siny_cosp.atan2(cosy_cosp);
    [roll, pitch, yaw]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_identity_mul() {
        /* identity * identity = identity */
        let id = quat_identity();
        let r = quat_mul(id, id);
        assert!((r.w - 1.0).abs() < EPS);
        assert!(r.x.abs() < EPS);
    }

    #[test]
    fn test_normalize_identity() {
        /* identity is already normalized */
        let n = quat_normalize(quat_identity());
        assert!((n.w - 1.0).abs() < EPS);
    }

    #[test]
    fn test_conjugate_reverses_rotation() {
        /* q * conj(q) = identity */
        let q = quat_from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2);
        let r = quat_mul(q, quat_conjugate(q));
        assert!((r.w - 1.0).abs() < EPS);
        assert!(r.x.abs() < EPS);
        assert!(r.y.abs() < EPS);
        assert!(r.z.abs() < EPS);
    }

    #[test]
    fn test_from_axis_angle_z90() {
        /* 90 degrees around Z rotates [1,0,0] to ~[0,1,0] */
        let q = quat_from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2);
        let v = quat_rotate_vec3(q, [1.0, 0.0, 0.0]);
        assert!(v[0].abs() < 1e-4);
        assert!((v[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_slerp_endpoints() {
        /* slerp at 0 = a, at 1 = b */
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_2);
        let r0 = quat_slerp(a, b, 0.0);
        assert!((r0.w - a.w).abs() < EPS);
        let r1 = quat_slerp(a, b, 1.0);
        assert!((r1.w - b.w).abs() < EPS);
    }

    #[test]
    fn test_slerp_midpoint_norm() {
        /* slerp midpoint should be normalized */
        let a = quat_identity();
        let b = quat_from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
        let m = quat_slerp(a, b, 0.5);
        let ns = quat_norm_sq(m).sqrt();
        assert!((ns - 1.0).abs() < EPS);
    }

    #[test]
    fn test_to_euler_zero() {
        /* identity quaternion -> zero euler */
        let e = quat_to_euler(quat_identity());
        assert!(e[0].abs() < EPS);
        assert!(e[1].abs() < EPS);
        assert!(e[2].abs() < EPS);
    }

    #[test]
    fn test_norm_sq_identity() {
        /* identity norm^2 = 1 */
        let ns = quat_norm_sq(quat_identity());
        assert!((ns - 1.0).abs() < EPS);
    }

    #[test]
    fn test_rotate_identity_no_change() {
        /* rotating by identity quaternion leaves vector unchanged */
        let q = quat_identity();
        let v = quat_rotate_vec3(q, [3.0, 4.0, 5.0]);
        assert!((v[0] - 3.0).abs() < EPS);
        assert!((v[1] - 4.0).abs() < EPS);
        assert!((v[2] - 5.0).abs() < EPS);
    }
}
