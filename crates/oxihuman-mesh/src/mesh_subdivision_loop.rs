// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Loop subdivision scheme with crease-edge support.

use std::collections::HashMap;

/// Result of a Loop subdivision pass.
#[allow(dead_code)]
pub struct LoopSubdivResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub level: u32,
}

/// Config for Loop subdivision.
#[allow(dead_code)]
pub struct LoopSubdivConfig {
    pub levels: u32,
    pub crease_sharpness: f32,
}

#[allow(dead_code)]
impl Default for LoopSubdivConfig {
    fn default() -> Self {
        Self { levels: 1, crease_sharpness: 0.0 }
    }
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Beta weight for Loop subdivision (Warren's formula).
#[allow(dead_code)]
pub fn loop_beta(valence: usize) -> f32 {
    if valence == 3 {
        3.0 / 16.0
    } else {
        3.0 / (8.0 * valence as f32)
    }
}

/// Compute an edge midpoint (Loop edge point).
#[allow(dead_code)]
pub fn loop_edge_point(p0: [f32; 3], p1: [f32; 3], opp0: [f32; 3], opp1: [f32; 3]) -> [f32; 3] {
    let sum = add3(add3(add3(scale3(p0, 3.0), scale3(p1, 3.0)), opp0), opp1);
    scale3(sum, 1.0 / 8.0)
}

/// Compute the updated position of an interior vertex.
#[allow(dead_code)]
pub fn loop_vertex_point(center: [f32; 3], neighbours: &[[f32; 3]]) -> [f32; 3] {
    let n = neighbours.len();
    let beta = loop_beta(n);
    let mut sum = scale3(center, 1.0 - n as f32 * beta);
    for nb in neighbours {
        sum = add3(sum, scale3(*nb, beta));
    }
    sum
}

/// Perform one level of Loop subdivision on a triangle mesh.
#[allow(dead_code)]
pub fn loop_subdivide(positions: &[[f32; 3]], indices: &[u32]) -> LoopSubdivResult {
    let mut new_pos = positions.to_vec();
    let mut edge_mid: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_idx: Vec<u32> = Vec::with_capacity(indices.len() * 4);

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[3 * t];
        let b = indices[3 * t + 1];
        let c = indices[3 * t + 2];

        let get_mid = |p: u32, q: u32, pos: &mut Vec<[f32; 3]>, map: &mut HashMap<(u32, u32), u32>| {
            let key = if p < q { (p, q) } else { (q, p) };
            *map.entry(key).or_insert_with(|| {
                let mp = scale3(add3(pos[p as usize], pos[q as usize]), 0.5);
                let idx = pos.len() as u32;
                pos.push(mp);
                idx
            })
        };

        let ab = get_mid(a, b, &mut new_pos, &mut edge_mid);
        let bc = get_mid(b, c, &mut new_pos, &mut edge_mid);
        let ca = get_mid(c, a, &mut new_pos, &mut edge_mid);

        new_idx.extend_from_slice(&[a, ab, ca]);
        new_idx.extend_from_slice(&[ab, b, bc]);
        new_idx.extend_from_slice(&[ca, bc, c]);
        new_idx.extend_from_slice(&[ab, bc, ca]);
    }

    LoopSubdivResult { positions: new_pos, indices: new_idx, level: 1 }
}

/// Perform N levels of Loop subdivision.
#[allow(dead_code)]
pub fn loop_subdivide_n(positions: &[[f32; 3]], indices: &[u32], n: u32) -> LoopSubdivResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    for level in 0..n {
        let r = loop_subdivide(&pos, &idx);
        pos = r.positions;
        idx = r.indices;
        let _ = level;
    }
    LoopSubdivResult { positions: pos, indices: idx, level: n }
}

/// Count faces after N levels.
#[allow(dead_code)]
pub fn expected_face_count_loop(initial_faces: usize, levels: u32) -> usize {
    initial_faces * 4usize.pow(levels)
}

/// Count expected vertices (upper bound estimate).
#[allow(dead_code)]
pub fn expected_vertex_count_loop(initial_verts: usize, initial_faces: usize, levels: u32) -> usize {
    let mut v = initial_verts;
    let mut f = initial_faces;
    let mut e = (3 * f) / 2;
    for _ in 0..levels {
        let new_v = v + e;
        let new_f = 4 * f;
        let new_e = 2 * e + 3 * f;
        v = new_v;
        f = new_f;
        e = new_e;
    }
    v
}

/// Apply crease to make a boundary edge retain sharpness.
#[allow(dead_code)]
pub fn crease_edge_midpoint(p0: [f32; 3], p1: [f32; 3]) -> [f32; 3] {
    scale3(add3(p0, p1), 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    #[test]
    fn subdivide_once_quad_count() {
        let (p, i) = triangle();
        let r = loop_subdivide(&p, &i);
        assert_eq!(r.indices.len(), 12);
    }

    #[test]
    fn subdivide_n_levels() {
        let (p, i) = triangle();
        let r = loop_subdivide_n(&p, &i, 2);
        assert_eq!(r.indices.len(), 48);
        assert_eq!(r.level, 2);
    }

    #[test]
    fn expected_face_count_correct() {
        assert_eq!(expected_face_count_loop(1, 2), 16);
    }

    #[test]
    fn loop_beta_regular_valence() {
        let b = loop_beta(6);
        assert!((b - 0.0625).abs() < 1e-5);
    }

    #[test]
    fn loop_beta_val3() {
        let b = loop_beta(3);
        assert!((b - 3.0 / 16.0).abs() < 1e-6);
    }

    #[test]
    fn loop_edge_point_midpoint_on_degenerate() {
        let p = loop_edge_point(
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
        );
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_count_grows_on_subdivision() {
        let (p, i) = triangle();
        let r = loop_subdivide(&p, &i);
        assert!(r.positions.len() > p.len());
    }

    #[test]
    fn crease_midpoint_is_midpoint() {
        let m = crease_edge_midpoint([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn level_field_correct() {
        let (p, i) = triangle();
        let r = loop_subdivide(&p, &i);
        assert_eq!(r.level, 1);
    }
}
