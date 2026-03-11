#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rigid body transform: position + uniform scale.

/// A rigid-body transform with position and uniform scale.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct BodyTransform {
    pub position: [f32; 3],
    pub scale: f32,
}

/// Create a new `BodyTransform`.
#[allow(dead_code)]
pub fn new_body_transform(position: [f32; 3], scale: f32) -> BodyTransform {
    BodyTransform { position, scale }
}

/// Transform a point by a `BodyTransform` (scale then translate).
#[allow(dead_code)]
pub fn transform_point(t: &BodyTransform, p: [f32; 3]) -> [f32; 3] {
    [
        p[0] * t.scale + t.position[0],
        p[1] * t.scale + t.position[1],
        p[2] * t.scale + t.position[2],
    ]
}

/// Transform a direction vector (scale only, no translation).
#[allow(dead_code)]
pub fn transform_vector(t: &BodyTransform, v: [f32; 3]) -> [f32; 3] {
    [v[0] * t.scale, v[1] * t.scale, v[2] * t.scale]
}

/// Inverse-transform a point back to local space.
#[allow(dead_code)]
pub fn inverse_transform(t: &BodyTransform, p: [f32; 3]) -> [f32; 3] {
    let inv_scale = if t.scale.abs() > 1e-9 { 1.0 / t.scale } else { 1.0 };
    [
        (p[0] - t.position[0]) * inv_scale,
        (p[1] - t.position[1]) * inv_scale,
        (p[2] - t.position[2]) * inv_scale,
    ]
}

/// Compose two transforms: apply `inner` first, then `outer`.
#[allow(dead_code)]
pub fn compose_transforms(outer: &BodyTransform, inner: &BodyTransform) -> BodyTransform {
    BodyTransform {
        position: [
            outer.position[0] + inner.position[0] * outer.scale,
            outer.position[1] + inner.position[1] * outer.scale,
            outer.position[2] + inner.position[2] * outer.scale,
        ],
        scale: outer.scale * inner.scale,
    }
}

/// Return an identity `BodyTransform`.
#[allow(dead_code)]
pub fn transform_identity() -> BodyTransform {
    BodyTransform { position: [0.0; 3], scale: 1.0 }
}

/// Return a `BodyTransform` with given scale and zero position.
#[allow(dead_code)]
pub fn transform_scale(scale: f32) -> BodyTransform {
    BodyTransform { position: [0.0; 3], scale }
}

/// Convert to a flat 4×4 column-major matrix (scale + translation, no rotation).
#[allow(dead_code)]
pub fn transform_to_matrix4(t: &BodyTransform) -> [f32; 16] {
    let s = t.scale;
    let [tx, ty, tz] = t.position;
    [
        s,   0.0, 0.0, 0.0,
        0.0, s,   0.0, 0.0,
        0.0, 0.0, s,   0.0,
        tx,  ty,  tz,  1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_transform() {
        let t = new_body_transform([1.0, 2.0, 3.0], 2.0);
        assert_eq!(t.scale, 2.0);
    }

    #[test]
    fn test_transform_point() {
        let t = new_body_transform([1.0, 0.0, 0.0], 2.0);
        let p = transform_point(&t, [1.0, 0.0, 0.0]);
        assert!((p[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_vector() {
        let t = new_body_transform([0.0; 3], 3.0);
        let v = transform_vector(&t, [1.0, 1.0, 1.0]);
        assert!((v[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_inverse_transform() {
        let t = new_body_transform([1.0, 2.0, 3.0], 2.0);
        let p = [3.0, 4.0, 5.0];
        let local = inverse_transform(&t, p);
        assert!((local[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compose_identity() {
        let id = transform_identity();
        let t = new_body_transform([1.0, 2.0, 3.0], 2.0);
        let composed = compose_transforms(&id, &t);
        assert!((composed.position[0] - t.position[0]).abs() < 1e-6);
    }

    #[test]
    fn test_transform_identity() {
        let id = transform_identity();
        let p = [3.0, 4.0, 5.0];
        let out = transform_point(&id, p);
        assert!((out[0] - p[0]).abs() < 1e-6);
    }

    #[test]
    fn test_transform_scale() {
        let t = transform_scale(0.5);
        let v = transform_vector(&t, [2.0, 0.0, 0.0]);
        assert!((v[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_matrix4_identity() {
        let t = transform_identity();
        let m = transform_to_matrix4(&t);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[15] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip() {
        let t = new_body_transform([5.0, -3.0, 2.0], 4.0);
        let p = [1.0, 1.0, 1.0];
        let world = transform_point(&t, p);
        let back = inverse_transform(&t, world);
        assert!((back[0] - p[0]).abs() < 1e-5);
        assert!((back[1] - p[1]).abs() < 1e-5);
        assert!((back[2] - p[2]).abs() < 1e-5);
    }
}
