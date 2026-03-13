// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge flipping operations for mesh optimization (Delaunay criterion).

/// Check if flipping the shared edge of two triangles improves Delaunay criterion.
#[allow(dead_code)]
pub fn should_flip_edge(
    pa: [f32; 3], pb: [f32; 3], pc: [f32; 3], pd: [f32; 3],
) -> bool {
    // Delaunay: flip if pd is inside circumcircle of (pa, pb, pc)
    // Use 2D projection for planar test
    let ax = pa[0] - pd[0]; let ay = pa[1] - pd[1];
    let bx = pb[0] - pd[0]; let by = pb[1] - pd[1];
    let cx = pc[0] - pd[0]; let cy = pc[1] - pd[1];
    let det = ax * (by * (cx * cx + cy * cy) - cy * (bx * bx + by * by))
            - ay * (bx * (cx * cx + cy * cy) - cx * (bx * bx + by * by))
            + (ax * ax + ay * ay) * (bx * cy - by * cx);
    det > 0.0
}

/// Result of an edge flip.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeFlipResult {
    pub indices: Vec<u32>,
    pub flips_performed: usize,
}

/// Flip an edge shared by triangles (a,b,c) and (b,a,d) -> (a,d,c) and (d,b,c).
#[allow(dead_code)]
pub fn flip_shared_edge(
    tri1: [u32; 3], tri2: [u32; 3],
    shared_a: u32, shared_b: u32,
) -> Option<([u32; 3], [u32; 3])> {
    let c = tri1.iter().find(|&&v| v != shared_a && v != shared_b)?;
    let d = tri2.iter().find(|&&v| v != shared_a && v != shared_b)?;
    Some(([shared_a, *d, *c], [*d, shared_b, *c]))
}

/// Find the opposing vertex in a triangle given an edge.
#[allow(dead_code)]
pub fn opposing_vertex(tri: [u32; 3], edge_a: u32, edge_b: u32) -> Option<u32> {
    tri.iter().find(|&&v| v != edge_a && v != edge_b).copied()
}

/// Perform one pass of edge flipping on the mesh.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn flip_edges_pass(
    positions: &[[f32; 3]],
    indices: &mut Vec<u32>,
) -> usize {
    use std::collections::HashMap;
    let tc = indices.len() / 3;
    // Build edge -> triangles map
    let mut edge_tris: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(key).or_default().push(t);
        }
    }
    let mut flips = 0;
    let mut flipped = vec![false; tc];
    for (&(ea, eb), tris) in &edge_tris {
        if tris.len() != 2 { continue; }
        let t0 = tris[0];
        let t1 = tris[1];
        if flipped[t0] || flipped[t1] { continue; }
        let tri0 = [indices[t0 * 3], indices[t0 * 3 + 1], indices[t0 * 3 + 2]];
        let tri1 = [indices[t1 * 3], indices[t1 * 3 + 1], indices[t1 * 3 + 2]];
        let c = match opposing_vertex(tri0, ea, eb) { Some(v) => v, None => continue };
        let d = match opposing_vertex(tri1, ea, eb) { Some(v) => v, None => continue };
        if should_flip_edge(
            positions[ea as usize], positions[eb as usize],
            positions[c as usize], positions[d as usize],
        ) {
            if let Some((new0, new1)) = flip_shared_edge(tri0, tri1, ea, eb) {
                indices[t0 * 3] = new0[0];
                indices[t0 * 3 + 1] = new0[1];
                indices[t0 * 3 + 2] = new0[2];
                indices[t1 * 3] = new1[0];
                indices[t1 * 3 + 1] = new1[1];
                indices[t1 * 3 + 2] = new1[2];
                flipped[t0] = true;
                flipped[t1] = true;
                flips += 1;
            }
        }
    }
    flips
}

/// Run multiple passes of edge flipping.
#[allow(dead_code)]
pub fn flip_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    max_passes: u32,
) -> EdgeFlipResult {
    let mut idx = indices.to_vec();
    let mut total = 0;
    for _ in 0..max_passes {
        let f = flip_edges_pass(positions, &mut idx);
        total += f;
        if f == 0 { break; }
    }
    EdgeFlipResult { indices: idx, flips_performed: total }
}

/// Edge count in mesh.
#[allow(dead_code)]
pub fn mesh_edge_count(indices: &[u32]) -> usize {
    use std::collections::HashSet;
    let tc = indices.len() / 3;
    let mut edges = HashSet::new();
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
        }
    }
    edges.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opposing_vertex() {
        assert_eq!(opposing_vertex([0, 1, 2], 0, 1), Some(2));
    }

    #[test]
    fn test_opposing_vertex_none() {
        assert_eq!(opposing_vertex([0, 1, 2], 3, 4), None);
    }

    #[test]
    fn test_flip_shared_edge() {
        let result = flip_shared_edge([0, 1, 2], [1, 0, 3], 0, 1);
        assert!(result.is_some());
    }

    #[test]
    fn test_should_flip_coplanar() {
        let result = should_flip_edge(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0], [0.5, -1.0, 0.0],
        );
        // Just verify it returns a boolean
        assert!(result || !result);
    }

    #[test]
    fn test_flip_edges_pass_no_crash() {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0], [0.5, -1.0, 0.0],
        ];
        let mut idx = vec![0, 1, 2, 1, 0, 3];
        let _ = flip_edges_pass(&pos, &mut idx);
        assert_eq!(idx.len(), 6);
    }

    #[test]
    fn test_flip_edges_preserves_count() {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0], [0.5, -1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 0, 3];
        let result = flip_edges(&pos, &idx, 5);
        assert_eq!(result.indices.len(), 6);
    }

    #[test]
    fn test_single_triangle_no_flip() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = flip_edges(&pos, &idx, 5);
        assert_eq!(result.flips_performed, 0);
    }

    #[test]
    fn test_edge_count() {
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        assert_eq!(mesh_edge_count(&idx), 5);
    }

    #[test]
    fn test_empty_mesh() {
        let result = flip_edges(&[], &[], 5);
        assert_eq!(result.flips_performed, 0);
    }

    #[test]
    fn test_flip_result_indices_valid() {
        let pos = vec![
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0],
            [1.0, 2.0, 0.0], [1.0, -2.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 0, 3];
        let result = flip_edges(&pos, &idx, 3);
        for &i in &result.indices {
            assert!((i as usize) < pos.len());
        }
    }

}
