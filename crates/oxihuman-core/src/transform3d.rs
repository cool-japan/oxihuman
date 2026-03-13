#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 3D transform (position + rotation quaternion + scale).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Transform3d {
    pub position: [f32; 3],
    /// Quaternion [x, y, z, w]
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
pub fn transform_identity() -> Transform3d {
    Transform3d {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn transform_translate(t: &Transform3d, delta: [f32; 3]) -> Transform3d {
    Transform3d {
        position: [
            t.position[0] + delta[0],
            t.position[1] + delta[1],
            t.position[2] + delta[2],
        ],
        rotation: t.rotation,
        scale: t.scale,
    }
}

/// Multiply two quaternions q1 * q2.
fn quat_mul_local(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

#[allow(dead_code)]
pub fn transform_rotate(t: &Transform3d, q: [f32; 4]) -> Transform3d {
    Transform3d {
        position: t.position,
        rotation: quat_mul_local(t.rotation, q),
        scale: t.scale,
    }
}

#[allow(dead_code)]
pub fn transform_scale_uniform(t: &Transform3d, s: f32) -> Transform3d {
    Transform3d {
        position: t.position,
        rotation: t.rotation,
        scale: [t.scale[0] * s, t.scale[1] * s, t.scale[2] * s],
    }
}

#[allow(dead_code)]
pub fn transform_to_mat4(t: &Transform3d) -> [[f32; 4]; 4] {
    let q = t.rotation;
    let qx = q[0];
    let qy = q[1];
    let qz = q[2];
    let qw = q[3];
    let sx = t.scale[0];
    let sy = t.scale[1];
    let sz = t.scale[2];
    let tx = t.position[0];
    let ty = t.position[1];
    let tz = t.position[2];

    [
        [
            (1.0 - 2.0 * (qy * qy + qz * qz)) * sx,
            (2.0 * (qx * qy - qw * qz)) * sy,
            (2.0 * (qx * qz + qw * qy)) * sz,
            tx,
        ],
        [
            (2.0 * (qx * qy + qw * qz)) * sx,
            (1.0 - 2.0 * (qx * qx + qz * qz)) * sy,
            (2.0 * (qy * qz - qw * qx)) * sz,
            ty,
        ],
        [
            (2.0 * (qx * qz - qw * qy)) * sx,
            (2.0 * (qy * qz + qw * qx)) * sy,
            (1.0 - 2.0 * (qx * qx + qy * qy)) * sz,
            tz,
        ],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[allow(dead_code)]
pub fn transform_lerp(a: &Transform3d, b: &Transform3d, fac: f32) -> Transform3d {
    let fac = fac.clamp(0.0, 1.0);
    Transform3d {
        position: [
            lerp_f32(a.position[0], b.position[0], fac),
            lerp_f32(a.position[1], b.position[1], fac),
            lerp_f32(a.position[2], b.position[2], fac),
        ],
        rotation: [
            lerp_f32(a.rotation[0], b.rotation[0], fac),
            lerp_f32(a.rotation[1], b.rotation[1], fac),
            lerp_f32(a.rotation[2], b.rotation[2], fac),
            lerp_f32(a.rotation[3], b.rotation[3], fac),
        ],
        scale: [
            lerp_f32(a.scale[0], b.scale[0], fac),
            lerp_f32(a.scale[1], b.scale[1], fac),
            lerp_f32(a.scale[2], b.scale[2], fac),
        ],
    }
}

/// Apply the transform to a 3D point (position only, no rotation/scale applied here).
#[allow(dead_code)]
pub fn transform_apply(t: &Transform3d, p: [f32; 3]) -> [f32; 3] {
    [
        t.position[0] + p[0] * t.scale[0],
        t.position[1] + p[1] * t.scale[1],
        t.position[2] + p[2] * t.scale[2],
    ]
}

/// Combine two transforms: returns a transform that first applies `a` then `b`.
#[allow(dead_code)]
pub fn transform_combine(a: &Transform3d, b: &Transform3d) -> Transform3d {
    Transform3d {
        position: [
            a.position[0] + b.position[0],
            a.position[1] + b.position[1],
            a.position[2] + b.position[2],
        ],
        rotation: quat_mul_local(a.rotation, b.rotation),
        scale: [
            a.scale[0] * b.scale[0],
            a.scale[1] * b.scale[1],
            a.scale[2] * b.scale[2],
        ],
    }
}

/// Returns the translation inverse (negated position, identity rotation, identity scale).
#[allow(dead_code)]
pub fn transform_inverse_translation(t: &Transform3d) -> Transform3d {
    Transform3d {
        position: [-t.position[0], -t.position[1], -t.position[2]],
        rotation: t.rotation,
        scale: t.scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_position() {
        let t = transform_identity();
        assert_eq!(t.position, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_identity_scale() {
        let t = transform_identity();
        assert_eq!(t.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_translate() {
        let t = transform_identity();
        let t2 = transform_translate(&t, [1.0, 2.0, 3.0]);
        assert!((t2.position[0] - 1.0).abs() < 1e-6);
        assert!((t2.position[1] - 2.0).abs() < 1e-6);
        assert!((t2.position[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_uniform() {
        let t = transform_identity();
        let t2 = transform_scale_uniform(&t, 2.0);
        assert!((t2.scale[0] - 2.0).abs() < 1e-6);
        assert!((t2.scale[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_mat4_identity() {
        let t = transform_identity();
        let m = transform_to_mat4(&t);
        assert!((m[0][0] - 1.0).abs() < 1e-6);
        assert!((m[1][1] - 1.0).abs() < 1e-6);
        assert!((m[2][2] - 1.0).abs() < 1e-6);
        assert!((m[3][3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_mat4_translation() {
        let t = transform_translate(&transform_identity(), [5.0, 6.0, 7.0]);
        let m = transform_to_mat4(&t);
        assert!((m[0][3] - 5.0).abs() < 1e-6);
        assert!((m[1][3] - 6.0).abs() < 1e-6);
        assert!((m[2][3] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_half() {
        let a = transform_identity();
        let b = transform_translate(&transform_identity(), [2.0, 0.0, 0.0]);
        let c = transform_lerp(&a, &b, 0.5);
        assert!((c.position[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_zero() {
        let a = transform_identity();
        let b = transform_translate(&transform_identity(), [2.0, 0.0, 0.0]);
        let c = transform_lerp(&a, &b, 0.0);
        assert!(c.position[0].abs() < 1e-6);
    }

    #[test]
    fn test_lerp_one() {
        let a = transform_identity();
        let b = transform_translate(&transform_identity(), [2.0, 0.0, 0.0]);
        let c = transform_lerp(&a, &b, 1.0);
        assert!((c.position[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_rotate_identity_unchanged() {
        let t = transform_identity();
        let q_id = [0.0f32, 0.0, 0.0, 1.0];
        let t2 = transform_rotate(&t, q_id);
        assert!((t2.rotation[3] - 1.0).abs() < 1e-6);
    }
}
