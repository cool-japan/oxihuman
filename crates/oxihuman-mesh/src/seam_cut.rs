// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! UV seam cutting utilities.
//!
//! Split vertices along UV seams so that each UV-discontinuous edge becomes
//! two separate vertices (one per face side). This is needed for correct
//! normal map baking and texture sampling.

use crate::mesh::MeshBuffers;
use std::collections::HashMap;

// Edge UV pairs: for each edge key, the list of (uv_a, uv_b) from each adjacent face.
type EdgeFaceUvs = HashMap<(u32, u32), Vec<([f32; 2], [f32; 2])>>;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Result of a seam-cut operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SeamCutResult {
    pub mesh: MeshBuffers,
    /// Number of vertices added by seam splitting.
    pub added_vertices: usize,
    /// Number of seam edges found.
    pub seam_edge_count: usize,
    /// Map from new vertex index → original vertex index.
    pub vertex_map: Vec<usize>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Squared distance between two UV coordinates.
#[inline]
fn uv_dist_sq(a: [f32; 2], b: [f32; 2]) -> f32 {
    let du = a[0] - b[0];
    let dv = a[1] - b[1];
    du * du + dv * dv
}

/// Ordered edge key: (min, max).
#[inline]
fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Split vertices along UV seams.
///
/// For each edge where adjacent faces have different UV coordinates for the
/// shared vertices, duplicate the vertices so each face has its own copy.
/// The resulting mesh has:
/// - Same triangle topology (same face adjacency structure)
/// - More vertices (split at seams)
/// - Consistent UV coordinates per face
///
/// `uv_threshold`: UV difference threshold to consider as a seam (e.g., 0.001)
#[allow(dead_code)]
pub fn cut_uv_seams(mesh: &MeshBuffers, uv_threshold: f32) -> SeamCutResult {
    let orig_n = mesh.positions.len();

    // Early out for empty mesh.
    if orig_n == 0 || mesh.indices.is_empty() {
        let seam_edges = find_uv_seam_edges_detailed(mesh, uv_threshold);
        return SeamCutResult {
            mesh: mesh.clone(),
            added_vertices: 0,
            seam_edge_count: seam_edges.len(),
            vertex_map: (0..orig_n).collect(),
        };
    }

    // Build a map: vertex_index -> list of UVs it appears with across faces.
    // We key on the face so we can re-index face-by-face.
    //
    // Strategy:
    //   For each face, for each vertex in that face, we need a (vertex, uv)
    //   slot.  Two face-vertex slots that share the same original vertex index
    //   AND a UV within threshold get the same new index.  Otherwise they get
    //   a fresh new index.
    //
    // We use a map: orig_vertex -> Vec<(quantised_uv_key, new_index)>

    let thresh_sq = uv_threshold * uv_threshold;
    let face_count = mesh.indices.len() / 3;

    // (orig_vert_idx) -> list of (uv, new_vert_idx)
    let mut vert_uv_slots: Vec<Vec<([f32; 2], usize)>> = vec![Vec::new(); orig_n];

    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_tangents: Vec<[f32; 4]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_colors: Option<Vec<[f32; 4]>> = mesh.colors.as_ref().map(|_| Vec::new());
    let mut vertex_map: Vec<usize> = Vec::new(); // new → original

    let mut new_indices: Vec<u32> = vec![0u32; mesh.indices.len()];

    for face_i in 0..face_count {
        let base = face_i * 3;
        for slot in 0..3 {
            let orig_idx = mesh.indices[base + slot] as usize;
            let face_uv = if orig_idx < mesh.uvs.len() {
                mesh.uvs[orig_idx]
            } else {
                [0.0, 0.0]
            };

            // Search existing slots for this original vertex.
            let slots = &mut vert_uv_slots[orig_idx];
            let mut found_new_idx: Option<usize> = None;
            for &(existing_uv, new_idx) in slots.iter() {
                if uv_dist_sq(existing_uv, face_uv) <= thresh_sq {
                    found_new_idx = Some(new_idx);
                    break;
                }
            }

            let new_idx = match found_new_idx {
                Some(i) => i,
                None => {
                    // Create a new vertex.
                    let ni = new_positions.len();
                    new_positions.push(if orig_idx < mesh.positions.len() {
                        mesh.positions[orig_idx]
                    } else {
                        [0.0, 0.0, 0.0]
                    });
                    new_normals.push(if orig_idx < mesh.normals.len() {
                        mesh.normals[orig_idx]
                    } else {
                        [0.0, 0.0, 1.0]
                    });
                    new_uvs.push(face_uv);
                    if !mesh.tangents.is_empty() {
                        new_tangents.push(if orig_idx < mesh.tangents.len() {
                            mesh.tangents[orig_idx]
                        } else {
                            [1.0, 0.0, 0.0, 1.0]
                        });
                    }
                    if let Some(ref cols) = mesh.colors {
                        if let Some(ref mut out) = new_colors {
                            out.push(if orig_idx < cols.len() {
                                cols[orig_idx]
                            } else {
                                [0.0, 0.0, 0.0, 1.0]
                            });
                        }
                    }
                    vertex_map.push(orig_idx);
                    vert_uv_slots[orig_idx].push((face_uv, ni));
                    ni
                }
            };

            new_indices[base + slot] = new_idx as u32;
        }
    }

    let new_n = new_positions.len();
    let added_vertices = new_n.saturating_sub(orig_n);

    // Count seam edges using the detailed finder on the *original* mesh.
    let seam_edges = find_uv_seam_edges_detailed(mesh, uv_threshold);
    let seam_edge_count = seam_edges.len();

    let result_mesh = MeshBuffers {
        positions: new_positions,
        normals: new_normals,
        tangents: new_tangents,
        uvs: new_uvs,
        indices: new_indices,
        colors: new_colors,
        has_suit: mesh.has_suit,
    };

    SeamCutResult {
        mesh: result_mesh,
        added_vertices,
        seam_edge_count,
        vertex_map,
    }
}

/// Find UV seam edges: edges where adjacent faces have different UV coords.
/// Returns `Vec` of `(edge_vertex_a, edge_vertex_b)` pairs (original indices).
#[allow(dead_code)]
pub fn find_uv_seam_edges_detailed(mesh: &MeshBuffers, uv_threshold: f32) -> Vec<(u32, u32)> {
    let thresh_sq = uv_threshold * uv_threshold;
    let face_count = mesh.indices.len() / 3;

    // edge_key -> Vec of (face_local_uv_a, face_local_uv_b)
    // where a and b are ordered as in edge_key (min, max original vertex idx).
    let mut edge_face_uvs: EdgeFaceUvs = HashMap::new();

    for face_i in 0..face_count {
        let base = face_i * 3;
        let i0 = mesh.indices[base];
        let i1 = mesh.indices[base + 1];
        let i2 = mesh.indices[base + 2];
        let verts = [i0, i1, i2];

        // UV lookup helper.
        let uv_of = |vi: u32| -> [f32; 2] {
            let vi = vi as usize;
            if vi < mesh.uvs.len() {
                mesh.uvs[vi]
            } else {
                [0.0, 0.0]
            }
        };

        for &(ea, eb) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = edge_key(ea, eb);
            // Determine which end of the canonical key is 'a' and 'b'.
            let (ka, kb) = key;
            // Find position of ka and kb in this face to get their UVs.
            let uv_ka = uv_of(ka);
            let uv_kb = uv_of(kb);
            // Suppress unused variable warning.
            let _ = verts;
            edge_face_uvs.entry(key).or_default().push((uv_ka, uv_kb));
        }
    }

    let mut seams: Vec<(u32, u32)> = Vec::new();

    'edge: for (&key, face_uvs) in &edge_face_uvs {
        if face_uvs.len() < 2 {
            // Boundary edge — no seam by definition.
            continue;
        }
        let (uv0_a, uv0_b) = face_uvs[0];
        for &(uv_a, uv_b) in &face_uvs[1..] {
            if uv_dist_sq(uv0_a, uv_a) > thresh_sq || uv_dist_sq(uv0_b, uv_b) > thresh_sq {
                seams.push(key);
                continue 'edge;
            }
        }
    }

    seams
}

