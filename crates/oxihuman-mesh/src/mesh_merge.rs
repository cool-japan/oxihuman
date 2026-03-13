// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Parameters controlling merge behaviour.
pub struct MergeParams {
    /// Deduplicate shared vertices (default false).
    pub weld_vertices: bool,
    /// Distance threshold for welding (default 1e-5).
    pub weld_epsilon: f32,
    /// Recompute normals after merge (default true).
    pub recompute_normals: bool,
}

impl Default for MergeParams {
    fn default() -> Self {
        Self {
            weld_vertices: false,
            weld_epsilon: 1e-5,
            recompute_normals: true,
        }
    }
}

/// Result returned by [`merge_with_params`].
pub struct MergeResult {
    /// The combined mesh.
    pub mesh: MeshBuffers,
    /// Vertex offset for each input mesh in the output vertex array.
    pub mesh_offsets: Vec<usize>,
    /// Total vertices across all input meshes (before welding).
    pub total_input_vertices: usize,
    /// Vertices in the output mesh (after optional welding).
    pub total_output_vertices: usize,
    /// How many vertices were removed by welding.
    pub vertices_welded: usize,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn make_empty_mesh() -> MeshBuffers {
    MeshBuffers {
        positions: Vec::new(),
        normals: Vec::new(),
        tangents: Vec::new(),
        uvs: Vec::new(),
        indices: Vec::new(),
        colors: None,
        has_suit: false,
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
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

/// Apply Rodrigues rotation to a single vector.
fn rodrigues(v: [f32; 3], axis: [f32; 3], sin_a: f32, cos_a: f32) -> [f32; 3] {
    let k = normalize3(axis);
    let kxv = cross3(k, v);
    let kdv = dot3(k, v);
    [
        v[0] * cos_a + kxv[0] * sin_a + k[0] * kdv * (1.0 - cos_a),
        v[1] * cos_a + kxv[1] * sin_a + k[1] * kdv * (1.0 - cos_a),
        v[2] * cos_a + kxv[2] * sin_a + k[2] * kdv * (1.0 - cos_a),
    ]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Merge two meshes into one (simple concatenation).
pub fn merge_two(a: &MeshBuffers, b: &MeshBuffers) -> MeshBuffers {
    let offset = a.positions.len() as u32;

    let mut positions = a.positions.clone();
    positions.extend_from_slice(&b.positions);

    let mut normals = a.normals.clone();
    normals.extend_from_slice(&b.normals);

    let mut tangents = a.tangents.clone();
    tangents.extend_from_slice(&b.tangents);

    let mut uvs = a.uvs.clone();
    uvs.extend_from_slice(&b.uvs);

    let mut indices = a.indices.clone();
    for &idx in &b.indices {
        indices.push(idx + offset);
    }

    // Merge optional color arrays.
    let colors = match (&a.colors, &b.colors) {
        (Some(ca), Some(cb)) => {
            let mut c = ca.clone();
            c.extend_from_slice(cb);
            Some(c)
        }
        (Some(ca), None) => {
            let mut c = ca.clone();
            c.extend(std::iter::repeat_n(
                [1.0f32, 1.0, 1.0, 1.0],
                b.positions.len(),
            ));
            Some(c)
        }
        (None, Some(cb)) => {
            let mut c: Vec<[f32; 4]> =
                std::iter::repeat_n([1.0f32, 1.0, 1.0, 1.0], a.positions.len()).collect();
            c.extend_from_slice(cb);
            Some(c)
        }
        (None, None) => None,
    };

    let mut result = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        colors,
        has_suit: a.has_suit || b.has_suit,
    };

    if result.normals.is_empty() && !result.positions.is_empty() {
        compute_normals(&mut result);
    }

    result
}

/// Merge a slice of meshes (simple concatenation, no welding).
pub fn merge_many(meshes: &[MeshBuffers]) -> MeshBuffers {
    if meshes.is_empty() {
        return make_empty_mesh();
    }
    let mut result = meshes[0].clone();
    for other in &meshes[1..] {
        result = merge_two(&result, other);
    }
    result
}

/// Concatenate mesh `other` into `base` in place.
pub fn append_mesh(base: &mut MeshBuffers, other: &MeshBuffers) {
    let merged = merge_two(base, other);
    *base = merged;
}

/// Merge with options: optional vertex welding and normal recomputation.
pub fn merge_with_params(meshes: &[MeshBuffers], params: &MergeParams) -> MergeResult {
    let mut mesh_offsets: Vec<usize> = Vec::with_capacity(meshes.len());
    let mut total_input_vertices: usize = 0;

    // Build merged mesh and record offsets.
    let mut merged = make_empty_mesh();
    for m in meshes {
        mesh_offsets.push(merged.positions.len());
        total_input_vertices += m.positions.len();
        merged = merge_two(&merged, m);
    }

    let vertices_welded = if params.weld_vertices && !merged.positions.is_empty() {
        let before = merged.positions.len();
        merged = weld_vertices(&merged, params.weld_epsilon);
        before - merged.positions.len()
    } else {
        0
    };

    if params.recompute_normals && !merged.positions.is_empty() {
        compute_normals(&mut merged);
    }

    let total_output_vertices = merged.positions.len();

    MergeResult {
        mesh: merged,
        mesh_offsets,
        total_input_vertices,
        total_output_vertices,
        vertices_welded,
    }
}

/// Brute-force vertex welding by proximity.
fn weld_vertices(mesh: &MeshBuffers, epsilon: f32) -> MeshBuffers {
    let n = mesh.positions.len();
    let eps2 = epsilon * epsilon;
    // map[i] = canonical index for vertex i
    let mut map: Vec<usize> = (0..n).collect();

    for i in 0..n {
        for j in 0..i {
            if map[j] == j {
                let dx = mesh.positions[i][0] - mesh.positions[j][0];
                let dy = mesh.positions[i][1] - mesh.positions[j][1];
                let dz = mesh.positions[i][2] - mesh.positions[j][2];
                if dx * dx + dy * dy + dz * dz < eps2 {
                    map[i] = map[j];
                    break;
                }
            }
        }
    }

    // Compact: assign new indices
    let mut new_index: Vec<Option<usize>> = vec![None; n];
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_tangents: Vec<[f32; 4]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut remap: Vec<usize> = vec![0; n];

    for i in 0..n {
        let canon = map[i];
        if new_index[canon].is_none() {
            new_index[canon] = Some(new_positions.len());
            new_positions.push(mesh.positions[canon]);
            if canon < mesh.normals.len() {
                new_normals.push(mesh.normals[canon]);
            }
            if canon < mesh.tangents.len() {
                new_tangents.push(mesh.tangents[canon]);
            }
            if canon < mesh.uvs.len() {
                new_uvs.push(mesh.uvs[canon]);
            }
        }
        remap[i] = new_index[canon].unwrap_or(0);
    }

    let new_indices: Vec<u32> = mesh
        .indices
        .iter()
        .map(|&idx| remap[idx as usize] as u32)
        .collect();

    MeshBuffers {
        positions: new_positions,
        normals: new_normals,
        tangents: new_tangents,
        uvs: new_uvs,
        indices: new_indices,
        colors: None,
        has_suit: mesh.has_suit,
    }
}

/// Split a mesh into connected components using union-find on vertex adjacency.
pub fn split_by_connectivity(mesh: &MeshBuffers) -> Vec<MeshBuffers> {
    let n = mesh.positions.len();
    if n == 0 {
        return Vec::new();
    }

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }

    for tri in mesh.indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        union(&mut parent, v0, v1);
        union(&mut parent, v0, v2);
    }

    // Normalise roots
    for i in 0..n {
        parent[i] = find(&mut parent, i);
    }

    // Group face indices by root
    use std::collections::HashMap;
    let mut component_faces: HashMap<usize, Vec<u32>> = HashMap::new();
    for tri in mesh.indices.chunks_exact(3) {
        let root = parent[tri[0] as usize];
        component_faces
            .entry(root)
            .or_default()
            .extend_from_slice(tri);
    }

    // For each component build a sub-mesh with compacted vertices
    let mut results: Vec<MeshBuffers> = Vec::new();
    let mut keys: Vec<usize> = component_faces.keys().copied().collect();
    keys.sort_unstable();

    for root in keys {
        let face_indices = &component_faces[&root];
        let mut old_to_new: HashMap<u32, u32> = HashMap::new();
        let mut new_pos: Vec<[f32; 3]> = Vec::new();
        let mut new_nor: Vec<[f32; 3]> = Vec::new();
        let mut new_tan: Vec<[f32; 4]> = Vec::new();
        let mut new_uvs: Vec<[f32; 2]> = Vec::new();
        let mut new_idx: Vec<u32> = Vec::new();

        for &old in face_indices {
            let new_v = *old_to_new.entry(old).or_insert_with(|| {
                let ni = new_pos.len() as u32;
                new_pos.push(mesh.positions[old as usize]);
                if (old as usize) < mesh.normals.len() {
                    new_nor.push(mesh.normals[old as usize]);
                }
                if (old as usize) < mesh.tangents.len() {
                    new_tan.push(mesh.tangents[old as usize]);
                }
                if (old as usize) < mesh.uvs.len() {
                    new_uvs.push(mesh.uvs[old as usize]);
                }
                ni
            });
            new_idx.push(new_v);
        }

        results.push(MeshBuffers {
            positions: new_pos,
            normals: new_nor,
            tangents: new_tan,
            uvs: new_uvs,
            indices: new_idx,
            colors: None,
            has_suit: mesh.has_suit,
        });
    }

    results
}

/// Extract a sub-mesh from a contiguous face range.
pub fn extract_face_range(mesh: &MeshBuffers, face_start: usize, face_count: usize) -> MeshBuffers {
    let idx_start = face_start * 3;
    let idx_end = (face_start + face_count) * 3;
    let idx_end = idx_end.min(mesh.indices.len());

    if idx_start >= mesh.indices.len() {
        return make_empty_mesh();
    }

    let face_indices = &mesh.indices[idx_start..idx_end];

    use std::collections::HashMap;
    let mut old_to_new: HashMap<u32, u32> = HashMap::new();
    let mut new_pos: Vec<[f32; 3]> = Vec::new();
    let mut new_nor: Vec<[f32; 3]> = Vec::new();
    let mut new_tan: Vec<[f32; 4]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_idx: Vec<u32> = Vec::new();

    for &old in face_indices {
        let new_v = *old_to_new.entry(old).or_insert_with(|| {
            let ni = new_pos.len() as u32;
            new_pos.push(mesh.positions[old as usize]);
            if (old as usize) < mesh.normals.len() {
                new_nor.push(mesh.normals[old as usize]);
            }
            if (old as usize) < mesh.tangents.len() {
                new_tan.push(mesh.tangents[old as usize]);
            }
            if (old as usize) < mesh.uvs.len() {
                new_uvs.push(mesh.uvs[old as usize]);
            }
            ni
        });
        new_idx.push(new_v);
    }

    MeshBuffers {
        positions: new_pos,
        normals: new_nor,
        tangents: new_tan,
        uvs: new_uvs,
        indices: new_idx,
        colors: None,
        has_suit: mesh.has_suit,
    }
}

/// Filter faces by predicate; returns a new mesh with only matching faces
/// and a compacted vertex array.
pub fn filter_faces(mesh: &MeshBuffers, keep: impl Fn(usize, [u32; 3]) -> bool) -> MeshBuffers {
    use std::collections::HashMap;
    let mut old_to_new: HashMap<u32, u32> = HashMap::new();
    let mut new_pos: Vec<[f32; 3]> = Vec::new();
    let mut new_nor: Vec<[f32; 3]> = Vec::new();
    let mut new_tan: Vec<[f32; 4]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_idx: Vec<u32> = Vec::new();

    for (fi, tri) in mesh.indices.chunks_exact(3).enumerate() {
        let face = [tri[0], tri[1], tri[2]];
        if !keep(fi, face) {
            continue;
        }
        for &old in &face {
            let new_v = *old_to_new.entry(old).or_insert_with(|| {
                let ni = new_pos.len() as u32;
                new_pos.push(mesh.positions[old as usize]);
                if (old as usize) < mesh.normals.len() {
                    new_nor.push(mesh.normals[old as usize]);
                }
                if (old as usize) < mesh.tangents.len() {
                    new_tan.push(mesh.tangents[old as usize]);
                }
                if (old as usize) < mesh.uvs.len() {
                    new_uvs.push(mesh.uvs[old as usize]);
                }
                ni
            });
            new_idx.push(new_v);
        }
    }

    MeshBuffers {
        positions: new_pos,
        normals: new_nor,
        tangents: new_tan,
        uvs: new_uvs,
        indices: new_idx,
        colors: None,
        has_suit: mesh.has_suit,
    }
}

/// Translate all vertices by `offset`, then recompute normals.
pub fn translate_mesh(mesh: &MeshBuffers, offset: [f32; 3]) -> MeshBuffers {
    let mut out = mesh.clone();
    for p in &mut out.positions {
        p[0] += offset[0];
        p[1] += offset[1];
        p[2] += offset[2];
    }
    compute_normals(&mut out);
    out
}

/// Scale all vertices by per-axis factors, then recompute normals.
pub fn scale_mesh(mesh: &MeshBuffers, scale: [f32; 3]) -> MeshBuffers {
    let mut out = mesh.clone();
    for p in &mut out.positions {
        p[0] *= scale[0];
        p[1] *= scale[1];
        p[2] *= scale[2];
    }
    compute_normals(&mut out);
    out
}

/// Rotate all vertices around origin by axis-angle (angle in radians).
pub fn rotate_mesh(mesh: &MeshBuffers, axis: [f32; 3], angle: f32) -> MeshBuffers {
    let mut out = mesh.clone();
    let sin_a = angle.sin();
    let cos_a = angle.cos();
    for p in &mut out.positions {
        *p = rodrigues(*p, axis, sin_a, cos_a);
    }
    for n in &mut out.normals {
        *n = rodrigues(*n, axis, sin_a, cos_a);
    }
    out
}

// ---------------------------------------------------------------------------
// New unified merge API (spec: mesh_merge second pass)
// ---------------------------------------------------------------------------

/// Configuration for the unified mesh merge operation.
pub struct MergeConfig {
    /// Threshold for welding nearby vertices (0.0 disables welding).
    pub weld_threshold: f32,
    /// Whether to recalculate normals after merging.
    pub recalculate_normals: bool,
    /// Whether to merge material slots.
    pub merge_materials: bool,
}

/// A named mesh input for the unified merge API.
pub struct MergeInput {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Vertex normals (may be empty).
    pub normals: Vec<[f32; 3]>,
    /// Triangle index triples.
    pub triangles: Vec<[u32; 3]>,
    /// Human-readable mesh name.
    pub name: String,
}

/// Result of merging multiple `MergeInput` objects together.
pub struct UnifiedMergeResult {
    /// All merged vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// All merged normals (may be empty if inputs had none).
    pub normals: Vec<[f32; 3]>,
    /// All merged triangles (indices into the merged `positions` array).
    pub triangles: Vec<[u32; 3]>,
    /// Number of input meshes that were merged.
    pub input_count: usize,
}

/// Build a `MergeConfig` with sensible defaults.
#[allow(dead_code)]
pub fn default_merge_config() -> MergeConfig {
    MergeConfig {
        weld_threshold: 0.0,
        recalculate_normals: true,
        merge_materials: false,
    }
}

/// Construct a `MergeInput` with positions and triangles.  Normals are left empty.
#[allow(dead_code)]
pub fn new_merge_input(
    positions: Vec<[f32; 3]>,
    triangles: Vec<[u32; 3]>,
    name: &str,
) -> MergeInput {
    MergeInput {
        positions,
        normals: Vec::new(),
        triangles,
        name: name.to_string(),
    }
}

/// Merge a slice of `MergeInput` meshes into one `UnifiedMergeResult`.
#[allow(dead_code)]
pub fn merge_meshes(inputs: &[MergeInput], _cfg: &MergeConfig) -> UnifiedMergeResult {
    let mut out_pos: Vec<[f32; 3]> = Vec::new();
    let mut out_nor: Vec<[f32; 3]> = Vec::new();
    let mut out_tri: Vec<[u32; 3]> = Vec::new();

    for inp in inputs {
        let base = out_pos.len() as u32;
        out_pos.extend_from_slice(&inp.positions);
        out_nor.extend_from_slice(&inp.normals);
        for &t in &inp.triangles {
            out_tri.push([t[0] + base, t[1] + base, t[2] + base]);
        }
    }

    UnifiedMergeResult {
        positions: out_pos,
        normals: out_nor,
        triangles: out_tri,
        input_count: inputs.len(),
    }
}

/// Number of vertices in a `UnifiedMergeResult`.
#[allow(dead_code)]
pub fn merge_result_vertex_count(r: &UnifiedMergeResult) -> usize {
    r.positions.len()
}

/// Number of triangles in a `UnifiedMergeResult`.
#[allow(dead_code)]
pub fn merge_result_face_count(r: &UnifiedMergeResult) -> usize {
    r.triangles.len()
}

/// Serialize a `UnifiedMergeResult` to a compact JSON string.
#[allow(dead_code)]
pub fn merge_result_to_json(r: &UnifiedMergeResult) -> String {
    format!(
        "{{\"input_count\":{},\"vertices\":{},\"triangles\":{}}}",
        r.input_count,
        r.positions.len(),
        r.triangles.len(),
    )
}

/// Basic validity check: all triangle indices must be within bounds.
#[allow(dead_code)]
pub fn validate_merge_result(r: &UnifiedMergeResult) -> bool {
    let n = r.positions.len() as u32;
    r.triangles.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute the axis-aligned bounding box `[min, max]` of a `UnifiedMergeResult`.
#[allow(dead_code)]
pub fn merge_bounding_box(r: &UnifiedMergeResult) -> [[f32; 3]; 2] {
    if r.positions.is_empty() {
        return [[0.0; 3]; 2];
    }
    let mut mn = r.positions[0];
    let mut mx = r.positions[0];
    for &p in &r.positions {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mn[2] = mn[2].min(p[2]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
        mx[2] = mx[2].max(p[2]);
    }
    [mn, mx]
}

/// Total vertex count across all inputs (before merging).
#[allow(dead_code)]
pub fn merge_input_total_vertices(inputs: &[MergeInput]) -> usize {
    inputs.iter().map(|i| i.positions.len()).sum()
}

/// Total triangle count across all inputs (before merging).
#[allow(dead_code)]
pub fn merge_input_total_faces(inputs: &[MergeInput]) -> usize {
    inputs.iter().map(|i| i.triangles.len()).sum()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        let normals = vec![[0.0f32, 0.0, 1.0]; n];
        let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; n];
        let uvs = vec![[0.0f32, 0.0]; n];
        MeshBuffers {
            positions,
            normals,
            tangents,
            uvs,
            indices,
            colors: None,
            has_suit: false,
        }
    }

    fn simple_tri() -> MeshBuffers {
        tri_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    fn simple_tri2() -> MeshBuffers {
        tri_mesh(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn test_merge_two_basic() {
        let a = simple_tri();
        let b = simple_tri2();
        let m = merge_two(&a, &b);
        assert_eq!(m.positions.len(), 6);
        assert_eq!(m.indices.len(), 6);
    }

    #[test]
    fn test_merge_two_index_offset() {
        let a = simple_tri();
        let b = simple_tri2();
        let m = merge_two(&a, &b);
        // b's first index should be offset by 3 (a's vertex count)
        assert_eq!(m.indices[3], 3);
        assert_eq!(m.indices[4], 4);
        assert_eq!(m.indices[5], 5);
    }

    #[test]
    fn test_merge_many_empty() {
        let m = merge_many(&[]);
        assert_eq!(m.positions.len(), 0);
        assert_eq!(m.indices.len(), 0);
    }

    #[test]
    fn test_merge_many_single() {
        let a = simple_tri();
        let m = merge_many(std::slice::from_ref(&a));
        assert_eq!(m.positions.len(), a.positions.len());
        assert_eq!(m.indices.len(), a.indices.len());
    }

    #[test]
    fn test_append_mesh() {
        let a = simple_tri();
        let b = simple_tri2();
        let mut base = a.clone();
        append_mesh(&mut base, &b);
        assert_eq!(base.positions.len(), 6);
        assert_eq!(base.indices.len(), 6);
    }

    #[test]
    fn test_extract_face_range() {
        let a = simple_tri();
        let b = simple_tri2();
        let merged = merge_two(&a, &b);
        // 2 faces total; extract second face only
        let sub = extract_face_range(&merged, 1, 1);
        assert_eq!(sub.positions.len(), 3);
        assert_eq!(sub.indices.len(), 3);
    }

    #[test]
    fn test_filter_faces() {
        let a = simple_tri();
        let b = simple_tri2();
        let merged = merge_two(&a, &b);
        // keep only face index 0
        let filtered = filter_faces(&merged, |fi, _| fi == 0);
        assert_eq!(filtered.positions.len(), 3);
        assert_eq!(filtered.indices.len(), 3);
    }

    #[test]
    fn test_translate_mesh() {
        let a = simple_tri();
        let t = translate_mesh(&a, [1.0, 2.0, 3.0]);
        assert!((t.positions[0][0] - 1.0).abs() < 1e-6);
        assert!((t.positions[0][1] - 2.0).abs() < 1e-6);
        assert!((t.positions[0][2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_mesh() {
        let a = simple_tri();
        let s = scale_mesh(&a, [2.0, 3.0, 4.0]);
        // vertex 1 is [1,0,0] → [2,0,0]
        assert!((s.positions[1][0] - 2.0).abs() < 1e-6);
        assert!((s.positions[1][1]).abs() < 1e-6);
    }

    #[test]
    fn test_rotate_mesh_zero() {
        let a = simple_tri();
        // Rotate by 0 radians — positions unchanged
        let r = rotate_mesh(&a, [0.0, 0.0, 1.0], 0.0);
        for (p, q) in a.positions.iter().zip(r.positions.iter()) {
            assert!((p[0] - q[0]).abs() < 1e-5);
            assert!((p[1] - q[1]).abs() < 1e-5);
            assert!((p[2] - q[2]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_split_by_connectivity_single() {
        let a = simple_tri();
        let parts = split_by_connectivity(&a);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].positions.len(), 3);
    }

    #[test]
    fn test_split_by_connectivity_two() {
        // Two disconnected triangles
        let a = simple_tri();
        let b = simple_tri2();
        let merged = merge_two(&a, &b);
        let parts = split_by_connectivity(&merged);
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_merge_with_params_default() {
        let a = simple_tri();
        let b = simple_tri2();
        let result = merge_with_params(&[a, b], &MergeParams::default());
        assert_eq!(result.mesh.positions.len(), 6);
        assert_eq!(result.total_input_vertices, 6);
        assert_eq!(result.total_output_vertices, 6);
        assert_eq!(result.vertices_welded, 0);
    }

    #[test]
    fn test_merge_result_offsets() {
        let a = simple_tri();
        let b = simple_tri2();
        let result = merge_with_params(&[a, b], &MergeParams::default());
        assert_eq!(result.mesh_offsets.len(), 2);
        assert_eq!(result.mesh_offsets[0], 0);
        assert_eq!(result.mesh_offsets[1], 3);
    }

    // Tests for the unified merge API.

    fn make_merge_input(name: &str) -> MergeInput {
        new_merge_input(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
            name,
        )
    }

    #[test]
    fn unified_merge_two_inputs() {
        let a = make_merge_input("a");
        let b = make_merge_input("b");
        let cfg = default_merge_config();
        let res = merge_meshes(&[a, b], &cfg);
        assert_eq!(res.input_count, 2);
        assert_eq!(res.positions.len(), 6);
        assert_eq!(res.triangles.len(), 2);
    }

    #[test]
    fn unified_merge_index_offset() {
        let a = make_merge_input("a");
        let b = make_merge_input("b");
        let cfg = default_merge_config();
        let res = merge_meshes(&[a, b], &cfg);
        // b's triangle should be offset by 3.
        assert_eq!(res.triangles[1], [3, 4, 5]);
    }

    #[test]
    fn unified_merge_validate() {
        let a = make_merge_input("a");
        let cfg = default_merge_config();
        let res = merge_meshes(&[a], &cfg);
        assert!(validate_merge_result(&res));
    }

    #[test]
    fn unified_merge_bounding_box() {
        let a = make_merge_input("a");
        let cfg = default_merge_config();
        let res = merge_meshes(&[a], &cfg);
        let bb = merge_bounding_box(&res);
        // min should be (0,0,0)
        assert!((bb[0][0]).abs() < 1e-6);
        // max x should be 1.0
        assert!((bb[1][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn unified_merge_input_totals() {
        let a = make_merge_input("a");
        let b = make_merge_input("b");
        assert_eq!(merge_input_total_vertices(&[a, b]), 6);
    }

    #[test]
    fn unified_merge_result_to_json() {
        let a = make_merge_input("a");
        let cfg = default_merge_config();
        let res = merge_meshes(&[a], &cfg);
        let json = merge_result_to_json(&res);
        assert!(json.contains("input_count"));
        assert!(json.contains("vertices"));
    }

    #[test]
    fn unified_merge_face_count() {
        let a = make_merge_input("a");
        let b = make_merge_input("b");
        let cfg = default_merge_config();
        let res = merge_meshes(&[a, b], &cfg);
        assert_eq!(merge_result_vertex_count(&res), 6);
        assert_eq!(merge_result_face_count(&res), 2);
    }
}
