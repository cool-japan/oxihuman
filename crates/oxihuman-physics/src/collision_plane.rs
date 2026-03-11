// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Infinite plane collision body and response.

#![allow(dead_code)]

/// An infinite plane defined by normal and distance from origin: n·x = d.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct InfinitePlane {
    /// Unit normal pointing away from the solid side.
    pub normal: [f32; 3],
    /// Signed distance: plane is at n·x = d.
    pub d: f32,
    /// Restitution coefficient.
    pub restitution: f32,
    /// Friction coefficient.
    pub friction: f32,
}

/// Create a new infinite plane.
#[allow(dead_code)]
pub fn new_infinite_plane(
    normal: [f32; 3],
    d: f32,
    restitution: f32,
    friction: f32,
) -> InfinitePlane {
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    let n = if len > f32::EPSILON {
        [normal[0] / len, normal[1] / len, normal[2] / len]
    } else {
        [0.0, 1.0, 0.0]
    };
    InfinitePlane {
        normal: n,
        d,
        restitution,
        friction,
    }
}

/// Signed distance from a point to the plane (positive = above plane).
#[allow(dead_code)]
pub fn plane_signed_dist(plane: &InfinitePlane, point: [f32; 3]) -> f32 {
    dot3(plane.normal, point) - plane.d
}

/// Check if a point is above the plane (on the normal side).
#[allow(dead_code)]
pub fn plane_point_above(plane: &InfinitePlane, point: [f32; 3]) -> bool {
    plane_signed_dist(plane, point) >= 0.0
}

/// Project a point onto the plane.
#[allow(dead_code)]
pub fn plane_project_point(plane: &InfinitePlane, point: [f32; 3]) -> [f32; 3] {
    let dist = plane_signed_dist(plane, point);
    [
        point[0] - dist * plane.normal[0],
        point[1] - dist * plane.normal[1],
        point[2] - dist * plane.normal[2],
    ]
}

/// Sphere-plane contact: returns penetration depth and contact normal (or None if no contact).
#[allow(dead_code)]
pub fn sphere_plane_contact(plane: &InfinitePlane, center: [f32; 3], radius: f32) -> Option<f32> {
    let dist = plane_signed_dist(plane, center);
    let penetration = radius - dist;
    if penetration > 0.0 {
        Some(penetration)
    } else {
        None
    }
}

/// Resolve sphere-plane collision: push sphere out and reflect velocity.
#[allow(dead_code)]
pub fn resolve_sphere_plane(
    plane: &InfinitePlane,
    center: &mut [f32; 3],
    velocity: &mut [f32; 3],
    radius: f32,
) {
    let dist = plane_signed_dist(plane, *center);
    let penetration = radius - dist;
    if penetration <= 0.0 {
        return;
    }
    // Push out
    center[0] += plane.normal[0] * penetration;
    center[1] += plane.normal[1] * penetration;
    center[2] += plane.normal[2] * penetration;

    // Reflect velocity
    let vn = dot3(*velocity, plane.normal);
    if vn < 0.0 {
        velocity[0] -= (1.0 + plane.restitution) * vn * plane.normal[0];
        velocity[1] -= (1.0 + plane.restitution) * vn * plane.normal[1];
        velocity[2] -= (1.0 + plane.restitution) * vn * plane.normal[2];

        // Apply friction to tangential component
        let vt = [
            velocity[0] - vn * plane.normal[0],
            velocity[1] - vn * plane.normal[1],
            velocity[2] - vn * plane.normal[2],
        ];
        velocity[0] -= vt[0] * plane.friction;
        velocity[1] -= vt[1] * plane.friction;
        velocity[2] -= vt[2] * plane.friction;
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Create a ground plane (y=0, normal up).
#[allow(dead_code)]
pub fn ground_plane(restitution: f32, friction: f32) -> InfinitePlane {
    new_infinite_plane([0.0, 1.0, 0.0], 0.0, restitution, friction)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground() -> InfinitePlane {
        ground_plane(0.5, 0.3)
    }

    #[test]
    fn signed_dist_above() {
        let p = ground();
        let dist = plane_signed_dist(&p, [0.0, 5.0, 0.0]);
        assert!((dist - 5.0).abs() < 1e-5);
    }

    #[test]
    fn signed_dist_below() {
        let p = ground();
        let dist = plane_signed_dist(&p, [0.0, -1.0, 0.0]);
        assert!(dist < 0.0);
    }

    #[test]
    fn point_above_true() {
        let p = ground();
        assert!(plane_point_above(&p, [0.0, 1.0, 0.0]));
    }

    #[test]
    fn point_below_false() {
        let p = ground();
        assert!(!plane_point_above(&p, [0.0, -0.5, 0.0]));
    }

    #[test]
    fn sphere_contact_penetrating() {
        let p = ground();
        let contact = sphere_plane_contact(&p, [0.0, 0.5, 0.0], 1.0);
        assert!(contact.is_some());
        let depth = contact.unwrap();
        assert!((depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn sphere_contact_no_penetration() {
        let p = ground();
        let contact = sphere_plane_contact(&p, [0.0, 5.0, 0.0], 1.0);
        assert!(contact.is_none());
    }

    #[test]
    fn resolve_pushes_sphere_out() {
        let p = ground();
        let mut center = [0.0, 0.4f32, 0.0];
        let mut vel = [0.0, -1.0f32, 0.0];
        resolve_sphere_plane(&p, &mut center, &mut vel, 1.0);
        assert!(center[1] >= 1.0 - 1e-4);
    }

    #[test]
    fn resolve_reflects_velocity() {
        let p = ground();
        let mut center = [0.0, 0.9f32, 0.0];
        let mut vel = [0.0, -2.0f32, 0.0];
        resolve_sphere_plane(&p, &mut center, &mut vel, 1.0);
        assert!(vel[1] > 0.0); // reflected upward
    }

    #[test]
    fn project_point_on_plane() {
        let p = ground();
        let projected = plane_project_point(&p, [3.0, 5.0, 2.0]);
        assert!((projected[1]).abs() < 1e-4);
        assert!((projected[0] - 3.0).abs() < 1e-4);
    }

    #[test]
    fn normal_normalized() {
        let plane = new_infinite_plane([3.0, 4.0, 0.0], 0.0, 0.5, 0.2);
        let len = dot3(plane.normal, plane.normal).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }
}
