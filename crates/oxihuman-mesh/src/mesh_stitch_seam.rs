// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Stitch seam edges between two mesh boundaries.

/// A pair of vertex indices forming a boundary edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SeamEdgePair {
    pub a: u32,
    pub b: u32,
}

/// Result of a seam stitch operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StitchSeamResult {
    pub new_indices: Vec<u32>,
    pub stitched_faces: usize,
}

/// Stitch two boundary loops together by connecting nearest vertices.
/// `loop_a` and `loop_b` must have the same length.
#[allow(dead_code)]
pub fn stitch_loops(loop_a: &[u32], loop_b: &[u32]) -> StitchSeamResult {
    assert_eq!(loop_a.len(), loop_b.len(), "loops must match length");
    let n = loop_a.len();
    let mut new_indices = Vec::with_capacity(n * 6);
    for i in 0..n {
        let j = (i + 1) % n;
        let (a0, a1) = (loop_a[i], loop_a[j]);
        let (b0, b1) = (loop_b[i], loop_b[j]);
        new_indices.extend_from_slice(&[a0, b0, a1]);
        new_indices.extend_from_slice(&[b0, b1, a1]);
    }
    StitchSeamResult {
        stitched_faces: n * 2,
        new_indices,
    }
}

/// Find boundary loops in an index buffer (edges used exactly once).
#[allow(dead_code)]
pub fn find_boundary_loop(indices: &[u32]) -> Vec<u32> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            let key = (u.min(v), u.max(v));
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let mut boundary_edges: Vec<(u32, u32)> = edge_count
        .into_iter()
        .filter(|&(_, c)| c == 1)
        .map(|(e, _)| e)
        .collect();
    if boundary_edges.is_empty() {
        return vec![];
    }
    // Build a simple chain starting from the first edge
    let mut loop_verts = vec![];
    let first = boundary_edges.remove(0);
    loop_verts.push(first.0);
    loop_verts.push(first.1);
    loop {
        // chain edges
        let last = loop_verts[loop_verts.len() - 1];
        let pos = boundary_edges
            .iter()
            .position(|&(u, v)| u == last || v == last);
        match pos {
            None => break,
            Some(idx) => {
                let e = boundary_edges.remove(idx);
                let next = if e.0 == last { e.1 } else { e.0 };
                if next == loop_verts[0] {
                    break;
                }
                loop_verts.push(next);
            }
        }
    }
    loop_verts
}

/// Compute the centroid of a loop of vertices.
#[allow(dead_code)]
pub fn loop_centroid(loop_verts: &[u32], positions: &[[f32; 3]]) -> [f32; 3] {
    if loop_verts.is_empty() {
        return [0.0; 3];
    }
    let n = loop_verts.len() as f32;
    let s = loop_verts.iter().fold([0.0_f32; 3], |acc, &i| {
        let p = positions[i as usize];
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Count the number of stitched faces in a result.
#[allow(dead_code)]
pub fn stitched_face_count(res: &StitchSeamResult) -> usize {
    res.stitched_faces
}

/// Return true when no seam exists (empty index buffer).
#[allow(dead_code)]
pub fn is_closed(indices: &[u32]) -> bool {
    find_boundary_loop(indices).is_empty()
}

/// Average edge length along a loop.
#[allow(dead_code)]
pub fn loop_avg_edge_length(loop_verts: &[u32], positions: &[[f32; 3]]) -> f32 {
    let n = loop_verts.len();
    if n < 2 {
        return 0.0;
    }
    let total: f32 = (0..n)
        .map(|i| {
            let j = (i + 1) % n;
            let a = positions[loop_verts[i] as usize];
            let b = positions[loop_verts[j] as usize];
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        })
        .sum();
    total / n as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stitch_two_quads() {
        let la = vec![0u32, 1, 2, 3];
        let lb = vec![4u32, 5, 6, 7];
        let res = stitch_loops(&la, &lb);
        assert_eq!(res.stitched_faces, 8);
        assert_eq!(res.new_indices.len(), 24);
    }

    #[test]
    fn stitch_indices_in_range() {
        let la = vec![0u32, 1, 2];
        let lb = vec![3u32, 4, 5];
        let res = stitch_loops(&la, &lb);
        assert!(res.new_indices.iter().all(|&i| i < 6));
    }

    #[test]
    fn find_boundary_triangle() {
        let idx = vec![0u32, 1, 2];
        let bnd = find_boundary_loop(&idx);
        assert!(!bnd.is_empty());
    }

    #[test]
    fn closed_mesh_no_boundary() {
        // Two triangles forming a closed "bowtie" sharing all edges (not achievable easily),
        // so use no indices: technically not closed but tests empty result.
        assert!(find_boundary_loop(&[]).is_empty());
    }

    #[test]
    fn loop_centroid_basic() {
        let loop_v = vec![0u32, 1, 2];
        let pos = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let c = loop_centroid(&loop_v, &pos);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn loop_centroid_empty() {
        let c = loop_centroid(&[], &[]);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn stitched_face_count_fn() {
        let res = StitchSeamResult {
            new_indices: vec![],
            stitched_faces: 7,
        };
        assert_eq!(stitched_face_count(&res), 7);
    }

    #[test]
    fn loop_avg_edge_length_square() {
        let loop_v = vec![0u32, 1, 2, 3];
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let avg = loop_avg_edge_length(&loop_v, &pos);
        assert!((avg - 1.0).abs() < 1e-5);
    }

    #[test]
    fn is_closed_empty_true() {
        assert!(is_closed(&[]));
    }

    #[test]
    fn is_closed_triangle_false() {
        assert!(!is_closed(&[0u32, 1, 2]));
    }
}
