// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Boundary vertex detection for triangle meshes.

use std::collections::{HashMap, HashSet};

/// Result of boundary vertex detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryVertexResult {
    pub boundary_vertices: Vec<usize>,
    pub total_vertices: usize,
}

/// Build an edge map counting how many triangles share each edge.
#[allow(dead_code)]
pub fn build_edge_map(indices: &[u32]) -> HashMap<(u32, u32), usize> {
    let mut map = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3];
        let i1 = indices[t * 3 + 1];
        let i2 = indices[t * 3 + 2];
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *map.entry(key).or_insert(0) += 1;
        }
    }
    map
}

/// Find all boundary edges (edges with exactly one adjacent triangle).
#[allow(dead_code)]
pub fn find_boundary_edges(indices: &[u32]) -> Vec<(u32, u32)> {
    build_edge_map(indices)
        .into_iter()
        .filter(|&(_, count)| count == 1)
        .map(|(edge, _)| edge)
        .collect()
}

/// Detect all boundary vertices.
#[allow(dead_code)]
pub fn detect_boundary_vertices(positions: &[[f32; 3]], indices: &[u32]) -> BoundaryVertexResult {
    let boundary_edges = find_boundary_edges(indices);
    let mut set = HashSet::new();
    for (a, b) in &boundary_edges {
        set.insert(*a as usize);
        set.insert(*b as usize);
    }
    let mut boundary_vertices: Vec<usize> = set.into_iter().collect();
    boundary_vertices.sort_unstable();
    BoundaryVertexResult {
        boundary_vertices,
        total_vertices: positions.len(),
    }
}

/// Check if a vertex is on the boundary.
#[allow(dead_code)]
pub fn is_boundary_vertex(result: &BoundaryVertexResult, vertex: usize) -> bool {
    result.boundary_vertices.contains(&vertex)
}

/// Count of boundary vertices.
#[allow(dead_code)]
pub fn boundary_count(result: &BoundaryVertexResult) -> usize {
    result.boundary_vertices.len()
}

/// Fraction of vertices on the boundary.
#[allow(dead_code)]
pub fn boundary_fraction(result: &BoundaryVertexResult) -> f32 {
    if result.total_vertices == 0 {
        return 0.0;
    }
    result.boundary_vertices.len() as f32 / result.total_vertices as f32
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn boundary_vertex_to_json(result: &BoundaryVertexResult) -> String {
    format!(
        "{{\"boundary_count\":{},\"total\":{},\"fraction\":{:.6}}}",
        result.boundary_vertices.len(),
        result.total_vertices,
        boundary_fraction(result)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_edge_map() {
        let indices = vec![0, 1, 2];
        let map = build_edge_map(&indices);
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_find_boundary_edges_single_tri() {
        let indices = vec![0, 1, 2];
        let edges = find_boundary_edges(&indices);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_closed_mesh_no_boundary() {
        // Two triangles sharing an edge
        let indices = vec![0, 1, 2, 2, 1, 3];
        let edges = find_boundary_edges(&indices);
        // 5 edges total, only inner edge is shared
        assert_eq!(edges.len(), 4);
    }

    #[test]
    fn test_detect_boundary_vertices() {
        let pos = vec![[0.0; 3]; 3];
        let indices = vec![0, 1, 2];
        let result = detect_boundary_vertices(&pos, &indices);
        assert_eq!(boundary_count(&result), 3);
    }

    #[test]
    fn test_is_boundary() {
        let result = BoundaryVertexResult {
            boundary_vertices: vec![0, 2],
            total_vertices: 3,
        };
        assert!(is_boundary_vertex(&result, 0));
        assert!(!is_boundary_vertex(&result, 1));
    }

    #[test]
    fn test_boundary_fraction() {
        let result = BoundaryVertexResult {
            boundary_vertices: vec![0, 1],
            total_vertices: 4,
        };
        assert!((boundary_fraction(&result) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_empty() {
        let result = detect_boundary_vertices(&[], &[]);
        assert_eq!(boundary_count(&result), 0);
    }

    #[test]
    fn test_boundary_fraction_empty() {
        let result = BoundaryVertexResult {
            boundary_vertices: vec![],
            total_vertices: 0,
        };
        assert!((boundary_fraction(&result)).abs() < 1e-9);
    }

    #[test]
    fn test_to_json() {
        let result = BoundaryVertexResult {
            boundary_vertices: vec![0],
            total_vertices: 3,
        };
        let j = boundary_vertex_to_json(&result);
        assert!(j.contains("\"boundary_count\":1"));
    }

    #[test]
    fn test_sorted_output() {
        let pos = vec![[0.0; 3]; 4];
        let indices = vec![0, 1, 2];
        let result = detect_boundary_vertices(&pos, &indices);
        for w in result.boundary_vertices.windows(2) {
            assert!(w[0] < w[1]);
        }
    }
}
