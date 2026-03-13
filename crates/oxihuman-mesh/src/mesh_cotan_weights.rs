// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cotangent Laplacian weights for mesh processing (smoothing, parameterization).

/// Cotangent weight for the edge (i, j) opposite vertex k in a triangle.
#[allow(dead_code)]
pub fn cotangent_weight_at(
    pi: [f32; 3], pj: [f32; 3], pk: [f32; 3],
) -> f32 {
    let ki = [pi[0] - pk[0], pi[1] - pk[1], pi[2] - pk[2]];
    let kj = [pj[0] - pk[0], pj[1] - pk[1], pj[2] - pk[2]];
    let dot = ki[0] * kj[0] + ki[1] * kj[1] + ki[2] * kj[2];
    let cross = [
        ki[1] * kj[2] - ki[2] * kj[1],
        ki[2] * kj[0] - ki[0] * kj[2],
        ki[0] * kj[1] - ki[1] * kj[0],
    ];
    let cross_len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if cross_len < 1e-12 {
        return 0.0;
    }
    dot / cross_len
}

/// Cotangent weight result for an edge.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CotanEdgeWeight {
    pub vertex_i: u32,
    pub vertex_j: u32,
    pub weight: f32,
}

/// Build all cotangent weights for edges in a triangle mesh.
#[allow(dead_code)]
pub fn build_cotan_weights(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<CotanEdgeWeight> {
    use std::collections::HashMap;
    let tc = indices.len() / 3;
    let mut edge_weights: HashMap<(u32, u32), f32> = HashMap::new();
    for t in 0..tc {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        // Edge (i0, i1) opposite i2
        let w01 = cotangent_weight_at(p0, p1, p2);
        // Edge (i1, i2) opposite i0
        let w12 = cotangent_weight_at(p1, p2, p0);
        // Edge (i2, i0) opposite i1
        let w20 = cotangent_weight_at(p2, p0, p1);
        for (a, b, w) in [(i0 as u32, i1 as u32, w01), (i1 as u32, i2 as u32, w12), (i2 as u32, i0 as u32, w20)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_weights.entry(key).or_insert(0.0) += w;
        }
    }
    edge_weights.into_iter().map(|((i, j), w)| CotanEdgeWeight {
        vertex_i: i, vertex_j: j, weight: w,
    }).collect()
}

/// Apply one step of cotangent Laplacian smoothing.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn cotan_smooth_step(
    positions: &mut [[f32; 3]],
    weights: &[CotanEdgeWeight],
    lambda: f32,
) {
    let n = positions.len();
    let mut laplacian = vec![[0.0f32; 3]; n];
    let mut weight_sum = vec![0.0f32; n];
    for cw in weights {
        let i = cw.vertex_i as usize;
        let j = cw.vertex_j as usize;
        let w = cw.weight.max(0.0);
        for k in 0..3 {
            laplacian[i][k] += w * (positions[j][k] - positions[i][k]);
            laplacian[j][k] += w * (positions[i][k] - positions[j][k]);
        }
        weight_sum[i] += w;
        weight_sum[j] += w;
    }
    for i in 0..n {
        if weight_sum[i] > 1e-12 {
            for k in 0..3 {
                positions[i][k] += lambda * laplacian[i][k] / weight_sum[i];
            }
        }
    }
}

/// Voronoi area at a vertex (for normalizing curvature).
#[allow(dead_code)]
pub fn vertex_voronoi_area(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex: usize,
) -> f32 {
    let tc = indices.len() / 3;
    let mut area = 0.0f32;
    for t in 0..tc {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 == vertex || i1 == vertex || i2 == vertex {
            area += triangle_area(positions[i0], positions[i1], positions[i2]) / 3.0;
        }
    }
    area
}

/// Count total edges.
#[allow(dead_code)]
pub fn cotan_edge_count(weights: &[CotanEdgeWeight]) -> usize {
    weights.len()
}

/// Average weight across all edges.
#[allow(dead_code)]
pub fn cotan_avg_weight(weights: &[CotanEdgeWeight]) -> f32 {
    if weights.is_empty() { return 0.0; }
    weights.iter().map(|w| w.weight).sum::<f32>() / weights.len() as f32
}

fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::f32::consts::PI;

    fn right_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], vec![0, 1, 2])
    }

    #[test]
    fn test_cotan_weight_right_angle() {
        let w = cotangent_weight_at([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        // cot(90) = 0
        assert!(w.abs() < 1e-5);
    }

    #[test]
    fn test_cotan_weight_45_deg() {
        let w = cotangent_weight_at([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        // cot(45) = 1
        assert!((w - 1.0).abs() < 1e-4);
        let _ = PI;
    }

    #[test]
    fn test_build_weights() {
        let (pos, idx) = right_tri();
        let weights = build_cotan_weights(&pos, &idx);
        assert_eq!(cotan_edge_count(&weights), 3);
    }

    #[test]
    fn test_smooth_step_no_crash() {
        let (mut pos, idx) = right_tri();
        let weights = build_cotan_weights(&pos, &idx);
        cotan_smooth_step(&mut pos, &weights, 0.5);
        assert_eq!(pos.len(), 3);
    }

    #[test]
    fn test_voronoi_area() {
        let (pos, idx) = right_tri();
        let a = vertex_voronoi_area(&pos, &idx, 0);
        assert!(a > 0.0);
    }

    #[test]
    fn test_avg_weight() {
        let (pos, idx) = right_tri();
        let weights = build_cotan_weights(&pos, &idx);
        let avg = cotan_avg_weight(&weights);
        assert!(avg.is_finite());
    }

    #[test]
    fn test_degenerate_triangle() {
        let w = cotangent_weight_at([0.0; 3], [0.0; 3], [0.0; 3]);
        assert!((w - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_edge_count_quad() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2, 0, 2, 3];
        let weights = build_cotan_weights(&pos, &idx);
        assert_eq!(cotan_edge_count(&weights), 5);
    }

    #[test]
    fn test_smooth_preserves_vertex_count() {
        let (mut pos, idx) = right_tri();
        let weights = build_cotan_weights(&pos, &idx);
        let n = pos.len();
        cotan_smooth_step(&mut pos, &weights, 0.1);
        assert_eq!(pos.len(), n);
    }

    #[test]
    fn test_empty_mesh() {
        let weights = build_cotan_weights(&[], &[]);
        assert!(weights.is_empty());
    }

}
