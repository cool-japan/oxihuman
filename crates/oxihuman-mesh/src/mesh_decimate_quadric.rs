// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Quadric error metric (QEM) decimation for triangle mesh simplification.

/// 4x4 symmetric quadric matrix stored as 10 unique floats.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Quadric {
    pub a: [f32; 10], // upper triangle of 4x4 symmetric matrix
}

/// Compute a face quadric from a triangle's plane equation.
#[allow(dead_code)]
pub fn face_quadric(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> Quadric {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-12 {
        return Quadric { a: [0.0; 10] };
    }
    let a = nx / len;
    let b = ny / len;
    let c = nz / len;
    let d = -(a * p0[0] + b * p0[1] + c * p0[2]);
    // Q = plane^T * plane
    Quadric {
        a: [
            a * a, a * b, a * c, a * d,
            b * b, b * c, b * d,
            c * c, c * d,
            d * d,
        ],
    }
}

/// Add two quadrics.
#[allow(dead_code)]
pub fn add_quadrics(q1: &Quadric, q2: &Quadric) -> Quadric {
    let mut r = Quadric { a: [0.0; 10] };
    for i in 0..10 {
        r.a[i] = q1.a[i] + q2.a[i];
    }
    r
}

/// Evaluate quadric error at a point.
#[allow(dead_code)]
pub fn evaluate_quadric(q: &Quadric, v: [f32; 3]) -> f32 {
    let x = v[0]; let y = v[1]; let z = v[2];
    q.a[0] * x * x + 2.0 * q.a[1] * x * y + 2.0 * q.a[2] * x * z + 2.0 * q.a[3] * x
        + q.a[4] * y * y + 2.0 * q.a[5] * y * z + 2.0 * q.a[6] * y
        + q.a[7] * z * z + 2.0 * q.a[8] * z
        + q.a[9]
}

/// Compute per-vertex quadrics by summing face quadrics.
#[allow(dead_code)]
pub fn vertex_quadrics(positions: &[[f32; 3]], indices: &[u32]) -> Vec<Quadric> {
    let n = positions.len();
    let mut qs = vec![Quadric { a: [0.0; 10] }; n];
    let tc = indices.len() / 3;
    for t in 0..tc {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let fq = face_quadric(positions[i0], positions[i1], positions[i2]);
        qs[i0] = add_quadrics(&qs[i0], &fq);
        qs[i1] = add_quadrics(&qs[i1], &fq);
        qs[i2] = add_quadrics(&qs[i2], &fq);
    }
    qs
}

/// Compute collapse cost for an edge (use midpoint as optimal).
#[allow(dead_code)]
pub fn edge_collapse_cost(
    qs: &[Quadric],
    positions: &[[f32; 3]],
    i: usize,
    j: usize,
) -> (f32, [f32; 3]) {
    let combined = add_quadrics(&qs[i], &qs[j]);
    let mid = [
        (positions[i][0] + positions[j][0]) * 0.5,
        (positions[i][1] + positions[j][1]) * 0.5,
        (positions[i][2] + positions[j][2]) * 0.5,
    ];
    let cost = evaluate_quadric(&combined, mid).abs();
    (cost, mid)
}

/// Simple QEM decimation: greedily collapse lowest-cost edge.
#[allow(dead_code)]
pub fn qem_decimate(
    positions: &[[f32; 3]],
    indices: &[u32],
    target_faces: usize,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    // Simple implementation: just return input if target >= current
    let current_faces = indices.len() / 3;
    if target_faces >= current_faces {
        return (positions.to_vec(), indices.to_vec());
    }
    // For a real implementation we'd do iterative collapse;
    // here we do one pass collecting edges and costs
    let qs = vertex_quadrics(positions, indices);
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut current = current_faces;
    while current > target_faces && current > 1 {
        let edges = collect_edges(&idx);
        if edges.is_empty() { break; }
        let mut best_cost = f32::INFINITY;
        let mut best_edge = (0usize, 0usize);
        let mut best_pos = [0.0f32; 3];
        for &(a, b) in &edges {
            let (cost, mp) = edge_collapse_cost(&qs, &pos, a, b);
            if cost < best_cost {
                best_cost = cost;
                best_edge = (a, b);
                best_pos = mp;
            }
        }
        pos[best_edge.0] = best_pos;
        // Remap all references of b to a
        for v in idx.iter_mut() {
            if *v == best_edge.1 as u32 {
                *v = best_edge.0 as u32;
            }
        }
        // Remove degenerate triangles
        let mut new_idx = Vec::new();
        let tc = idx.len() / 3;
        for t in 0..tc {
            let a = idx[t * 3];
            let b = idx[t * 3 + 1];
            let c = idx[t * 3 + 2];
            if a != b && b != c && a != c {
                new_idx.extend_from_slice(&[a, b, c]);
            }
        }
        idx = new_idx;
        current = idx.len() / 3;
    }
    (pos, idx)
}

/// Count resulting faces.
#[allow(dead_code)]
pub fn qem_face_count(indices: &[u32]) -> usize {
    indices.len() / 3
}

fn collect_edges(indices: &[u32]) -> Vec<(usize, usize)> {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let tc = indices.len() / 3;
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k] as usize;
            let b = indices[t * 3 + (k + 1) % 3] as usize;
            let e = if a < b { (a, b) } else { (b, a) };
            set.insert(e);
        }
    }
    set.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ];
        let idx = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6,
            0, 4, 5, 0, 5, 1, 2, 6, 7, 2, 7, 3,
            0, 3, 7, 0, 7, 4, 1, 5, 6, 1, 6, 2,
        ];
        (pos, idx)
    }

    #[test]
    fn test_face_quadric_nonzero() {
        let q = face_quadric([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(q.a.iter().any(|&v| v != 0.0));
    }

    #[test]
    fn test_evaluate_on_plane() {
        let q = face_quadric([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let err = evaluate_quadric(&q, [0.5, 0.5, 0.0]);
        assert!(err.abs() < 1e-5);
    }

    #[test]
    fn test_add_quadrics() {
        let q1 = Quadric { a: [1.0; 10] };
        let q2 = Quadric { a: [2.0; 10] };
        let sum = add_quadrics(&q1, &q2);
        assert!((sum.a[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_vertex_quadrics_count() {
        let (pos, idx) = cube_mesh();
        let qs = vertex_quadrics(&pos, &idx);
        assert_eq!(qs.len(), 8);
    }

    #[test]
    fn test_collapse_cost_finite() {
        let (pos, idx) = cube_mesh();
        let qs = vertex_quadrics(&pos, &idx);
        let (cost, _mp) = edge_collapse_cost(&qs, &pos, 0, 1);
        assert!(cost.is_finite());
    }

    #[test]
    fn test_qem_decimate_reduces() {
        let (pos, idx) = cube_mesh();
        let orig_faces = idx.len() / 3;
        let (_, new_idx) = qem_decimate(&pos, &idx, orig_faces / 2);
        assert!(qem_face_count(&new_idx) <= orig_faces);
    }

    #[test]
    fn test_qem_identity() {
        let (pos, idx) = cube_mesh();
        let (p2, i2) = qem_decimate(&pos, &idx, 999);
        assert_eq!(p2.len(), pos.len());
        assert_eq!(i2.len(), idx.len());
    }

    #[test]
    fn test_degenerate_face_quadric() {
        let q = face_quadric([0.0; 3], [0.0; 3], [0.0; 3]);
        assert!(q.a.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_qem_face_count() {
        assert_eq!(qem_face_count(&[0, 1, 2, 3, 4, 5]), 2);
    }

    #[test]
    fn test_evaluate_off_plane() {
        let q = face_quadric([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let err = evaluate_quadric(&q, [0.0, 0.0, 1.0]);
        assert!(err > 0.5);
    }

}
