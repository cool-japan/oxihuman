// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Convert triangle pairs to quads.

#[derive(Debug, Clone)]
pub struct TrisToQuadsResult {
    pub quad_faces: Vec<[u32; 4]>,
    pub remaining_tris: Vec<[u32; 3]>,
    pub quads_formed: usize,
    pub tris_remaining: usize,
}

fn face_normal(positions: &[[f32; 3]], tri: [u32; 3]) -> [f32; 3] {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-10);
    [n[0] / l, n[1] / l, n[2] / l]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn shared_edge(a: [u32; 3], b: [u32; 3]) -> Option<(u32, u32)> {
    for i in 0..3 {
        for j in 0..3 {
            let v0 = a[i];
            let v1 = a[(i + 1) % 3];
            let w0 = b[j];
            let w1 = b[(j + 1) % 3];
            if (v0 == w1 && v1 == w0) || (v0 == w0 && v1 == w1) {
                return Some((v0, v1));
            }
        }
    }
    None
}

/// Find the opposite vertex in a triangle given a shared edge.
fn opposite_vertex(tri: [u32; 3], e0: u32, e1: u32) -> Option<u32> {
    tri.iter().copied().find(|&v| v != e0 && v != e1)
}

/// Try to merge two triangles sharing an edge into a quad.
pub fn try_merge_to_quad(
    positions: &[[f32; 3]],
    a: [u32; 3],
    b: [u32; 3],
    angle_threshold_deg: f32,
) -> Option<[u32; 4]> {
    let threshold = angle_threshold_deg * std::f32::consts::PI / 180.0;
    let na = face_normal(positions, a);
    let nb = face_normal(positions, b);
    let dot = dot3(na, nb).clamp(-1.0, 1.0);
    if dot.acos() > threshold {
        return None;
    }
    let (e0, e1) = shared_edge(a, b)?;
    let va = opposite_vertex(a, e0, e1)?;
    let vb = opposite_vertex(b, e0, e1)?;
    /* order quad vertices: va, e0, vb, e1 */
    Some([va, e0, vb, e1])
}

/// Convert all compatible triangle pairs to quads.
pub fn tris_to_quads(
    positions: &[[f32; 3]],
    indices: &[u32],
    angle_threshold_deg: f32,
) -> TrisToQuadsResult {
    let face_count = indices.len() / 3;
    let mut tris: Vec<[u32; 3]> = indices
        .chunks(3)
        .filter(|c| c.len() == 3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let mut quads = Vec::new();
    let mut used = vec![false; face_count];
    for i in 0..face_count {
        if used[i] {
            continue;
        }
        for j in i + 1..face_count {
            if used[j] {
                continue;
            }
            if let Some(q) = try_merge_to_quad(positions, tris[i], tris[j], angle_threshold_deg) {
                quads.push(q);
                used[i] = true;
                used[j] = true;
                break;
            }
        }
    }
    let remaining: Vec<[u32; 3]> = tris
        .iter()
        .enumerate()
        .filter(|(i, _)| !used[*i])
        .map(|(_, &t)| t)
        .collect();
    let quads_formed = quads.len();
    let tris_remaining = remaining.len();
    tris.clear();
    TrisToQuadsResult {
        quad_faces: quads,
        remaining_tris: remaining,
        quads_formed,
        tris_remaining,
    }
}

pub fn quads_to_tri_indices(quads: &[[u32; 4]]) -> Vec<u32> {
    quads
        .iter()
        .flat_map(|&q| [q[0], q[1], q[2], q[0], q[2], q[3]])
        .collect()
}

pub fn quad_count(result: &TrisToQuadsResult) -> usize {
    result.quads_formed
}
pub fn tri_count(result: &TrisToQuadsResult) -> usize {
    result.tris_remaining
}
pub fn total_faces(result: &TrisToQuadsResult) -> usize {
    result.quads_formed + result.tris_remaining
}

pub fn validate_quads(quads: &[[u32; 4]], vertex_count: usize) -> bool {
    quads
        .iter()
        .all(|q| q.iter().all(|&v| (v as usize) < vertex_count))
}

pub fn merge_ratio(result: &TrisToQuadsResult, original_tris: usize) -> f32 {
    if original_tris == 0 {
        return 0.0;
    }
    result.quads_formed as f32 * 2.0 / original_tris as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris_square() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_tris_to_quads_merges() {
        let (pos, idx) = two_tris_square();
        let res = tris_to_quads(&pos, &idx, 10.0);
        assert_eq!(res.quads_formed, 1);
        assert_eq!(res.tris_remaining, 0);
    }

    #[test]
    fn test_shared_edge() {
        let a = [0u32, 1, 2];
        let b = [0u32, 2, 3];
        let e = shared_edge(a, b);
        assert!(e.is_some());
    }

    #[test]
    fn test_quad_count() {
        let (pos, idx) = two_tris_square();
        let res = tris_to_quads(&pos, &idx, 10.0);
        assert_eq!(quad_count(&res), 1);
    }

    #[test]
    fn test_tri_count_no_merge() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.5, -1.0, 1.0],
        ];
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        let res = tris_to_quads(&pos, &idx, 5.0);
        assert_eq!(res.tris_remaining, 2);
    }

    #[test]
    fn test_quads_to_tri_indices() {
        let quads = vec![[0u32, 1, 2, 3]];
        let idx = quads_to_tri_indices(&quads);
        assert_eq!(idx.len(), 6);
    }

    #[test]
    fn test_validate_quads() {
        let quads = vec![[0u32, 1, 2, 3]];
        assert!(validate_quads(&quads, 4));
        assert!(!validate_quads(&quads, 3));
    }

    #[test]
    fn test_total_faces() {
        let (pos, idx) = two_tris_square();
        let res = tris_to_quads(&pos, &idx, 10.0);
        assert_eq!(total_faces(&res), 1);
    }

    #[test]
    fn test_merge_ratio() {
        let (pos, idx) = two_tris_square();
        let res = tris_to_quads(&pos, &idx, 10.0);
        assert!((merge_ratio(&res, 2) - 1.0).abs() < 1e-5);
    }
}
