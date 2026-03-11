// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;
use std::collections::HashMap;

// ── helpers ──────────────────────────────────────────────────────────────────

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn scale2(v: [f32; 2], s: f32) -> [f32; 2] {
    [v[0] * s, v[1] * s]
}

fn canonical_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

// ── midpoint helper ───────────────────────────────────────────────────────────

/// Build edge → midpoint-vertex-index map (simple average, no Loop weights).
/// Returns (map, new_positions, new_uvs).
#[allow(clippy::type_complexity)]
fn build_midpoint_map(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> (HashMap<(u32, u32), u32>, Vec<[f32; 3]>, Vec<[f32; 2]>) {
    let base_count = positions.len() as u32;
    let mut map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_pos: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut next_idx = base_count;

    for tri in indices.chunks_exact(3) {
        let verts = [tri[0], tri[1], tri[2]];
        for i in 0..3 {
            let a = verts[i];
            let b = verts[(i + 1) % 3];
            let key = canonical_edge(a, b);
            map.entry(key).or_insert_with(|| {
                let mid_pos = scale3(add3(positions[a as usize], positions[b as usize]), 0.5);
                let mid_uv = scale2(add2(uvs[a as usize], uvs[b as usize]), 0.5);
                new_pos.push(mid_pos);
                new_uvs.push(mid_uv);
                let idx = next_idx;
                next_idx += 1;
                idx
            });
        }
    }

    (map, new_pos, new_uvs)
}

// ── Loop subdivision (one iteration) ─────────────────────────────────────────

fn loop_subdivide_once(mesh: &MeshBuffers) -> MeshBuffers {
    let positions = &mesh.positions;
    let uvs = &mesh.uvs;
    let indices = &mesh.indices;
    let n_orig = positions.len();

    // --- Step 1: gather edge data (face count per edge, opposite vertices) ---
    // edge_key → (face_count, [opp0, opp1])
    let mut edge_faces: HashMap<(u32, u32), (u32, [u32; 2])> = HashMap::new();

    for tri in indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        let edges = [(v0, v1, v2), (v1, v2, v0), (v2, v0, v1)];
        for (a, b, opp) in edges {
            let key = canonical_edge(a, b);
            let entry = edge_faces.entry(key).or_insert((0, [0, 0]));
            if entry.0 < 2 {
                entry.1[entry.0 as usize] = opp;
            }
            entry.0 += 1;
        }
    }

    // --- Step 2: compute edge midpoints using Loop weights ---
    let base_count = n_orig as u32;
    let mut edge_to_idx: HashMap<(u32, u32), u32> = HashMap::new();
    let mut mid_positions: Vec<[f32; 3]> = Vec::new();
    let mut mid_uvs: Vec<[f32; 2]> = Vec::new();
    let mut next_idx = base_count;

    // Collect all edges deterministically (iterate over triangles)
    for tri in indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        let edge_triplets = [(v0, v1, v2), (v1, v2, v0), (v2, v0, v1)];
        for (a, b, _opp) in edge_triplets {
            let key = canonical_edge(a, b);
            if edge_to_idx.contains_key(&key) {
                continue;
            }
            let (count, opps) = edge_faces[&key];
            let pa = positions[key.0 as usize];
            let pb = positions[key.1 as usize];
            let mid_pos = if count == 1 {
                // boundary: simple midpoint
                scale3(add3(pa, pb), 0.5)
            } else {
                // interior: Loop weights 3/8*(a+b) + 1/8*(opp0+opp1)
                let p_opp0 = positions[opps[0] as usize];
                let p_opp1 = positions[opps[1] as usize];
                let ab = scale3(add3(pa, pb), 3.0 / 8.0);
                let opps_sum = scale3(add3(p_opp0, p_opp1), 1.0 / 8.0);
                add3(ab, opps_sum)
            };
            let uva = uvs[key.0 as usize];
            let uvb = uvs[key.1 as usize];
            let mid_uv = scale2(add2(uva, uvb), 0.5);

            mid_positions.push(mid_pos);
            mid_uvs.push(mid_uv);
            edge_to_idx.insert(key, next_idx);
            next_idx += 1;
        }
    }

    // --- Step 3: update original vertex positions ---
    // Build adjacency: for each vertex, collect neighbor indices and boundary info
    let mut neighbors: Vec<Vec<u32>> = vec![Vec::new(); n_orig];
    // boundary_neighbors[v] = list of vertices connected via boundary edges
    let mut boundary_neighbors: Vec<Vec<u32>> = vec![Vec::new(); n_orig];

    for (&(a, b), &(count, _)) in &edge_faces {
        let a_u = a as usize;
        let b_u = b as usize;
        if !neighbors[a_u].contains(&b) {
            neighbors[a_u].push(b);
        }
        if !neighbors[b_u].contains(&a) {
            neighbors[b_u].push(a);
        }
        if count == 1 {
            if !boundary_neighbors[a_u].contains(&b) {
                boundary_neighbors[a_u].push(b);
            }
            if !boundary_neighbors[b_u].contains(&a) {
                boundary_neighbors[b_u].push(a);
            }
        }
    }

    let mut updated_positions: Vec<[f32; 3]> = positions.to_vec();
    for i in 0..n_orig {
        let v = positions[i];
        let bn = &boundary_neighbors[i];
        if bn.len() >= 2 {
            // boundary vertex: 3/4 * v + 1/8 * (left + right)
            let vl = positions[bn[0] as usize];
            let vr = positions[bn[1] as usize];
            let new_v = add3(scale3(v, 3.0 / 4.0), scale3(add3(vl, vr), 1.0 / 8.0));
            updated_positions[i] = new_v;
        } else {
            // interior vertex
            let n_val = neighbors[i].len();
            if n_val == 0 {
                continue;
            }
            let beta = if n_val == 3 {
                3.0 / 16.0
            } else {
                3.0 / (8.0 * n_val as f32)
            };
            let mut sum = [0.0f32; 3];
            for &nb in &neighbors[i] {
                sum = add3(sum, positions[nb as usize]);
            }
            let new_v = add3(scale3(v, 1.0 - n_val as f32 * beta), scale3(sum, beta));
            updated_positions[i] = new_v;
        }
    }

    // Append midpoint positions after updated originals
    let all_positions: Vec<[f32; 3]> = updated_positions.into_iter().chain(mid_positions).collect();

    // UVs: originals unchanged + midpoints
    let all_uvs: Vec<[f32; 2]> = uvs.iter().copied().chain(mid_uvs).collect();

    // --- Step 4: rebuild topology ---
    let mut new_indices: Vec<u32> = Vec::with_capacity(indices.len() * 4);
    for tri in indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        let m01 = edge_to_idx[&canonical_edge(v0, v1)];
        let m12 = edge_to_idx[&canonical_edge(v1, v2)];
        let m20 = edge_to_idx[&canonical_edge(v2, v0)];
        // 4 sub-triangles
        new_indices.extend_from_slice(&[v0, m01, m20]);
        new_indices.extend_from_slice(&[v1, m12, m01]);
        new_indices.extend_from_slice(&[v2, m20, m12]);
        new_indices.extend_from_slice(&[m01, m12, m20]);
    }

    let mut result = MeshBuffers {
        positions: all_positions,
        normals: vec![[0.0, 1.0, 0.0]; 0], // will be filled by compute_normals
        uvs: all_uvs,
        tangents: vec![],
        colors: None,
        indices: new_indices,
        has_suit: mesh.has_suit,
    };
    result.normals = vec![[0.0, 1.0, 0.0]; result.positions.len()];
    compute_normals(&mut result);
    result
}

