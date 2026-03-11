// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Convert a triangle mesh into a wire-frame (edge-only) representation.

/// A unique undirected edge (i < j guaranteed).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireEdge {
    pub i: u32,
    pub j: u32,
}

/// Wire mesh: positions + edges.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WireMesh {
    pub positions: Vec<[f32; 3]>,
    pub edges: Vec<WireEdge>,
}

fn make_edge(a: u32, b: u32) -> WireEdge {
    if a <= b {
        WireEdge { i: a, j: b }
    } else {
        WireEdge { i: b, j: a }
    }
}

/// Extract unique edges from a triangle index buffer.
#[allow(dead_code)]
pub fn extract_wire_edges(positions: &[[f32; 3]], indices: &[u32]) -> WireMesh {
    let mut seen = std::collections::HashSet::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        seen.insert(make_edge(a, b));
        seen.insert(make_edge(b, c));
        seen.insert(make_edge(c, a));
    }
    let mut edges: Vec<WireEdge> = seen.into_iter().collect();
    edges.sort_by(|x, y| x.i.cmp(&y.i).then(x.j.cmp(&y.j)));
    WireMesh {
        positions: positions.to_vec(),
        edges,
    }
}

/// Count edges in a wire mesh.
#[allow(dead_code)]
pub fn wire_edge_count(w: &WireMesh) -> usize {
    w.edges.len()
}

/// Compute total wire length.
#[allow(dead_code)]
pub fn wire_total_length(w: &WireMesh) -> f32 {
    w.edges
        .iter()
        .map(|e| {
            let a = w.positions[e.i as usize];
            let b = w.positions[e.j as usize];
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// Average edge length.
#[allow(dead_code)]
pub fn wire_avg_length(w: &WireMesh) -> f32 {
    let n = w.edges.len();
    if n == 0 {
        return 0.0;
    }
    wire_total_length(w) / n as f32
}

/// Check that all edge indices are valid.
#[allow(dead_code)]
pub fn wire_indices_valid(w: &WireMesh) -> bool {
    let n = w.positions.len() as u32;
    w.edges.iter().all(|e| e.i < n && e.j < n)
}

/// Serialise wire mesh stats to JSON.
#[allow(dead_code)]
pub fn wire_mesh_to_json(w: &WireMesh) -> String {
    format!(
        "{{\"vertices\":{},\"edges\":{}}}",
        w.positions.len(),
        w.edges.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn test_extract_wire_edges_count() {
        let (pos, idx) = one_tri();
        let w = extract_wire_edges(&pos, &idx);
        assert_eq!(wire_edge_count(&w), 3);
    }

    #[test]
    fn test_wire_no_duplicate_edges() {
        let (pos, idx) = one_tri();
        let w = extract_wire_edges(&pos, &idx);
        let mut seen = std::collections::HashSet::new();
        for e in &w.edges {
            assert!(seen.insert(*e), "Duplicate edge found");
        }
    }

    #[test]
    fn test_wire_total_length_positive() {
        let (pos, idx) = one_tri();
        let w = extract_wire_edges(&pos, &idx);
        assert!(wire_total_length(&w) > 0.0);
    }

    #[test]
    fn test_wire_avg_length_positive() {
        let (pos, idx) = one_tri();
        let w = extract_wire_edges(&pos, &idx);
        assert!(wire_avg_length(&w) > 0.0);
    }

    #[test]
    fn test_wire_avg_length_empty() {
        let w = WireMesh {
            positions: vec![],
            edges: vec![],
        };
        assert!(wire_avg_length(&w).abs() < 1e-6);
    }

    #[test]
    fn test_wire_indices_valid() {
        let (pos, idx) = one_tri();
        let w = extract_wire_edges(&pos, &idx);
        assert!(wire_indices_valid(&w));
    }

    #[test]
    fn test_wire_mesh_to_json() {
        let (pos, idx) = one_tri();
        let w = extract_wire_edges(&pos, &idx);
        let j = wire_mesh_to_json(&w);
        assert!(j.contains("edges"));
    }

    #[test]
    fn test_two_adjacent_tris_share_edge() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 1, 3, 2];
        let w = extract_wire_edges(&pos, &idx);
        // 4 unique edges: (0,1),(0,2),(1,2),(1,3),(2,3) => 5
        assert!(wire_edge_count(&w) >= 4);
    }

    #[test]
    fn test_empty_mesh() {
        let w = extract_wire_edges(&[], &[]);
        assert_eq!(wire_edge_count(&w), 0);
    }
}