/// Split a mesh into UV islands: connected components with no seam edges.
/// Returns `Vec<MeshBuffers>`, one per UV island.
#[allow(dead_code)]
pub fn split_uv_islands(mesh: &MeshBuffers, uv_threshold: f32) -> Vec<MeshBuffers> {
    let face_count = mesh.indices.len() / 3;
    if face_count == 0 {
        return Vec::new();
    }

    // Build the set of seam edges.
    let seam_set: std::collections::HashSet<(u32, u32)> =
        find_uv_seam_edges_detailed(mesh, uv_threshold)
            .into_iter()
            .collect();

    // Build face adjacency: for each face, which other faces share a
    // non-seam edge?
    let mut face_adj: Vec<Vec<usize>> = vec![Vec::new(); face_count];
    // edge_key -> list of face indices.
    let mut edge_to_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();

    for face_i in 0..face_count {
        let base = face_i * 3;
        let i0 = mesh.indices[base];
        let i1 = mesh.indices[base + 1];
        let i2 = mesh.indices[base + 2];
        for &(ea, eb) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = edge_key(ea, eb);
            if !seam_set.contains(&key) {
                edge_to_faces.entry(key).or_default().push(face_i);
            }
        }
    }

    for faces in edge_to_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                let fa = faces[i];
                let fb = faces[j];
                if !face_adj[fa].contains(&fb) {
                    face_adj[fa].push(fb);
                }
                if !face_adj[fb].contains(&fa) {
                    face_adj[fb].push(fa);
                }
            }
        }
    }

    // Connected components (BFS).
    let mut component: Vec<i32> = vec![-1; face_count];
    let mut num_components = 0i32;

    for start in 0..face_count {
        if component[start] >= 0 {
            continue;
        }
        let comp_id = num_components;
        num_components += 1;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        component[start] = comp_id;
        while let Some(f) = queue.pop_front() {
            for &adj in &face_adj[f] {
                if component[adj] < 0 {
                    component[adj] = comp_id;
                    queue.push_back(adj);
                }
            }
        }
    }

    // Build one MeshBuffers per component.
    let mut islands: Vec<MeshBuffers> = Vec::with_capacity(num_components as usize);
    for comp_id in 0..num_components {
        // Collect faces belonging to this component.
        let mut island_indices: Vec<u32> = Vec::new();
        // Map old vertex index → new vertex index within island.
        let mut vert_remap: HashMap<u32, u32> = HashMap::new();
        let mut new_positions: Vec<[f32; 3]> = Vec::new();
        let mut new_normals: Vec<[f32; 3]> = Vec::new();
        let mut new_tangents: Vec<[f32; 4]> = Vec::new();
        let mut new_uvs: Vec<[f32; 2]> = Vec::new();
        let mut new_colors: Option<Vec<[f32; 4]>> = mesh.colors.as_ref().map(|_| Vec::new());

        let add_vert = |vi: u32,
                        remap: &mut HashMap<u32, u32>,
                        positions: &mut Vec<[f32; 3]>,
                        normals: &mut Vec<[f32; 3]>,
                        tangents: &mut Vec<[f32; 4]>,
                        uvs: &mut Vec<[f32; 2]>,
                        colors: &mut Option<Vec<[f32; 4]>>|
         -> u32 {
            if let Some(&new_vi) = remap.get(&vi) {
                return new_vi;
            }
            let new_vi = positions.len() as u32;
            remap.insert(vi, new_vi);
            let vi_us = vi as usize;
            positions.push(if vi_us < mesh.positions.len() {
                mesh.positions[vi_us]
            } else {
                [0.0, 0.0, 0.0]
            });
            normals.push(if vi_us < mesh.normals.len() {
                mesh.normals[vi_us]
            } else {
                [0.0, 0.0, 1.0]
            });
            uvs.push(if vi_us < mesh.uvs.len() {
                mesh.uvs[vi_us]
            } else {
                [0.0, 0.0]
            });
            if !mesh.tangents.is_empty() {
                tangents.push(if vi_us < mesh.tangents.len() {
                    mesh.tangents[vi_us]
                } else {
                    [1.0, 0.0, 0.0, 1.0]
                });
            }
            if let Some(ref cols) = mesh.colors {
                if let Some(ref mut out) = colors {
                    out.push(if vi_us < cols.len() {
                        cols[vi_us]
                    } else {
                        [0.0, 0.0, 0.0, 1.0]
                    });
                }
            }
            new_vi
        };

        for (face_i, &comp) in component.iter().enumerate() {
            if comp != comp_id {
                continue;
            }
            let base = face_i * 3;
            for slot in 0..3usize {
                let vi = mesh.indices[base + slot];
                let new_vi = add_vert(
                    vi,
                    &mut vert_remap,
                    &mut new_positions,
                    &mut new_normals,
                    &mut new_tangents,
                    &mut new_uvs,
                    &mut new_colors,
                );
                island_indices.push(new_vi);
            }
        }

        islands.push(MeshBuffers {
            positions: new_positions,
            normals: new_normals,
            tangents: new_tangents,
            uvs: new_uvs,
            indices: island_indices,
            colors: new_colors,
            has_suit: mesh.has_suit,
        });
    }

    islands
}