// ── midpoint subdivision (one iteration) ─────────────────────────────────────

fn midpoint_subdivide_once(mesh: &MeshBuffers) -> MeshBuffers {
    let n_orig = mesh.positions.len();
    let (map, new_pos, new_uvs) = build_midpoint_map(&mesh.positions, &mesh.uvs, &mesh.indices);

    let all_positions: Vec<[f32; 3]> = mesh.positions.iter().copied().chain(new_pos).collect();
    let all_uvs: Vec<[f32; 2]> = mesh.uvs.iter().copied().chain(new_uvs).collect();

    let _ = n_orig; // used implicitly via base_count inside build_midpoint_map

    let mut new_indices: Vec<u32> = Vec::with_capacity(mesh.indices.len() * 4);
    for tri in mesh.indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        let m01 = map[&canonical_edge(v0, v1)];
        let m12 = map[&canonical_edge(v1, v2)];
        let m20 = map[&canonical_edge(v2, v0)];
        new_indices.extend_from_slice(&[v0, m01, m20]);
        new_indices.extend_from_slice(&[v1, m12, m01]);
        new_indices.extend_from_slice(&[v2, m20, m12]);
        new_indices.extend_from_slice(&[m01, m12, m20]);
    }

    let mut result = MeshBuffers {
        positions: all_positions,
        normals: vec![[0.0, 1.0, 0.0]; 0],
        uvs: all_uvs,
        tangents: vec![],
        colors: None,
        indices: new_indices,
        has_suit: mesh.has_suit,
    };
    result.normals = vec![[0.0, 1.0, 0.0]; result.positions.len()];
    compute_normals(&mut result);
    result
}

