// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Per-vertex ambient occlusion baking with BVH-accelerated ray casting.
//!
//! # Algorithm
//!
//! For each vertex:
//!
//! 1. Offset the origin slightly along the vertex normal (by `bias`) to avoid
//!    self-intersection with the face that owns this vertex.
//! 2. Generate `ray_count` directions in the hemisphere oriented to the normal
//!    using cosine-weighted sampling (square-to-hemisphere transform) with a
//!    deterministic LCG so results are reproducible.
//! 3. Cast each ray against the mesh's BVH. If any triangle is hit within
//!    `max_distance`, the ray is considered occluded.
//! 4. AO = unoccluded_rays / total_rays.  AO = 1.0 means fully unoccluded.

use std::f32::consts::TAU;

// ── Public API types ──────────────────────────────────────────────────────────

/// Configuration for ambient occlusion baking.
#[derive(Debug, Clone)]
pub struct AoConfig {
    /// Number of hemisphere rays per vertex.  Default 64.
    pub ray_count: u32,
    /// Maximum ray distance for occlusion testing.  Default 1.0.
    pub max_distance: f32,
    /// Ray origin bias along the vertex normal to avoid self-intersection.
    pub bias: f32,
}

impl Default for AoConfig {
    fn default() -> Self {
        Self {
            ray_count: 64,
            max_distance: 1.0,
            bias: 1e-4,
        }
    }
}

/// Minimal mesh representation used as input for AO baking.
/// Mirrors the fields of `oxihuman_mesh::mesh::MeshBuffers` that are needed
/// without pulling in a dependency cycle.
pub struct MeshBuffers<'a> {
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub indices: &'a [u32],
}

/// Bake per-vertex ambient occlusion.
///
/// Returns a `Vec<f32>` with one value per vertex, in `[0.0, 1.0]`.
/// `1.0` = fully unoccluded, `0.0` = fully occluded.
pub fn bake_ambient_occlusion(mesh: &MeshBuffers<'_>, config: &AoConfig) -> Vec<f32> {
    let nv = mesh.positions.len();
    if nv == 0 || config.ray_count == 0 {
        return vec![1.0; nv];
    }

    let bvh = AoBvh::build(mesh.positions, mesh.indices);

    let mut ao_out = Vec::with_capacity(nv);

    for v in 0..nv {
        let pos = mesh.positions[v];
        let normal = if v < mesh.normals.len() {
            normalize3f(mesh.normals[v])
        } else {
            [0.0, 0.0, 1.0]
        };

        // Offset origin by bias along normal
        let origin = [
            pos[0] + normal[0] * config.bias,
            pos[1] + normal[1] * config.bias,
            pos[2] + normal[2] * config.bias,
        ];

        // Build deterministic LCG seed per vertex (wrapping arithmetic to avoid overflow)
        let seed = (v as u64)
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut lcg = Lcg::new(seed);

        let mut unoccluded = 0u32;
        for ray_idx in 0..config.ray_count {
            let dir = cosine_hemisphere_sample(normal, ray_idx, config.ray_count, &mut lcg);
            let hit = bvh.ray_cast(origin, dir, config.max_distance);
            if !hit {
                unoccluded += 1;
            }
        }

        ao_out.push(unoccluded as f32 / config.ray_count as f32);
    }

    ao_out
}

// ── LCG RNG ───────────────────────────────────────────────────────────────────

/// Deterministic linear congruential generator for reproducible sampling.
pub struct Lcg {
    state: u64,
}

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    /// Advance and return the next pseudo-random `f32` in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        // Knuth multiplicative LCG
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Extract upper 23 bits → mantissa of f32 in [1,2), subtract 1
        let mantissa = (self.state >> 41) as u32;
        let bits = 0x3f80_0000_u32 | mantissa;
        f32::from_bits(bits) - 1.0
    }
}

// ── Cosine-weighted hemisphere sampling ───────────────────────────────────────

