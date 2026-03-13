#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Quad-dominant remeshing stub.

#[allow(dead_code)]
pub struct QuadRemeshResult {
    pub verts: Vec<[f32; 3]>,
    pub quads: Vec<[u32; 4]>,
    pub tris: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn remesh_to_quads(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    _target_edge_len: f32,
) -> QuadRemeshResult {
    let quads = pair_triangles_to_quads(tris);
    let used = quads.len() * 2;
    let remaining: Vec<[u32; 3]> = if used < tris.len() {
        tris[used..].to_vec()
    } else {
        Vec::new()
    };
    QuadRemeshResult {
        verts: verts.to_vec(),
        quads,
        tris: remaining,
    }
}

#[allow(dead_code)]
pub fn pair_triangles_to_quads(tris: &[[u32; 3]]) -> Vec<[u32; 4]> {
    let mut quads = Vec::new();
    let mut i = 0;
    while i + 1 < tris.len() {
        let t0 = tris[i];
        let t1 = tris[i + 1];
        quads.push([t0[0], t0[1], t1[2], t0[2]]);
        i += 2;
    }
    quads
}

#[allow(dead_code)]
pub fn quad_count(result: &QuadRemeshResult) -> usize {
    result.quads.len()
}

#[allow(dead_code)]
pub fn tri_count_remaining(result: &QuadRemeshResult) -> usize {
    result.tris.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn sample_tris() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [1, 3, 2]]
    }

    #[test]
    fn pair_two_tris_to_one_quad() {
        let tris = sample_tris();
        let quads = pair_triangles_to_quads(&tris);
        assert_eq!(quads.len(), 1);
    }

    #[test]
    fn pair_single_tri_yields_no_quads() {
        let tris = vec![[0u32, 1, 2]];
        let quads = pair_triangles_to_quads(&tris);
        assert!(quads.is_empty());
    }

    #[test]
    fn remesh_returns_correct_vert_count() {
        let verts = sample_verts();
        let tris = sample_tris();
        let result = remesh_to_quads(&verts, &tris, 0.5);
        assert_eq!(result.verts.len(), 4);
    }

    #[test]
    fn quad_count_matches() {
        let verts = sample_verts();
        let tris = sample_tris();
        let result = remesh_to_quads(&verts, &tris, 0.5);
        assert_eq!(quad_count(&result), 1);
    }

    #[test]
    fn tri_count_remaining_is_zero_for_even_tris() {
        let verts = sample_verts();
        let tris = sample_tris();
        let result = remesh_to_quads(&verts, &tris, 0.5);
        assert_eq!(tri_count_remaining(&result), 0);
    }

    #[test]
    fn odd_tri_count_leaves_one_remaining() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 2.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 2], [1, 3, 2], [2, 3, 4]];
        let result = remesh_to_quads(&verts, &tris, 0.5);
        assert_eq!(tri_count_remaining(&result), 1);
    }

    #[test]
    fn empty_tris_gives_empty_result() {
        let verts = sample_verts();
        let result = remesh_to_quads(&verts, &[], 0.5);
        assert!(result.quads.is_empty());
        assert!(result.tris.is_empty());
    }

    #[test]
    fn quad_indices_come_from_source_tris() {
        let tris = vec![[0u32, 1, 2], [1, 3, 2]];
        let quads = pair_triangles_to_quads(&tris);
        assert_eq!(quads[0][0], 0);
        assert_eq!(quads[0][1], 1);
    }

    #[test]
    fn four_tris_gives_two_quads() {
        let tris = vec![[0u32, 1, 2], [1, 3, 2], [0, 2, 4], [2, 5, 4]];
        let quads = pair_triangles_to_quads(&tris);
        assert_eq!(quads.len(), 2);
    }

    #[test]
    fn remesh_large_edge_len_still_works() {
        let verts = sample_verts();
        let tris = sample_tris();
        let result = remesh_to_quads(&verts, &tris, 100.0);
        assert_eq!(result.verts.len(), verts.len());
    }
}
