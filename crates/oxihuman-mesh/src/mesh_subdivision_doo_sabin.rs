// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Doo-Sabin subdivision scheme.

use std::f32::consts::PI;

/// Result of a Doo-Sabin subdivision step.
#[allow(dead_code)]
pub struct DooSabinResult {
    pub positions: Vec<[f32; 3]>,
    pub face_sizes: Vec<usize>,
    pub indices: Vec<u32>,
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Doo-Sabin weight for index k in an n-sided polygon.
#[allow(dead_code)]
pub fn doo_sabin_weight(n: usize, k: usize) -> f32 {
    if n == 1 {
        return 1.0;
    }
    let nf = n as f32;
    let kf = k as f32;
    if k == 0 {
        (nf + 5.0) / (4.0 * nf)
    } else {
        (3.0 + 2.0 * (2.0 * PI * kf / nf).cos()) / (4.0 * nf)
    }
}

/// Compute the new vertex for corner i of an n-gon given its vertex positions.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn doo_sabin_new_vertex(verts: &[[f32; 3]], i: usize) -> [f32; 3] {
    let n = verts.len();
    let mut result = [0.0f32; 3];
    for j in 0..n {
        let k = (j + n - i) % n;
        result = add3(result, scale3(verts[j], doo_sabin_weight(n, k)));
    }
    result
}

/// Generate new face vertices for a single polygon.
#[allow(dead_code)]
pub fn doo_sabin_face_vertices(verts: &[[f32; 3]]) -> Vec<[f32; 3]> {
    (0..verts.len()).map(|i| doo_sabin_new_vertex(verts, i)).collect()
}

/// Check that weights for an n-gon sum to 1.
#[allow(dead_code)]
pub fn doo_sabin_weights_sum(n: usize) -> f32 {
    (0..n).map(|k| doo_sabin_weight(n, k)).sum()
}

/// Estimate output face count for a simple polygon mesh.
#[allow(dead_code)]
pub fn doo_sabin_face_count_estimate(
    polygon_count: usize,
    edge_count: usize,
    vertex_count: usize,
) -> usize {
    polygon_count + edge_count + vertex_count
}

/// Build a simple Doo-Sabin result for a single quad.
#[allow(dead_code)]
pub fn doo_sabin_single_quad(verts: &[[f32; 3]; 4]) -> DooSabinResult {
    let new_verts = doo_sabin_face_vertices(verts);
    let positions = new_verts;
    let n = 4;
    let face_sizes = vec![n];
    let indices = (0..n as u32).collect();
    DooSabinResult { positions, face_sizes, indices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_sum_to_one_quad() {
        let s = doo_sabin_weights_sum(4);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_sum_to_one_tri() {
        let s = doo_sabin_weights_sum(3);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_sum_to_one_hex() {
        let s = doo_sabin_weights_sum(6);
        assert!((s - 1.0).abs() < 1e-4);
    }

    #[test]
    fn center_weight_larger_than_opposite() {
        let w0 = doo_sabin_weight(4, 0);
        let w2 = doo_sabin_weight(4, 2);
        assert!(w0 > w2);
    }

    #[test]
    fn face_vertices_count_matches() {
        let verts = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]];
        let nv = doo_sabin_face_vertices(&verts);
        assert_eq!(nv.len(), 4);
    }

    #[test]
    fn new_vertex_within_polygon() {
        let verts = vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[2.0,2.0,0.0],[0.0,2.0,0.0]];
        let v = doo_sabin_new_vertex(&verts, 0);
        assert!(v[0] >= 0.0 && v[0] <= 2.0);
        assert!(v[1] >= 0.0 && v[1] <= 2.0);
    }

    #[test]
    fn single_quad_result_structure() {
        let verts = [[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]];
        let r = doo_sabin_single_quad(&verts);
        assert_eq!(r.positions.len(), 4);
        assert_eq!(r.indices.len(), 4);
    }

    #[test]
    fn face_count_estimate() {
        let fc = doo_sabin_face_count_estimate(6, 12, 8);
        assert_eq!(fc, 26);
    }

    #[test]
    fn pi_cos_used() {
        let _ = PI;
        let w = doo_sabin_weight(6, 1);
        assert!(w > 0.0);
    }
}
