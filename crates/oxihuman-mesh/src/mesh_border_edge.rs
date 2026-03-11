// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Border edge detection and analysis for triangle meshes.

use std::collections::HashMap;

/// A directed edge represented as (start, end) vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectedEdge {
    pub start: u32,
    pub end: u32,
}

/// Result of border edge analysis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BorderResult {
    pub border_edges: Vec<[u32; 2]>,
    pub border_vertex_set: Vec<u32>,
}

/// Build a half-edge map: for each directed edge, count occurrences.
#[allow(dead_code)]
pub fn build_edge_count_map(indices: &[u32]) -> HashMap<(u32, u32), u32> {
    let mut map = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        for &(s, e) in &[(a, b), (b, c), (c, a)] {
            let key = if s < e { (s, e) } else { (e, s) };
            *map.entry(key).or_insert(0) += 1;
        }
    }
    map
}

/// Detect all border (boundary) edges -- edges shared by exactly one triangle.
#[allow(dead_code)]
pub fn detect_border_edges(indices: &[u32]) -> Vec<[u32; 2]> {
    let map = build_edge_count_map(indices);
    map.into_iter()
        .filter(|&(_, count)| count == 1)
        .map(|((a, b), _)| [a, b])
        .collect()
}

/// Full border analysis returning edges and unique border vertices.
#[allow(dead_code)]
pub fn border_analysis(indices: &[u32]) -> BorderResult {
    let edges = detect_border_edges(indices);
    let mut verts: Vec<u32> = edges.iter().flat_map(|e| e.iter().copied()).collect();
    verts.sort();
    verts.dedup();
    BorderResult {
        border_edges: edges,
        border_vertex_set: verts,
    }
}

/// Return the count of border edges.
#[allow(dead_code)]
pub fn border_edge_count(result: &BorderResult) -> usize {
    result.border_edges.len()
}

/// Return the count of unique border vertices.
#[allow(dead_code)]
pub fn border_vertex_count(result: &BorderResult) -> usize {
    result.border_vertex_set.len()
}

/// Check if a specific vertex is on the border.
#[allow(dead_code)]
pub fn is_border_vertex(result: &BorderResult, vertex: u32) -> bool {
    result.border_vertex_set.contains(&vertex)
}

/// Compute total border edge length.
#[allow(dead_code)]
pub fn border_total_length(positions: &[[f32; 3]], result: &BorderResult) -> f32 {
    result
        .border_edges
        .iter()
        .map(|e| {
            let a = positions[e[0] as usize];
            let b = positions[e[1] as usize];
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// Convert border result to JSON.
#[allow(dead_code)]
pub fn border_result_to_json(result: &BorderResult) -> String {
    format!(
        "{{\"border_edge_count\":{},\"border_vertex_count\":{}}}",
        result.border_edges.len(),
        result.border_vertex_set.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Single triangle => all 3 edges are border
    fn single_tri_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    // Two triangles sharing an edge => that edge is NOT border
    fn two_tri_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    fn positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_single_tri_all_border() {
        let edges = detect_border_edges(&single_tri_indices());
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_two_tris_shared_edge() {
        let result = border_analysis(&two_tri_indices());
        // shared edge (1,2) is not border; 4 edges are border
        assert_eq!(border_edge_count(&result), 4);
    }

    #[test]
    fn test_border_vertex_count() {
        let result = border_analysis(&single_tri_indices());
        assert_eq!(border_vertex_count(&result), 3);
    }

    #[test]
    fn test_is_border_vertex() {
        let result = border_analysis(&single_tri_indices());
        assert!(is_border_vertex(&result, 0));
        assert!(is_border_vertex(&result, 1));
        assert!(is_border_vertex(&result, 2));
    }

    #[test]
    fn test_empty_mesh() {
        let result = border_analysis(&[]);
        assert_eq!(border_edge_count(&result), 0);
        assert_eq!(border_vertex_count(&result), 0);
    }

    #[test]
    fn test_border_total_length() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = border_analysis(&single_tri_indices());
        let len = border_total_length(&pos, &result);
        assert!(len > 0.0);
    }

    #[test]
    fn test_border_result_to_json() {
        let result = border_analysis(&single_tri_indices());
        let json = border_result_to_json(&result);
        assert!(json.contains("\"border_edge_count\":3"));
    }

    #[test]
    fn test_build_edge_count_map() {
        let map = build_edge_count_map(&single_tri_indices());
        assert_eq!(map.len(), 3);
        for &v in map.values() {
            assert_eq!(v, 1);
        }
    }

    #[test]
    fn test_two_tris_border_vertices() {
        let result = border_analysis(&two_tri_indices());
        assert_eq!(border_vertex_count(&result), 4);
    }

    #[test]
    fn test_directed_edge_eq() {
        let e1 = DirectedEdge { start: 0, end: 1 };
        let e2 = DirectedEdge { start: 0, end: 1 };
        assert_eq!(e1, e2);
    }
}
