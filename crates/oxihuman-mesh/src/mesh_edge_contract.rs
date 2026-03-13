// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge contraction operations for mesh simplification.

/// Result of an edge contraction.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeContractResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub contracted_count: usize,
}

/// Compute edge length between two vertices.
#[allow(dead_code)]
pub fn edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Find the shortest edge in a mesh, returns (vertex_a, vertex_b, length).
#[allow(dead_code)]
pub fn find_shortest_edge(positions: &[[f32; 3]], indices: &[u32]) -> Option<(u32, u32, f32)> {
    let tri_count = indices.len() / 3;
    let mut best: Option<(u32, u32, f32)> = None;
    for t in 0..tri_count {
        let verts = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for &(a, b) in &[
            (verts[0], verts[1]),
            (verts[1], verts[2]),
            (verts[2], verts[0]),
        ] {
            let len = edge_length(positions[a as usize], positions[b as usize]);
            if best.is_none_or(|(_, _, bl)| len < bl) {
                let key = if a < b { (a, b) } else { (b, a) };
                best = Some((key.0, key.1, len));
            }
        }
    }
    best
}

/// Contract a single edge by merging vertex `b` into vertex `a`.
#[allow(dead_code)]
pub fn contract_edge(
    positions: &[[f32; 3]],
    indices: &[u32],
    a: u32,
    b: u32,
) -> EdgeContractResult {
    let mid = [
        (positions[a as usize][0] + positions[b as usize][0]) * 0.5,
        (positions[a as usize][1] + positions[b as usize][1]) * 0.5,
        (positions[a as usize][2] + positions[b as usize][2]) * 0.5,
    ];
    let mut new_pos = positions.to_vec();
    new_pos[a as usize] = mid;

    let mut new_indices = Vec::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let mut tri = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for v in &mut tri {
            if *v == b {
                *v = a;
            }
        }
        // Skip degenerate triangles
        if tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2] {
            new_indices.extend_from_slice(&tri);
        }
    }

    EdgeContractResult {
        positions: new_pos,
        indices: new_indices,
        contracted_count: 1,
    }
}

/// Contract N shortest edges.
#[allow(dead_code)]
pub fn contract_n_edges(positions: &[[f32; 3]], indices: &[u32], n: usize) -> EdgeContractResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut total = 0;
    for _ in 0..n {
        if let Some((a, b, _)) = find_shortest_edge(&pos, &idx) {
            let result = contract_edge(&pos, &idx, a, b);
            pos = result.positions;
            idx = result.indices;
            total += 1;
        } else {
            break;
        }
    }
    EdgeContractResult {
        positions: pos,
        indices: idx,
        contracted_count: total,
    }
}

/// Triangle count.
#[allow(dead_code)]
pub fn triangle_count(result: &EdgeContractResult) -> usize {
    result.indices.len() / 3
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn contract_to_json(result: &EdgeContractResult) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{},\"contracted\":{}}}",
        result.positions.len(),
        triangle_count(result),
        result.contracted_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn test_edge_length() {
        let l = edge_length([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_find_shortest() {
        let (pos, idx) = simple_mesh();
        let result = find_shortest_edge(&pos, &idx);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_shortest_empty() {
        assert!(find_shortest_edge(&[], &[]).is_none());
    }

    #[test]
    fn test_contract_edge() {
        let (pos, idx) = simple_mesh();
        let result = contract_edge(&pos, &idx, 0, 1);
        assert_eq!(result.contracted_count, 1);
    }

    #[test]
    fn test_contract_removes_degenerate() {
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = contract_edge(&pos, &idx, 0, 1);
        // After contracting 0 and 1, triangle becomes degenerate
        assert_eq!(triangle_count(&result), 0);
    }

    #[test]
    fn test_contract_n() {
        let (pos, idx) = simple_mesh();
        let result = contract_n_edges(&pos, &idx, 1);
        assert_eq!(result.contracted_count, 1);
    }

    #[test]
    fn test_contract_n_zero() {
        let (pos, idx) = simple_mesh();
        let result = contract_n_edges(&pos, &idx, 0);
        assert_eq!(result.contracted_count, 0);
    }

    #[test]
    fn test_triangle_count() {
        let result = EdgeContractResult {
            positions: vec![[0.0; 3]; 3],
            indices: vec![0, 1, 2],
            contracted_count: 0,
        };
        assert_eq!(triangle_count(&result), 1);
    }

    #[test]
    fn test_to_json() {
        let result = EdgeContractResult {
            positions: vec![[0.0; 3]; 3],
            indices: vec![0, 1, 2],
            contracted_count: 1,
        };
        let j = contract_to_json(&result);
        assert!(j.contains("\"contracted\":1"));
    }

    #[test]
    fn test_midpoint_placement() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = contract_edge(&pos, &idx, 0, 1);
        assert!((result.positions[0][0] - 1.0).abs() < 1e-6);
    }
}