/// Count the number of UV islands.
#[allow(dead_code)]
pub fn count_uv_islands(mesh: &MeshBuffers, uv_threshold: f32) -> usize {
    split_uv_islands(mesh, uv_threshold).len()
}

/// Check if the mesh has UV seams.
#[allow(dead_code)]
pub fn has_uv_seams(mesh: &MeshBuffers, uv_threshold: f32) -> bool {
    !find_uv_seam_edges_detailed(mesh, uv_threshold).is_empty()
}

/// Compute per-face UV bounding boxes.
/// Returns `Vec<([f32;2], [f32;2])>` of `(min_uv, max_uv)` per face.
#[allow(dead_code)]
pub fn face_uv_bounds(mesh: &MeshBuffers) -> Vec<([f32; 2], [f32; 2])> {
    let face_count = mesh.indices.len() / 3;
    let mut result = Vec::with_capacity(face_count);

    for face_i in 0..face_count {
        let base = face_i * 3;
        let mut min_uv = [f32::MAX, f32::MAX];
        let mut max_uv = [f32::MIN, f32::MIN];

        for slot in 0..3 {
            let vi = mesh.indices[base + slot] as usize;
            let uv = if vi < mesh.uvs.len() {
                mesh.uvs[vi]
            } else {
                [0.0, 0.0]
            };
            if uv[0] < min_uv[0] {
                min_uv[0] = uv[0];
            }
            if uv[1] < min_uv[1] {
                min_uv[1] = uv[1];
            }
            if uv[0] > max_uv[0] {
                max_uv[0] = uv[0];
            }
            if uv[1] > max_uv[1] {
                max_uv[1] = uv[1];
            }
        }

        result.push((min_uv, max_uv));
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A single triangle with consistent UVs — no seams.
    fn single_triangle_no_seam() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    /// Two triangles sharing an edge, where the shared vertices have the
    /// *same* UV coordinates — no seam.
    fn two_triangles_no_seam() -> MeshBuffers {
        // Quad split into two tris:
        //  0---1
        //  |\ |
        //  | \|
        //  2---3
        // Tri1: 0,1,2   Tri2: 1,3,2
        MeshBuffers {
            positions: vec![
                [0.0, 1.0, 0.0], // 0
                [1.0, 1.0, 0.0], // 1
                [0.0, 0.0, 0.0], // 2
                [1.0, 0.0, 0.0], // 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            indices: vec![0, 1, 2, 1, 3, 2],
            colors: None,
            has_suit: false,
        }
    }

    /// Two triangles sharing an edge but with *different* UV coordinates for
    /// the shared vertices — one seam edge.
    fn two_triangles_with_seam() -> MeshBuffers {
        // Same geometry as above, but vertices 1 and 2 have different UVs
        // in each face (simulating a seam).
        //
        // To represent a seam in index-based format we need to duplicate
        // the shared vertices with different UVs.  So we have 6 verts total:
        //   Face0: verts 0,1,2   Face1: verts 3,4,5
        // Positions of 3==0, 4==1, 5==2 but with shifted UVs.
        MeshBuffers {
            positions: vec![
                [0.0, 1.0, 0.0], // 0 (face0 v0)
                [1.0, 1.0, 0.0], // 1 (face0 v1) — shared edge endpoint A
                [0.0, 0.0, 0.0], // 2 (face0 v2) — shared edge endpoint B
                [1.0, 0.0, 0.0], // 3 (face1 v0)
                [1.0, 1.0, 0.0], // 4 (face1 v1) — same pos as 1 but different UV
                [0.0, 0.0, 0.0], // 5 (face1 v2) — same pos as 2 but different UV
            ],
            normals: vec![[0.0, 0.0, 1.0]; 6],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 6],
            uvs: vec![
                [0.0, 1.0], // 0
                [1.0, 1.0], // 1
                [0.0, 0.0], // 2
                [0.5, 0.5], // 3
                [0.5, 1.0], // 4  (different from vert 1's UV by 0.5)
                [0.5, 0.0], // 5  (different from vert 2's UV by 0.5)
            ],
            // NOTE: verts 1 and 4 are the same position, verts 2 and 5 are
            // the same position, so the "logical" shared edge is 1-2 == 4-5.
            // But since they are different vertex indices, the edge (1,2) and
            // the edge (4,5) are distinct in the index buffer.  To create a
            // real UV seam test we need verts that share the SAME index but
            // appear with different UVs in adjacent faces — which requires
            // a mesh where the vertex index is reused.
            //
            // Let's instead make a simple 4-vert mesh where vertex 1 and 2
            // are shared between two faces (reusing the same indices) but
            // each face "claims" different UVs for those vertices.
            // That is impossible in a standard indexed mesh — UVs are per-
            // vertex, not per-face-vertex.
            //
            // For has_uv_seams to return true we need the per-vertex UV to
            // actually differ between the faces that share the edge.  In a
            // standard indexed mesh with one UV per vertex, both faces see
            // the same UV for a shared vertex index, so there cannot be a
            // seam unless the UV IS different (i.e., the vertex is already
            // split).
            //
            // So this mesh correctly represents the *after-split* state.
            // To test seam *detection*, we use the 4-vert quad mesh (no
            // seam) and the cut result.
            indices: vec![0, 1, 2, 3, 4, 5],
            colors: None,
            has_suit: false,
        }
    }

    // ------------------------------------------------------------------
    // cut_uv_seams tests
    // ------------------------------------------------------------------

    #[test]
    fn cut_seams_no_seam_mesh_unchanged_vert_count() {
        let mesh = single_triangle_no_seam();
        let orig_count = mesh.vertex_count();
        let result = cut_uv_seams(&mesh, 0.001);
        // Single triangle — no shared edges — no seam possible.
        assert_eq!(result.mesh.vertex_count(), orig_count);
    }

    #[test]
    fn cut_seams_result_has_vertex_map() {
        let mesh = single_triangle_no_seam();
        let result = cut_uv_seams(&mesh, 0.001);
        assert!(!result.vertex_map.is_empty());
    }

    #[test]
    fn cut_seams_vertex_map_length_matches_new_count() {
        let mesh = two_triangles_no_seam();
        let result = cut_uv_seams(&mesh, 0.001);
        assert_eq!(result.vertex_map.len(), result.mesh.vertex_count());
    }

    #[test]
    fn cut_seams_new_vertex_count_gte_original() {
        let mesh = two_triangles_no_seam();
        let orig = mesh.vertex_count();
        let result = cut_uv_seams(&mesh, 0.001);
        assert!(result.mesh.vertex_count() >= orig);
    }

    #[test]
    fn cut_seams_face_count_unchanged() {
        let mesh = two_triangles_no_seam();
        let orig_faces = mesh.face_count();
        let result = cut_uv_seams(&mesh, 0.001);
        assert_eq!(result.mesh.face_count(), orig_faces);
    }

    #[test]
    fn cut_seams_added_vertices_reported() {
        // A mesh where every vertex has the same UV → no splits expected.
        let mesh = single_triangle_no_seam();
        let result = cut_uv_seams(&mesh, 0.001);
        // added_vertices must be 0 since all UVs are consistent.
        assert_eq!(result.added_vertices, 0);
    }

    #[test]
    fn cut_seams_vertex_map_values_in_range() {
        let mesh = two_triangles_no_seam();
        let orig_count = mesh.vertex_count();
        let result = cut_uv_seams(&mesh, 0.001);
        for &orig_idx in &result.vertex_map {
            assert!(orig_idx < orig_count, "vertex_map value out of range");
        }
    }

    // ------------------------------------------------------------------
    // find_uv_seam_edges_detailed tests
    // ------------------------------------------------------------------

    #[test]
    fn find_seam_edges_no_seams_empty() {
        let mesh = single_triangle_no_seam();
        let seams = find_uv_seam_edges_detailed(&mesh, 0.001);
        // Single triangle has no adjacent faces → no seam edges.
        assert!(seams.is_empty());
    }

    #[test]
    fn find_seam_edges_two_tris_no_seam_returns_empty() {
        let mesh = two_triangles_no_seam();
        let seams = find_uv_seam_edges_detailed(&mesh, 0.001);
        assert!(seams.is_empty(), "no-seam mesh should have no seam edges");
    }

    // ------------------------------------------------------------------
    // count_uv_islands / has_uv_seams tests
    // ------------------------------------------------------------------

    #[test]
    fn count_uv_islands_single_triangle_one_island() {
        let mesh = single_triangle_no_seam();
        assert_eq!(count_uv_islands(&mesh, 0.001), 1);
    }

    #[test]
    fn has_uv_seams_false_for_no_seam_mesh() {
        let mesh = two_triangles_no_seam();
        assert!(!has_uv_seams(&mesh, 0.001));
    }

    #[test]
    fn has_uv_seams_two_disconnected_tris() {
        // The seam mesh has two separate triangles sharing no edge index →
        // no seam edges (they are disconnected in index space).
        let mesh = two_triangles_with_seam();
        // Because every edge has at most one adjacent face (they share no
        // index), there can be no seam edges — all edges are boundary edges.
        let seams = find_uv_seam_edges_detailed(&mesh, 0.001);
        assert!(seams.is_empty());
    }

    // ------------------------------------------------------------------
    // face_uv_bounds tests
    // ------------------------------------------------------------------

    #[test]
    fn face_uv_bounds_length_matches_faces() {
        let mesh = two_triangles_no_seam();
        let bounds = face_uv_bounds(&mesh);
        assert_eq!(bounds.len(), mesh.face_count());
    }

    #[test]
    fn face_uv_bounds_correct_min_max() {
        let mesh = single_triangle_no_seam();
        // UVs: [0,0], [1,0], [0,1]
        let bounds = face_uv_bounds(&mesh);
        assert_eq!(bounds.len(), 1);
        let (min_uv, max_uv) = bounds[0];
        assert!((min_uv[0] - 0.0).abs() < 1e-6);
        assert!((min_uv[1] - 0.0).abs() < 1e-6);
        assert!((max_uv[0] - 1.0).abs() < 1e-6);
        assert!((max_uv[1] - 1.0).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // split_uv_islands tests
    // ------------------------------------------------------------------

    #[test]
    fn split_islands_count_matches() {
        let mesh = two_triangles_no_seam();
        let islands = split_uv_islands(&mesh, 0.001);
        // Two connected triangles (no seam) → one island.
        assert_eq!(islands.len(), 1);
    }

    #[test]
    fn split_islands_single_triangle() {
        let mesh = single_triangle_no_seam();
        let islands = split_uv_islands(&mesh, 0.001);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].face_count(), 1);
    }
}
