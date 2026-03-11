// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Penetration depth computation for various shape pairs.

#![allow(dead_code)]

/// Computes sphere-sphere penetration depth. Positive = penetrating.
#[allow(dead_code)]
pub fn sphere_sphere_depth(ca: [f32; 3], ra: f32, cb: [f32; 3], rb: f32) -> f32 {
    let dx = cb[0] - ca[0];
    let dy = cb[1] - ca[1];
    let dz = cb[2] - ca[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    ra + rb - dist
}

/// Computes sphere-plane penetration depth. Plane: dot(normal, x) = d.
/// Positive = sphere center is on the positive side, penetrating into plane.
#[allow(dead_code)]
pub fn sphere_plane_depth(center: [f32; 3], radius: f32, normal: [f32; 3], d: f32) -> f32 {
    let signed_dist =
        center[0] * normal[0] + center[1] * normal[1] + center[2] * normal[2] - d;
    radius - signed_dist
}

/// Computes AABB-AABB penetration depth along the minimum overlap axis.
/// Returns 0 if not overlapping.
#[allow(dead_code)]
pub fn aabb_aabb_depth(min_a: [f32; 3], max_a: [f32; 3], min_b: [f32; 3], max_b: [f32; 3]) -> f32 {
    let mut min_overlap = f32::INFINITY;
    for i in 0..3 {
        let overlap = (max_a[i].min(max_b[i])) - (min_a[i].max(min_b[i]));
        if overlap <= 0.0 {
            return 0.0;
        }
        if overlap < min_overlap {
            min_overlap = overlap;
        }
    }
    min_overlap
}

/// Computes the penetration of a point into a sphere (positive = inside).
#[allow(dead_code)]
pub fn point_sphere_depth(pt: [f32; 3], center: [f32; 3], radius: f32) -> f32 {
    let dx = pt[0] - center[0];
    let dy = pt[1] - center[1];
    let dz = pt[2] - center[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    radius - dist
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    #[test]
    fn test_sphere_sphere_touching() {
        // Two spheres of radius 1.0 exactly touching: depth = 0
        let d = sphere_sphere_depth([0.0, 0.0, 0.0], 1.0, [2.0, 0.0, 0.0], 1.0);
        assert!(d.abs() < EPS);
    }

    #[test]
    fn test_sphere_sphere_overlapping() {
        let d = sphere_sphere_depth([0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], 1.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_sphere_sphere_separated() {
        let d = sphere_sphere_depth([0.0, 0.0, 0.0], 0.5, [3.0, 0.0, 0.0], 0.5);
        assert!(d < 0.0);
    }

    #[test]
    fn test_sphere_plane_above() {
        // Sphere at y=2, radius=1, plane y=0 (normal=[0,1,0], d=0)
        let d = sphere_plane_depth([0.0, 2.0, 0.0], 1.0, [0.0, 1.0, 0.0], 0.0);
        assert!(d < 0.0); // not penetrating
    }

    #[test]
    fn test_sphere_plane_penetrating() {
        // Sphere center at y=0.5, radius=1, plane y=0 → depth = 1 - 0.5 = 0.5
        let d = sphere_plane_depth([0.0, 0.5, 0.0], 1.0, [0.0, 1.0, 0.0], 0.0);
        assert!((d - 0.5).abs() < EPS);
    }

    #[test]
    fn test_aabb_aabb_no_overlap() {
        let d = aabb_aabb_depth([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
        assert!(d.abs() < EPS);
    }

    #[test]
    fn test_aabb_aabb_overlapping() {
        let d = aabb_aabb_depth([0.0, 0.0, 0.0], [2.0, 2.0, 2.0], [1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
        assert!(d > 0.0);
    }

    #[test]
    fn test_point_sphere_inside() {
        let d = point_sphere_depth([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!((d - 1.0).abs() < EPS);
    }

    #[test]
    fn test_point_sphere_outside() {
        let d = point_sphere_depth([2.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!(d < 0.0);
    }
}
