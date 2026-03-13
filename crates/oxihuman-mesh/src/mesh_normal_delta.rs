// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Compute delta normals for blend shape / morph targets.
//!
//! In GLTF, morph targets can store not just position deltas but also normal
//! deltas. Without normal deltas, shading will be incorrect for large
//! deformations. This module provides types and functions to compute, store,
//! apply and serialise those delta normals.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

// ─── Math helpers ────────────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_len_sq(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[inline]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_len_sq(v).sqrt()
}

// ─── Public math utilities ────────────────────────────────────────────────────

/// Normalize a normal vector, safely returning `[0, 1, 0]` for zero-length input.
pub fn safe_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Check whether two normal arrays are approximately equal within `tol`.
///
/// Returns `false` if lengths differ.
pub fn normals_approx_equal(a: &[[f32; 3]], b: &[[f32; 3]], tol: f32) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (na, nb) in a.iter().zip(b.iter()) {
        let d = vec3_sub(*na, *nb);
        if vec3_len(d) > tol {
            return false;
        }
    }
    true
}

// ─── Vertex-normal computation ────────────────────────────────────────────────

/// Compute per-vertex normals from raw positions and indices.
///
/// Accumulated area-weighted face normals are normalised per vertex.
/// Returns `vertex_count` normals; vertices not referenced by any face receive
/// `[0, 1, 0]`.
pub fn compute_vertex_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex_count: usize,
) -> Vec<[f32; 3]> {
    let mut accum = vec![[0.0f32; 3]; vertex_count];

    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= vertex_count || i1 >= vertex_count || i2 >= vertex_count {
            continue;
        }
        let e1 = vec3_sub(positions[i1], positions[i0]);
        let e2 = vec3_sub(positions[i2], positions[i0]);
        let face_n = vec3_cross(e1, e2);
        accum[i0] = vec3_add(accum[i0], face_n);
        accum[i1] = vec3_add(accum[i1], face_n);
        accum[i2] = vec3_add(accum[i2], face_n);
    }

    accum.into_iter().map(safe_normalize).collect()
}

// ─── Core types ───────────────────────────────────────────────────────────────

/// Delta normal for a single vertex in one morph target.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalDelta {
    /// Index of the vertex this delta applies to.
    pub vertex_index: u32,
    /// Normal delta: `target_normal − base_normal`.
    pub dn: [f32; 3],
}

/// Full set of normal deltas for one morph target.
#[derive(Debug, Clone)]
pub struct MorphNormalDeltas {
    /// Human-readable name for the morph target (e.g. `"smile"`).
    pub target_name: String,
    /// Sparse list of per-vertex normal deltas.
    pub deltas: Vec<NormalDelta>,
    /// Total number of vertices in the mesh (determines flat buffer size).
    pub vertex_count: usize,
}

impl MorphNormalDeltas {
    /// Create an empty delta set for a morph target.
    pub fn new(target_name: &str, vertex_count: usize) -> Self {
        Self {
            target_name: target_name.to_owned(),
            deltas: Vec::new(),
            vertex_count,
        }
    }

    /// Append a single vertex normal delta.
    pub fn add(&mut self, vi: u32, dn: [f32; 3]) {
        self.deltas.push(NormalDelta {
            vertex_index: vi,
            dn,
        });
    }

    /// Count deltas whose magnitude exceeds `1e-6`.
    pub fn count_nonzero(&self) -> usize {
        self.deltas.iter().filter(|d| vec3_len(d.dn) > 1e-6).count()
    }

    /// Apply the delta normals to `base_normals` at the given blend `weight`.
    ///
    /// Returns a new normal array with the deltas added and each result
    /// re-normalised.
    pub fn apply(&self, base_normals: &[[f32; 3]], weight: f32) -> Vec<[f32; 3]> {
        let mut out = base_normals.to_vec();
        for d in &self.deltas {
            let vi = d.vertex_index as usize;
            if vi < out.len() {
                let scaled = vec3_scale(d.dn, weight);
                out[vi] = safe_normalize(vec3_add(out[vi], scaled));
            }
        }
        out
    }

