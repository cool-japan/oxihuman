//! Boundary edge and vertex detection for triangle meshes.
//!
//! A boundary edge is one that is shared by exactly one face.
//! This module detects all such edges, the vertices that touch them,
//! and organises them into ordered boundary loops.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Config / Result types
// ---------------------------------------------------------------------------

/// Configuration for boundary detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryDetectConfig {
    /// Reserved for future use (e.g. tolerance).
    pub reserved: u32,
}

/// Result of a boundary-detection pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryInfo {
    /// Boundary edges stored as (min_v, max_v) pairs.
    pub edges: Vec<(u32, u32)>,
    /// Set of vertex indices that lie on a boundary edge.
    pub vertices: Vec<u32>,
    /// Directed adjacency used for loop tracing: next[v] = w.
    pub(crate) next: HashMap<u32, u32>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Returns the default boundary-detection configuration.
#[allow(dead_code)]
pub fn default_boundary_config() -> BoundaryDetectConfig {
    BoundaryDetectConfig { reserved: 0 }
}

/// Detects boundary edges and vertices in a triangle mesh.
///
/// `faces` is a slice of triangles (each a `[u32; 3]` of vertex indices).
/// `n_verts` is the total number of vertices (used only for validation).
#[allow(dead_code)]
pub fn detect_boundary(
    faces: &[[u32; 3]],
    _n_verts: usize,
    _cfg: &BoundaryDetectConfig,
) -> BoundaryInfo {
    // Count how many times each directed half-edge appears.
    // A boundary half-edge (v0→v1) is one whose reverse (v1→v0) does NOT exist.
    let mut half_edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for face in faces {
        let [a, b, c] = *face;
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            *half_edge_count.entry((u, v)).or_insert(0) += 1;
        }
    }

    // An undirected edge is a boundary edge if exactly one of its two directed
    // variants is present in the half-edge map (the other has count 0).
    let mut boundary_set: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
    let mut next: HashMap<u32, u32> = HashMap::new();

    for (&(u, v), &cnt) in &half_edge_count {
        // If the reverse direction is absent, (u→v) is a boundary half-edge.
        if cnt > 0 && *half_edge_count.get(&(v, u)).unwrap_or(&0) == 0 {
            let key = if u < v { (u, v) } else { (v, u) };
            boundary_set.insert(key);
            // Record directed adjacency for loop tracing (u → v).
            next.insert(u, v);
        }
    }

    let mut edges: Vec<(u32, u32)> = boundary_set.into_iter().collect();
    edges.sort_unstable();

    let mut vert_set: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &(u, v) in &edges {
        vert_set.insert(u);
        vert_set.insert(v);
    }
    let mut vertices: Vec<u32> = vert_set.into_iter().collect();
    vertices.sort_unstable();

    BoundaryInfo { edges, vertices, next }
}

/// Returns the number of boundary edges.
#[allow(dead_code)]
pub fn boundary_edge_count(info: &BoundaryInfo) -> usize {
    info.edges.len()
}

/// Returns the number of boundary vertices.
#[allow(dead_code)]
pub fn boundary_vertex_count(info: &BoundaryInfo) -> usize {
    info.vertices.len()
}

/// Returns `true` if the given vertex index lies on a boundary edge.
#[allow(dead_code)]
pub fn is_boundary_vertex(info: &BoundaryInfo, vert_idx: u32) -> bool {
    info.vertices.binary_search(&vert_idx).is_ok()
}

/// Returns `true` if the edge (v0, v1) is a boundary edge (order-independent).
#[allow(dead_code)]
pub fn is_boundary_edge(info: &BoundaryInfo, v0: u32, v1: u32) -> bool {
    let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
    info.edges.binary_search(&key).is_ok()
}

/// Traces and returns all boundary loops as ordered vertex sequences.
///
/// Each inner `Vec<u32>` starts and ends at the same vertex (the start vertex
/// is NOT repeated at the end; callers should close the loop themselves if needed).
#[allow(dead_code)]
pub fn boundary_loops(info: &BoundaryInfo) -> Vec<Vec<u32>> {
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut loops: Vec<Vec<u32>> = Vec::new();

    for &start in info.next.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut loop_verts: Vec<u32> = Vec::new();
        let mut cur = start;
        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            loop_verts.push(cur);
            match info.next.get(&cur) {
                Some(&nxt) => cur = nxt,
                None => break,
            }
        }
        if !loop_verts.is_empty() {
            loops.push(loop_verts);
        }
    }
    loops
}

