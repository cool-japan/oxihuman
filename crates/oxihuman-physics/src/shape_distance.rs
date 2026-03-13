#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Distance queries between geometric shapes.

/// Wrapper for shape-distance results.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeDistance {
    pub distance: f32,
    pub closest_a: [f32; 3],
    pub closest_b: [f32; 3],
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
pub fn sphere_sphere_distance(
    c1: [f32; 3], r1: f32,
    c2: [f32; 3], r2: f32,
) -> f32 {
    let d = vec3_len(vec3_sub(c1, c2));
    (d - r1 - r2).max(0.0)
}

#[allow(dead_code)]
pub fn sphere_box_distance(
    sc: [f32; 3], sr: f32,
    bc: [f32; 3], bh: [f32; 3],
) -> f32 {
    // Closest point on box to sphere center.
    let cx = (sc[0] - bc[0]).clamp(-bh[0], bh[0]) + bc[0];
    let cy = (sc[1] - bc[1]).clamp(-bh[1], bh[1]) + bc[1];
    let cz = (sc[2] - bc[2]).clamp(-bh[2], bh[2]) + bc[2];
    let d = vec3_len(vec3_sub(sc, [cx, cy, cz]));
    (d - sr).max(0.0)
}

#[allow(dead_code)]
pub fn box_box_distance(
    c1: [f32; 3], h1: [f32; 3],
    c2: [f32; 3], h2: [f32; 3],
) -> f32 {
    let mut dist_sq = 0.0f32;
    for i in 0..3 {
        let lo1 = c1[i] - h1[i];
        let hi1 = c1[i] + h1[i];
        let lo2 = c2[i] - h2[i];
        let hi2 = c2[i] + h2[i];
        if hi1 < lo2 {
            dist_sq += (lo2 - hi1) * (lo2 - hi1);
        } else if hi2 < lo1 {
            dist_sq += (lo1 - hi2) * (lo1 - hi2);
        }
    }
    dist_sq.sqrt()
}

#[allow(dead_code)]
pub fn capsule_capsule_distance(
    a1: [f32; 3], b1: [f32; 3], r1: f32,
    a2: [f32; 3], b2: [f32; 3], r2: f32,
) -> f32 {
    // Simplified: distance between midpoints minus radii and half-lengths.
    let m1 = [(a1[0]+b1[0])*0.5, (a1[1]+b1[1])*0.5, (a1[2]+b1[2])*0.5];
    let m2 = [(a2[0]+b2[0])*0.5, (a2[1]+b2[1])*0.5, (a2[2]+b2[2])*0.5];
    let d = vec3_len(vec3_sub(m1, m2));
    (d - r1 - r2).max(0.0)
}

#[allow(dead_code)]
pub fn point_shape_distance(point: [f32; 3], center: [f32; 3], radius: f32) -> f32 {
    (vec3_len(vec3_sub(point, center)) - radius).max(0.0)
}

#[allow(dead_code)]
pub fn closest_points_on_shapes(
    c1: [f32; 3], r1: f32,
    c2: [f32; 3], r2: f32,
) -> ([f32; 3], [f32; 3]) {
    let diff = vec3_sub(c1, c2);
    let d = vec3_len(diff);
    if d < 1e-10 {
        return (c1, c2);
    }
    let n = [diff[0] / d, diff[1] / d, diff[2] / d];
    let pa = [c1[0] - n[0] * r1, c1[1] - n[1] * r1, c1[2] - n[2] * r1];
    let pb = [c2[0] + n[0] * r2, c2[1] + n[1] * r2, c2[2] + n[2] * r2];
    (pa, pb)
}

#[allow(dead_code)]
pub fn distance_is_zero(d: f32) -> bool {
    d.abs() < 1e-6
}

#[allow(dead_code)]
pub fn signed_distance(center: [f32; 3], radius: f32, point: [f32; 3]) -> f32 {
    vec3_len(vec3_sub(point, center)) - radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_sphere_no_overlap() {
        let d = sphere_sphere_distance([0.0, 0.0, 0.0], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!((d - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_sphere_sphere_overlap() {
        let d = sphere_sphere_distance([0.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], 2.0);
        assert_eq!(d, 0.0);
    }

    #[test]
    fn test_sphere_box() {
        let d = sphere_box_distance([3.0, 0.0, 0.0], 0.5, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((d - 1.5).abs() < 1e-4);
    }

    #[test]
    fn test_box_box_no_overlap() {
        let d = box_box_distance([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [5.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((d - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_box_box_overlap() {
        let d = box_box_distance([0.0, 0.0, 0.0], [2.0, 2.0, 2.0], [1.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        assert_eq!(d, 0.0);
    }

    #[test]
    fn test_capsule_capsule() {
        let d = capsule_capsule_distance(
            [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5,
            [5.0, 0.0, 0.0], [5.0, 1.0, 0.0], 0.5,
        );
        assert!(d > 0.0);
    }

    #[test]
    fn test_point_shape() {
        let d = point_shape_distance([3.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!((d - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_closest_points() {
        let (a, b) = closest_points_on_shapes([0.0, 0.0, 0.0], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!(a[0] > 0.0);
        assert!(b[0] < 5.0);
    }

    #[test]
    fn test_distance_is_zero() {
        assert!(distance_is_zero(0.0));
        assert!(!distance_is_zero(1.0));
    }

    #[test]
    fn test_signed_distance() {
        let d = signed_distance([0.0, 0.0, 0.0], 1.0, [0.5, 0.0, 0.0]);
        assert!(d < 0.0); // inside
        let d2 = signed_distance([0.0, 0.0, 0.0], 1.0, [2.0, 0.0, 0.0]);
        assert!(d2 > 0.0); // outside
    }
}
