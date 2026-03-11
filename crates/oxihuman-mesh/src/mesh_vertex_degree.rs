// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vertex degree (valence) analysis for mesh vertices.

use std::collections::HashSet;

/// Degree information for all vertices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexDegreeResult {
    pub degrees: Vec<usize>,
}

/// Compute the degree (number of unique adjacent vertices) for each vertex.
#[allow(dead_code)]
pub fn compute_vertex_degrees(vertex_count: usize, indices: &[u32]) -> VertexDegreeResult {
    let mut adj: Vec<HashSet<u32>> = vec![HashSet::new(); vertex_count];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for k in 0..3 {
            let a = vs[k] as usize;
            let b = vs[(k + 1) % 3];
            if a < vertex_count {
                adj[a].insert(b);
            }
            let b_usize = b as usize;
            if b_usize < vertex_count {
                adj[b_usize].insert(vs[k]);
            }
        }
    }
    let degrees = adj.iter().map(|s| s.len()).collect();
    VertexDegreeResult { degrees }
}

/// Get degree of a specific vertex.
#[allow(dead_code)]
pub fn vertex_degree(result: &VertexDegreeResult, vertex: usize) -> usize {
    result.degrees.get(vertex).copied().unwrap_or(0)
}

/// Average vertex degree.
#[allow(dead_code)]
pub fn avg_degree(result: &VertexDegreeResult) -> f32 {
    if result.degrees.is_empty() {
        return 0.0;
    }
    let sum: usize = result.degrees.iter().sum();
    sum as f32 / result.degrees.len() as f32
}

/// Maximum degree.
#[allow(dead_code)]
pub fn max_degree(result: &VertexDegreeResult) -> usize {
    result.degrees.iter().copied().max().unwrap_or(0)
}

/// Minimum non-zero degree.
#[allow(dead_code)]
pub fn min_nonzero_degree(result: &VertexDegreeResult) -> usize {
    result
        .degrees
        .iter()
        .copied()
        .filter(|&d| d > 0)
        .min()
        .unwrap_or(0)
}

/// Count irregular vertices (degree != 6 for interior triangle mesh).
#[allow(dead_code)]
pub fn irregular_vertex_count(result: &VertexDegreeResult, regular_degree: usize) -> usize {
    result
        .degrees
        .iter()
        .filter(|&&d| d > 0 && d != regular_degree)
        .count()
}

/// Find all vertices with a specific degree.
#[allow(dead_code)]
pub fn vertices_with_degree(result: &VertexDegreeResult, degree: usize) -> Vec<usize> {
    result
        .degrees
        .iter()
        .enumerate()
        .filter(|(_, &d)| d == degree)
        .map(|(i, _)| i)
        .collect()
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn degree_result_to_json(result: &VertexDegreeResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"avg_degree\":{:.4},\"max_degree\":{}}}",
        result.degrees.len(),
        avg_degree(result),
        max_degree(result),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> Vec<u32> {
        vec![0, 1, 2]
    }
    fn two_tris() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_single_tri_degrees() {
        let r = compute_vertex_degrees(3, &single_tri());
        assert_eq!(vertex_degree(&r, 0), 2);
    }

    #[test]
    fn test_two_tris_shared_vertex() {
        let r = compute_vertex_degrees(4, &two_tris());
        assert_eq!(vertex_degree(&r, 1), 3); // connects to 0, 2, 3
    }

    #[test]
    fn test_avg_degree() {
        let r = compute_vertex_degrees(3, &single_tri());
        assert!((avg_degree(&r) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_degree() {
        let r = compute_vertex_degrees(4, &two_tris());
        assert_eq!(max_degree(&r), 3);
    }

    #[test]
    fn test_min_nonzero_degree() {
        let r = compute_vertex_degrees(4, &two_tris());
        assert_eq!(min_nonzero_degree(&r), 2);
    }

    #[test]
    fn test_irregular_vertex_count() {
        let r = compute_vertex_degrees(3, &single_tri());
        // regular_degree=2, all are regular
        assert_eq!(irregular_vertex_count(&r, 2), 0);
    }

    #[test]
    fn test_vertices_with_degree() {
        let r = compute_vertex_degrees(4, &two_tris());
        let v3 = vertices_with_degree(&r, 3);
        assert!(v3.contains(&1));
        assert!(v3.contains(&2));
    }

    #[test]
    fn test_empty_mesh() {
        let r = compute_vertex_degrees(0, &[]);
        assert_eq!(max_degree(&r), 0);
    }

    #[test]
    fn test_degree_result_to_json() {
        let r = compute_vertex_degrees(3, &single_tri());
        let json = degree_result_to_json(&r);
        assert!(json.contains("\"vertex_count\":3"));
    }

    #[test]
    fn test_vertex_degree_oob() {
        let r = compute_vertex_degrees(3, &single_tri());
        assert_eq!(vertex_degree(&r, 100), 0);
    }
}