/// Returns `true` if the mesh has no boundary edges (i.e. is a closed manifold).
#[allow(dead_code)]
pub fn mesh_is_closed(info: &BoundaryInfo) -> bool {
    info.edges.is_empty()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> BoundaryDetectConfig {
        default_boundary_config()
    }

    /// Single triangle — all three edges are boundary edges.
    #[test]
    fn test_single_triangle_all_boundary() {
        let faces = vec![[0u32, 1, 2]];
        let info = detect_boundary(&faces, 3, &cfg());
        assert_eq!(boundary_edge_count(&info), 3);
        assert_eq!(boundary_vertex_count(&info), 3);
    }

    /// Two triangles sharing one edge — shared edge is interior, rest are boundary.
    #[test]
    fn test_two_triangles_shared_edge() {
        // Triangles: (0,1,2) and (1,3,2) share edge 1-2.
        let faces = vec![[0u32, 1, 2], [1, 3, 2]];
        let info = detect_boundary(&faces, 4, &cfg());
        // Edges: 0-1, 0-2 (from first tri boundary sides), 1-3, 2-3 (from second tri)
        // Shared edge 1-2 is interior.
        assert_eq!(boundary_edge_count(&info), 4);
    }

    /// A closed tetrahedron has no boundary edges.
    #[test]
    fn test_closed_tetrahedron_no_boundary() {
        let faces = vec![
            [0u32, 1, 2],
            [0, 2, 3],
            [0, 3, 1],
            [1, 3, 2],
        ];
        let info = detect_boundary(&faces, 4, &cfg());
        assert!(mesh_is_closed(&info));
        assert_eq!(boundary_edge_count(&info), 0);
        assert_eq!(boundary_vertex_count(&info), 0);
    }

    /// `is_boundary_vertex` identifies correct vertices.
    #[test]
    fn test_is_boundary_vertex() {
        let faces = vec![[0u32, 1, 2]];
        let info = detect_boundary(&faces, 3, &cfg());
        assert!(is_boundary_vertex(&info, 0));
        assert!(is_boundary_vertex(&info, 1));
        assert!(is_boundary_vertex(&info, 2));
        assert!(!is_boundary_vertex(&info, 99));
    }

    /// `is_boundary_edge` is order-independent.
    #[test]
    fn test_is_boundary_edge_order_independent() {
        let faces = vec![[0u32, 1, 2]];
        let info = detect_boundary(&faces, 3, &cfg());
        assert!(is_boundary_edge(&info, 0, 1));
        assert!(is_boundary_edge(&info, 1, 0)); // reversed
        assert!(!is_boundary_edge(&info, 0, 99));
    }

    /// Shared interior edge is not a boundary edge.
    #[test]
    fn test_shared_edge_not_boundary() {
        let faces = vec![[0u32, 1, 2], [0, 2, 3]];
        let info = detect_boundary(&faces, 4, &cfg());
        // Edge 0-2 is shared, so not boundary.
        assert!(!is_boundary_edge(&info, 0, 2));
        assert!(!is_boundary_edge(&info, 2, 0));
    }

    /// `boundary_loops` returns at least one loop for an open mesh.
    #[test]
    fn test_boundary_loops_single_triangle() {
        let faces = vec![[0u32, 1, 2]];
        let info = detect_boundary(&faces, 3, &cfg());
        let loops = boundary_loops(&info);
        // All three vertices form one loop.
        assert!(!loops.is_empty());
        let total: usize = loops.iter().map(|l| l.len()).sum();
        assert_eq!(total, 3);
    }

    /// Empty mesh has no boundary.
    #[test]
    fn test_empty_mesh() {
        let faces: Vec<[u32; 3]> = vec![];
        let info = detect_boundary(&faces, 0, &cfg());
        assert!(mesh_is_closed(&info));
        assert_eq!(boundary_edge_count(&info), 0);
        assert_eq!(boundary_vertex_count(&info), 0);
    }

    /// A quad strip (two tris, open mesh) — boundary vertex count equals 4.
    #[test]
    fn test_quad_strip_boundary_vertex_count() {
        // Vertices: 0-1-2-3 as a quad split into two triangles.
        let faces = vec![[0u32, 1, 2], [1, 3, 2]];
        let info = detect_boundary(&faces, 4, &cfg());
        assert_eq!(boundary_vertex_count(&info), 4);
    }
}
