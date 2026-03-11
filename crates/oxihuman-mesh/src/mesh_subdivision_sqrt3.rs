// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! √3-subdivision scheme (Kobbelt 2000).

use std::f32::consts::PI;

/// Result of one √3 subdivision pass.
#[allow(dead_code)]
pub struct Sqrt3SubdivResult {
    pub positions: Vec<[f32; 3]>,
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

/// Alpha weight for √3 scheme.
#[allow(dead_code)]
pub fn sqrt3_alpha(n: usize) -> f32 {
    let nf = n as f32;
    (4.0 - 2.0 * (2.0 * PI / nf).cos()) / 9.0
}

/// Compute centroid of a triangle.
#[allow(dead_code)]
pub fn triangle_centroid(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    scale3(add3(add3(p0, p1), p2), 1.0 / 3.0)
}

/// Updated vertex position for √3 scheme.
#[allow(dead_code)]
pub fn sqrt3_updated_vertex(center: [f32; 3], neighbours: &[[f32; 3]]) -> [f32; 3] {
    let n = neighbours.len();
    let alpha = sqrt3_alpha(n);
    let mut nb_sum = [0.0f32; 3];
    for nb in neighbours {
        nb_sum = add3(nb_sum, *nb);
    }
    let avg_nb = scale3(nb_sum, 1.0 / n as f32);
    add3(scale3(center, 1.0 - alpha), scale3(avg_nb, alpha))
}

/// Perform one √3 subdivision step (triangle split only, no flip step).
#[allow(dead_code)]
pub fn sqrt3_subdivide_step(positions: &[[f32; 3]], indices: &[u32]) -> Sqrt3SubdivResult {
    let mut new_pos = positions.to_vec();
    let mut new_idx: Vec<u32> = Vec::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[3 * t] as usize;
        let b = indices[3 * t + 1] as usize;
        let c = indices[3 * t + 2] as usize;
        let cen = triangle_centroid(positions[a], positions[b], positions[c]);
        let ci = new_pos.len() as u32;
        new_pos.push(cen);
        new_idx.extend_from_slice(&[a as u32, b as u32, ci]);
        new_idx.extend_from_slice(&[b as u32, c as u32, ci]);
        new_idx.extend_from_slice(&[c as u32, a as u32, ci]);
    }
    Sqrt3SubdivResult { positions: new_pos, indices: new_idx }
}

/// Face count after one √3 subdivision.
#[allow(dead_code)]
pub fn sqrt3_face_count(initial: usize) -> usize {
    initial * 3
}

/// Vertex count added per step.
#[allow(dead_code)]
pub fn sqrt3_new_vertices(face_count: usize) -> usize {
    face_count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], vec![0,1,2])
    }

    #[test]
    fn one_step_triples_faces() {
        let (p, i) = triangle();
        let r = sqrt3_subdivide_step(&p, &i);
        assert_eq!(r.indices.len(), 9);
    }

    #[test]
    fn centroid_correct() {
        let c = triangle_centroid([0.0,0.0,0.0],[3.0,0.0,0.0],[0.0,3.0,0.0]);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn alpha_positive() {
        let a = sqrt3_alpha(6);
        assert!(a > 0.0 && a < 1.0);
    }

    #[test]
    fn alpha_decreases_with_valence() {
        let a3 = sqrt3_alpha(3);
        let a8 = sqrt3_alpha(8);
        assert!(a3 > a8);
    }

    #[test]
    fn sqrt3_face_count_correct() {
        assert_eq!(sqrt3_face_count(4), 12);
    }

    #[test]
    fn vertex_count_grows() {
        let (p, i) = triangle();
        let r = sqrt3_subdivide_step(&p, &i);
        assert_eq!(r.positions.len(), p.len() + 1);
    }

    #[test]
    fn updated_vertex_no_change_on_same_neighbors() {
        let center = [1.0, 1.0, 0.0];
        let neighbors = vec![[1.0, 1.0, 0.0]; 6];
        let v = sqrt3_updated_vertex(center, &neighbors);
        assert!((v[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn new_vertices_equals_face_count() {
        assert_eq!(sqrt3_new_vertices(10), 10);
    }

    #[test]
    fn pi_usage_in_alpha() {
        let _ = PI;
        let a = sqrt3_alpha(4);
        assert!(a > 0.0);
    }
}
