// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dual mesh computation — place a vertex at each face centroid, connect adjacent faces.

// ─── Structures ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct DualMesh {
    /// One dual vertex per original face (face centroid).
    pub positions: Vec<[f32; 3]>,
    /// Edges connecting adjacent face centroids.
    pub edges: Vec<[usize; 2]>,
    /// Maps original face index to dual vertex index.
    pub face_to_dual_vertex: Vec<usize>,
    pub original_vertex_count: usize,
    pub original_face_count: usize,
}

// ─── Functions ───────────────────────────────────────────────────────────────

/// Compute the centroid of a triangular face.
#[allow(dead_code)]
pub fn face_centroid(positions: &[[f32; 3]], face: [u32; 3]) -> [f32; 3] {
    let a = positions[face[0] as usize];
    let b = positions[face[1] as usize];
    let c = positions[face[2] as usize];
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

/// Build a per-face adjacency list: faces that share an edge are adjacent.
#[allow(dead_code)]
pub fn build_face_adjacency(indices: &[u32], face_count: usize) -> Vec<Vec<usize>> {
    use std::collections::HashMap;

    // Map each directed edge to the face that owns it.
    let mut edge_to_face: HashMap<(u32, u32), usize> = HashMap::new();
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); face_count];

    for face_idx in 0..face_count {
        let base = face_idx * 3;
        let v = [indices[base], indices[base + 1], indices[base + 2]];
        for k in 0..3usize {
            let a = v[k];
            let b = v[(k + 1) % 3];
            // Canonical undirected edge key.
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&other) = edge_to_face.get(&key) {
                // This edge is shared – mark both faces as adjacent.
                if !adjacency[face_idx].contains(&other) {
                    adjacency[face_idx].push(other);
                }
                if !adjacency[other].contains(&face_idx) {
                    adjacency[other].push(face_idx);
                }
            } else {
                edge_to_face.insert(key, face_idx);
            }
        }
    }
    adjacency
}

/// Compute the dual mesh of a triangle mesh.
#[allow(dead_code)]
pub fn compute_dual_mesh(positions: &[[f32; 3]], indices: &[u32]) -> DualMesh {
    let face_count = indices.len() / 3;
    let original_vertex_count = positions.len();

    // Dual vertex at each face centroid.
    let mut dual_positions: Vec<[f32; 3]> = Vec::with_capacity(face_count);
    let mut face_to_dual_vertex: Vec<usize> = Vec::with_capacity(face_count);

    for face_idx in 0..face_count {
        let base = face_idx * 3;
        let face = [indices[base], indices[base + 1], indices[base + 2]];
        dual_positions.push(face_centroid(positions, face));
        face_to_dual_vertex.push(face_idx);
    }

    // Build edges between adjacent face centroids.
    let adjacency = build_face_adjacency(indices, face_count);
    let mut edges: Vec<[usize; 2]> = Vec::new();
    for (fi, neighbors) in adjacency.iter().enumerate() {
        for &fj in neighbors {
            if fi < fj {
                edges.push([fi, fj]);
            }
        }
    }

    DualMesh {
        positions: dual_positions,
        edges,
        face_to_dual_vertex,
        original_vertex_count,
        original_face_count: face_count,
    }
}

/// Number of dual vertices (equals original face count).
#[allow(dead_code)]
pub fn dual_vertex_count(dual: &DualMesh) -> usize {
    dual.positions.len()
}

/// Number of dual edges.
#[allow(dead_code)]
pub fn dual_edge_count(dual: &DualMesh) -> usize {
    dual.edges.len()
}

/// Slice of dual vertex positions.
#[allow(dead_code)]
pub fn dual_to_positions(dual: &DualMesh) -> &[[f32; 3]] {
    &dual.positions
}

