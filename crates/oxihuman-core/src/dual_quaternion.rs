// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dual quaternion for rigid body transforms (rotation + translation).

#![allow(dead_code)]

use crate::quaternion_ops::{
    quat, quat_conjugate, quat_dot, quat_identity, quat_mul, quat_normalize, Quat,
};

/// A dual quaternion: real part (rotation) + dual part (translation encoded).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualQuat {
    /// Real part (unit quaternion for rotation).
    pub real: Quat,
    /// Dual part (encodes translation as 0.5 * t * real).
    pub dual: Quat,
}

/// Create a dual quaternion from real and dual parts.
#[allow(dead_code)]
pub fn dual_quat(real: Quat, dual: Quat) -> DualQuat {
    DualQuat { real, dual }
}

/// Identity dual quaternion (no rotation, no translation).
#[allow(dead_code)]
pub fn dq_identity() -> DualQuat {
    DualQuat {
        real: quat_identity(),
        dual: quat(0.0, 0.0, 0.0, 0.0),
    }
}

/// Create a dual quaternion from a unit rotation quaternion and translation vector.
#[allow(dead_code)]
pub fn dq_from_rot_trans(rot: &Quat, trans: [f32; 3]) -> DualQuat {
    let t = quat(trans[0], trans[1], trans[2], 0.0);
    let dual = quat_mul(&t, rot);
    DualQuat {
        real: *rot,
        dual: quat(dual.x * 0.5, dual.y * 0.5, dual.z * 0.5, dual.w * 0.5),
    }
}

/// Multiply two dual quaternions: result = a * b.
#[allow(dead_code)]
pub fn dq_mul(a: &DualQuat, b: &DualQuat) -> DualQuat {
    let real = quat_mul(&a.real, &b.real);
    let dual_ab = quat_mul(&a.real, &b.dual);
    let dual_ba = quat_mul(&a.dual, &b.real);
    DualQuat {
        real,
        dual: quat(
            dual_ab.x + dual_ba.x,
            dual_ab.y + dual_ba.y,
            dual_ab.z + dual_ba.z,
            dual_ab.w + dual_ba.w,
        ),
    }
}

/// Conjugate of a dual quaternion (for inversion of unit DQ).
#[allow(dead_code)]
pub fn dq_conjugate(dq: &DualQuat) -> DualQuat {
    DualQuat {
        real: quat_conjugate(&dq.real),
        dual: quat_conjugate(&dq.dual),
    }
}

/// Normalize a dual quaternion so the real part is unit.
#[allow(dead_code)]
pub fn dq_normalize(dq: &DualQuat) -> DualQuat {
    let n = (dq.real.x * dq.real.x
        + dq.real.y * dq.real.y
        + dq.real.z * dq.real.z
        + dq.real.w * dq.real.w)
        .sqrt();
    if n < 1e-10 {
        return dq_identity();
    }
    let inv = 1.0 / n;
    DualQuat {
        real: quat(
            dq.real.x * inv,
            dq.real.y * inv,
            dq.real.z * inv,
            dq.real.w * inv,
        ),
        dual: quat(
            dq.dual.x * inv,
            dq.dual.y * inv,
            dq.dual.z * inv,
            dq.dual.w * inv,
        ),
    }
}

/// Extract the translation vector from a unit dual quaternion.
#[allow(dead_code)]
pub fn dq_get_translation(dq: &DualQuat) -> [f32; 3] {
    let rc = quat_conjugate(&dq.real);
    let t = quat_mul(
        &quat(
            dq.dual.x * 2.0,
            dq.dual.y * 2.0,
            dq.dual.z * 2.0,
            dq.dual.w * 2.0,
        ),
        &rc,
    );
    [t.x, t.y, t.z]
}

/// Extract the rotation part from a dual quaternion.
#[allow(dead_code)]
pub fn dq_get_rotation(dq: &DualQuat) -> Quat {
    quat_normalize(&dq.real)
}

