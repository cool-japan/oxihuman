#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mark seam edges based on UV discontinuities.

#[allow(dead_code)]
pub struct UvSeamResult {
    pub seam_edges: Vec<(u32, u32)>,
}

#[allow(dead_code)]
pub fn seam_from_uv(tris: &[[u32; 3]], uvs_per_tri: &[[[f32; 2]; 3]]) -> UvSeamResult {
    use std::collections::HashMap;

    if tris.len() != uvs_per_tri.len() {
        return UvSeamResult { seam_edges: vec![] };
    }

    type UvPair = ([f32; 2], [f32; 2]);
    type EdgeUvMap = HashMap<(u32, u32), Vec<UvPair>>;
    // Map edge (sorted vert pair) -> list of (uv_a, uv_b) per triangle
    let mut edge_uvs: EdgeUvMap = HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        for e in 0..3 {
            let a = tri[e];
            let b = tri[(e + 1) % 3];
            let uv_a = uvs_per_tri[ti][e];
            let uv_b = uvs_per_tri[ti][(e + 1) % 3];
            let (key, uv_pair) = if a < b {
                ((a, b), (uv_a, uv_b))
            } else {
                ((b, a), (uv_b, uv_a))
            };
            edge_uvs.entry(key).or_default().push(uv_pair);
        }
    }

    let mut seam_edges = vec![];
    for ((a, b), uvs) in &edge_uvs {
        if uvs.len() < 2 {
            continue;
        }
        let (uv_a0, uv_a1) = uvs[0];
        let (uv_b0, uv_b1) = uvs[1];
        if uv_edge_is_seam(uv_a0, uv_a1, uv_b0, uv_b1, 1e-4) {
            seam_edges.push((*a, *b));
        }
    }
    UvSeamResult { seam_edges }
}

#[allow(dead_code)]
pub fn uv_edge_is_seam(
    uv_a0: [f32; 2],
    uv_a1: [f32; 2],
    uv_b0: [f32; 2],
    uv_b1: [f32; 2],
    tol: f32,
) -> bool {
    let match_fwd = uv_dist(uv_a0, uv_b0) < tol && uv_dist(uv_a1, uv_b1) < tol;
    let match_rev = uv_dist(uv_a0, uv_b1) < tol && uv_dist(uv_a1, uv_b0) < tol;
    !(match_fwd || match_rev)
}

#[allow(dead_code)]
pub fn seam_count(result: &UvSeamResult) -> usize {
    result.seam_edges.len()
}

fn uv_dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seam_count_empty() {
        let r = UvSeamResult { seam_edges: vec![] };
        assert_eq!(seam_count(&r), 0);
    }

    #[test]
    fn seam_from_uv_mismatched_len() {
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2]];
        let r = seam_from_uv(&tris, &[]);
        assert_eq!(seam_count(&r), 0);
    }

    #[test]
    fn no_seam_when_uvs_match() {
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2], [0, 1, 3]];
        let uvs: Vec<[[f32; 2]; 3]> = vec![
            [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            [[0.0, 0.0], [1.0, 0.0], [0.5, -1.0]],
        ];
        let r = seam_from_uv(&tris, &uvs);
        assert_eq!(seam_count(&r), 0);
    }

    #[test]
    fn seam_detected_when_uvs_differ() {
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2], [0, 1, 3]];
        let uvs: Vec<[[f32; 2]; 3]> = vec![
            [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            [[0.5, 0.5], [0.9, 0.5], [0.5, -1.0]],
        ];
        let r = seam_from_uv(&tris, &uvs);
        assert!(seam_count(&r) > 0);
    }

    #[test]
    fn uv_edge_is_seam_matching_forward() {
        assert!(!uv_edge_is_seam(
            [0.0, 0.0], [1.0, 0.0],
            [0.0, 0.0], [1.0, 0.0],
            1e-4
        ));
    }

    #[test]
    fn uv_edge_is_seam_matching_reverse() {
        assert!(!uv_edge_is_seam(
            [0.0, 0.0], [1.0, 0.0],
            [1.0, 0.0], [0.0, 0.0],
            1e-4
        ));
    }

    #[test]
    fn uv_edge_is_seam_different_uvs() {
        assert!(uv_edge_is_seam(
            [0.0, 0.0], [1.0, 0.0],
            [0.5, 0.5], [1.5, 0.5],
            1e-4
        ));
    }

    #[test]
    fn seam_from_uv_empty() {
        let r = seam_from_uv(&[], &[]);
        assert_eq!(seam_count(&r), 0);
    }
}