    /// Remove deltas whose magnitude is at or below `threshold`.
    pub fn prune(&mut self, threshold: f32) {
        self.deltas.retain(|d| vec3_len(d.dn) > threshold);
    }

    /// Serialise as a flat `Vec<f32>` with `3 * vertex_count` elements.
    ///
    /// Vertices not present in `deltas` are represented as `[0, 0, 0]`.
    pub fn to_flat(&self) -> Vec<f32> {
        let mut out = vec![0.0f32; self.vertex_count * 3];
        for d in &self.deltas {
            let vi = d.vertex_index as usize;
            if vi < self.vertex_count {
                let base = vi * 3;
                out[base] += d.dn[0];
                out[base + 1] += d.dn[1];
                out[base + 2] += d.dn[2];
            }
        }
        out
    }

    /// Deserialise from a flat buffer produced by [`MorphNormalDeltas::to_flat`].
    ///
    /// Only non-zero entries are stored as sparse deltas.
    pub fn from_flat(name: &str, data: &[f32]) -> Self {
        let vertex_count = data.len() / 3;
        let mut out = Self::new(name, vertex_count);
        for (vi, chunk) in data.chunks_exact(3).enumerate() {
            let dn = [chunk[0], chunk[1], chunk[2]];
            if vec3_len(dn) > 1e-10 {
                out.add(vi as u32, dn);
            }
        }
        out
    }
}

// ─── Compute deltas ───────────────────────────────────────────────────────────

/// Compute delta normals between `base_mesh` and a fully-applied `morph_mesh`.
///
/// The morph mesh is the mesh after the morph target is applied at weight 1.
/// Normals are re-computed from positions rather than taken from the stored
/// normal buffer so that the delta is geometric and not affected by stale data.
pub fn compute_normal_deltas(
    base_mesh: &MeshBuffers,
    morph_mesh: &MeshBuffers,
    target_name: &str,
) -> MorphNormalDeltas {
    let vertex_count = base_mesh.positions.len();
    let base_normals =
        compute_vertex_normals(&base_mesh.positions, &base_mesh.indices, vertex_count);
    let morph_normals = compute_vertex_normals(
        &morph_mesh.positions,
        &morph_mesh.indices,
        morph_mesh.positions.len(),
    );

    let mut result = MorphNormalDeltas::new(target_name, vertex_count);
    let cmp_len = vertex_count.min(morph_normals.len());
    for vi in 0..cmp_len {
        let dn = vec3_sub(morph_normals[vi], base_normals[vi]);
        if vec3_len(dn) > 1e-7 {
            result.add(vi as u32, dn);
        }
    }
    result
}

/// Compute delta normals for multiple morph targets in a single pass.
///
/// Each entry in `morph_meshes` is `(morphed_mesh, target_name)`.
pub fn compute_batch_normal_deltas(
    base_mesh: &MeshBuffers,
    morph_meshes: &[(&MeshBuffers, &str)],
) -> Vec<MorphNormalDeltas> {
    morph_meshes
        .iter()
        .map(|(morph, name)| compute_normal_deltas(base_mesh, morph, name))
        .collect()
}

// ─── Apply helpers ────────────────────────────────────────────────────────────

/// Blend multiple morph normal deltas simultaneously.
///
/// Each tuple in `deltas` is `(delta_set, weight)`. All deltas are accumulated
/// linearly and the result is re-normalised per vertex.
pub fn apply_morph_normals(
    base_normals: &[[f32; 3]],
    deltas: &[(&MorphNormalDeltas, f32)],
) -> Vec<[f32; 3]> {
    let n = base_normals.len();
    // Accumulate raw (unnormalised) normal contributions.
    let mut accum: Vec<[f32; 3]> = base_normals.to_vec();
    for (delta_set, weight) in deltas {
        for d in &delta_set.deltas {
            let vi = d.vertex_index as usize;
            if vi < n {
                let contribution = vec3_scale(d.dn, *weight);
                accum[vi] = vec3_add(accum[vi], contribution);
            }
        }
    }
    accum.into_iter().map(safe_normalize).collect()
}

