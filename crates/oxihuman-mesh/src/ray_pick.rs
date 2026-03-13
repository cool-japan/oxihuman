// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Internal math helpers
// ---------------------------------------------------------------------------

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 {
        scale3(v, 1.0 / l)
    } else {
        [0.0, 0.0, 1.0]
    }
}

// ---------------------------------------------------------------------------
// Möller-Trumbore ray-triangle intersection
// ---------------------------------------------------------------------------

/// Returns `Some((t, u, v))` where t is the ray parameter and (u, v) are
/// barycentric coordinates of the hit point (w = 1 - u - v).
fn moller_trumbore(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<(f32, f32, f32)> {
    const EPSILON: f32 = 1e-7;

    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let h = cross3(ray_dir, edge2);
    let det = dot3(edge1, h);

    if det.abs() < EPSILON {
        return None; // Ray is parallel to triangle
    }

    let inv_det = 1.0 / det;
    let s = sub3(ray_origin, v0);
    let u = dot3(s, h) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = cross3(s, edge1);
    let v = dot3(ray_dir, q) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = dot3(edge2, q) * inv_det;
    if t < EPSILON {
        return None; // Intersection is behind the ray origin
    }

    Some((t, u, v))
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A ray defined by an origin and a (ideally normalized) direction.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Ray {
    /// Create a ray with the given origin and direction (not normalized).
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self { origin, direction }
    }

    /// Create a ray and auto-normalize the direction vector.
    pub fn normalized(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            origin,
            direction: normalize3(direction),
        }
    }

    /// Return the point along the ray at parameter `t`: `origin + t * direction`.
    pub fn at(&self, t: f32) -> [f32; 3] {
        add3(self.origin, scale3(self.direction, t))
    }

    /// Create a ray from normalized device coordinates (NDC, [-1, 1]) using a
    /// simple orthographic camera. `camera_dir` is the viewing direction.
    pub fn from_screen_ortho(
        screen_x: f32,
        screen_y: f32,
        camera_pos: [f32; 3],
        camera_dir: [f32; 3],
    ) -> Self {
        // Build an orthonormal basis from camera_dir
        let fwd = normalize3(camera_dir);

        // Choose an "up" vector that is not parallel to fwd
        let world_up = if fwd[1].abs() < 0.9 {
            [0.0f32, 1.0, 0.0]
        } else {
            [1.0f32, 0.0, 0.0]
        };
        let right = normalize3(cross3(fwd, world_up));
        let up = normalize3(cross3(right, fwd));

        // Shift the ray origin by the NDC offset in the camera plane
        let origin = [
            camera_pos[0] + right[0] * screen_x + up[0] * screen_y,
            camera_pos[1] + right[1] * screen_x + up[1] * screen_y,
            camera_pos[2] + right[2] * screen_x + up[2] * screen_y,
        ];

        Self {
            origin,
            direction: fwd,
        }
    }
}

/// Result of a successful ray-mesh intersection.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Index of the hit face (triangle index, i.e. `indices[face_index*3..]`).
    pub face_index: usize,
    /// Index of the nearest vertex of the hit face.
    pub vertex_index: usize,
    /// World-space hit point.
    pub hit_point: [f32; 3],
    /// Distance along the ray to the hit.
    pub distance: f32,
    /// Barycentric coordinates `[w, u, v]` where w = 1 - u - v.
    pub barycentric: [f32; 3],
    /// Face normal at the hit point.
    pub normal: [f32; 3],
}

/// Parameters that control picking behaviour.
#[derive(Debug, Clone)]
pub struct PickParams {
    /// Maximum allowed ray distance (default `f32::MAX`).
    pub max_distance: f32,
    /// Skip back-facing triangles when `true` (default `true`).
    pub backface_culling: bool,
    /// Distance-to-ray threshold for vertex picking (default `0.01`).
    pub vertex_pick_radius: f32,
}

impl Default for PickParams {
    fn default() -> Self {
        Self {
            max_distance: f32::MAX,
            backface_culling: true,
            vertex_pick_radius: 0.01,
        }
    }
}

// ---------------------------------------------------------------------------
// Core picking functions
// ---------------------------------------------------------------------------