/// Euclidean length of each dual edge.
#[allow(dead_code)]
pub fn dual_edge_lengths(dual: &DualMesh) -> Vec<f32> {
    dual.edges
        .iter()
        .map(|&[a, b]| {
            let pa = dual.positions[a];
            let pb = dual.positions[b];
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dz = pb[2] - pa[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .collect()
}

/// Average dual edge length; returns 0 if no edges.
#[allow(dead_code)]
pub fn average_dual_edge_length(dual: &DualMesh) -> f32 {
    let lengths = dual_edge_lengths(dual);
    if lengths.is_empty() {
        return 0.0;
    }
    lengths.iter().sum::<f32>() / lengths.len() as f32
}

/// Axis-aligned bounding box of dual vertex positions (min, max).
#[allow(dead_code)]
pub fn dual_mesh_bounds(dual: &DualMesh) -> ([f32; 3], [f32; 3]) {
    if dual.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = dual.positions[0];
    let mut mx = dual.positions[0];
    for p in &dual.positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

/// Returns `true` if the dual mesh has at least one edge.
#[allow(dead_code)]
pub fn is_dual_connected(dual: &DualMesh) -> bool {
    !dual.edges.is_empty()
}

/// Per-dual-vertex adjacency list.
#[allow(dead_code)]
pub fn dual_to_graph_adjacency(dual: &DualMesh) -> Vec<Vec<usize>> {
    let n = dual_vertex_count(dual);
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &[a, b] in &dual.edges {
        adj[a].push(b);
        adj[b].push(a);
    }
    adj
}

/// Degree of a dual vertex (number of adjacent dual vertices).
#[allow(dead_code)]
pub fn dual_vertex_degree(dual: &DualMesh, vertex_idx: usize) -> usize {
    let adj = dual_to_graph_adjacency(dual);
    if vertex_idx < adj.len() {
        adj[vertex_idx].len()
    } else {
        0
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // A simple tetrahedron: 4 vertices, 4 triangular faces.
    fn tetra_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, // face 0
            0, 1, 3, // face 1
            0, 2, 3, // face 2
            1, 2, 3, // face 3
        ];
        (positions, indices)
    }

    #[test]
    fn test_face_centroid_unit_triangle() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let c = face_centroid(&positions, [0, 1, 2]);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
        assert!(c[2].abs() < 1e-5);
    }

    #[test]
    fn test_dual_of_tetrahedron_has_4_vertices() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        assert_eq!(dual_vertex_count(&dual), 4);
    }

    #[test]
    fn test_dual_original_face_count() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        assert_eq!(dual.original_face_count, 4);
    }

    #[test]
    fn test_dual_original_vertex_count() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        assert_eq!(dual.original_vertex_count, 4);
    }

    #[test]
    fn test_face_to_dual_vertex_mapping() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        for (i, &dv) in dual.face_to_dual_vertex.iter().enumerate() {
            assert_eq!(i, dv);
        }
    }

    #[test]
    fn test_build_face_adjacency_tetrahedron() {
        let (_, idx) = tetra_mesh();
        let adj = build_face_adjacency(&idx, 4);
        // In a tetrahedron each face shares edges with all 3 other faces.
        for neighbors in &adj {
            assert_eq!(neighbors.len(), 3);
        }
    }

    #[test]
    fn test_dual_vertex_count_function() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        assert_eq!(dual_vertex_count(&dual), dual.positions.len());
    }

    #[test]
    fn test_dual_edge_count_tetrahedron() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        // 4 faces each adjacent to 3 → 6 shared edges.
        assert_eq!(dual_edge_count(&dual), 6);
    }

    #[test]
    fn test_dual_edge_lengths_positive() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        let lengths = dual_edge_lengths(&dual);
        for l in &lengths {
            assert!(*l > 0.0);
        }
    }

    #[test]
    fn test_average_dual_edge_length() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        let avg = average_dual_edge_length(&dual);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_dual_mesh_bounds() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        let (mn, mx) = dual_mesh_bounds(&dual);
        for k in 0..3 {
            assert!(mn[k] <= mx[k]);
        }
    }

    #[test]
    fn test_is_dual_connected() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        assert!(is_dual_connected(&dual));
    }

    #[test]
    fn test_dual_to_graph_adjacency_degree() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        let adj = dual_to_graph_adjacency(&dual);
        for neighbors in &adj {
            assert_eq!(neighbors.len(), 3);
        }
    }

    #[test]
    fn test_dual_vertex_degree() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        assert_eq!(dual_vertex_degree(&dual, 0), 3);
    }

    #[test]
    fn test_dual_to_positions_slice() {
        let (pos, idx) = tetra_mesh();
        let dual = compute_dual_mesh(&pos, &idx);
        let slice = dual_to_positions(&dual);
        assert_eq!(slice.len(), 4);
    }
}
