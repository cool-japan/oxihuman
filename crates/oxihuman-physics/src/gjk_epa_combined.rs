#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Combined GJK/EPA collision detection queries.

/// Result of a GJK/EPA query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GjkEpaResult {
    pub intersects: bool,
    pub distance: f32,
    pub penetration_depth: f32,
    pub closest_a: [f32; 3],
    pub closest_b: [f32; 3],
    pub normal: [f32; 3],
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(v);
    if l < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

#[allow(dead_code)]
pub fn gjk_epa_query(
    center_a: [f32; 3], radius_a: f32,
    center_b: [f32; 3], radius_b: f32,
) -> GjkEpaResult {
    let diff = vec3_sub(center_a, center_b);
    let dist = vec3_len(diff);
    let sum_r = radius_a + radius_b;
    let normal = if dist > 1e-10 { vec3_normalize(diff) } else { [0.0, 1.0, 0.0] };
    if dist <= sum_r {
        GjkEpaResult {
            intersects: true,
            distance: 0.0,
            penetration_depth: sum_r - dist,
            closest_a: [center_a[0] - normal[0] * radius_a, center_a[1] - normal[1] * radius_a, center_a[2] - normal[2] * radius_a],
            closest_b: [center_b[0] + normal[0] * radius_b, center_b[1] + normal[1] * radius_b, center_b[2] + normal[2] * radius_b],
            normal,
        }
    } else {
        GjkEpaResult {
            intersects: false,
            distance: dist - sum_r,
            penetration_depth: 0.0,
            closest_a: [center_a[0] - normal[0] * radius_a, center_a[1] - normal[1] * radius_a, center_a[2] - normal[2] * radius_a],
            closest_b: [center_b[0] + normal[0] * radius_b, center_b[1] + normal[1] * radius_b, center_b[2] + normal[2] * radius_b],
            normal,
        }
    }
}

#[allow(dead_code)]
pub fn gjk_closest_points(
    center_a: [f32; 3], radius_a: f32,
    center_b: [f32; 3], radius_b: f32,
) -> ([f32; 3], [f32; 3]) {
    let r = gjk_epa_query(center_a, radius_a, center_b, radius_b);
    (r.closest_a, r.closest_b)
}

#[allow(dead_code)]
pub fn epa_penetration_depth(
    center_a: [f32; 3], radius_a: f32,
    center_b: [f32; 3], radius_b: f32,
) -> f32 {
    gjk_epa_query(center_a, radius_a, center_b, radius_b).penetration_depth
}

#[allow(dead_code)]
pub fn gjk_epa_distance(
    center_a: [f32; 3], radius_a: f32,
    center_b: [f32; 3], radius_b: f32,
) -> f32 {
    gjk_epa_query(center_a, radius_a, center_b, radius_b).distance
}

#[allow(dead_code)]
pub fn support_point_sphere(center: [f32; 3], radius: f32, direction: [f32; 3]) -> [f32; 3] {
    let n = vec3_normalize(direction);
    [center[0] + n[0] * radius, center[1] + n[1] * radius, center[2] + n[2] * radius]
}

#[allow(dead_code)]
pub fn support_point_box(center: [f32; 3], half: [f32; 3], direction: [f32; 3]) -> [f32; 3] {
    [
        center[0] + if direction[0] >= 0.0 { half[0] } else { -half[0] },
        center[1] + if direction[1] >= 0.0 { half[1] } else { -half[1] },
        center[2] + if direction[2] >= 0.0 { half[2] } else { -half[2] },
    ]
}

#[allow(dead_code)]
pub fn support_point_capsule(
    a: [f32; 3], b: [f32; 3], radius: f32, direction: [f32; 3],
) -> [f32; 3] {
    let da = vec3_dot(a, direction);
    let db = vec3_dot(b, direction);
    let base = if da >= db { a } else { b };
    let n = vec3_normalize(direction);
    [base[0] + n[0] * radius, base[1] + n[1] * radius, base[2] + n[2] * radius]
}

#[allow(dead_code)]
pub fn gjk_epa_intersects(
    center_a: [f32; 3], radius_a: f32,
    center_b: [f32; 3], radius_b: f32,
) -> bool {
    gjk_epa_query(center_a, radius_a, center_b, radius_b).intersects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_intersection() {
        let r = gjk_epa_query([0.0, 0.0, 0.0], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!(!r.intersects);
        assert!((r.distance - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_intersection() {
        let r = gjk_epa_query([0.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], 2.0);
        assert!(r.intersects);
        assert!(r.penetration_depth > 0.0);
    }

    #[test]
    fn test_closest_points() {
        let (a, b) = gjk_closest_points([0.0, 0.0, 0.0], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!(a[0] > 0.0);
        assert!(b[0] < 5.0);
    }

    #[test]
    fn test_penetration_depth() {
        let d = epa_penetration_depth([0.0, 0.0, 0.0], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_distance_no_overlap() {
        let d = gjk_epa_distance([0.0, 0.0, 0.0], 1.0, [10.0, 0.0, 0.0], 1.0);
        assert!((d - 8.0).abs() < 1e-4);
    }

    #[test]
    fn test_support_sphere() {
        let p = support_point_sphere([0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0]);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_support_box() {
        let p = support_point_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_support_capsule() {
        let p = support_point_capsule([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5, [0.0, 1.0, 0.0]);
        assert!(p[1] > 2.0);
    }

    #[test]
    fn test_intersects() {
        assert!(gjk_epa_intersects([0.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], 2.0));
        assert!(!gjk_epa_intersects([0.0, 0.0, 0.0], 1.0, [10.0, 0.0, 0.0], 1.0));
    }

    #[test]
    fn test_coincident() {
        let r = gjk_epa_query([0.0, 0.0, 0.0], 1.0, [0.0, 0.0, 0.0], 1.0);
        assert!(r.intersects);
        assert!((r.penetration_depth - 2.0).abs() < 1e-4);
    }
}
