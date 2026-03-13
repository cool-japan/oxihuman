// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! BVH (Bounding Volume Hierarchy) acceleration structure for triangle meshes.
//!
//! Complements `octree.rs` (which handles points) by targeting triangle faces.
//! Uses median-split on the longest centroid axis and Möller-Trumbore ray-triangle
//! intersection.

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

// ---------------------------------------------------------------------------
// Möller-Trumbore ray-triangle intersection (local, no cross-module dependency)
// ---------------------------------------------------------------------------

/// Returns `Some((t, u, v))` where t is distance along ray, and (u,v) are
/// barycentric coordinates on the triangle. Also works for back-faces
/// (det can be negative).
#[allow(dead_code)]
fn moller_trumbore(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<(f32, f32, f32)> {
    const EPSILON: f32 = 1e-8;
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let h = cross3(dir, e2);
    let det = dot3(e1, h);
    if det.abs() < EPSILON {
        return None; // Ray is parallel to triangle plane.
    }
    let inv_det = 1.0 / det;
    let s = sub3(origin, v0);
    let u = inv_det * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = inv_det * dot3(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv_det * dot3(e2, q);
    if t > EPSILON {
        Some((t, u, v))
    } else {
        None // Intersection is behind the ray origin.
    }
}

// ---------------------------------------------------------------------------
// BvhAabb
// ---------------------------------------------------------------------------

/// An axis-aligned bounding box (AABB) for a set of triangles.
#[derive(Debug, Clone)]
pub struct BvhAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BvhAabb {
    /// Create an empty AABB (min = +inf, max = -inf).
    pub fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    /// Expand this AABB to include point `p`.
    pub fn expand_with_point(&mut self, p: [f32; 3]) {
        self.min[0] = self.min[0].min(p[0]);
        self.min[1] = self.min[1].min(p[1]);
        self.min[2] = self.min[2].min(p[2]);
        self.max[0] = self.max[0].max(p[0]);
        self.max[1] = self.max[1].max(p[1]);
        self.max[2] = self.max[2].max(p[2]);
    }

    /// Expand this AABB to include another AABB.
    pub fn expand_with_aabb(&mut self, other: &BvhAabb) {
        self.min[0] = self.min[0].min(other.min[0]);
        self.min[1] = self.min[1].min(other.min[1]);
        self.min[2] = self.min[2].min(other.min[2]);
        self.max[0] = self.max[0].max(other.max[0]);
        self.max[1] = self.max[1].max(other.max[1]);
        self.max[2] = self.max[2].max(other.max[2]);
    }

    /// Center of this AABB.
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Surface area of the AABB (2*(dx*dy + dy*dz + dz*dx)).
    pub fn surface_area(&self) -> f32 {
        let dx = (self.max[0] - self.min[0]).max(0.0);
        let dy = (self.max[1] - self.min[1]).max(0.0);
        let dz = (self.max[2] - self.min[2]).max(0.0);
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    /// Slab test ray-AABB intersection.
    ///
    /// `inv_dir` = `[1/dx, 1/dy, 1/dz]` (precomputed by caller).
    /// Returns `true` if the ray hits the AABB.
    pub fn intersects_ray(&self, origin: [f32; 3], inv_dir: [f32; 3]) -> bool {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;

        #[allow(clippy::needless_range_loop)]
        for i in 0..3 {
            let t1 = (self.min[i] - origin[i]) * inv_dir[i];
            let t2 = (self.max[i] - origin[i]) * inv_dir[i];
            let lo = t1.min(t2);
            let hi = t1.max(t2);
            t_min = t_min.max(lo);
            t_max = t_max.min(hi);
        }

        t_max >= t_min && t_max >= 0.0
    }

    /// Compute a tight AABB for a single triangle.
    pub fn from_triangle(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> Self {
        let mut aabb = BvhAabb::empty();
        aabb.expand_with_point(v0);
        aabb.expand_with_point(v1);
        aabb.expand_with_point(v2);
        aabb
    }
}

// ---------------------------------------------------------------------------
// RayHit
// ---------------------------------------------------------------------------

/// Result of a ray-mesh intersection test.
#[derive(Debug, Clone)]
pub struct RayHit {
    /// Triangle face index (i.e., the face is `mesh.indices[face_index*3 .. face_index*3+3]`).
    pub face_index: usize,
    /// Distance along the ray.
    pub t: f32,
    /// Barycentric coordinates `(w, u, v)` where `w = 1 - u - v`.
    pub barycentric: [f32; 3],
    /// Hit position in world space.
    pub position: [f32; 3],
}

// ---------------------------------------------------------------------------
// BvhNode (private)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
enum BvhNode {
    Leaf {
        aabb: BvhAabb,
        face_indices: Vec<usize>,
    },
    Internal {
        aabb: BvhAabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    fn aabb(&self) -> &BvhAabb {
        match self {
            BvhNode::Leaf { aabb, .. } => aabb,
            BvhNode::Internal { aabb, .. } => aabb,
        }
    }

    fn depth(&self) -> usize {
        match self {
            BvhNode::Leaf { .. } => 1,
            BvhNode::Internal { left, right, .. } => 1 + left.depth().max(right.depth()),
        }
    }

    fn node_count(&self) -> usize {
        match self {
            BvhNode::Leaf { .. } => 1,
            BvhNode::Internal { left, right, .. } => 1 + left.node_count() + right.node_count(),
        }
    }

    /// Build a BVH node from `face_indices` using median-split on the longest centroid axis.
    fn build(mesh: &MeshBuffers, mut face_indices: Vec<usize>, max_leaf_size: usize) -> Self {
        // Compute tight AABB over all triangles.
        let mut aabb = BvhAabb::empty();
        for &fi in &face_indices {
            let (v0, v1, v2) = face_verts(mesh, fi);
            aabb.expand_with_point(v0);
            aabb.expand_with_point(v1);
            aabb.expand_with_point(v2);
        }

        if face_indices.len() <= max_leaf_size {
            return BvhNode::Leaf { aabb, face_indices };
        }

        // Compute AABB of centroids to find the longest split axis.
        let mut centroid_aabb = BvhAabb::empty();
        for &fi in &face_indices {
            let (v0, v1, v2) = face_verts(mesh, fi);
            let centroid = scale3(add3(add3(v0, v1), v2), 1.0 / 3.0);
            centroid_aabb.expand_with_point(centroid);
        }

        let ext = [
            centroid_aabb.max[0] - centroid_aabb.min[0],
            centroid_aabb.max[1] - centroid_aabb.min[1],
            centroid_aabb.max[2] - centroid_aabb.min[2],
        ];

        let axis = if ext[0] >= ext[1] && ext[0] >= ext[2] {
            0
        } else if ext[1] >= ext[2] {
            1
        } else {
            2
        };

        // Sort by centroid on the chosen axis and split at the median.
        face_indices.sort_unstable_by(|&a, &b| {
            let ca = face_centroid(mesh, a)[axis];
            let cb = face_centroid(mesh, b)[axis];
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = face_indices.len() / 2;
        let right_faces = face_indices.split_off(mid);
        let left_faces = face_indices;

        // Guard against degenerate splits (all faces on one side).
        if left_faces.is_empty() || right_faces.is_empty() {
            let all = if left_faces.is_empty() {
                right_faces
            } else {
                left_faces
            };
            return BvhNode::Leaf {
                aabb,
                face_indices: all,
            };
        }

        let left = Box::new(BvhNode::build(mesh, left_faces, max_leaf_size));
        let right = Box::new(BvhNode::build(mesh, right_faces, max_leaf_size));

        BvhNode::Internal { aabb, left, right }
    }

    /// Find the closest intersection along the ray. Updates `best` in-place.
    fn intersect_closest(
        &self,
        mesh: &MeshBuffers,
        origin: [f32; 3],
        dir: [f32; 3],
        inv_dir: [f32; 3],
        best: &mut Option<RayHit>,
    ) {
        if !self.aabb().intersects_ray(origin, inv_dir) {
            return;
        }

        match self {
            BvhNode::Leaf { face_indices, .. } => {
                for &fi in face_indices {
                    if let Some(hit) = test_face(mesh, fi, origin, dir) {
                        let keep = best.as_ref().is_none_or(|b| hit.t < b.t);
                        if keep {
                            *best = Some(hit);
                        }
                    }
                }
            }
            BvhNode::Internal { left, right, .. } => {
                left.intersect_closest(mesh, origin, dir, inv_dir, best);
                right.intersect_closest(mesh, origin, dir, inv_dir, best);
            }
        }
    }

    /// Collect all intersections.
    fn intersect_all(
        &self,
        mesh: &MeshBuffers,
        origin: [f32; 3],
        dir: [f32; 3],
        inv_dir: [f32; 3],
        results: &mut Vec<RayHit>,
    ) {
        if !self.aabb().intersects_ray(origin, inv_dir) {
            return;
        }

        match self {
            BvhNode::Leaf { face_indices, .. } => {
                for &fi in face_indices {
                    if let Some(hit) = test_face(mesh, fi, origin, dir) {
                        results.push(hit);
                    }
                }
            }
            BvhNode::Internal { left, right, .. } => {
                left.intersect_all(mesh, origin, dir, inv_dir, results);
                right.intersect_all(mesh, origin, dir, inv_dir, results);
            }
        }
    }

    /// Early-exit any-hit test within `max_t`.
    fn intersects_any(
        &self,
        mesh: &MeshBuffers,
        origin: [f32; 3],
        dir: [f32; 3],
        inv_dir: [f32; 3],
        max_t: f32,
    ) -> bool {
        if !self.aabb().intersects_ray(origin, inv_dir) {
            return false;
        }

        match self {
            BvhNode::Leaf { face_indices, .. } => {
                for &fi in face_indices {
                    if let Some(hit) = test_face(mesh, fi, origin, dir) {
                        if hit.t <= max_t {
                            return true;
                        }
                    }
                }
                false
            }
            BvhNode::Internal { left, right, .. } => {
                left.intersects_any(mesh, origin, dir, inv_dir, max_t)
                    || right.intersects_any(mesh, origin, dir, inv_dir, max_t)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[inline]
fn face_verts(mesh: &MeshBuffers, fi: usize) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let base = fi * 3;
    let v0 = mesh.positions[mesh.indices[base] as usize];
    let v1 = mesh.positions[mesh.indices[base + 1] as usize];
    let v2 = mesh.positions[mesh.indices[base + 2] as usize];
    (v0, v1, v2)
}

#[inline]
fn face_centroid(mesh: &MeshBuffers, fi: usize) -> [f32; 3] {
    let (v0, v1, v2) = face_verts(mesh, fi);
    scale3(add3(add3(v0, v1), v2), 1.0 / 3.0)
}

/// Test a single face against a ray. Returns `Some(RayHit)` on intersection.
fn test_face(mesh: &MeshBuffers, fi: usize, origin: [f32; 3], dir: [f32; 3]) -> Option<RayHit> {
    let (v0, v1, v2) = face_verts(mesh, fi);
    let (t, u, v) = moller_trumbore(origin, dir, v0, v1, v2)?;
    let w = 1.0 - u - v;
    let position = add3(add3(scale3(v0, w), scale3(v1, u)), scale3(v2, v));
    Some(RayHit {
        face_index: fi,
        t,
        barycentric: [w, u, v],
        position,
    })
}

// ---------------------------------------------------------------------------
// Bvh
// ---------------------------------------------------------------------------

/// BVH acceleration structure for a triangle mesh.
pub struct Bvh {
    root: BvhNode,
    /// Number of triangle faces in the mesh this BVH was built from.
    pub face_count: usize,
}

impl Bvh {
    /// Build a BVH from a mesh using median-split on the longest centroid axis.
    ///
    /// `max_leaf_size`: maximum number of triangles per leaf node (e.g., 4).
    pub fn build(mesh: &MeshBuffers, max_leaf_size: usize) -> Self {
        let fc = mesh.face_count();
        let face_indices: Vec<usize> = (0..fc).collect();
        let max_leaf = max_leaf_size.max(1);

        let root = if face_indices.is_empty() {
            // Empty mesh — create a sentinel leaf.
            BvhNode::Leaf {
                aabb: BvhAabb::empty(),
                face_indices: Vec::new(),
            }
        } else {
            BvhNode::build(mesh, face_indices, max_leaf)
        };

        Bvh {
            root,
            face_count: fc,
        }
    }

    /// Find the closest ray intersection.
    ///
    /// `direction` does not need to be normalised (but should be for meaningful
    /// `t` values). Returns `None` if no triangle is hit.
    pub fn intersect_ray(
        &self,
        mesh: &MeshBuffers,
        origin: [f32; 3],
        direction: [f32; 3],
    ) -> Option<RayHit> {
        let inv_dir = [1.0 / direction[0], 1.0 / direction[1], 1.0 / direction[2]];
        let mut best: Option<RayHit> = None;
        self.root
            .intersect_closest(mesh, origin, direction, inv_dir, &mut best);
        best
    }

    /// Find all ray intersections (not just the closest).
    pub fn intersect_ray_all(
        &self,
        mesh: &MeshBuffers,
        origin: [f32; 3],
        direction: [f32; 3],
    ) -> Vec<RayHit> {
        let inv_dir = [1.0 / direction[0], 1.0 / direction[1], 1.0 / direction[2]];
        let mut results = Vec::new();
        self.root
            .intersect_all(mesh, origin, direction, inv_dir, &mut results);
        results
    }

    /// Check if any intersection exists within `max_t` (early-exit version).
    pub fn intersects_ray_any(
        &self,
        mesh: &MeshBuffers,
        origin: [f32; 3],
        direction: [f32; 3],
        max_t: f32,
    ) -> bool {
        let inv_dir = [1.0 / direction[0], 1.0 / direction[1], 1.0 / direction[2]];
        self.root
            .intersects_any(mesh, origin, direction, inv_dir, max_t)
    }

    /// Depth of the BVH tree (number of levels).
    pub fn depth(&self) -> usize {
        self.root.depth()
    }

    /// Total number of nodes in the BVH tree.
    pub fn node_count(&self) -> usize {
        self.root.node_count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn single_triangle_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: false,
        }
    }

    /// Mesh with N triangles arranged as a grid in the XY plane.
    /// Each cell is two triangles; grid is `n × n` quads, so `2*n*n` triangles.
    fn grid_mesh(n: u32) -> MeshBuffers {
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for y in 0..=n {
            for x in 0..=n {
                positions.push([x as f32, y as f32, 0.0f32]);
            }
        }
        let stride = n + 1;
        for y in 0..n {
            for x in 0..n {
                let i0 = y * stride + x;
                let i1 = i0 + 1;
                let i2 = i0 + stride;
                let i3 = i2 + 1;
                indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
            }
        }
        let nv = positions.len();
        MeshBuffers {
            normals: vec![[0.0, 0.0, 1.0]; nv],
            uvs: vec![[0.0, 0.0]; nv],
            tangents: vec![],
            colors: None,
            has_suit: false,
            positions,
            indices,
        }
    }

    /// Brute-force closest hit for comparison.
    fn brute_intersect(mesh: &MeshBuffers, origin: [f32; 3], dir: [f32; 3]) -> Option<RayHit> {
        let fc = mesh.face_count();
        let mut best: Option<RayHit> = None;
        for fi in 0..fc {
            if let Some(hit) = test_face(mesh, fi, origin, dir) {
                let keep = best.as_ref().is_none_or(|b| hit.t < b.t);
                if keep {
                    best = Some(hit);
                }
            }
        }
        best
    }

    // ── BvhAabb tests ─────────────────────────────────────────────────────────

    #[test]
    fn bvh_aabb_intersects_ray_hit() {
        let aabb = BvhAabb {
            min: [-1.0f32; 3],
            max: [1.0f32; 3],
        };
        let origin = [0.0f32, 0.0, 5.0];
        let dir = [0.0f32, 0.0, -1.0];
        let inv_dir = [1.0 / dir[0], 1.0 / dir[1], 1.0 / dir[2]];
        assert!(
            aabb.intersects_ray(origin, inv_dir),
            "ray should hit the unit cube"
        );
    }

    #[test]
    fn bvh_aabb_intersects_ray_miss() {
        let aabb = BvhAabb {
            min: [-1.0f32; 3],
            max: [1.0f32; 3],
        };
        // Ray shoots in +X and misses the cube entirely.
        let origin = [5.0f32, 5.0, 5.0];
        let dir = [1.0f32, 0.0, 0.0];
        let inv_dir = [1.0 / dir[0], 1.0 / dir[1], 1.0 / dir[2]];
        assert!(!aabb.intersects_ray(origin, inv_dir), "ray should miss");
    }

    // ── Bvh build tests ───────────────────────────────────────────────────────

    #[test]
    fn bvh_build_single_triangle() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);
        assert_eq!(bvh.face_count, 1);
        assert!(bvh.depth() >= 1);
        assert!(bvh.node_count() >= 1);
    }

    #[test]
    fn bvh_build_many_triangles() {
        let mesh = grid_mesh(8); // 128 triangles
        let bvh = Bvh::build(&mesh, 4);
        assert_eq!(bvh.face_count, 128);
        assert!(bvh.depth() > 1, "deep mesh should produce depth > 1");
        assert!(bvh.node_count() > 1);
    }

    // ── Ray intersection tests ────────────────────────────────────────────────

    #[test]
    fn bvh_intersect_ray_hits_triangle() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);

        // Ray from +Z pointing -Z, aimed at the interior of the triangle.
        let origin = [0.2f32, 0.2, 2.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hit = bvh.intersect_ray(&mesh, origin, dir);
        assert!(hit.is_some(), "expected hit, got None");
        let hit = hit.expect("should succeed");
        assert_eq!(hit.face_index, 0);
        assert!(
            (hit.t - 2.0).abs() < 1e-4,
            "expected t ≈ 2.0, got {}",
            hit.t
        );
    }

    #[test]
    fn bvh_intersect_ray_miss() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);

        // Ray aimed well outside the triangle.
        let origin = [10.0f32, 10.0, 2.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hit = bvh.intersect_ray(&mesh, origin, dir);
        assert!(hit.is_none(), "expected miss, got {:?}", hit.map(|h| h.t));
    }

    #[test]
    fn bvh_intersect_ray_all_count() {
        // Grid mesh with triangles in the Z=0 plane.
        // A ray shooting through the grid should hit exactly the 2 triangles
        // sharing a single quad cell around (0.5, 0.5).
        let mesh = grid_mesh(4); // 32 triangles in 4x4 quads
        let bvh = Bvh::build(&mesh, 4);

        let origin = [0.5f32, 0.5, 2.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hits = bvh.intersect_ray_all(&mesh, origin, dir);
        // The point (0.5, 0.5) is covered by 2 triangles in the first quad.
        assert!(
            !hits.is_empty(),
            "expected at least one hit, got {}",
            hits.len()
        );
    }

    #[test]
    fn bvh_intersects_ray_any_true() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);

        let origin = [0.2f32, 0.2, 2.0];
        let dir = [0.0f32, 0.0, -1.0];
        assert!(
            bvh.intersects_ray_any(&mesh, origin, dir, 100.0),
            "expected any-hit = true"
        );
    }

    #[test]
    fn bvh_intersects_ray_any_false_beyond_max_t() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);

        // Triangle is at t = 2.0 along the ray; use max_t = 1.0 so it's out of range.
        let origin = [0.2f32, 0.2, 2.0];
        let dir = [0.0f32, 0.0, -1.0];
        assert!(
            !bvh.intersects_ray_any(&mesh, origin, dir, 1.0),
            "expected any-hit = false when max_t < actual t"
        );
    }

    #[test]
    fn bvh_depth_positive() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);
        assert!(bvh.depth() >= 1, "depth must be at least 1");
    }

    #[test]
    fn bvh_ray_hit_position_on_triangle() {
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);

        let origin = [0.25f32, 0.25, 3.0];
        let dir = [0.0f32, 0.0, -1.0];
        let hit = bvh.intersect_ray(&mesh, origin, dir).expect("expected hit");

        // Position should be at (0.25, 0.25, 0.0) — on the triangle.
        assert!(
            (hit.position[0] - 0.25).abs() < 1e-4,
            "x mismatch: {}",
            hit.position[0]
        );
        assert!(
            (hit.position[1] - 0.25).abs() < 1e-4,
            "y mismatch: {}",
            hit.position[1]
        );
        assert!(
            hit.position[2].abs() < 1e-4,
            "z mismatch: {}",
            hit.position[2]
        );
    }

    #[test]
    fn bvh_vs_brute_force_same_result() {
        let mesh = grid_mesh(6); // 72 triangles
        let bvh = Bvh::build(&mesh, 4);

        let queries: &[([f32; 3], [f32; 3])] = &[
            ([0.5, 0.5, 5.0], [0.0, 0.0, -1.0]),
            ([2.5, 3.1, 5.0], [0.0, 0.0, -1.0]),
            ([5.9, 5.9, 5.0], [0.0, 0.0, -1.0]),
            ([0.1, 0.1, 5.0], [0.0, 0.0, -1.0]),
        ];

        for &(origin, dir) in queries {
            let bvh_hit = bvh.intersect_ray(&mesh, origin, dir);
            let brute_hit = brute_intersect(&mesh, origin, dir);

            match (bvh_hit, brute_hit) {
                (None, None) => {}
                (Some(b), Some(bf)) => {
                    assert!(
                        (b.t - bf.t).abs() < 1e-4,
                        "origin={origin:?}: BVH t={} vs brute t={}",
                        b.t,
                        bf.t
                    );
                }
                (bvh_res, brute_res) => {
                    panic!(
                        "origin={origin:?}: BVH={:?} brute={:?}",
                        bvh_res.map(|h| h.t),
                        brute_res.map(|h| h.t)
                    );
                }
            }
        }
    }

    #[test]
    fn bvh_backface_hit_returns_some() {
        // Triangle in XY plane, normal +Z. Ray coming from -Z pointing +Z
        // hits the back face; Möller-Trumbore allows back-face hits (det can be
        // negative). The function returns Some when the triangle is intersected
        // regardless of winding.
        let mesh = single_triangle_mesh();
        let bvh = Bvh::build(&mesh, 4);

        let origin = [0.2f32, 0.2, -2.0]; // behind the triangle
        let dir = [0.0f32, 0.0, 1.0]; // pointing +Z (towards the back face)
        let hit = bvh.intersect_ray(&mesh, origin, dir);
        assert!(
            hit.is_some(),
            "back-face ray should still intersect (two-sided)"
        );
    }
}
