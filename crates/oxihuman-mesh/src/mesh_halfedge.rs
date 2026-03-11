// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Half-edge data structure for adjacency queries on triangle meshes.
//!
//! For each directed half-edge the structure stores:
//! * the destination vertex,
//! * the owning face,
//! * the next half-edge in the face loop,
//! * the twin half-edge (opposite direction on the shared edge).
//!
//! Boundary half-edges have `face == usize::MAX`.

#![allow(dead_code)]

/// Configuration for the half-edge builder.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalfEdgeMeshConfig {
    /// If `true`, isolated vertices (not referenced by any face) are retained.
    pub keep_isolated_verts: bool,
}

/// Returns a sensible default [`HalfEdgeMeshConfig`].
#[allow(dead_code)]
pub fn default_halfedge_config() -> HalfEdgeMeshConfig {
    HalfEdgeMeshConfig { keep_isolated_verts: false }
}

/// A single directed half-edge.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HalfEdge {
    /// Destination vertex index.
    pub vertex: usize,
    /// Face that owns this half-edge (`usize::MAX` for boundary).
    pub face: usize,
    /// Next half-edge in the face loop (or boundary loop).
    pub next: usize,
    /// Twin half-edge index (`usize::MAX` if on boundary).
    pub twin: usize,
}

/// The full half-edge mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalfEdgeMesh {
    /// All half-edges.
    pub half_edges: Vec<HalfEdge>,
    /// For each vertex: one outgoing half-edge index.
    pub vertex_edge: Vec<usize>,
    /// For each face: one half-edge index belonging to it.
    pub face_edge: Vec<usize>,
    /// Number of vertices in the source mesh.
    pub vertex_count: usize,
    /// Number of faces in the source mesh.
    pub face_count: usize,
    /// Configuration.
    pub config: HalfEdgeMeshConfig,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Build a [`HalfEdgeMesh`] from a triangle mesh.
