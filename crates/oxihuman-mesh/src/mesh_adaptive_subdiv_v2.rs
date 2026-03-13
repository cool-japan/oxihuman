// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Adaptive subdivision based on curvature threshold (v2).

use std::collections::HashMap;

/// Config for adaptive subdivision v2.
#[allow(dead_code)]
pub struct AdaptiveSubdivV2Config {
    pub curvature_threshold: f32,
    pub max_levels: u32,
}

#[allow(dead_code)]
impl Default for AdaptiveSubdivV2Config {
    fn default() -> Self {
        Self { curvature_threshold: 0.1, max_levels: 4 }
    }
}

/// Result of adaptive subdivision v2.
#[allow(dead_code)]
pub struct AdaptiveSubdivV2Result {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub faces_subdivided: usize,
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

/// Compute the dihedral angle between two face normals.
#[allow(dead_code)]
pub fn dihedral_angle_v2(n0: [f32; 3], n1: [f32; 3]) -> f32 {
    let cos_a = dot3(n0, n1).clamp(-1.0, 1.0);
    cos_a.acos()
}

/// Compute the face normal.
#[allow(dead_code)]
pub fn face_normal_v2(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let l = len3(n);
    if l < 1e-10 { n } else { scale3(n, 1.0 / l) }
}

/// Estimate curvature at a face as max edge length (proxy).
#[allow(dead_code)]
pub fn face_curvature_proxy(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let l0 = len3(sub3(p1, p0));
    let l1 = len3(sub3(p2, p1));
    let l2 = len3(sub3(p0, p2));
    l0.max(l1).max(l2)
}

/// Check if a face needs subdivision based on curvature threshold.
#[allow(dead_code)]
pub fn needs_subdivision(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], threshold: f32) -> bool {
    face_curvature_proxy(p0, p1, p2) > threshold
}

/// Perform one pass of adaptive subdivision.
#[allow(dead_code)]
pub fn adaptive_subdivide_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &AdaptiveSubdivV2Config,
) -> AdaptiveSubdivV2Result {
    let mut new_pos = positions.to_vec();
    let mut new_idx: Vec<u32> = Vec::new();
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut subdivided = 0usize;

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[3 * t];
        let b = indices[3 * t + 1];
        let c = indices[3 * t + 2];
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        let pc = positions[c as usize];

        if needs_subdivision(pa, pb, pc, config.curvature_threshold) {
            let get_mid = |p: u32, q: u32, pos: &mut Vec<[f32; 3]>, map: &mut HashMap<(u32, u32), u32>| {
                let key = if p < q { (p, q) } else { (q, p) };
                *map.entry(key).or_insert_with(|| {
                    let mp = scale3(add3(pos[p as usize], pos[q as usize]), 0.5);
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
            subdivided += 1;
        } else {
            new_idx.extend_from_slice(&[a, b, c]);
        }
    }

    AdaptiveSubdivV2Result { positions: new_pos, indices: new_idx, faces_subdivided: subdivided }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_subdivision_below_threshold() {
        let pos = vec![[0.0,0.0,0.0],[0.01,0.0,0.0],[0.0,0.01,0.0]];
        let idx = vec![0,1,2];
        let cfg = AdaptiveSubdivV2Config { curvature_threshold: 1.0, max_levels: 2 };
        let r = adaptive_subdivide_v2(&pos, &idx, &cfg);
        assert_eq!(r.faces_subdivided, 0);
        assert_eq!(r.indices.len(), 3);
    }

    #[test]
    fn subdivision_above_threshold() {
        let pos = vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[0.0,2.0,0.0]];
        let idx = vec![0,1,2];
        let cfg = AdaptiveSubdivV2Config { curvature_threshold: 1.0, max_levels: 2 };
        let r = adaptive_subdivide_v2(&pos, &idx, &cfg);
        assert_eq!(r.faces_subdivided, 1);
        assert_eq!(r.indices.len(), 12);
    }

    #[test]
    fn face_normal_unit_length() {
        let n = face_normal_v2([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]);
        let l = n[0]*n[0]+n[1]*n[1]+n[2]*n[2];
        assert!((l - 1.0).abs() < 1e-5);
    }

    #[test]
    fn curvature_proxy_positive() {
        let c = face_curvature_proxy([0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]);
        assert!(c > 0.0);
    }

    #[test]
    fn needs_subdivision_true() {
        assert!(needs_subdivision([0.0,0.0,0.0],[5.0,0.0,0.0],[0.0,5.0,0.0], 1.0));
    }

    #[test]
    fn needs_subdivision_false() {
        assert!(!needs_subdivision([0.0,0.0,0.0],[0.1,0.0,0.0],[0.0,0.1,0.0], 1.0));
    }

    #[test]
    fn dihedral_same_normals_zero() {
        let n = [0.0, 0.0, 1.0];
        let a = dihedral_angle_v2(n, n);
        assert!(a.abs() < 1e-5);
    }

    #[test]
    fn dihedral_opposite_normals_pi() {
        use std::f32::consts::PI;
        let n0 = [0.0, 0.0, 1.0];
        let n1 = [0.0, 0.0, -1.0];
        let a = dihedral_angle_v2(n0, n1);
        assert!((a - PI).abs() < 1e-5);
    }

    #[test]
    fn default_config() {
        let c = AdaptiveSubdivV2Config::default();
        assert_eq!(c.max_levels, 4);
    }
}