/// Transform a 3D point by a unit dual quaternion.
/// Decomposes into rotation + translation then applies each in order.
#[allow(dead_code)]
pub fn dq_transform_point(dq: &DualQuat, p: [f32; 3]) -> [f32; 3] {
    use crate::quaternion_ops::quat_rotate_vec;
    let rot = quat_normalize(&dq.real);
    let rotated = quat_rotate_vec(&rot, p);
    let trans = dq_get_translation(dq);
    [
        rotated[0] + trans[0],
        rotated[1] + trans[1],
        rotated[2] + trans[2],
    ]
}

/// Dot product of dual quaternion real parts.
#[allow(dead_code)]
pub fn dq_dot(a: &DualQuat, b: &DualQuat) -> f32 {
    quat_dot(&a.real, &b.real)
}

/// Check approximate equality.
#[allow(dead_code)]
pub fn dq_approx_eq(a: &DualQuat, b: &DualQuat, eps: f32) -> bool {
    let re = |x: f32, y: f32| (x - y).abs() < eps;
    re(a.real.x, b.real.x)
        && re(a.real.y, b.real.y)
        && re(a.real.z, b.real.z)
        && re(a.real.w, b.real.w)
        && re(a.dual.x, b.dual.x)
        && re(a.dual.y, b.dual.y)
        && re(a.dual.z, b.dual.z)
        && re(a.dual.w, b.dual.w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quaternion_ops::quat_identity;

    #[test]
    fn test_identity() {
        let id = dq_identity();
        assert!((id.real.w - 1.0).abs() < 1e-5);
        assert!(id.dual.w.abs() < 1e-5);
    }

    #[test]
    fn test_from_rot_trans_no_rotation() {
        let rot = quat_identity();
        let dq = dq_from_rot_trans(&rot, [1.0, 2.0, 3.0]);
        let trans = dq_get_translation(&dq);
        assert!((trans[0] - 1.0).abs() < 1e-5);
        assert!((trans[1] - 2.0).abs() < 1e-5);
        assert!((trans[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_mul_identity() {
        let id = dq_identity();
        let dq = dq_from_rot_trans(&quat_identity(), [1.0, 0.0, 0.0]);
        let result = dq_mul(&dq, &id);
        assert!(dq_approx_eq(&result, &dq, 1e-5));
    }

    #[test]
    fn test_normalize() {
        let id = dq_identity();
        let n = dq_normalize(&id);
        assert!((n.real.w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_transform_point_translation_only() {
        let rot = quat_identity();
        let dq = dq_normalize(&dq_from_rot_trans(&rot, [1.0, 0.0, 0.0]));
        let p = dq_transform_point(&dq, [0.0, 0.0, 0.0]);
        assert!((p[0] - 1.0).abs() < 1e-4);
        assert!(p[1].abs() < 1e-4);
        assert!(p[2].abs() < 1e-4);
    }

    #[test]
    fn test_conjugate_real() {
        let dq = dq_from_rot_trans(&quat_identity(), [1.0, 2.0, 3.0]);
        let c = dq_conjugate(&dq);
        assert!((c.real.w - dq.real.w).abs() < 1e-5);
        assert!((c.real.x + dq.real.x).abs() < 1e-5);
    }

    #[test]
    fn test_get_rotation_identity() {
        let id = dq_identity();
        let rot = dq_get_rotation(&id);
        assert!((rot.w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_dot_identity_with_self() {
        let id = dq_identity();
        let d = dq_dot(&id, &id);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_approx_eq() {
        let a = dq_identity();
        let b = dq_identity();
        assert!(dq_approx_eq(&a, &b, 1e-5));
    }

    #[test]
    fn test_from_rot_trans_zero_translation() {
        let rot = quat_identity();
        let dq = dq_from_rot_trans(&rot, [0.0, 0.0, 0.0]);
        let trans = dq_get_translation(&dq);
        assert!(trans[0].abs() < 1e-5);
        assert!(trans[1].abs() < 1e-5);
        assert!(trans[2].abs() < 1e-5);
    }
}