///
/// * `vertex_count` – number of vertices.
/// * `indices`      – flat triangle index buffer (length = `3 * face_count`).
#[allow(dead_code)]
pub fn build_halfedge_mesh(
    vertex_count: usize,
    indices: &[u32],
    config: HalfEdgeMeshConfig,
) -> HalfEdgeMesh {
    let face_count = indices.len() / 3;
    let he_count = face_count * 3;
    let mut half_edges: Vec<HalfEdge> = Vec::with_capacity(he_count);
    let mut vertex_edge = vec![usize::MAX; vertex_count];
    let mut face_edge = vec![usize::MAX; face_count];

    // Create half-edges.
    #[allow(clippy::needless_range_loop)]
    for f in 0..face_count {
        let base = f * 3;
        let he_base = f * 3;
        for k in 0..3 {
            let src = indices[base + k] as usize;
            let dst = indices[base + (k + 1) % 3] as usize;
            let next = he_base + (k + 1) % 3;
            half_edges.push(HalfEdge { vertex: dst, face: f, next, twin: usize::MAX });
            if vertex_edge[src] == usize::MAX { vertex_edge[src] = he_base + k; }
        }
        face_edge[f] = he_base;
    }

    // Link twins using a HashMap keyed on (src, dst).
    use std::collections::HashMap;
    let mut edge_map: HashMap<(usize, usize), usize> = HashMap::with_capacity(he_count);
    for f in 0..face_count {
        let base = f * 3;
        for k in 0..3 {
            let src = indices[base + k] as usize;
            let dst = indices[base + (k + 1) % 3] as usize;
            let he_idx = f * 3 + k;
            edge_map.insert((src, dst), he_idx);
        }
    }
    #[allow(clippy::needless_range_loop)]
    for i in 0..half_edges.len() {
        if half_edges[i].twin == usize::MAX {
            // src of this half-edge is the vertex BEFORE dst in the face.
            let f = half_edges[i].face;
            let base = f * 3;
            // find k such that f*3+k == i
            let k = i - f * 3;
            let src = indices[base + k] as usize;
            let dst = half_edges[i].vertex;
            if let Some(&twin_idx) = edge_map.get(&(dst, src)) {
                half_edges[i].twin = twin_idx;
            }
        }
    }

    HalfEdgeMesh { half_edges, vertex_edge, face_edge, vertex_count, face_count, config }
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

/// Return the twin half-edge index of `he`, or `usize::MAX` if on boundary.
#[allow(dead_code)]
pub fn halfedge_twin(mesh: &HalfEdgeMesh, he: usize) -> usize {
    mesh.half_edges[he].twin
}

/// Return the next half-edge index in the face loop.
#[allow(dead_code)]
pub fn halfedge_next(mesh: &HalfEdgeMesh, he: usize) -> usize {
    mesh.half_edges[he].next
}

/// Return the destination vertex of a half-edge.
#[allow(dead_code)]
pub fn halfedge_vertex(mesh: &HalfEdgeMesh, he: usize) -> usize {
    mesh.half_edges[he].vertex
}

/// Return the face owning a half-edge (`usize::MAX` for boundary).
#[allow(dead_code)]
pub fn halfedge_face(mesh: &HalfEdgeMesh, he: usize) -> usize {
    mesh.half_edges[he].face
}

/// Total number of half-edges in the mesh.
#[allow(dead_code)]
pub fn halfedge_count(mesh: &HalfEdgeMesh) -> usize {
    mesh.half_edges.len()
}

/// Returns `true` if the half-edge is on the boundary (no twin).
#[allow(dead_code)]
pub fn halfedge_is_boundary(mesh: &HalfEdgeMesh, he: usize) -> bool {
    mesh.half_edges[he].twin == usize::MAX
}

/// Collect the one-ring of vertex `v` (all neighbouring vertex indices).
#[allow(dead_code)]
pub fn halfedge_vertex_one_ring(mesh: &HalfEdgeMesh, v: usize) -> Vec<usize> {
    let start = mesh.vertex_edge[v];
    if start == usize::MAX { return vec![]; }
    let mut result = Vec::new();
    let mut he = start;
    loop {
        result.push(mesh.half_edges[he].vertex);
        // Go to previous half-edge in the same face (prev = next->next for triangles).
        let next1 = mesh.half_edges[he].next;
        let next2 = mesh.half_edges[next1].next;
        let twin = mesh.half_edges[next2].twin;
        if twin == usize::MAX || twin == start { break; }
        he = twin;
        if he == start { break; }
    }
    result
}

/// Collect all boundary loops as lists of vertex indices.
#[allow(dead_code)]
pub fn halfedge_mesh_boundary_loops(mesh: &HalfEdgeMesh) -> Vec<Vec<usize>> {
    let mut visited = vec![false; mesh.half_edges.len()];
    let mut loops = Vec::new();

    for start in 0..mesh.half_edges.len() {
        if !visited[start] && mesh.half_edges[start].twin == usize::MAX {
            // Walk the boundary loop.
            let mut loop_verts = Vec::new();
            let mut he = start;
            // For boundary traversal: advance to next boundary half-edge.
            loop {
                visited[he] = true;
                loop_verts.push(mesh.half_edges[he].vertex);
                // Find next boundary edge: go to next in face, then keep going
                // until we find one without a twin.
                let mut cur = mesh.half_edges[he].next;
                while mesh.half_edges[cur].twin != usize::MAX {
                    cur = mesh.half_edges[mesh.half_edges[cur].twin].next;
                }
                he = cur;
                if he == start || visited[he] { break; }
            }
            if !loop_verts.is_empty() {
                loops.push(loop_verts);
            }
        }
    }
    loops
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> (usize, Vec<u32>) {
        (3, vec![0u32, 1, 2])
    }

    fn quad_mesh() -> (usize, Vec<u32>) {
        // Two triangles: (0,1,2) and (0,2,3)
        (4, vec![0u32, 1, 2, 0, 2, 3])
    }

    #[test]
    fn test_single_tri_he_count() {
        let (vc, idx) = single_tri();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        assert_eq!(halfedge_count(&mesh), 3);
    }

    #[test]
    fn test_single_tri_all_boundary() {
        let (vc, idx) = single_tri();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        for he in 0..3 {
            assert!(halfedge_is_boundary(&mesh, he), "he {he} should be boundary");
        }
    }

    #[test]
    fn test_quad_twin_linking() {
        let (vc, idx) = quad_mesh();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        // Shared edge is (0,2) and (2,0); find the two half-edges.
        let mut found_linked = false;
        for he in 0..mesh.half_edges.len() {
            let twin = mesh.half_edges[he].twin;
            if twin != usize::MAX {
                found_linked = true;
                // Twin of twin should be original.
                assert_eq!(mesh.half_edges[twin].twin, he);
            }
        }
        assert!(found_linked, "expected at least one twin pair");
    }

    #[test]
    fn test_face_count() {
        let (vc, idx) = quad_mesh();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        assert_eq!(mesh.face_count, 2);
    }

    #[test]
    fn test_vertex_count() {
        let (vc, idx) = quad_mesh();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        assert_eq!(mesh.vertex_count, 4);
    }

    #[test]
    fn test_halfedge_next_cycles_face() {
        let (vc, idx) = single_tri();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        // Walking next three times should return to start.
        let start = 0;
        let n1 = halfedge_next(&mesh, start);
        let n2 = halfedge_next(&mesh, n1);
        let n3 = halfedge_next(&mesh, n2);
        assert_eq!(n3, start);
    }

    #[test]
    fn test_halfedge_face_accessor() {
        let (vc, idx) = quad_mesh();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        for he in 0..6 {
            assert!(halfedge_face(&mesh, he) < 2);
        }
    }

    #[test]
    fn test_boundary_loops_single_tri() {
        let (vc, idx) = single_tri();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        let loops = halfedge_mesh_boundary_loops(&mesh);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 3);
    }

    #[test]
    fn test_empty_mesh() {
        let mesh = build_halfedge_mesh(0, &[], default_halfedge_config());
        assert_eq!(halfedge_count(&mesh), 0);
        assert_eq!(halfedge_mesh_boundary_loops(&mesh).len(), 0);
    }

    #[test]
    fn test_halfedge_vertex_accessor() {
        let (vc, idx) = single_tri();
        let mesh = build_halfedge_mesh(vc, &idx, default_halfedge_config());
        // HE 0: src=0, dst=1; HE 1: src=1, dst=2; HE 2: src=2, dst=0
        assert_eq!(halfedge_vertex(&mesh, 0), 1);
        assert_eq!(halfedge_vertex(&mesh, 1), 2);
        assert_eq!(halfedge_vertex(&mesh, 2), 0);
    }
}
