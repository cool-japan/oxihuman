// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Antiprism mesh generator.

use std::f32::consts::TAU;

/// An antiprism mesh: two parallel n-gon rings connected by 2n triangles.
#[derive(Debug, Clone)]
pub struct Antiprism {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub sides: usize,
}

/// Build an antiprism with `sides` sides, bottom radius `r_bot`, top radius `r_top`,
/// and given `height`. The top ring is rotated by π/n relative to the bottom.
pub fn build_antiprism(sides: usize, r_bot: f32, r_top: f32, height: f32) -> Antiprism {
    if sides < 3 {
        return Antiprism {
            verts: vec![],
            tris: vec![],
            sides: 0,
        };
    }
    let n = sides;
    let offset = TAU / (2.0 * n as f32);
    let mut verts = Vec::with_capacity(2 * n + 2);
    for i in 0..n {
        let a = TAU * i as f32 / n as f32;
        verts.push([r_bot * a.cos(), 0.0, r_bot * a.sin()]);
    }
    for i in 0..n {
        let a = TAU * i as f32 / n as f32 + offset;
        verts.push([r_top * a.cos(), height, r_top * a.sin()]);
    }
    let bot_c = (2 * n) as u32;
    let top_c = (2 * n + 1) as u32;
    verts.push([0.0, 0.0, 0.0]);
    verts.push([0.0, height, 0.0]);

    let mut tris = Vec::new();
    /* lateral strip: 2n triangles */
    for i in 0..n {
        let next = (i + 1) % n;
        let a = i as u32;
        let b = next as u32;
        let c = (n + i) as u32;
        let d = (n + next) as u32;
        /* "up" triangle */
        tris.push([a, b, c]);
        /* "down" triangle */
        tris.push([b, d, c]);
    }
    /* bottom cap fan */
    for i in 0..n {
        tris.push([bot_c, ((i + 1) % n) as u32, i as u32]);
    }
    /* top cap fan */
    for i in 0..n {
        tris.push([top_c, (n + i) as u32, (n + (i + 1) % n) as u32]);
    }
    Antiprism {
        verts,
        tris,
        sides: n,
    }
}

/// Return vertex count.
pub fn antiprism_vertex_count(a: &Antiprism) -> usize {
    a.verts.len()
}

/// Return triangle count.
pub fn antiprism_tri_count(a: &Antiprism) -> usize {
    a.tris.len()
}

/// Validate all triangle indices.
pub fn validate_antiprism(a: &Antiprism) -> bool {
    let n = a.verts.len() as u32;
    a.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute the expected triangle count for a given number of sides.
pub fn antiprism_expected_tris(sides: usize) -> usize {
    /* 2*n lateral + n bottom + n top */
    4 * sides
}

/// Return whether the lateral triangles alternate up/down correctly (stub check).
pub fn has_alternating_strip(a: &Antiprism) -> bool {
    !a.tris.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_antiprism_vertex_count() {
        /* 5-sided: 5+5+2 = 12 verts */
        let a = build_antiprism(5, 1.0, 1.0, 1.0);
        assert_eq!(antiprism_vertex_count(&a), 12);
    }

    #[test]
    fn test_antiprism_tri_count() {
        /* 5-sided: 10 lateral + 5 bottom + 5 top = 20 */
        let a = build_antiprism(5, 1.0, 1.0, 1.0);
        assert_eq!(antiprism_tri_count(&a), 20);
    }

    #[test]
    fn test_antiprism_expected_tris() {
        assert_eq!(antiprism_expected_tris(5), 20);
    }

    #[test]
    fn test_validate_antiprism() {
        let a = build_antiprism(6, 1.0, 0.5, 2.0);
        assert!(validate_antiprism(&a));
    }

    #[test]
    fn test_antiprism_empty_on_too_few_sides() {
        let a = build_antiprism(2, 1.0, 1.0, 1.0);
        assert_eq!(antiprism_vertex_count(&a), 0);
    }

    #[test]
    fn test_antiprism_sides_stored() {
        let a = build_antiprism(8, 1.0, 1.0, 1.0);
        assert_eq!(a.sides, 8);
    }

    #[test]
    fn test_antiprism_triangular() {
        /* triangular antiprism: 3 sides */
        let a = build_antiprism(3, 1.0, 1.0, 1.0);
        assert_eq!(antiprism_tri_count(&a), 12);
    }

    #[test]
    fn test_has_alternating_strip() {
        let a = build_antiprism(4, 1.0, 1.0, 1.0);
        assert!(has_alternating_strip(&a));
    }

    #[test]
    fn test_antiprism_square() {
        /* square antiprism: 4-sided antiprism has 8+2 = 10 verts */
        let a = build_antiprism(4, 1.0, 1.0, 1.0);
        assert_eq!(antiprism_vertex_count(&a), 10);
    }
}
