#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Flip quads to improve diagonal orientation.

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct QuadFlipResult {
    pub flipped_count: usize,
    pub tris: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn quad_diagonal_lengths(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
) -> (f32, f32) {
    // diagonal 0-2 vs diagonal 1-3
    let d02 = {
        let dx = v2[0] - v0[0];
        let dy = v2[1] - v0[1];
        let dz = v2[2] - v0[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };
    let d13 = {
        let dx = v3[0] - v1[0];
        let dy = v3[1] - v1[1];
        let dz = v3[2] - v1[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };
    (d02, d13)
}

#[allow(dead_code)]
pub fn should_flip_quad(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> bool {
    let (d02, d13) = quad_diagonal_lengths(v0, v1, v2, v3);
    d13 < d02
}

#[allow(dead_code)]
pub fn quad_to_tris_flipped(q: [u32; 4], flip: bool) -> [[u32; 3]; 2] {
    if flip {
        // split along 1-3
        [[q[1], q[2], q[3]], [q[3], q[0], q[1]]]
    } else {
        // split along 0-2
        [[q[0], q[1], q[2]], [q[2], q[3], q[0]]]
    }
}

#[allow(dead_code)]
pub fn flip_quads(verts: &[[f32; 3]], quads: &[[u32; 4]]) -> QuadFlipResult {
    let mut flipped_count = 0;
    let mut tris = Vec::with_capacity(quads.len() * 2);
    for q in quads {
        let v0 = verts[q[0] as usize];
        let v1 = verts[q[1] as usize];
        let v2 = verts[q[2] as usize];
        let v3 = verts[q[3] as usize];
        let flip = should_flip_quad(v0, v1, v2, v3);
        if flip {
            flipped_count += 1;
        }
        let pair = quad_to_tris_flipped(*q, flip);
        tris.push(pair[0]);
        tris.push(pair[1]);
    }
    QuadFlipResult { flipped_count, tris }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn square_diagonals_equal() {
        let v = square_verts();
        let (d02, d13) = quad_diagonal_lengths(v[0], v[1], v[2], v[3]);
        assert!((d02 - d13).abs() < 1e-5);
    }

    #[test]
    fn flip_result_two_tris_per_quad() {
        let v = square_verts();
        let quads = vec![[0u32, 1, 2, 3]];
        let r = flip_quads(&v, &quads);
        assert_eq!(r.tris.len(), 2);
    }

    #[test]
    fn no_flip_for_square() {
        let v = square_verts();
        let quads = vec![[0u32, 1, 2, 3]];
        let r = flip_quads(&v, &quads);
        // square has equal diagonals so should_flip returns false
        assert_eq!(r.flipped_count, 0);
    }

    #[test]
    fn quad_to_tris_no_flip_indices() {
        let q = [0u32, 1, 2, 3];
        let tris = quad_to_tris_flipped(q, false);
        assert_eq!(tris[0], [0, 1, 2]);
        assert_eq!(tris[1], [2, 3, 0]);
    }

    #[test]
    fn quad_to_tris_flip_indices() {
        let q = [0u32, 1, 2, 3];
        let tris = quad_to_tris_flipped(q, true);
        assert_eq!(tris[0], [1, 2, 3]);
        assert_eq!(tris[1], [3, 0, 1]);
    }

    #[test]
    fn empty_quads_result() {
        let v: Vec<[f32; 3]> = vec![];
        let r = flip_quads(&v, &[]);
        assert_eq!(r.tris.len(), 0);
        assert_eq!(r.flipped_count, 0);
    }

    #[test]
    fn flip_for_asymmetric_quad() {
        // Make diagonal 1-3 shorter
        let v = [
            [0.0f32, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.5, 0.0, 0.0001],
        ];
        let flip = should_flip_quad(v[0], v[1], v[2], v[3]);
        // d13 close to 0, d02 close to 2, so flip = true
        assert!(flip);
    }

    #[test]
    fn diagonal_lengths_positive() {
        let v = square_verts();
        let (d02, d13) = quad_diagonal_lengths(v[0], v[1], v[2], v[3]);
        assert!(d02 > 0.0);
        assert!(d13 > 0.0);
    }
}
