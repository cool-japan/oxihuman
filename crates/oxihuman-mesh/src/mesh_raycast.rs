// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ray–triangle intersection using the Möller–Trumbore algorithm.

#![allow(dead_code)]

/// A ray defined by an origin and a unit direction.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Ray origin.
    pub origin: [f32; 3],
    /// Ray direction (should be normalised for correct `t` values).
    pub direction: [f32; 3],
}

/// The result of a ray–triangle intersection test.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    /// Distance along the ray.
    pub t: f32,
    /// Barycentric u coordinate.
    pub u: f32,
    /// Barycentric v coordinate.
    pub v: f32,
    /// Triangle index.
    pub triangle: usize,
}

#[inline]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Möller–Trumbore ray–triangle intersection.
/// Returns `Some(RayHit)` if the ray hits the triangle (front-face only, t > 0).
#[allow(dead_code)]
pub fn ray_triangle(
    ray: &Ray,
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    triangle: usize,
) -> Option<RayHit> {
    let edge1 = sub(v1, v0);
    let edge2 = sub(v2, v0);
    let h = cross(ray.direction, edge2);
    let det = dot(edge1, h);
    if det.abs() < 1e-8 {
        return None; // parallel
    }
    let inv_det = 1.0 / det;
    let s = sub(ray.origin, v0);
    let u = dot(s, h) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, edge1);
    let v = dot(ray.direction, q) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = dot(edge2, q) * inv_det;
    if t < 1e-8 {
        return None;
    }
    Some(RayHit { t, u, v, triangle })
}

/// Cast `ray` against a triangle soup; return the closest hit.
#[allow(dead_code)]
pub fn raycast(ray: &Ray, positions: &[[f32; 3]], indices: &[u32]) -> Option<RayHit> {
    let tri_count = indices.len() / 3;
    let mut closest: Option<RayHit> = None;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        if let Some(hit) = ray_triangle(ray, positions[ia], positions[ib], positions[ic], t) {
            if closest.is_none_or(|c: RayHit| hit.t < c.t) {
                closest = Some(hit);
            }
        }
    }
    closest
}

/// Return all hits (unsorted) against a triangle soup.
#[allow(dead_code)]
pub fn raycast_all(ray: &Ray, positions: &[[f32; 3]], indices: &[u32]) -> Vec<RayHit> {
    let tri_count = indices.len() / 3;
    let mut hits = Vec::new();
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        if let Some(hit) = ray_triangle(ray, positions[ia], positions[ib], positions[ic], t) {
            hits.push(hit);
        }
    }
    hits
}

/// Compute the hit point in world space.
#[allow(dead_code)]
pub fn hit_point(ray: &Ray, hit: &RayHit) -> [f32; 3] {
    let d = ray.direction;
    let o = ray.origin;
    [
        o[0] + d[0] * hit.t,
        o[1] + d[1] * hit.t,
        o[2] + d[2] * hit.t,
    ]
}

/// Return a normalised direction from `from` to `to`.
#[allow(dead_code)]
pub fn ray_direction(from: [f32; 3], to: [f32; 3]) -> [f32; 3] {
    let d = sub(to, from);
    let len = dot(d, d).sqrt().max(f32::EPSILON);
    [d[0] / len, d[1] / len, d[2] / len]
}

/// Serialise a hit as JSON.
#[allow(dead_code)]
pub fn hit_to_json(hit: &RayHit) -> String {
    format!(
        "{{\"t\":{:.6},\"u\":{:.6},\"v\":{:.6},\"triangle\":{}}}",
        hit.t, hit.u, hit.v, hit.triangle
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_ray() -> Ray {
        Ray {
            origin: [0.0, 0.0, -5.0],
            direction: [0.0, 0.0, 1.0],
        }
    }

    fn front_tri() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn test_hit_front_face() {
        let ray = unit_ray();
        let (v0, v1, v2) = front_tri();
        let hit = ray_triangle(&ray, v0, v1, v2, 0);
        assert!(hit.is_some());
    }

    #[test]
    fn test_miss_parallel() {
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let (v0, v1, v2) = front_tri();
        let hit = ray_triangle(&ray, v0, v1, v2, 0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_t_value() {
        let ray = unit_ray();
        let (v0, v1, v2) = front_tri();
        let hit = ray_triangle(&ray, v0, v1, v2, 0).unwrap();
        assert!((hit.t - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_uv_valid() {
        let ray = unit_ray();
        let (v0, v1, v2) = front_tri();
        let hit = ray_triangle(&ray, v0, v1, v2, 0).unwrap();
        assert!(hit.u >= 0.0 && hit.v >= 0.0 && hit.u + hit.v <= 1.0);
    }

    #[test]
    fn test_raycast_finds_hit() {
        let positions = vec![[-1.0, -1.0, 0.0f32], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let ray = unit_ray();
        let hit = raycast(&ray, &positions, &indices);
        assert!(hit.is_some());
    }

    #[test]
    fn test_raycast_all_count() {
        let positions = vec![[-1.0, -1.0, 0.0f32], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let ray = unit_ray();
        let hits = raycast_all(&ray, &positions, &indices);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_hit_point() {
        let ray = unit_ray();
        let (v0, v1, v2) = front_tri();
        let hit = ray_triangle(&ray, v0, v1, v2, 0).unwrap();
        let pt = hit_point(&ray, &hit);
        assert!((pt[2]).abs() < 1e-4);
    }

    #[test]
    fn test_ray_direction() {
        let d = ray_direction([0.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
        assert!((d[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hit_to_json() {
        let hit = RayHit {
            t: 5.0,
            u: 0.3,
            v: 0.3,
            triangle: 0,
        };
        let json = hit_to_json(&hit);
        assert!(json.contains("triangle"));
    }

    #[test]
    fn test_behind_ray_miss() {
        let ray = Ray {
            origin: [0.0, 0.0, 5.0],
            direction: [0.0, 0.0, 1.0],
        };
        let (v0, v1, v2) = front_tri();
        let hit = ray_triangle(&ray, v0, v1, v2, 0);
        assert!(hit.is_none());
    }
}
