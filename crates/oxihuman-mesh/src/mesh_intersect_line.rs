// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh-line intersection utilities.

/// Möller–Trumbore ray–triangle intersection.
/// Returns the distance `t` along the ray if intersection occurs (t > 0).
pub fn ray_triangle_intersect(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let h = cross3(dir, e2);
    let a = dot3(e1, h);
    if a.abs() < 1e-8 {
        return None;
    }
    let f = 1.0 / a;
    let s = sub3(origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = f * dot3(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(e2, q);
    if t > 1e-8 {
        Some(t)
    } else {
        None
    }
}

/// Line segment–triangle intersection (segment from `a` to `b`).
pub fn line_segment_triangle_intersect(
    a: [f32; 3],
    b: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> bool {
    let dir = sub3(b, a);
    let len = (dot3(dir, dir)).sqrt();
    if len < f32::EPSILON {
        return false;
    }
    let unit = [dir[0] / len, dir[1] / len, dir[2] / len];
    match ray_triangle_intersect(a, unit, v0, v1, v2) {
        Some(t) => t <= len,
        None => false,
    }
}

/// Count how many triangles (index triples) a ray intersects.
pub fn ray_mesh_intersect_count(
    origin: [f32; 3],
    dir: [f32; 3],
    positions: &[[f32; 3]],
    indices: &[u32],
) -> usize {
    indices
        .chunks(3)
        .filter(|tri| {
            if tri.len() < 3 {
                return false;
            }
            let v0 = positions[tri[0] as usize];
            let v1 = positions[tri[1] as usize];
            let v2 = positions[tri[2] as usize];
            ray_triangle_intersect(origin, dir, v0, v1, v2).is_some()
        })
        .count()
}

/// Point along a ray at parameter `t`.
pub fn ray_point_at(origin: [f32; 3], dir: [f32; 3], t: f32) -> [f32; 3] {
    [
        origin[0] + dir[0] * t,
        origin[1] + dir[1] * t,
        origin[2] + dir[2] * t,
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn test_ray_hits_triangle() {
        let (v0, v1, v2) = unit_triangle();
        let t = ray_triangle_intersect([0.25, 0.25, 1.0], [0.0, 0.0, -1.0], v0, v1, v2);
        assert!(t.is_some());
    }

    #[test]
    fn test_ray_misses_triangle() {
        let (v0, v1, v2) = unit_triangle();
        let t = ray_triangle_intersect([5.0, 5.0, 1.0], [0.0, 0.0, -1.0], v0, v1, v2);
        assert!(t.is_none());
    }

    #[test]
    fn test_segment_intersects() {
        let (v0, v1, v2) = unit_triangle();
        let hit =
            line_segment_triangle_intersect([0.25, 0.25, 1.0], [0.25, 0.25, -1.0], v0, v1, v2);
        assert!(hit);
    }

    #[test]
    fn test_segment_no_intersect_too_short() {
        /* segment stops before the triangle */
        let (v0, v1, v2) = unit_triangle();
        let hit = line_segment_triangle_intersect([0.25, 0.25, 2.0], [0.25, 0.25, 1.1], v0, v1, v2);
        assert!(!hit);
    }

    #[test]
    fn test_ray_point_at() {
        let p = ray_point_at([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
        assert!((p[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_mesh_intersect_count() {
        let (v0, v1, v2) = unit_triangle();
        let pos = vec![v0, v1, v2];
        let idx = vec![0u32, 1, 2];
        let count = ray_mesh_intersect_count([0.25, 0.25, 1.0], [0.0, 0.0, -1.0], &pos, &idx);
        assert_eq!(count, 1);
    }
}