// ── public API ────────────────────────────────────────────────────────────────

/// Subdivide a mesh using the Loop subdivision scheme.
/// Each iteration quadruples the face count.
/// `iterations`: 1–4 are practical (more causes memory/time issues).
pub fn loop_subdivide(mesh: &MeshBuffers, iterations: u32) -> MeshBuffers {
    let mut current = mesh.clone();
    for _ in 0..iterations {
        current = loop_subdivide_once(&current);
    }
    current
}

/// Simple midpoint subdivision (no smoothing): just splits each triangle into 4.
/// Faster than Loop but doesn't smooth vertex positions.
pub fn midpoint_subdivide(mesh: &MeshBuffers, iterations: u32) -> MeshBuffers {
    let mut current = mesh.clone();
    for _ in 0..iterations {
        current = midpoint_subdivide_once(&current);
    }
    current
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangles() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2, 1, 3, 2],
            has_suit: true,
        }
    }

    #[test]
    fn midpoint_one_iter_quadruples_faces() {
        let mesh = two_triangles();
        let result = midpoint_subdivide(&mesh, 1);
        assert_eq!(
            result.face_count(),
            8,
            "2 tris * 4 = 8 tris after 1 midpoint iteration"
        );
    }

    #[test]
    fn midpoint_vertex_count_increases() {
        let mesh = two_triangles();
        let result = midpoint_subdivide(&mesh, 1);
        assert!(
            result.vertex_count() > mesh.vertex_count(),
            "vertex count should increase after subdivision"
        );
    }

    #[test]
    fn loop_one_iter_quadruples_faces() {
        let mesh = two_triangles();
        let result = loop_subdivide(&mesh, 1);
        assert_eq!(
            result.face_count(),
            8,
            "2 tris * 4 = 8 tris after 1 Loop iteration"
        );
    }

    #[test]
    fn loop_uv_count_matches_position_count() {
        let mesh = two_triangles();
        let result = loop_subdivide(&mesh, 1);
        assert_eq!(
            result.uvs.len(),
            result.positions.len(),
            "UVs must match position count after Loop subdivision"
        );
    }

    #[test]
    fn loop_zero_iterations_unchanged() {
        let mesh = two_triangles();
        let result = loop_subdivide(&mesh, 0);
        assert_eq!(result.vertex_count(), mesh.vertex_count());
        assert_eq!(result.face_count(), mesh.face_count());
    }

    #[test]
    fn loop_two_iterations_sixteen_faces() {
        let mesh = two_triangles();
        let result = loop_subdivide(&mesh, 2);
        assert_eq!(
            result.face_count(),
            32,
            "2 tris * 4^2 = 32 tris after 2 iterations"
        );
    }

    #[test]
    fn midpoint_check_index_bounds() {
        let mesh = two_triangles();
        let result = midpoint_subdivide(&mesh, 1);
        let n = result.vertex_count() as u32;
        for &idx in &result.indices {
            assert!(idx < n, "index {} out of bounds (vertex count {})", idx, n);
        }
    }
}
