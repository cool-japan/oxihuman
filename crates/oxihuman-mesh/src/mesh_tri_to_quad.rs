// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pair adjacent triangles into quads (greedy pairing).

use std::collections::HashMap;

/// A merged quad.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuadPair {
    pub verts: [u32; 4],
    pub tri_a: usize,
    pub tri_b: usize,
}

/// Result of tri-to-quad conversion.
#[allow(dead_code)]
pub struct TriToQuadResult {
    pub quads: Vec<QuadPair>,
    pub remaining_tris: Vec<[u32; 3]>,
    pub original_tri_count: usize,
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn face_normal(positions: &[[f32; 3]], i0: u32, i1: u32, i2: u32) -> [f32; 3] {
    let p0 = positions[i0 as usize];
    let p1 = positions[i1 as usize];
    let p2 = positions[i2 as usize];
    normalize3(cross3(sub3(p1, p0), sub3(p2, p0)))
}

/// Compute planarity score: how coplanar are two triangles (dot product of normals).
fn planarity_score(positions: &[[f32; 3]], ta: [u32; 3], tb: [u32; 3]) -> f32 {
    let na = face_normal(positions, ta[0], ta[1], ta[2]);
    let nb = face_normal(positions, tb[0], tb[1], tb[2]);
    dot3(na, nb)
}

/// Build a quad from two triangles sharing an edge.
fn merge_tris(ta: [u32; 3], tb: [u32; 3], shared_a: (u32, u32)) -> Option<[u32; 4]> {
    let (va, vb) = shared_a;
    let free_a = ta.iter().find(|&&v| v != va && v != vb).copied()?;
    let free_b = tb.iter().find(|&&v| v != va && v != vb).copied()?;
    Some([free_a, va, free_b, vb])
}

/// Greedily pair triangles into quads based on planarity.
#[allow(dead_code)]
pub fn triangles_to_quads(
    positions: &[[f32; 3]],
    indices: &[u32],
    min_planarity: f32,
) -> TriToQuadResult {
    let n_tri = indices.len() / 3;
    let tris: Vec<[u32; 3]> = (0..n_tri)
        .map(|t| [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]])
        .collect();
    let mut edge_to_tri: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (t, tri) in tris.iter().enumerate() {
        for e in 0..3 {
            let a = tri[e];
            let b = tri[(e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_tri.entry(key).or_default().push(t);
        }
    }
    let mut paired = vec![false; n_tri];
    let mut quads = Vec::new();
    let mut quad_order: Vec<(f32, usize, usize, (u32, u32))> = Vec::new();
    let mut edge_seen = HashMap::new();
    for t in 0..n_tri {
        for e in 0..3 {
            let a = tris[t][e];
            let b = tris[t][(e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_seen.contains_key(&key) {
                continue;
            }
            edge_seen.insert(key, true);
            if let Some(faces) = edge_to_tri.get(&key) {
                if faces.len() == 2 {
                    let ta = faces[0];
                    let tb = faces[1];
                    let score = planarity_score(positions, tris[ta], tris[tb]);
                    if score >= min_planarity {
                        quad_order.push((score, ta, tb, key));
                    }
                }
            }
        }
    }
    quad_order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, ta, tb, (va, vb)) in &quad_order {
        if paired[*ta] || paired[*tb] {
            continue;
        }
        if let Some(quad) = merge_tris(tris[*ta], tris[*tb], (*va, *vb)) {
            quads.push(QuadPair {
                verts: quad,
                tri_a: *ta,
                tri_b: *tb,
            });
            paired[*ta] = true;
            paired[*tb] = true;
        }
    }
    let remaining_tris = (0..n_tri)
        .filter(|&t| !paired[t])
        .map(|t| tris[t])
        .collect();
    TriToQuadResult {
        quads,
        remaining_tris,
        original_tri_count: n_tri,
    }
}

/// Count of successfully merged quads.
#[allow(dead_code)]
pub fn quad_count_t2q(result: &TriToQuadResult) -> usize {
    result.quads.len()
}

/// Count of remaining unpaired triangles.
#[allow(dead_code)]
pub fn remaining_tri_count(result: &TriToQuadResult) -> usize {
    result.remaining_tris.len()
}

/// Quad-to-triangle ratio.
#[allow(dead_code)]
pub fn quadification_ratio(result: &TriToQuadResult) -> f32 {
    if result.original_tri_count == 0 {
        return 0.0;
    }
    (result.quads.len() * 2) as f32 / result.original_tri_count as f32
}

/// Export quads as flat 4-index buffer.
#[allow(dead_code)]
pub fn quads_to_flat_buffer(result: &TriToQuadResult) -> Vec<u32> {
    result.quads.iter().flat_map(|q| q.verts).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_coplanar_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        (positions, indices)
    }

    #[test]
    fn coplanar_tris_merge_to_quad() {
        let (pos, idx) = two_coplanar_tris();
        let result = triangles_to_quads(&pos, &idx, 0.9);
        assert_eq!(quad_count_t2q(&result), 1);
    }

    #[test]
    fn no_remaining_after_merge() {
        let (pos, idx) = two_coplanar_tris();
        let result = triangles_to_quads(&pos, &idx, 0.9);
        assert_eq!(remaining_tri_count(&result), 0);
    }

    #[test]
    fn high_planarity_threshold_prevents_merge() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.0, 1.0],
            [0.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let result = triangles_to_quads(&positions, &indices, 1.0);
        assert_eq!(quad_count_t2q(&result), 0);
    }

    #[test]
    fn quadification_ratio_correct() {
        let (pos, idx) = two_coplanar_tris();
        let result = triangles_to_quads(&pos, &idx, 0.9);
        let ratio = quadification_ratio(&result);
        assert!((ratio - 1.0).abs() < 1e-5);
    }

    #[test]
    fn quads_to_flat_buffer_correct() {
        let (pos, idx) = two_coplanar_tris();
        let result = triangles_to_quads(&pos, &idx, 0.9);
        let buf = quads_to_flat_buffer(&result);
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn original_tri_count_stored() {
        let (pos, idx) = two_coplanar_tris();
        let result = triangles_to_quads(&pos, &idx, 0.9);
        assert_eq!(result.original_tri_count, 2);
    }

    #[test]
    fn empty_mesh_no_quads() {
        let result = triangles_to_quads(&[], &[], 0.9);
        assert_eq!(quad_count_t2q(&result), 0);
    }

    #[test]
    fn single_tri_no_pair() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx: Vec<u32> = vec![0, 1, 2];
        let result = triangles_to_quads(&pos, &idx, 0.9);
        assert_eq!(quad_count_t2q(&result), 0);
        assert_eq!(remaining_tri_count(&result), 1);
    }

    #[test]
    fn quad_verts_in_bounds() {
        let (pos, idx) = two_coplanar_tris();
        let result = triangles_to_quads(&pos, &idx, 0.9);
        let n = pos.len() as u32;
        for q in &result.quads {
            for v in q.verts {
                assert!(v < n);
            }
        }
    }

    #[test]
    fn merge_tris_produces_four_verts() {
        let ta = [0u32, 1, 2];
        let tb = [0u32, 2, 3];
        let q = merge_tris(ta, tb, (0, 2));
        assert!(q.is_some());
        assert_eq!(q.unwrap().len(), 4);
    }
}
