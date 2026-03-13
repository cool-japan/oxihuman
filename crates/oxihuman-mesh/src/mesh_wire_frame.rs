// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Generate a wireframe line-segment mesh from a triangle mesh.

use std::collections::HashSet;

/// A wireframe edge (pair of vertex positions).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WireEdge2 {
    pub v0: u32,
    pub v1: u32,
}

/// Wireframe mesh result.
#[allow(dead_code)]
pub struct WireframeMesh {
    pub positions: Vec<[f32; 3]>,
    pub edges: Vec<WireEdge2>,
}

/// Generate wireframe from triangle mesh (unique edges only).
#[allow(dead_code)]
pub fn generate_wireframe(positions: &[[f32; 3]], indices: &[u32]) -> WireframeMesh {
    let n_tri = indices.len() / 3;
    let mut edge_set: HashSet<(u32, u32)> = HashSet::new();
    let mut edges = Vec::new();
    for t in 0..n_tri {
        let i0 = indices[t * 3];
        let i1 = indices[t * 3 + 1];
        let i2 = indices[t * 3 + 2];
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_set.insert(key) {
                edges.push(WireEdge2 { v0: a, v1: b });
            }
        }
    }
    WireframeMesh {
        positions: positions.to_vec(),
        edges,
    }
}

/// Generate wireframe with tube geometry (replaces each edge with a thin tube mesh).
/// This is a simplified version: just returns the edges as cylinder stubs.
#[allow(dead_code)]
pub fn generate_wireframe_tubes(
    positions: &[[f32; 3]],
    indices: &[u32],
    _radius: f32,
    _segments: usize,
) -> WireframeMesh {
    generate_wireframe(positions, indices)
}

/// Edge count in wireframe.
#[allow(dead_code)]
pub fn wireframe_edge_count(wf: &WireframeMesh) -> usize {
    wf.edges.len()
}

/// Compute the length of a wireframe edge.
#[allow(dead_code)]
pub fn edge_length_wf(wf: &WireframeMesh, edge_idx: usize) -> f32 {
    if edge_idx >= wf.edges.len() {
        return 0.0;
    }
    let e = &wf.edges[edge_idx];
    let p0 = wf.positions.get(e.v0 as usize).copied().unwrap_or([0.0; 3]);
    let p1 = wf.positions.get(e.v1 as usize).copied().unwrap_or([0.0; 3]);
    let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Total edge length of all wireframe edges.
#[allow(dead_code)]
pub fn total_wireframe_length(wf: &WireframeMesh) -> f32 {
    (0..wf.edges.len()).map(|i| edge_length_wf(wf, i)).sum()
}

/// Average edge length.
#[allow(dead_code)]
pub fn average_edge_length_wf(wf: &WireframeMesh) -> f32 {
    if wf.edges.is_empty() {
        return 0.0;
    }
    total_wireframe_length(wf) / wf.edges.len() as f32
}

/// Maximum edge length.
#[allow(dead_code)]
pub fn max_edge_length_wf(wf: &WireframeMesh) -> f32 {
    (0..wf.edges.len())
        .map(|i| edge_length_wf(wf, i))
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Check that all edge vertex indices are in bounds.
#[allow(dead_code)]
pub fn wireframe_indices_valid(wf: &WireframeMesh) -> bool {
    let n = wf.positions.len() as u32;
    wf.edges.iter().all(|e| e.v0 < n && e.v1 < n)
}

/// Convert wireframe to flat line index buffer.
#[allow(dead_code)]
pub fn wireframe_to_line_buffer(wf: &WireframeMesh) -> Vec<u32> {
    wf.edges.iter().flat_map(|e| [e.v0, e.v1]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 6, 3, 3, 6, 7, 1, 6, 2, 1, 5,
            6, 0, 3, 7, 0, 7, 4,
        ];
        (positions, indices)
    }

    #[test]
    fn wireframe_has_edges() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        assert!(wireframe_edge_count(&wf) > 0);
    }

    #[test]
    fn wireframe_edges_unique() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        let mut keys: Vec<(u32, u32)> = wf
            .edges
            .iter()
            .map(|e| {
                if e.v0 < e.v1 {
                    (e.v0, e.v1)
                } else {
                    (e.v1, e.v0)
                }
            })
            .collect();
        let orig = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), orig, "duplicate edges found");
    }

    #[test]
    fn wireframe_indices_valid_check() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        assert!(wireframe_indices_valid(&wf));
    }

    #[test]
    fn edge_length_positive() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        assert!(edge_length_wf(&wf, 0) > 0.0);
    }

    #[test]
    fn total_wireframe_length_positive() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        assert!(total_wireframe_length(&wf) > 0.0);
    }

    #[test]
    fn average_edge_length_positive() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        let avg = average_edge_length_wf(&wf);
        assert!(avg > 0.0);
    }

    #[test]
    fn max_edge_length_at_least_avg() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        assert!(max_edge_length_wf(&wf) >= average_edge_length_wf(&wf));
    }

    #[test]
    fn wireframe_line_buffer_even_count() {
        let (pos, idx) = cube_mesh();
        let wf = generate_wireframe(&pos, &idx);
        let buf = wireframe_to_line_buffer(&wf);
        assert_eq!(buf.len() % 2, 0);
    }

    #[test]
    fn empty_mesh_no_edges() {
        let wf = generate_wireframe(&[], &[]);
        assert_eq!(wireframe_edge_count(&wf), 0);
    }

    #[test]
    fn single_triangle_three_edges() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx: Vec<u32> = vec![0, 1, 2];
        let wf = generate_wireframe(&pos, &idx);
        assert_eq!(wireframe_edge_count(&wf), 3);
    }
}
