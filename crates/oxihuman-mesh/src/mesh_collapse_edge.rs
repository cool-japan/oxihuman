// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge collapse utilities for mesh simplification.

/// An edge defined by two vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollapseEdgeV2 {
    pub v0: u32,
    pub v1: u32,
    pub cost: u32, // quantized cost (millionths)
}

/// Result after a series of edge collapses.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseResultV2 {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub collapsed_count: usize,
}

/// Compute edge length.
#[allow(dead_code)]
pub fn edge_length_v2(positions: &[[f32; 3]], v0: u32, v1: u32) -> f32 {
    let a = positions[v0 as usize];
    let b = positions[v1 as usize];
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Midpoint of an edge.
#[allow(dead_code)]
pub fn edge_midpoint_v2(positions: &[[f32; 3]], v0: u32, v1: u32) -> [f32; 3] {
    let a = positions[v0 as usize];
    let b = positions[v1 as usize];
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Collapse a single edge: replace v1 with v0, remove degenerate triangles.
#[allow(dead_code)]
pub fn collapse_single_edge(
    positions: &mut [[f32; 3]],
    indices: &mut Vec<u32>,
    v0: u32,
    v1: u32,
) -> bool {
    if v0 == v1 {
        return false;
    }
    // Move v0 to midpoint
    let mid = edge_midpoint_v2(positions, v0, v1);
    positions[v0 as usize] = mid;
    // Replace all v1 references with v0
    for idx in indices.iter_mut() {
        if *idx == v1 {
            *idx = v0;
        }
    }
    // Remove degenerate triangles (where two or more indices are the same)
    let tri_count = indices.len() / 3;
    let mut keep = Vec::with_capacity(tri_count * 3);
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        if a != b && b != c && c != a {
            keep.push(a);
            keep.push(b);
            keep.push(c);
        }
    }
    *indices = keep;
    true
}

/// Find the shortest edge in the mesh.
#[allow(dead_code)]
pub fn find_shortest_edge(positions: &[[f32; 3]], indices: &[u32]) -> Option<(u32, u32)> {
    let tri_count = indices.len() / 3;
    if tri_count == 0 {
        return None;
    }
    let mut best = (indices[0], indices[1]);
    let mut best_len = f32::MAX;
    for t in 0..tri_count {
        let edges = [
            (indices[t * 3], indices[t * 3 + 1]),
            (indices[t * 3 + 1], indices[t * 3 + 2]),
            (indices[t * 3 + 2], indices[t * 3]),
        ];
        for (a, b) in edges {
            let l = edge_length_v2(positions, a, b);
            if l < best_len {
                best_len = l;
                best = (a, b);
            }
        }
    }
    Some(best)
}

/// Count unique edges in the index buffer.
#[allow(dead_code)]
pub fn unique_edge_count(indices: &[u32]) -> usize {
    let mut edges = std::collections::HashSet::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        for (s, e) in [(a, b), (b, c), (c, a)] {
            let key = if s < e { (s, e) } else { (e, s) };
            edges.insert(key);
        }
    }
    edges.len()
}

/// Collapse the n shortest edges iteratively.
#[allow(dead_code)]
pub fn collapse_n_edges(positions: &[[f32; 3]], indices: &[u32], n: usize) -> CollapseResultV2 {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut collapsed = 0;
    for _ in 0..n {
        if let Some((v0, v1)) = find_shortest_edge(&pos, &idx) {
            if collapse_single_edge(&mut pos, &mut idx, v0, v1) {
                collapsed += 1;
            }
        } else {
            break;
        }
    }
    CollapseResultV2 {
        new_positions: pos,
        new_indices: idx,
        collapsed_count: collapsed,
    }
}

/// Convert collapse result to JSON.
#[allow(dead_code)]
pub fn collapse_result_to_json_v2(result: &CollapseResultV2) -> String {
    format!(
        "{{\"collapsed\":{},\"vertices\":{},\"triangles\":{}}}",
        result.collapsed_count,
        result.new_positions.len(),
        result.new_indices.len() / 3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn test_edge_length() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
        let l = edge_length_v2(&pos, 0, 1);
        assert!((l - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_edge_midpoint() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let m = edge_midpoint_v2(&pos, 0, 1);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_find_shortest_edge() {
        let (pos, idx) = two_tri_mesh();
        let edge = find_shortest_edge(&pos, &idx);
        assert!(edge.is_some());
    }

    #[test]
    fn test_find_shortest_empty() {
        assert!(find_shortest_edge(&[], &[]).is_none());
    }

    #[test]
    fn test_unique_edge_count() {
        let (_, idx) = two_tri_mesh();
        assert_eq!(unique_edge_count(&idx), 5);
    }

    #[test]
    fn test_collapse_single() {
        let (mut pos, mut idx) = two_tri_mesh();
        let ok = collapse_single_edge(&mut pos, &mut idx, 0, 1);
        assert!(ok);
    }

    #[test]
    fn test_collapse_n_edges() {
        let (pos, idx) = two_tri_mesh();
        let result = collapse_n_edges(&pos, &idx, 1);
        assert_eq!(result.collapsed_count, 1);
    }

    #[test]
    fn test_collapse_result_to_json() {
        let (pos, idx) = two_tri_mesh();
        let result = collapse_n_edges(&pos, &idx, 0);
        let json = collapse_result_to_json_v2(&result);
        assert!(json.contains("\"collapsed\":0"));
    }

    #[test]
    fn test_collapse_same_vertex() {
        let (mut pos, mut idx) = two_tri_mesh();
        assert!(!collapse_single_edge(&mut pos, &mut idx, 0, 0));
    }

    #[test]
    fn test_collapse_edge_struct() {
        let e = CollapseEdgeV2 {
            v0: 0,
            v1: 1,
            cost: 100,
        };
        assert_eq!(e.v0, 0);
        assert_eq!(e.cost, 100);
    }
}