/// Find the nearest face intersected by a ray.
pub fn pick_face(mesh: &MeshBuffers, ray: &Ray, params: &PickParams) -> Option<PickResult> {
    let results = pick_all_faces(mesh, ray, params);
    results.into_iter().next()
}

/// Find all faces intersected by a ray, sorted by distance (nearest first).
pub fn pick_all_faces(mesh: &MeshBuffers, ray: &Ray, params: &PickParams) -> Vec<PickResult> {
    let indices = &mesh.indices;
    let positions = &mesh.positions;
    let num_faces = indices.len() / 3;

    let mut hits: Vec<PickResult> = Vec::new();

    for fi in 0..num_faces {
        let i0 = indices[fi * 3] as usize;
        let i1 = indices[fi * 3 + 1] as usize;
        let i2 = indices[fi * 3 + 2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        // Compute face normal
        let edge1 = sub3(v1, v0);
        let edge2 = sub3(v2, v0);
        let face_normal = normalize3(cross3(edge1, edge2));

        // Backface culling: skip if ray direction and normal point the same way
        if params.backface_culling && dot3(face_normal, ray.direction) > 0.0 {
            continue;
        }

        let Some((t, u, v)) = moller_trumbore(ray.origin, ray.direction, v0, v1, v2) else {
            continue;
        };

        if t > params.max_distance {
            continue;
        }

        let hit_point = ray.at(t);
        let w = 1.0 - u - v;

        // Find the nearest vertex to the hit point
        let d0 = len3(sub3(hit_point, v0));
        let d1 = len3(sub3(hit_point, v1));
        let d2 = len3(sub3(hit_point, v2));
        let vertex_index = if d0 <= d1 && d0 <= d2 {
            i0
        } else if d1 <= d2 {
            i1
        } else {
            i2
        };

        hits.push(PickResult {
            face_index: fi,
            vertex_index,
            hit_point,
            distance: t,
            barycentric: [w, u, v],
            normal: face_normal,
        });
    }

    hits.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

/// Find the nearest vertex to a ray (by closest-point-on-ray distance).
///
/// Returns `(vertex_index, distance_to_ray)` for the nearest vertex whose
/// distance to the ray is within `params.vertex_pick_radius`.
pub fn pick_vertex(mesh: &MeshBuffers, ray: &Ray, params: &PickParams) -> Option<(usize, f32)> {
    let mut best: Option<(usize, f32)> = None;

    for (vi, pos) in mesh.positions.iter().enumerate() {
        let d = point_to_ray_distance(*pos, ray);
        if d > params.vertex_pick_radius {
            continue;
        }
        if best.is_none_or(|(_, bd)| d < bd) {
            best = Some((vi, d));
        }
    }

    best
}

// ---------------------------------------------------------------------------
// Selection helpers
// ---------------------------------------------------------------------------

/// Select all vertices whose 2-D projection falls within the screen-space
/// rectangle defined by `box_min` and `box_max`.
pub fn box_select_vertices(
    mesh: &MeshBuffers,
    box_min: [f32; 2],
    box_max: [f32; 2],
    _camera_dir: [f32; 3],
    project: fn(&[f32; 3]) -> [f32; 2],
) -> Vec<usize> {
    mesh.positions
        .iter()
        .enumerate()
        .filter_map(|(vi, pos)| {
            let p2 = project(pos);
            if p2[0] >= box_min[0]
                && p2[0] <= box_max[0]
                && p2[1] >= box_min[1]
                && p2[1] <= box_max[1]
            {
                Some(vi)
            } else {
                None
            }
        })
        .collect()
}

/// Select all vertices within a sphere of given `radius` centred at `center`.
pub fn sphere_select_vertices(mesh: &MeshBuffers, center: [f32; 3], radius: f32) -> Vec<usize> {
    let r2 = radius * radius;
    mesh.positions
        .iter()
        .enumerate()
        .filter_map(|(vi, pos)| {
            let d = sub3(*pos, center);
            let dist2 = dot3(d, d);
            if dist2 <= r2 {
                Some(vi)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Math utilities
// ---------------------------------------------------------------------------

/// Project a 3-D point onto a ray; returns the ray parameter `t` such that
/// `ray.at(t)` is the closest point on the ray to `point`.
pub fn project_onto_ray(point: [f32; 3], ray: &Ray) -> f32 {
    dot3(sub3(point, ray.origin), ray.direction)
}

/// Closest point on a ray to a given point.
pub fn closest_ray_point(point: [f32; 3], ray: &Ray) -> [f32; 3] {
    let t = project_onto_ray(point, ray);
    ray.at(t)
}

/// Distance from a point to the nearest point on a ray.
pub fn point_to_ray_distance(point: [f32; 3], ray: &Ray) -> f32 {
    // d = |cross(ray.dir, point - ray.origin)|
    let diff = sub3(point, ray.origin);
    let c = cross3(ray.direction, diff);
    len3(c)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Build a simple one-triangle mesh in the XY-plane facing +Z.
    ///
    /// Vertices: (0,0,0), (1,0,0), (0,1,0)
    /// Face normal: (0,0,1)
    fn single_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    /// Build a two-triangle mesh (a quad split into two tris).
    fn two_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    // -----------------------------------------------------------------------
    // Ray construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_ray_new() {
        let ray = Ray::new([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
        assert_eq!(ray.origin, [0.0, 0.0, 5.0]);
        assert_eq!(ray.direction, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn test_ray_at() {
        let ray = Ray::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = ray.at(3.0);
        assert!((p[0] - 3.0).abs() < 1e-6);
        assert!(p[1].abs() < 1e-6);
        assert!(p[2].abs() < 1e-6);
    }

    #[test]
    fn test_ray_normalized() {
        let ray = Ray::normalized([0.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
        let l = len3(ray.direction);
        assert!(
            (l - 1.0).abs() < 1e-6,
            "direction should be unit length, got {l}"
        );
        assert!((ray.direction[0] - 1.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // point_to_ray_distance
    // -----------------------------------------------------------------------

    #[test]
    fn test_point_to_ray_distance_zero() {
        // A point on the ray should have distance 0
        let ray = Ray::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let d = point_to_ray_distance([2.0, 0.0, 0.0], &ray);
        assert!(d.abs() < 1e-6, "expected 0, got {d}");
    }

    #[test]
    fn test_point_to_ray_distance_perp() {
        // A point 1 unit perpendicular to the ray
        let ray = Ray::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let d = point_to_ray_distance([0.0, 1.0, 0.0], &ray);
        assert!((d - 1.0).abs() < 1e-6, "expected 1.0, got {d}");
    }

    // -----------------------------------------------------------------------
    // project_onto_ray / closest_ray_point
    // -----------------------------------------------------------------------

    #[test]
    fn test_project_onto_ray() {
        let ray = Ray::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let t = project_onto_ray([4.0, 2.0, 0.0], &ray);
        assert!((t - 4.0).abs() < 1e-6, "expected t=4.0, got {t}");
    }

    #[test]
    fn test_closest_ray_point() {
        let ray = Ray::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let cp = closest_ray_point([3.0, 5.0, 7.0], &ray);
        // Closest point on X-axis to (3,5,7) is (3,0,0)
        assert!((cp[0] - 3.0).abs() < 1e-6);
        assert!(cp[1].abs() < 1e-6);
        assert!(cp[2].abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // pick_face
    // -----------------------------------------------------------------------

    #[test]
    fn test_pick_face_hit() {
        let mesh = single_tri_mesh();
        // Ray from above the centroid, pointing down
        let ray = Ray::new([0.25, 0.25, 5.0], [0.0, 0.0, -1.0]);
        let params = PickParams::default();
        let result = pick_face(&mesh, &ray, &params);
        assert!(result.is_some(), "expected a hit");
        let r = result.expect("should succeed");
        assert_eq!(r.face_index, 0);
        assert!(
            (r.distance - 5.0).abs() < 1e-4,
            "expected t~5, got {}",
            r.distance
        );
        assert!((r.hit_point[2]).abs() < 1e-4);
    }

    #[test]
    fn test_pick_face_miss() {
        let mesh = single_tri_mesh();
        // Ray aimed well outside the triangle
        let ray = Ray::new([5.0, 5.0, 5.0], [0.0, 0.0, -1.0]);
        let params = PickParams::default();
        let result = pick_face(&mesh, &ray, &params);
        assert!(result.is_none(), "expected no hit");
    }

    // -----------------------------------------------------------------------
    // pick_all_faces
    // -----------------------------------------------------------------------

    #[test]
    fn test_pick_all_faces() {
        let mesh = two_tri_mesh();
        // Two-tri mesh: v0=(0,0,0), v1=(1,0,0), v2=(1,1,0), v3=(0,1,0)
        // face 0: (v0,v1,v2) — lower-right triangle
        // face 1: (v0,v2,v3) — upper-left triangle
        //
        // A ray at (0.8, 0.2) lies strictly inside face 0 and misses face 1.
        let ray_f0 = Ray::new([0.8, 0.2, 5.0], [0.0, 0.0, -1.0]);
        let params = PickParams::default();
        let hits_f0 = pick_all_faces(&mesh, &ray_f0, &params);
        assert_eq!(
            hits_f0.len(),
            1,
            "ray at (0.8,0.2) should hit face 0 only, got {}",
            hits_f0.len()
        );
        assert_eq!(hits_f0[0].face_index, 0, "should hit face 0");

        // A ray at (0.2, 0.8) lies strictly inside face 1 and misses face 0.
        let ray_f1 = Ray::new([0.2, 0.8, 5.0], [0.0, 0.0, -1.0]);
        let hits_f1 = pick_all_faces(&mesh, &ray_f1, &params);
        assert_eq!(
            hits_f1.len(),
            1,
            "ray at (0.2,0.8) should hit face 1 only, got {}",
            hits_f1.len()
        );
        assert_eq!(hits_f1[0].face_index, 1, "should hit face 1");

        // Results must be sorted by distance (nearest first)
        for w in hits_f0.windows(2) {
            assert!(
                w[0].distance <= w[1].distance,
                "hits should be sorted by distance"
            );
        }
    }

    // -----------------------------------------------------------------------
    // pick_vertex
    // -----------------------------------------------------------------------

    #[test]
    fn test_pick_vertex() {
        let mesh = single_tri_mesh();
        // Ray very close to vertex (0,0,0), passing alongside it
        let ray = Ray::new([0.001, 0.001, 5.0], [0.0, 0.0, -1.0]);
        let params = PickParams {
            vertex_pick_radius: 0.1,
            ..Default::default()
        };
        let result = pick_vertex(&mesh, &ray, &params);
        assert!(result.is_some(), "expected vertex 0 to be selected");
        let (vi, _d) = result.expect("should succeed");
        assert_eq!(vi, 0, "expected vertex index 0, got {vi}");
    }

    // -----------------------------------------------------------------------
    // sphere_select_vertices
    // -----------------------------------------------------------------------

    #[test]
    fn test_sphere_select_vertices() {
        let mesh = single_tri_mesh();
        // Sphere around origin with radius 0.5 — should capture vertex 0 only
        let selected = sphere_select_vertices(&mesh, [0.0, 0.0, 0.0], 0.5);
        assert!(selected.contains(&0), "vertex 0 should be selected");
        assert!(
            !selected.contains(&1),
            "vertex 1 is at distance 1 — should not be selected"
        );
        assert!(
            !selected.contains(&2),
            "vertex 2 is at distance 1 — should not be selected"
        );
    }

    // -----------------------------------------------------------------------
    // box_select_vertices
    // -----------------------------------------------------------------------

    #[test]
    fn test_box_select_vertices() {
        let mesh = single_tri_mesh();
        // Simple XY projection ignoring Z
        fn proj_xy(p: &[f32; 3]) -> [f32; 2] {
            [p[0], p[1]]
        }
        // Box covering only the origin area
        let selected =
            box_select_vertices(&mesh, [-0.1, -0.1], [0.1, 0.1], [0.0, 0.0, -1.0], proj_xy);
        assert_eq!(selected, vec![0], "only vertex 0 should be inside the box");
    }
}