/// Generate a cosine-weighted hemisphere direction oriented to `normal`.
///
/// Combines a stratified Hammersley-like sequence component with an LCG
/// jitter to produce visually smooth, low-variance AO estimates.
fn cosine_hemisphere_sample(
    normal: [f32; 3],
    ray_idx: u32,
    ray_count: u32,
    lcg: &mut Lcg,
) -> [f32; 3] {
    // Square-to-disk Malley method via cosine mapping
    // u in [0,1): stratified primary
    let u = (ray_idx as f32 + lcg.next_f32()) / ray_count.max(1) as f32;
    // v in [0,1): Van der Corput radical inverse for decorrelated 2nd axis
    let v = radical_inverse_base2(ray_idx);

    let phi = TAU * u;
    let cos_theta = (1.0 - v).max(0.0).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

    let lx = sin_theta * phi.cos();
    let ly = sin_theta * phi.sin();
    let lz = cos_theta;

    // Build tangent frame from normal
    let (tangent, bitangent) = make_tangent_frame(normal);

    [
        lx * tangent[0] + ly * bitangent[0] + lz * normal[0],
        lx * tangent[1] + ly * bitangent[1] + lz * normal[1],
        lx * tangent[2] + ly * bitangent[2] + lz * normal[2],
    ]
}

/// Van der Corput radical inverse in base 2.
#[inline]
fn radical_inverse_base2(n: u32) -> f32 {
    let mut bits = n.reverse_bits();
    // Reinterpret as float mantissa in [1,2)
    bits = 0x3f80_0000_u32 | (bits >> 9);
    f32::from_bits(bits) - 1.0
}

/// Build an orthonormal (tangent, bitangent) frame for the given normal.
fn make_tangent_frame(normal: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if normal[1].abs() < 0.999 {
        [0.0f32, 1.0, 0.0]
    } else {
        [1.0f32, 0.0, 0.0]
    };
    let tangent = normalize3f(cross3f(up, normal));
    let bitangent = cross3f(normal, tangent);
    (tangent, bitangent)
}

// ── BVH for ray-triangle intersection ────────────────────────────────────────

/// Axis-aligned bounding box.
#[derive(Clone, Debug)]
struct Aabb {
    min: [f32; 3],
    max: [f32; 3],
}

impl Aabb {
    fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    fn extend_point(&mut self, p: [f32; 3]) {
        for ((&pv, mn), mx) in p.iter().zip(self.min.iter_mut()).zip(self.max.iter_mut()) {
            if pv < *mn {
                *mn = pv;
            }
            if pv > *mx {
                *mx = pv;
            }
        }
    }

    #[allow(dead_code)]
    fn extend_aabb(&mut self, other: &Aabb) {
        for i in 0..3 {
            if other.min[i] < self.min[i] {
                self.min[i] = other.min[i];
            }
            if other.max[i] > self.max[i] {
                self.max[i] = other.max[i];
            }
        }
    }

    fn centroid(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Slab test for ray-AABB intersection.  Returns `true` if the ray with
    /// origin `ro` and direction `rd` intersects this AABB within `[0, t_max]`.
    fn ray_intersect(&self, ro: [f32; 3], rd_inv: [f32; 3], t_max: f32) -> bool {
        let mut t_min = 0.0f32;
        let mut t_far = t_max;

        for i in 0..3 {
            let t1 = (self.min[i] - ro[i]) * rd_inv[i];
            let t2 = (self.max[i] - ro[i]) * rd_inv[i];
            let (ta, tb) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_min = t_min.max(ta);
            t_far = t_far.min(tb);
        }

        t_min <= t_far
    }
}

/// A BVH leaf holds a list of triangle indices.
#[derive(Clone)]
struct BvhNode {
    bounds: Aabb,
    /// If `left == right == u32::MAX`, this is a leaf; `tris` contains triangle IDs.
    left: u32,
    right: u32,
    /// Leaf triangle primitive indices (into `AoBvh::tris`).
    tri_start: u32,
    tri_count: u32,
}

/// BVH for ray-triangle queries.
pub struct AoBvh {
    nodes: Vec<BvhNode>,
    /// Flat list of triangle vertex data: `[a, b, c]` as `[f32;3]` triples.
    tris: Vec<[[f32; 3]; 3]>,
}

impl AoBvh {
    /// Build a BVH from a mesh's vertex positions and triangle indices.
    pub fn build(positions: &[[f32; 3]], indices: &[u32]) -> Self {
        // Extract triangles
        let mut tris: Vec<[[f32; 3]; 3]> = Vec::with_capacity(indices.len() / 3);
        for chunk in indices.chunks(3) {
            if chunk.len() < 3 {
                continue;
            }
            let a = chunk[0] as usize;
            let b = chunk[1] as usize;
            let c = chunk[2] as usize;
            if a >= positions.len() || b >= positions.len() || c >= positions.len() {
                continue;
            }
            tris.push([positions[a], positions[b], positions[c]]);
        }

        if tris.is_empty() {
            return AoBvh {
                nodes: Vec::new(),
                tris,
            };
        }

        let n_tris = tris.len();
        // Build leaf indices list
        let mut prim_indices: Vec<u32> = (0..n_tris as u32).collect();
        let mut nodes = Vec::with_capacity(n_tris * 2);

        let root_idx = Self::build_recursive(&tris, &mut prim_indices, &mut nodes, 0, n_tris, 0);
        let _ = root_idx;

        // Reorder triangles per prim_indices for cache locality
        let ordered_tris: Vec<[[f32; 3]; 3]> =
            prim_indices.iter().map(|&i| tris[i as usize]).collect();

        AoBvh {
            nodes,
            tris: ordered_tris,
        }
    }

