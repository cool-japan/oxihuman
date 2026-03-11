// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge bisection (midpoint subdivision) for triangle meshes.

/// Result of edge bisection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeBisectResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub original_vertex_count: usize,
}

/// Compute midpoint of two 3D points.
#[allow(dead_code)]
pub fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Bisect all edges of a triangle mesh, creating 4 triangles per original.
#[allow(dead_code)]
pub fn bisect_all_edges(positions: &[[f32; 3]], indices: &[u32]) -> EdgeBisectResult {
    use std::collections::HashMap;

    let original_count = positions.len();
    let mut new_positions = positions.to_vec();
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_indices = Vec::new();

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3];
        let i1 = indices[t * 3 + 1];
        let i2 = indices[t * 3 + 2];

        let m01 = get_or_create_mid(&mut new_positions, &mut edge_map, i0, i1, positions);
        let m12 = get_or_create_mid(&mut new_positions, &mut edge_map, i1, i2, positions);
        let m20 = get_or_create_mid(&mut new_positions, &mut edge_map, i2, i0, positions);

        new_indices.extend_from_slice(&[i0, m01, m20]);
        new_indices.extend_from_slice(&[m01, i1, m12]);
        new_indices.extend_from_slice(&[m20, m12, i2]);
        new_indices.extend_from_slice(&[m01, m12, m20]);
    }

    EdgeBisectResult {
        positions: new_positions,
        indices: new_indices,
        original_vertex_count: original_count,
    }
}

#[allow(dead_code)]
fn get_or_create_mid(
    positions: &mut Vec<[f32; 3]>,
    edge_map: &mut std::collections::HashMap<(u32, u32), u32>,
    a: u32,
    b: u32,
    orig: &[[f32; 3]],
) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = edge_map.get(&key) {
        return idx;
    }
    let mid = midpoint(orig[a as usize], orig[b as usize]);
    let idx = positions.len() as u32;
    positions.push(mid);
    edge_map.insert(key, idx);
    idx
}

/// Vertex count after bisection.
#[allow(dead_code)]
pub fn bisect_vertex_count(result: &EdgeBisectResult) -> usize {
    result.positions.len()
}

/// Triangle count after bisection.
#[allow(dead_code)]
pub fn bisect_triangle_count(result: &EdgeBisectResult) -> usize {
    result.indices.len() / 3
}

/// Count of new vertices added.
#[allow(dead_code)]
pub fn new_vertex_count(result: &EdgeBisectResult) -> usize {
    result.positions.len() - result.original_vertex_count
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn bisect_to_json(result: &EdgeBisectResult) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{},\"new_vertices\":{}}}",
        bisect_vertex_count(result),
        bisect_triangle_count(result),
        new_vertex_count(result)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midpoint() {
        let m = midpoint([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_bisect_single_triangle() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = bisect_all_edges(&pos, &idx);
        assert_eq!(bisect_vertex_count(&result), 6);
        assert_eq!(bisect_triangle_count(&result), 4);
    }

    #[test]
    fn test_new_vertex_count() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = bisect_all_edges(&pos, &idx);
        assert_eq!(new_vertex_count(&result), 3);
    }

    #[test]
    fn test_bisect_empty() {
        let result = bisect_all_edges(&[], &[]);
        assert_eq!(bisect_vertex_count(&result), 0);
        assert_eq!(bisect_triangle_count(&result), 0);
    }

    #[test]
    fn test_shared_edge_reuse() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let result = bisect_all_edges(&pos, &idx);
        assert_eq!(bisect_vertex_count(&result), 9);
        assert_eq!(bisect_triangle_count(&result), 8);
    }

    #[test]
    fn test_indices_valid() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = bisect_all_edges(&pos, &idx);
        let max_idx = result.positions.len() as u32;
        assert!(result.indices.iter().all(|&i| i < max_idx));
    }

    #[test]
    fn test_original_preserved() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = bisect_all_edges(&pos, &idx);
        for (i, p) in pos.iter().enumerate() {
            assert!((result.positions[i][0] - p[0]).abs() < 1e-9);
        }
    }

    #[test]
    fn test_to_json() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = bisect_all_edges(&pos, &idx);
        let j = bisect_to_json(&result);
        assert!(j.contains("\"vertices\":6"));
    }

    #[test]
    fn test_midpoint_position() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = bisect_all_edges(&pos, &idx);
        let has_mid = result
            .positions
            .iter()
            .any(|p| (p[0] - 1.0).abs() < 1e-6 && (p[1]).abs() < 1e-6);
        assert!(has_mid);
    }

    #[test]
    fn test_two_separate_triangles() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 3, 4, 5];
        let result = bisect_all_edges(&pos, &idx);
        assert_eq!(bisect_triangle_count(&result), 8);
    }
}