// ─── Tangent-space conversion ─────────────────────────────────────────────────

/// Convert a world-space normal delta to tangent space.
///
/// `normal`, `tangent`, and `bitangent` form the TBN basis at the vertex.
/// All three basis vectors must already be normalised.
///
/// Returns a tangent-space `[dx, dy, dz]` triple.
#[allow(clippy::too_many_arguments)]
pub fn to_tangent_space_delta(
    dn_world: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
) -> [f32; 3] {
    [
        vec3_dot(dn_world, tangent),
        vec3_dot(dn_world, bitangent),
        vec3_dot(dn_world, normal),
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn flat_triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    /// A triangle mesh with two faces sharing an edge (a simple quad split).
    fn two_triangle_mesh() -> MeshBuffers {
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

    /// Create a mesh where all vertices are raised on Z (morphed up).
    fn raised_mesh(base: &MeshBuffers, dz: f32) -> MeshBuffers {
        let positions: Vec<[f32; 3]> = base
            .positions
            .iter()
            .map(|p| [p[0], p[1], p[2] + dz])
            .collect();
        MeshBuffers::from_morph(MB {
            positions,
            normals: base.normals.clone(),
            uvs: base.uvs.clone(),
            indices: base.indices.clone(),
            has_suit: false,
        })
    }

    // ── 1. safe_normalize ─────────────────────────────────────────────────────

    #[test]
    fn safe_normalize_unit_vector_unchanged() {
        let v = [0.0f32, 0.0, 1.0];
        let n = safe_normalize(v);
        assert!((n[2] - 1.0).abs() < 1e-6, "expected [0,0,1], got {:?}", n);
    }

    #[test]
    fn safe_normalize_zero_returns_up() {
        let n = safe_normalize([0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 1.0, 0.0]);
    }

    // ── 2. NormalDelta / MorphNormalDeltas ───────────────────────────────────

    #[test]
    fn morph_normal_deltas_new_empty() {
        let mnd = MorphNormalDeltas::new("test", 100);
        assert_eq!(mnd.vertex_count, 100);
        assert_eq!(mnd.deltas.len(), 0);
        assert_eq!(mnd.target_name, "test");
    }

    #[test]
    fn morph_normal_deltas_add_and_count() {
        let mut mnd = MorphNormalDeltas::new("smile", 10);
        mnd.add(0, [0.1, 0.0, 0.0]);
        mnd.add(1, [0.0, 0.1, 0.0]);
        mnd.add(2, [0.0, 0.0, 0.0]); // zero delta
        assert_eq!(mnd.deltas.len(), 3);
        assert_eq!(mnd.count_nonzero(), 2);
    }

    #[test]
    fn morph_normal_deltas_prune() {
        let mut mnd = MorphNormalDeltas::new("t", 5);
        mnd.add(0, [1.0, 0.0, 0.0]);
        mnd.add(1, [0.000_000_001, 0.0, 0.0]); // below threshold
        mnd.add(2, [0.5, 0.5, 0.0]);
        mnd.prune(1e-3);
        assert_eq!(mnd.deltas.len(), 2);
    }

    // ── 3. to_flat / from_flat round-trip ─────────────────────────────────────

    #[test]
    fn flat_roundtrip_preserves_deltas() {
        let mut mnd = MorphNormalDeltas::new("rt", 6);
        mnd.add(0, [0.1, 0.2, 0.3]);
        mnd.add(3, [0.4, 0.5, 0.6]);
        let flat = mnd.to_flat();
        assert_eq!(flat.len(), 6 * 3);

        let back = MorphNormalDeltas::from_flat("rt", &flat);
        assert_eq!(back.vertex_count, 6);
        assert_eq!(back.count_nonzero(), 2);

        // Check specific values
        let d0 = back.deltas.iter().find(|d| d.vertex_index == 0).expect("should succeed");
        assert!((d0.dn[0] - 0.1).abs() < 1e-5);
        assert!((d0.dn[1] - 0.2).abs() < 1e-5);
    }

    #[test]
    fn to_flat_size_is_three_times_vertex_count() {
        let mut mnd = MorphNormalDeltas::new("size_test", 50);
        mnd.add(7, [0.0, 0.0, 1.0]);
        let flat = mnd.to_flat();
        assert_eq!(flat.len(), 150);
    }

    // ── 4. apply ─────────────────────────────────────────────────────────────

    #[test]
    fn apply_weight_zero_leaves_normals_unchanged() {
        let base = vec![[0.0f32, 0.0, 1.0], [0.0, 1.0, 0.0]];
        let mut mnd = MorphNormalDeltas::new("w0", 2);
        mnd.add(0, [0.5, 0.0, 0.0]);
        let out = mnd.apply(&base, 0.0);
        // weight=0 → dn scaled to zero → normalise(base_normal + [0,0,0]) = base_normal
        assert!(normals_approx_equal(&out, &base, 1e-5));
    }

    #[test]
    fn apply_weight_one_adds_delta() {
        let base = vec![[0.0f32, 0.0, 1.0]];
        let mut mnd = MorphNormalDeltas::new("w1", 1);
        // Add a delta that would tilt the normal toward +X
        mnd.add(0, [0.5, 0.0, 0.0]);
        let out = mnd.apply(&base, 1.0);
        // Result should be normalised([0.5, 0, 1.0])
        let expected = safe_normalize([0.5, 0.0, 1.0]);
        assert!((out[0][0] - expected[0]).abs() < 1e-5);
        assert!((out[0][2] - expected[2]).abs() < 1e-5);
    }

    // ── 5. compute_normal_deltas ──────────────────────────────────────────────

    #[test]
    fn compute_normal_deltas_identical_meshes_yields_zero() {
        let base = flat_triangle_mesh();
        let morph = base.clone();
        let deltas = compute_normal_deltas(&base, &morph, "identity");
        assert_eq!(deltas.count_nonzero(), 0);
    }

    #[test]
    fn compute_normal_deltas_detects_change() {
        let base = two_triangle_mesh();
        // Tilt one vertex to change face normals
        let mut morph_positions = base.positions.clone();
        morph_positions[0][2] = 0.5;
        let morph = MeshBuffers::from_morph(MB {
            positions: morph_positions,
            normals: base.normals.clone(),
            uvs: base.uvs.clone(),
            indices: base.indices.clone(),
            has_suit: false,
        });
        let deltas = compute_normal_deltas(&base, &morph, "tilt");
        // Tilting a shared vertex must change at least one normal
        assert!(
            deltas.count_nonzero() > 0,
            "expected non-zero deltas after tilt"
        );
    }

    // ── 6. compute_batch_normal_deltas ────────────────────────────────────────

    #[test]
    fn batch_same_as_individual() {
        let base = two_triangle_mesh();
        let morph1 = raised_mesh(&base, 0.1);
        let morph2 = raised_mesh(&base, 0.3);

        let batch = compute_batch_normal_deltas(&base, &[(&morph1, "a"), (&morph2, "b")]);
        let single_a = compute_normal_deltas(&base, &morph1, "a");
        let single_b = compute_normal_deltas(&base, &morph2, "b");

        assert_eq!(batch[0].count_nonzero(), single_a.count_nonzero());
        assert_eq!(batch[1].count_nonzero(), single_b.count_nonzero());
    }

    // ── 7. apply_morph_normals ────────────────────────────────────────────────

    #[test]
    fn apply_morph_normals_no_deltas_unchanged() {
        let base_normals = vec![[0.0f32, 0.0, 1.0]; 4];
        let empty = MorphNormalDeltas::new("empty", 4);
        let out = apply_morph_normals(&base_normals, &[(&empty, 1.0)]);
        assert!(normals_approx_equal(&out, &base_normals, 1e-5));
    }

    #[test]
    fn apply_morph_normals_two_deltas_blended() {
        let base_normals = vec![[0.0f32, 0.0, 1.0]];

        let mut d1 = MorphNormalDeltas::new("d1", 1);
        d1.add(0, [0.2, 0.0, 0.0]);

        let mut d2 = MorphNormalDeltas::new("d2", 1);
        d2.add(0, [0.0, 0.2, 0.0]);

        let out = apply_morph_normals(&base_normals, &[(&d1, 0.5), (&d2, 0.5)]);
        // Result should be safe_normalize([0.0 + 0.1 + 0.0, 0.0 + 0.0 + 0.1, 1.0])
        let expected = safe_normalize([0.1, 0.1, 1.0]);
        assert!((out[0][0] - expected[0]).abs() < 1e-5, "X mismatch");
        assert!((out[0][1] - expected[1]).abs() < 1e-5, "Y mismatch");
    }

    // ── 8. to_tangent_space_delta ─────────────────────────────────────────────

    #[test]
    fn tangent_space_delta_identity_basis() {
        // When TBN = identity, world delta == tangent-space delta.
        let dn = [0.1f32, 0.2, 0.3];
        let n = [0.0f32, 0.0, 1.0]; // Z = normal
        let t = [1.0f32, 0.0, 0.0]; // X = tangent
        let b = [0.0f32, 1.0, 0.0]; // Y = bitangent
        let ts = to_tangent_space_delta(dn, n, t, b);
        assert!((ts[0] - 0.1).abs() < 1e-6); // dot(dn, T) = 0.1
        assert!((ts[1] - 0.2).abs() < 1e-6); // dot(dn, B) = 0.2
        assert!((ts[2] - 0.3).abs() < 1e-6); // dot(dn, N) = 0.3
    }

    // ── 9. compute_vertex_normals ─────────────────────────────────────────────

    #[test]
    fn compute_vertex_normals_flat_triangle_points_z() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let normals = compute_vertex_normals(&positions, &indices, 3);
        for n in &normals {
            assert!(n[2] > 0.9, "expected +Z, got {:?}", n);
        }
    }

    #[test]
    fn compute_vertex_normals_output_len_equals_vertex_count() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let normals = compute_vertex_normals(&positions, &indices, 5);
        assert_eq!(normals.len(), 5);
    }

    // ── 10. normals_approx_equal ──────────────────────────────────────────────

    #[test]
    fn normals_approx_equal_length_mismatch_false() {
        let a = vec![[0.0f32, 0.0, 1.0]; 3];
        let b = vec![[0.0f32, 0.0, 1.0]; 2];
        assert!(!normals_approx_equal(&a, &b, 1e-4));
    }

    #[test]
    fn normals_approx_equal_exact_match() {
        let a = vec![[0.0f32, 0.0, 1.0], [0.0, 1.0, 0.0]];
        let b = a.clone();
        assert!(normals_approx_equal(&a, &b, 0.0));
    }

    // ── Extra: write flat buffer to /tmp/ ─────────────────────────────────────

    #[test]
    fn to_flat_written_to_tmp() {
        let mut mnd = MorphNormalDeltas::new("tmp_test", 4);
        mnd.add(1, [0.1, 0.2, 0.3]);
        let flat = mnd.to_flat();
        let bytes: Vec<u8> = flat.iter().flat_map(|f| f.to_le_bytes()).collect();
        std::fs::write("/tmp/normal_delta_flat.bin", &bytes).expect("should write to /tmp/");
        let read_bytes = std::fs::read("/tmp/normal_delta_flat.bin").expect("should succeed");
        assert_eq!(read_bytes.len(), 4 * 3 * 4); // 4 verts * 3 floats * 4 bytes
    }
}