    fn build_recursive(
        tris: &[[[f32; 3]; 3]],
        prim_indices: &mut Vec<u32>,
        nodes: &mut Vec<BvhNode>,
        start: usize,
        end: usize,
        depth: u32,
    ) -> u32 {
        let node_idx = nodes.len() as u32;

        // Compute bounding box
        let mut bounds = Aabb::empty();
        for &idx in &prim_indices[start..end] {
            let tri = tris[idx as usize];
            bounds.extend_point(tri[0]);
            bounds.extend_point(tri[1]);
            bounds.extend_point(tri[2]);
        }

        let count = end - start;

        // Leaf threshold: ≤ 4 triangles or max depth
        if count <= 4 || depth >= 24 {
            let node = BvhNode {
                bounds,
                left: u32::MAX,
                right: u32::MAX,
                tri_start: start as u32,
                tri_count: count as u32,
            };
            nodes.push(node);
            return node_idx;
        }

        // Choose split axis: largest extent
        let extent = [
            bounds.max[0] - bounds.min[0],
            bounds.max[1] - bounds.min[1],
            bounds.max[2] - bounds.min[2],
        ];
        let axis = if extent[0] >= extent[1] && extent[0] >= extent[2] {
            0
        } else if extent[1] >= extent[2] {
            1
        } else {
            2
        };

        // Surface area heuristic split: partition around centroid midpoint
        let mid_val = bounds.centroid()[axis];
        // Stable partition: collect left/right into two vecs then merge
        let (left_prim, right_prim): (Vec<u32>, Vec<u32>) =
            prim_indices[start..end].iter().copied().partition(|&idx| {
                let tri = tris[idx as usize];
                let centroid = [
                    (tri[0][0] + tri[1][0] + tri[2][0]) / 3.0,
                    (tri[0][1] + tri[1][1] + tri[2][1]) / 3.0,
                    (tri[0][2] + tri[1][2] + tri[2][2]) / 3.0,
                ];
                centroid[axis] < mid_val
            });
        let left_len = left_prim.len();
        prim_indices[start..start + left_len].copy_from_slice(&left_prim);
        prim_indices[start + left_len..end].copy_from_slice(&right_prim);
        let split_pos = start + left_len;

        let split = if split_pos == start || split_pos == end {
            start + count / 2
        } else {
            split_pos
        };

        // Push placeholder node, then fill after children
        nodes.push(BvhNode {
            bounds: bounds.clone(),
            left: u32::MAX,
            right: u32::MAX,
            tri_start: 0,
            tri_count: 0,
        });

        let left = Self::build_recursive(tris, prim_indices, nodes, start, split, depth + 1);
        let right = Self::build_recursive(tris, prim_indices, nodes, split, end, depth + 1);

        nodes[node_idx as usize].left = left;
        nodes[node_idx as usize].right = right;

        node_idx
    }

    /// Cast a ray and return `true` if any triangle is hit within `t_max`.
    pub fn ray_cast(&self, origin: [f32; 3], direction: [f32; 3], t_max: f32) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        // Precompute inv direction (with sign handling)
        let rd_inv = [
            if direction[0].abs() > 1e-30 {
                1.0 / direction[0]
            } else {
                f32::INFINITY
            },
            if direction[1].abs() > 1e-30 {
                1.0 / direction[1]
            } else {
                f32::INFINITY
            },
            if direction[2].abs() > 1e-30 {
                1.0 / direction[2]
            } else {
                f32::INFINITY
            },
        ];

        // Iterative stack-based traversal
        let mut stack = [0u32; 64];
        let mut stack_top: usize = 1;
        stack[0] = 0;

