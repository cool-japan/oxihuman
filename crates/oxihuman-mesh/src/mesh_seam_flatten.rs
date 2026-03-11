// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct UvSeam {
    pub vertex_pairs: Vec<(usize, usize)>,
}

pub fn new_uv_seam() -> UvSeam {
    UvSeam {
        vertex_pairs: Vec::new(),
    }
}

pub fn seam_add_pair(s: &mut UvSeam, a: usize, b: usize) {
    s.vertex_pairs.push((a, b));
}

pub fn seam_pair_count(s: &UvSeam) -> usize {
    s.vertex_pairs.len()
}

pub fn seam_contains_vertex(s: &UvSeam, v: usize) -> bool {
    s.vertex_pairs.iter().any(|&(a, b)| a == v || b == v)
}

pub fn seam_flatten_uv(uvs: &[[f32; 2]], _seam: &UvSeam) -> Vec<[f32; 2]> {
    /* stub: return copy */
    uvs.to_vec()
}

pub fn seam_boundary_length(positions: &[[f32; 3]], seam: &UvSeam) -> f32 {
    seam.vertex_pairs
        .iter()
        .map(|&(a, b)| {
            if a < positions.len() && b < positions.len() {
                let pa = positions[a];
                let pb = positions[b];
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
            } else {
                0.0
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uv_seam() {
        /* starts empty */
        let s = new_uv_seam();
        assert_eq!(seam_pair_count(&s), 0);
    }

    #[test]
    fn test_seam_add_pair() {
        /* pair count increases */
        let mut s = new_uv_seam();
        seam_add_pair(&mut s, 0, 1);
        assert_eq!(seam_pair_count(&s), 1);
    }

    #[test]
    fn test_seam_contains_vertex() {
        /* finds vertex in pair */
        let mut s = new_uv_seam();
        seam_add_pair(&mut s, 3, 7);
        assert!(seam_contains_vertex(&s, 3));
        assert!(!seam_contains_vertex(&s, 5));
    }

    #[test]
    fn test_seam_flatten_uv_returns_copy() {
        /* flatten returns identical copy */
        let uvs = vec![[0.1f32, 0.2], [0.5, 0.6]];
        let s = new_uv_seam();
        let out = seam_flatten_uv(&uvs, &s);
        assert_eq!(out.len(), 2);
        assert!((out[0][0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_seam_boundary_length() {
        /* computes edge length between pair */
        let positions = vec![[0.0f32, 0.0, 0.0], [3.0, 4.0, 0.0]];
        let mut s = new_uv_seam();
        seam_add_pair(&mut s, 0, 1);
        let len = seam_boundary_length(&positions, &s);
        assert!((len - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_seam_boundary_length_empty() {
        /* empty seam has zero length */
        let positions = vec![[0.0f32, 0.0, 0.0]];
        let s = new_uv_seam();
        assert!(seam_boundary_length(&positions, &s).abs() < 1e-6);
    }
}
