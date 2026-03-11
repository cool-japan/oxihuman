// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact normal computation helpers.

#![allow(dead_code)]

/// Compute the contact normal between two spheres (from center_a toward center_b).
/// Returns a unit vector, or `[0,1,0]` if centers are coincident.
#[allow(dead_code)]
pub fn normal_sphere_sphere(center_a: [f32; 3], center_b: [f32; 3]) -> [f32; 3] {
    let dx = center_b[0] - center_a[0];
    let dy = center_b[1] - center_a[1];
    let dz = center_b[2] - center_a[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-9 {
        return [0.0, 1.0, 0.0];
    }
    [dx / len, dy / len, dz / len]
}

/// Compute the contact normal from an AABB surface to an interior/exterior point.
/// Returns the axis normal pointing outward from the closest face.
#[allow(dead_code)]
pub fn normal_box_point(box_min: [f32; 3], box_max: [f32; 3], point: [f32; 3]) -> [f32; 3] {
    // Find closest face by smallest penetration on each axis
    let mut min_dist = f32::INFINITY;
    let mut normal = [0.0f32; 3];

    let faces: [(f32, [f32; 3]); 6] = [
        (point[0] - box_min[0], [-1.0, 0.0, 0.0]),
        (box_max[0] - point[0], [1.0, 0.0, 0.0]),
        (point[1] - box_min[1], [0.0, -1.0, 0.0]),
        (box_max[1] - point[1], [0.0, 1.0, 0.0]),
        (point[2] - box_min[2], [0.0, 0.0, -1.0]),
        (box_max[2] - point[2], [0.0, 0.0, 1.0]),
    ];

    for (dist, n) in &faces {
        let d = dist.abs();
        if d < min_dist {
            min_dist = d;
            normal = *n;
        }
    }
    normal
}

/// Flip a normal vector (negate all components).
#[allow(dead_code)]
pub fn flip_normal(n: [f32; 3]) -> [f32; 3] {
    [-n[0], -n[1], -n[2]]
}

/// Compute the dot product of a normal and a vector.
#[allow(dead_code)]
pub fn normal_dot(n: [f32; 3], v: [f32; 3]) -> f32 {
    n[0] * v[0] + n[1] * v[1] + n[2] * v[2]
}

/// Return true if the normal has unit length (within tolerance).
#[allow(dead_code)]
pub fn is_valid_normal(n: [f32; 3]) -> bool {
    let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    (len_sq - 1.0).abs() < 1e-4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_sphere_sphere_unit_length() {
        let n = normal_sphere_sphere([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(is_valid_normal(n));
    }

    #[test]
    fn test_normal_sphere_sphere_direction() {
        let n = normal_sphere_sphere([0.0; 3], [3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
        assert!(n[1].abs() < 1e-6);
        assert!(n[2].abs() < 1e-6);
    }

    #[test]
    fn test_normal_sphere_coincident_fallback() {
        let n = normal_sphere_sphere([1.0; 3], [1.0; 3]);
        assert!((n[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normal_box_point_left_face() {
        // Point near left face of unit box
        let n = normal_box_point([0.0; 3], [1.0; 3], [0.01, 0.5, 0.5]);
        assert_eq!(n, [-1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_flip_normal() {
        let n = [1.0, 0.0, 0.0];
        let f = flip_normal(n);
        assert!((f[0] - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn test_normal_dot_perpendicular() {
        let d = normal_dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(d.abs() < 1e-9);
    }

    #[test]
    fn test_normal_dot_parallel() {
        let d = normal_dot([1.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_valid_normal_unit() {
        assert!(is_valid_normal([1.0, 0.0, 0.0]));
        assert!(is_valid_normal([0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_is_valid_normal_not_unit() {
        assert!(!is_valid_normal([2.0, 0.0, 0.0]));
        assert!(!is_valid_normal([0.0; 3]));
    }
}