        while stack_top > 0 {
            stack_top -= 1;
            let node_idx = stack[stack_top] as usize;
            if node_idx >= self.nodes.len() {
                continue;
            }
            let node = &self.nodes[node_idx];

            if !node.bounds.ray_intersect(origin, rd_inv, t_max) {
                continue;
            }

            if node.left == u32::MAX {
                // Leaf: test all triangles
                let tri_end = (node.tri_start + node.tri_count) as usize;
                for ti in node.tri_start as usize..tri_end.min(self.tris.len()) {
                    let tri = &self.tris[ti];
                    if let Some(t) = moller_trumbore(origin, direction, tri[0], tri[1], tri[2]) {
                        if t > 0.0 && t < t_max {
                            return true;
                        }
                    }
                }
            } else {
                // Internal node: push children
                if stack_top + 2 < stack.len() {
                    stack[stack_top] = node.left;
                    stack[stack_top + 1] = node.right;
                    stack_top += 2;
                }
            }
        }

        false
    }
}

// ── Möller–Trumbore ray-triangle intersection ─────────────────────────────────

/// Möller–Trumbore algorithm.  Returns the ray parameter `t` at the
/// intersection point, or `None` if the ray misses the triangle or is parallel.
fn moller_trumbore(
    origin: [f32; 3],
    direction: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let edge1 = sub3f(v1, v0);
    let edge2 = sub3f(v2, v0);

    let h = cross3f(direction, edge2);
    let det = dot3f(edge1, h);

    // Back-face and near-parallel culling
    if det.abs() < 1e-8 {
        return None;
    }

    let inv_det = 1.0 / det;
    let s = sub3f(origin, v0);
    let u = dot3f(s, h) * inv_det;

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = cross3f(s, edge1);
    let v = dot3f(direction, q) * inv_det;

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = dot3f(edge2, q) * inv_det;
    if t > 1e-6 {
        Some(t)
    } else {
        None
    }
}

// ── f32 math helpers ──────────────────────────────────────────────────────────

