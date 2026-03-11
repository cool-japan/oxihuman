// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ray-mesh intersection query stub.

/// A 3-D ray defined by origin and direction.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Ray {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        let d = normalize3(direction);
        Self {
            origin,
            direction: d,
        }
    }

    pub fn at(&self, t: f32) -> [f32; 3] {
        [
            self.origin[0] + t * self.direction[0],
            self.origin[1] + t * self.direction[1],
            self.origin[2] + t * self.direction[2],
        ]
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
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

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Result of a ray-triangle intersection test.
#[derive(Debug, Clone, Copy)]
pub struct RayHitResult {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub triangle_index: u32,
    pub hit_point: [f32; 3],
}

/// Möller–Trumbore ray-triangle intersection test.
pub fn ray_triangle_intersect(
    ray: &Ray,
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<(f32, f32, f32)> {
    /* Returns (t, u, v) if intersection at t > 0 */
    const EPS: f32 = 1e-7;
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let h = cross3(ray.direction, edge2);
    let a = dot3(edge1, h);
    if a.abs() < EPS {
        return None;
    }
    let f = 1.0 / a;
    let s = sub3(ray.origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, edge1);
    let v = f * dot3(ray.direction, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(edge2, q);
    if t < EPS {
        return None;
    }
    Some((t, u, v))
}

/// Find the nearest ray-mesh intersection.
pub fn intersect_ray_mesh(
    ray: &Ray,
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Option<RayHitResult> {
    /* Test all triangles and return the nearest hit */
    let mut nearest: Option<RayHitResult> = None;
    for (i, tri) in tris.iter().enumerate() {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        if let Some((t, u, v)) = ray_triangle_intersect(ray, verts[i0], verts[i1], verts[i2]) {
            let better = nearest.as_ref().is_none_or(|n| t < n.t);
            if better {
                nearest = Some(RayHitResult {
                    t,
                    u,
                    v,
                    triangle_index: i as u32,
                    hit_point: ray.at(t),
                });
            }
        }
    }
    nearest
}

/// Find ALL ray-mesh intersections (not just the nearest).
pub fn intersect_ray_mesh_all(
    ray: &Ray,
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Vec<RayHitResult> {
    /* Return all hits sorted by ascending t */
    let mut hits = Vec::new();
    for (i, tri) in tris.iter().enumerate() {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        if let Some((t, u, v)) = ray_triangle_intersect(ray, verts[i0], verts[i1], verts[i2]) {
            hits.push(RayHitResult {
                t,
                u,
                v,
                triangle_index: i as u32,
                hit_point: ray.at(t),
            });
        }
    }
    hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    hits
}

/// Count how many triangles the ray intersects.
pub fn count_ray_intersections(ray: &Ray, verts: &[[f32; 3]], tris: &[[u32; 3]]) -> usize {
    intersect_ray_mesh_all(ray, verts, tris).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    type TriangleData = ([[f32; 3]; 3], Vec<[f32; 3]>, Vec<[u32; 3]>);

    #[allow(clippy::type_complexity)]
    fn make_triangle() -> TriangleData {
        let v = [[-1.0f32, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];
        (v, v.to_vec(), vec![[0, 1, 2]])
    }

    #[test]
    fn test_ray_at() {
        let r = Ray::new([0.0; 3], [0.0, 0.0, 1.0]);
        let pt = r.at(3.0);
        assert!((pt[2] - 3.0).abs() < 1e-5 /* point at t=3 */);
    }

    #[test]
    fn test_ray_triangle_hit() {
        let (v, _, _) = make_triangle();
        let ray = Ray::new([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let result = ray_triangle_intersect(&ray, v[0], v[1], v[2]);
        assert!(result.is_some() /* ray hits triangle */);
    }

    #[test]
    fn test_ray_triangle_miss() {
        let (v, _, _) = make_triangle();
        let ray = Ray::new([10.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let result = ray_triangle_intersect(&ray, v[0], v[1], v[2]);
        assert!(result.is_none() /* ray misses triangle */);
    }

    #[test]
    fn test_intersect_mesh_finds_hit() {
        let (_, verts, tris) = make_triangle();
        let ray = Ray::new([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let hit = intersect_ray_mesh(&ray, &verts, &tris);
        assert!(hit.is_some() /* hit found */);
    }

    #[test]
    fn test_intersect_mesh_no_hit() {
        let (_, verts, tris) = make_triangle();
        let ray = Ray::new([10.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let hit = intersect_ray_mesh(&ray, &verts, &tris);
        assert!(hit.is_none() /* no hit */);
    }

    #[test]
    fn test_intersect_all_count() {
        let (_, verts, tris) = make_triangle();
        let ray = Ray::new([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        assert_eq!(
            count_ray_intersections(&ray, &verts, &tris),
            1 /* one hit */
        );
    }

    #[test]
    fn test_intersect_all_sorted() {
        let verts = vec![
            [-1.0f32, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0f32, -1.0, 2.0],
            [1.0, -1.0, 2.0],
            [0.0, 1.0, 2.0],
        ];
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let ray = Ray::new([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let hits = intersect_ray_mesh_all(&ray, &verts, &tris);
        if hits.len() >= 2 {
            assert!(hits[0].t <= hits[1].t /* sorted ascending */);
        }
    }

    #[test]
    fn test_empty_mesh_no_hit() {
        let ray = Ray::new([0.0; 3], [0.0, 0.0, 1.0]);
        assert!(intersect_ray_mesh(&ray, &[], &[]).is_none() /* no triangles */);
    }

    #[test]
    fn test_hit_point_on_triangle_plane() {
        let (_, verts, tris) = make_triangle();
        let ray = Ray::new([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        if let Some(hit) = intersect_ray_mesh(&ray, &verts, &tris) {
            assert!(hit.hit_point[2].abs() < 1e-4 /* hit is on z=0 plane */);
        }
    }
}
