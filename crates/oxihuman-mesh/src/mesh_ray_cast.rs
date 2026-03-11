// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

const EPSILON: f32 = 1e-7;

/// A ray defined by origin and direction.
#[allow(dead_code)]
#[derive(Clone)]
pub struct RayCast {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

/// Result of a ray-triangle intersection.
#[allow(dead_code)]
pub struct RayCastHit {
    pub t: f32,
    pub face: usize,
    pub bary: [f32; 3],
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Möller–Trumbore ray-triangle intersection.
#[allow(dead_code)]
pub fn ray_triangle_intersect_rc(
    ray: &RayCast,
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let h = cross3(ray.direction, e2);
    let a = dot3(e1, h);
    if a.abs() < EPSILON {
        return None;
    }
    let f = 1.0 / a;
    let s = sub3(ray.origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = f * dot3(ray.direction, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(e2, q);
    if t > EPSILON {
        Some(t)
    } else {
        None
    }
}

/// Cast a ray against a triangle mesh and return the nearest hit.
#[allow(dead_code)]
pub fn ray_cast_mesh(ray: &RayCast, positions: &[[f32; 3]], indices: &[u32]) -> Option<RayCastHit> {
    let mut best: Option<RayCastHit> = None;
    let n = indices.len() / 3;
    for fi in 0..n {
        let a = positions[indices[fi * 3] as usize];
        let b = positions[indices[fi * 3 + 1] as usize];
        let c = positions[indices[fi * 3 + 2] as usize];
        if let Some(t) = ray_triangle_intersect_rc(ray, a, b, c) {
            if best.as_ref().is_none_or(|bh: &RayCastHit| t < bh.t) {
                best = Some(RayCastHit {
                    t,
                    face: fi,
                    bary: [0.0, 0.0, 0.0],
                });
            }
        }
    }
    best
}

/// Normalize direction for a ray.
#[allow(dead_code)]
pub fn ray_cast_normalize(ray: &mut RayCast) {
    let d = &ray.direction;
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len > EPSILON {
        ray.direction = [d[0] / len, d[1] / len, d[2] / len];
    }
}

/// Evaluate a point along the ray at parameter t.
#[allow(dead_code)]
pub fn ray_at(ray: &RayCast, t: f32) -> [f32; 3] {
    [
        ray.origin[0] + ray.direction[0] * t,
        ray.origin[1] + ray.direction[1] * t,
        ray.origin[2] + ray.direction[2] * t,
    ]
}

/// Count how many triangles a ray hits.
#[allow(dead_code)]
pub fn ray_hit_count(ray: &RayCast, positions: &[[f32; 3]], indices: &[u32]) -> usize {
    let n = indices.len() / 3;
    (0..n)
        .filter(|&fi| {
            let a = positions[indices[fi * 3] as usize];
            let b = positions[indices[fi * 3 + 1] as usize];
            let c = positions[indices[fi * 3 + 2] as usize];
            ray_triangle_intersect_rc(ray, a, b, c).is_some()
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    #[test]
    fn hits_triangle_center() {
        let ray = RayCast {
            origin: [0.25, 0.25, 1.0],
            direction: [0.0, 0.0, -1.0],
        };
        let (pos, idx) = simple_triangle();
        let hit = ray_cast_mesh(&ray, &pos, &idx);
        assert!(hit.is_some());
    }

    #[test]
    fn misses_outside() {
        let ray = RayCast {
            origin: [2.0, 2.0, 1.0],
            direction: [0.0, 0.0, -1.0],
        };
        let (pos, idx) = simple_triangle();
        assert!(ray_cast_mesh(&ray, &pos, &idx).is_none());
    }

    #[test]
    fn ray_at_correct() {
        let ray = RayCast {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let p = ray_at(&ray, 3.0);
        assert!((p[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_unit_length() {
        let mut ray = RayCast {
            origin: [0.0; 3],
            direction: [3.0, 4.0, 0.0],
        };
        ray_cast_normalize(&mut ray);
        let len =
            (ray.direction[0].powi(2) + ray.direction[1].powi(2) + ray.direction[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn hit_count_one() {
        let ray = RayCast {
            origin: [0.25, 0.25, 1.0],
            direction: [0.0, 0.0, -1.0],
        };
        let (pos, idx) = simple_triangle();
        assert_eq!(ray_hit_count(&ray, &pos, &idx), 1);
    }

    #[test]
    fn hit_count_zero() {
        let ray = RayCast {
            origin: [5.0, 5.0, 1.0],
            direction: [0.0, 0.0, -1.0],
        };
        let (pos, idx) = simple_triangle();
        assert_eq!(ray_hit_count(&ray, &pos, &idx), 0);
    }

    #[test]
    fn triangle_intersect_direct() {
        let ray = RayCast {
            origin: [0.1, 0.1, 2.0],
            direction: [0.0, 0.0, -1.0],
        };
        let t = ray_triangle_intersect_rc(&ray, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(t.is_some_and(|v| v > 0.0));
    }

    #[test]
    fn behind_ray_no_hit() {
        let ray = RayCast {
            origin: [0.1, 0.1, -2.0],
            direction: [0.0, 0.0, -1.0],
        };
        let t = ray_triangle_intersect_rc(&ray, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(t.is_none());
    }

    #[test]
    fn empty_mesh_no_hit() {
        let ray = RayCast {
            origin: [0.0, 0.0, 1.0],
            direction: [0.0, 0.0, -1.0],
        };
        let hit = ray_cast_mesh(&ray, &[], &[]);
        assert!(hit.is_none());
    }

    #[test]
    fn closest_face_returned() {
        let ray = RayCast {
            origin: [0.1, 0.1, 5.0],
            direction: [0.0, 0.0, -1.0],
        };
        let pos = vec![
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 3.0],
            [1.0, 0.0, 3.0],
            [0.0, 1.0, 3.0],
        ];
        let idx = vec![0, 1, 2, 3, 4, 5];
        let hit = ray_cast_mesh(&ray, &pos, &idx).unwrap();
        assert_eq!(hit.face, 1); // z=3 is closer from z=5 going -z
    }
}