#[inline]
fn sub3f(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3f(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3f(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn normalize3f(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ── Backward-compat shims from old stub ──────────────────────────────────────

/// Legacy parameter struct.
pub struct AoParams {
    pub num_rays: u32,
    pub max_distance: f32,
    pub bias: f32,
}

/// Create legacy `AoParams`.
pub fn new_ao_params(num_rays: u32) -> AoParams {
    AoParams {
        num_rays,
        max_distance: 1.0,
        bias: 0.001,
    }
}

/// Legacy hemisphere sample (Hammersley).
pub fn ao_sample_hemisphere(normal: [f32; 3], sample_idx: u32, total: u32) -> [f32; 3] {
    let mut lcg = Lcg::new(sample_idx as u64);
    cosine_hemisphere_sample(normal, sample_idx, total, &mut lcg)
}

/// Legacy stub: always returns 1.0.
pub fn ao_estimate(_normal: [f32; 3], _num_rays: u32) -> f32 {
    1.0
}

/// Convert AO value to greyscale RGB.
pub fn ao_to_color(ao: f32) -> [f32; 3] {
    let v = ao.clamp(0.0, 1.0);
    [v, v, v]
}

/// Validate legacy params.
pub fn ao_params_is_valid(p: &AoParams) -> bool {
    p.num_rays > 0 && p.max_distance > 0.0 && p.bias >= 0.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple single-triangle mesh.
    fn single_triangle() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0]; 3];
        let indices = vec![0, 1, 2];
        (positions, normals, indices)
    }

    /// Build a simple quad mesh (two triangles on the XY plane).
    fn quad_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let indices = vec![0, 1, 2, 0, 2, 3];
        (positions, normals, indices)
    }

    #[test]
    fn ao_values_in_range() {
        let (positions, normals, indices) = quad_mesh();
        let mesh = MeshBuffers {
            positions: &positions,
            normals: &normals,
            indices: &indices,
        };
        let config = AoConfig {
            ray_count: 32,
            ..Default::default()
        };
        let ao = bake_ambient_occlusion(&mesh, &config);
        assert_eq!(ao.len(), positions.len());
        for &v in &ao {
            assert!((0.0..=1.0).contains(&v), "AO value {v} is outside [0,1]");
        }
    }

    #[test]
    fn ao_is_one_for_isolated_vertex() {
        // A mesh with a single triangle; a vertex far above it will see a clear
        // hemisphere if the only triangle is below.
        // We place an isolated point (not on any triangle face) at height 10.
        // Since our test only has one triangle near z=0 and the vertex is near z=10,
        // most rays should be unoccluded and AO near 1.0.
        let mut positions = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            // Isolated vertex far above
            [0.0, 0.0, 10.0],
        ];
        let mut normals = vec![[0.0, 0.0, 1.0]; 4];
        let indices = vec![0, 1, 2]; // only the bottom triangle

        let mesh = MeshBuffers {
            positions: &positions,
            normals: &normals,
            indices: &indices,
        };
        let config = AoConfig {
            ray_count: 64,
            max_distance: 5.0, // can't reach the bottom triangle from z=10
            bias: 1e-4,
        };
        let ao = bake_ambient_occlusion(&mesh, &config);
        // Vertex 3 (at z=10) should be fully unoccluded (all rays miss the triangle)
        assert!(
            ao[3] > 0.9,
            "isolated vertex at height 10 should have AO near 1.0, got {}",
            ao[3]
        );
        // Suppress unused-variable warnings
        let _ = (&mut positions, &mut normals);
    }

    #[test]
    fn ao_bvh_ray_cast_hit() {
        // A single triangle in front of the ray origin
        let positions = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
        let indices = vec![0, 1, 2];
        let bvh = AoBvh::build(&positions, &indices);
        // Ray from (0.1, 0.1, 0) pointing +Z should hit the triangle at z=1
        let hit = bvh.ray_cast([0.25, 0.25, 0.0], [0.0, 0.0, 1.0], 2.0);
        assert!(hit, "BVH should report a hit");
    }

    #[test]
    fn ao_bvh_ray_cast_miss() {
        let positions = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
        let indices = vec![0, 1, 2];
        let bvh = AoBvh::build(&positions, &indices);
        // Ray pointing away from the triangle
        let hit = bvh.ray_cast([0.25, 0.25, 0.0], [0.0, 0.0, -1.0], 2.0);
        assert!(!hit, "BVH should not report a hit for reversed ray");
    }

    #[test]
    fn moller_trumbore_hit() {
        let v0 = [0.0, 0.0, 1.0f32];
        let v1 = [1.0, 0.0, 1.0];
        let v2 = [0.0, 1.0, 1.0];
        let t = moller_trumbore([0.25, 0.25, 0.0], [0.0, 0.0, 1.0], v0, v1, v2);
        assert!(t.is_some());
        assert!((t.expect("should succeed") - 1.0).abs() < 1e-5, "t should be ≈ 1.0");
    }

    #[test]
    fn moller_trumbore_miss() {
        let v0 = [0.0, 0.0, 1.0f32];
        let v1 = [1.0, 0.0, 1.0];
        let v2 = [0.0, 1.0, 1.0];
        // Ray misses (far outside triangle)
        let t = moller_trumbore([5.0, 5.0, 0.0], [0.0, 0.0, 1.0], v0, v1, v2);
        assert!(t.is_none());
    }

    #[test]
    fn lcg_produces_values_in_range() {
        let mut lcg = Lcg::new(42);
        for _ in 0..1000 {
            let v = lcg.next_f32();
            assert!((0.0..1.0).contains(&v), "LCG out of range: {v}");
        }
    }

    #[test]
    fn bake_ao_empty_mesh_returns_empty() {
        let mesh = MeshBuffers {
            positions: &[],
            normals: &[],
            indices: &[],
        };
        let ao = bake_ambient_occlusion(&mesh, &AoConfig::default());
        assert!(ao.is_empty());
    }

    // ── Legacy stub tests ─────────────────────────────────────────────────────

    #[test]
    fn test_new_ao_params() {
        let p = new_ao_params(16);
        assert_eq!(p.num_rays, 16);
        assert!(ao_params_is_valid(&p));
    }

    #[test]
    fn test_ao_estimate_stub() {
        let v = ao_estimate([0.0, 1.0, 0.0], 32);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ao_to_color() {
        let c = ao_to_color(0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ao_sample_hemisphere_normalized() {
        let s = ao_sample_hemisphere([0.0, 1.0, 0.0], 0, 16);
        let len = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-3,
            "sample should be unit length, got {len}"
        );
    }

    #[test]
    fn test_ao_params_invalid() {
        let p = AoParams {
            num_rays: 0,
            max_distance: 1.0,
            bias: 0.0,
        };
        assert!(!ao_params_is_valid(&p));
    }

    #[test]
    fn single_triangle_ao_all_in_range() {
        let (positions, normals, indices) = single_triangle();
        let mesh = MeshBuffers {
            positions: &positions,
            normals: &normals,
            indices: &indices,
        };
        let config = AoConfig {
            ray_count: 16,
            ..Default::default()
        };
        let ao = bake_ambient_occlusion(&mesh, &config);
        assert_eq!(ao.len(), 3);
        for v in ao {
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
