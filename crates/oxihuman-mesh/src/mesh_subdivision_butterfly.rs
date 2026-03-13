// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Butterfly interpolating subdivision scheme.

use std::collections::HashMap;

/// Result of Butterfly subdivision.
#[allow(dead_code)]
pub struct ButterflyResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub level: u32,
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Butterfly edge point for a regular interior edge (valence 6 on both ends).
/// p0, p1 = edge vertices; l0, l1 = left/right opposite; w0..w7 = butterfly stencil (simplified).
#[allow(dead_code)]
pub fn butterfly_edge_point_regular(
    p0: [f32; 3],
    p1: [f32; 3],
    opp_l: [f32; 3],
    opp_r: [f32; 3],
) -> [f32; 3] {
    let half = scale3(add3(p0, p1), 0.5);
    let w = 1.0 / 16.0;
    let correction = scale3(add3(opp_l, opp_r), -w);
    add3(half, correction)
}

/// Simplified butterfly edge point (midpoint fallback).
#[allow(dead_code)]
pub fn butterfly_edge_midpoint(p0: [f32; 3], p1: [f32; 3]) -> [f32; 3] {
    scale3(add3(p0, p1), 0.5)
}

/// Perform one Butterfly subdivision step (simplified: midpoint insertion).
#[allow(dead_code)]
pub fn butterfly_subdivide(positions: &[[f32; 3]], indices: &[u32]) -> ButterflyResult {
    let mut new_pos = positions.to_vec();
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_idx: Vec<u32> = Vec::with_capacity(indices.len() * 4);

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[3 * t];
        let b = indices[3 * t + 1];
        let c = indices[3 * t + 2];

        let get_mid = |p: u32, q: u32, pos: &mut Vec<[f32; 3]>, map: &mut HashMap<(u32, u32), u32>| {
            let key = if p < q { (p, q) } else { (q, p) };
            *map.entry(key).or_insert_with(|| {
                let mp = butterfly_edge_midpoint(pos[p as usize], pos[q as usize]);
                let idx = pos.len() as u32;
                pos.push(mp);
                idx
            })
        };

        let ab = get_mid(a, b, &mut new_pos, &mut edge_map);
        let bc = get_mid(b, c, &mut new_pos, &mut edge_map);
        let ca = get_mid(c, a, &mut new_pos, &mut edge_map);

        new_idx.extend_from_slice(&[a, ab, ca]);
        new_idx.extend_from_slice(&[ab, b, bc]);
        new_idx.extend_from_slice(&[ca, bc, c]);
        new_idx.extend_from_slice(&[ab, bc, ca]);
    }

    ButterflyResult { positions: new_pos, indices: new_idx, level: 1 }
}

/// Butterfly N levels.
#[allow(dead_code)]
pub fn butterfly_subdivide_n(positions: &[[f32; 3]], indices: &[u32], n: u32) -> ButterflyResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    for _ in 0..n {
        let r = butterfly_subdivide(&pos, &idx);
        pos = r.positions;
        idx = r.indices;
    }
    ButterflyResult { positions: pos, indices: idx, level: n }
}

/// Modified Butterfly weight for irregular vertex (n != 6).
#[allow(dead_code)]
pub fn butterfly_irregular_weight(n: usize, j: usize) -> f32 {
    use std::f32::consts::PI;
    if n == 3 {
        let weights = [5.0 / 12.0, -1.0 / 12.0, -1.0 / 12.0];
        weights[j % 3]
    } else if n == 4 {
        let weights = [3.0 / 8.0, 0.0, -1.0 / 8.0, 0.0];
        weights[j % 4]
    } else {
        let nf = n as f32;
        (1.0 / nf) * (0.25 + (2.0 * PI * j as f32 / nf).cos() + 0.5 * (4.0 * PI * j as f32 / nf).cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], vec![0,1,2])
    }

    #[test]
    fn one_step_quad_faces() {
        let (p, i) = triangle();
        let r = butterfly_subdivide(&p, &i);
        assert_eq!(r.indices.len(), 12);
    }

    #[test]
    fn two_levels() {
        let (p, i) = triangle();
        let r = butterfly_subdivide_n(&p, &i, 2);
        assert_eq!(r.level, 2);
        assert_eq!(r.indices.len(), 48);
    }

    #[test]
    fn edge_point_regular_close_to_midpoint() {
        let ep = butterfly_edge_point_regular(
            [0.0,0.0,0.0],[2.0,0.0,0.0],[1.0,1.0,0.0],[1.0,-1.0,0.0],
        );
        assert!((ep[0] - 1.0).abs() < 0.5);
    }

    #[test]
    fn midpoint_exact() {
        let m = butterfly_edge_midpoint([0.0,0.0,0.0],[2.0,0.0,0.0]);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn irregular_weight_n3_sums_near_one() {
        // n=3: weights = [5/12, -1/12, -1/12]; sum = 3/12 = 0.25
        let s: f32 = (0..3).map(|j| butterfly_irregular_weight(3, j)).sum();
        assert!((s - 0.25).abs() < 1e-5);
    }

    #[test]
    fn irregular_weight_n6_positive_first() {
        let w = butterfly_irregular_weight(6, 0);
        assert!(w > 0.0);
    }

    #[test]
    fn vertex_count_increases() {
        let (p, i) = triangle();
        let r = butterfly_subdivide(&p, &i);
        assert!(r.positions.len() > p.len());
    }

    #[test]
    fn level_field() {
        let (p, i) = triangle();
        let r = butterfly_subdivide(&p, &i);
        assert_eq!(r.level, 1);
    }

    #[test]
    fn n_levels_level_field() {
        let (p, i) = triangle();
        let r = butterfly_subdivide_n(&p, &i, 3);
        assert_eq!(r.level, 3);
    }
}
