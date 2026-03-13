// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Trapezohedron (dual of antiprism) mesh generator.

use std::f32::consts::TAU;

/// A trapezohedron mesh: dual of an antiprism, with kite-shaped faces.
/// Represented as a triangle mesh (each kite split into 2 triangles).
#[derive(Debug, Clone)]
pub struct Trapezohedron {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub sides: usize,
}

/// Build a trapezohedron with `sides` kite faces per hemisphere.
/// `r` is equatorial radius, `h_equator` is the equatorial ring height offset,
/// `h_apex` is the apex height above the equatorial band.
pub fn build_trapezohedron(sides: usize, r: f32, h_equator: f32, h_apex: f32) -> Trapezohedron {
    if sides < 3 {
        return Trapezohedron {
            verts: vec![],
            tris: vec![],
            sides: 0,
        };
    }
    let n = sides;
    /* Two interleaved equatorial rings offset by pi/n, plus two apices */
    let mut verts = Vec::new();
    /* bottom equatorial ring */
    for i in 0..n {
        let a = TAU * i as f32 / n as f32;
        verts.push([r * a.cos(), -h_equator, r * a.sin()]);
    }
    /* top equatorial ring, rotated by pi/n */
    let off = TAU / (2.0 * n as f32);
    for i in 0..n {
        let a = TAU * i as f32 / n as f32 + off;
        verts.push([r * a.cos(), h_equator, r * a.sin()]);
    }
    let bot_apex = (2 * n) as u32;
    let top_apex = (2 * n + 1) as u32;
    verts.push([0.0, -h_apex, 0.0]);
    verts.push([0.0, h_apex, 0.0]);

    let mut tris = Vec::new();
    /* each kite = 4 verts: bottom_ring[i], bottom_ring[i+1], top_ring[i], top_ring[i+1]
    plus a connection to apex; split each kite into 2 tris */
    for i in 0..n {
        let next = (i + 1) % n;
        let bl = i as u32;
        let br = next as u32;
        let tl = (n + i) as u32;
        let tr = (n + next) as u32;
        /* top kite: top_apex → tl → tr */
        tris.push([top_apex, tl, tr]);
        /* equatorial quad split: bl-br-tr-tl */
        tris.push([bl, br, tr]);
        tris.push([bl, tr, tl]);
        /* bottom kite: bot_apex → br → bl */
        tris.push([bot_apex, br, bl]);
    }
    Trapezohedron {
        verts,
        tris,
        sides: n,
    }
}

/// Return vertex count.
pub fn trapezohedron_vertex_count(t: &Trapezohedron) -> usize {
    t.verts.len()
}

/// Return triangle count.
pub fn trapezohedron_tri_count(t: &Trapezohedron) -> usize {
    t.tris.len()
}

/// Expected triangle count: 4 tris per kite face.
pub fn trapezohedron_expected_tris(sides: usize) -> usize {
    4 * sides
}

/// Validate all triangle indices.
pub fn validate_trapezohedron(t: &Trapezohedron) -> bool {
    let n = t.verts.len() as u32;
    t.tris
        .iter()
        .all(|tri| tri[0] < n && tri[1] < n && tri[2] < n)
}

/// Compute total surface area.
pub fn trapezohedron_surface_area(t: &Trapezohedron) -> f32 {
    t.tris
        .iter()
        .map(|tri| {
            let a = t.verts[tri[0] as usize];
            let b = t.verts[tri[1] as usize];
            let c = t.verts[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt() * 0.5
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trapezohedron_vertex_count() {
        /* 3-sided: 3+3+2 = 8 */
        let t = build_trapezohedron(3, 1.0, 0.5, 1.5);
        assert_eq!(trapezohedron_vertex_count(&t), 8);
    }

    #[test]
    fn test_trapezohedron_tri_count() {
        /* 3*4 = 12 */
        let t = build_trapezohedron(3, 1.0, 0.5, 1.5);
        assert_eq!(trapezohedron_tri_count(&t), 12);
    }

    #[test]
    fn test_trapezohedron_expected_tris() {
        assert_eq!(trapezohedron_expected_tris(5), 20);
    }

    #[test]
    fn test_validate_trapezohedron() {
        let t = build_trapezohedron(5, 1.0, 0.5, 2.0);
        assert!(validate_trapezohedron(&t));
    }

    #[test]
    fn test_trapezohedron_empty_on_too_few() {
        let t = build_trapezohedron(2, 1.0, 0.5, 1.0);
        assert_eq!(trapezohedron_vertex_count(&t), 0);
    }

    #[test]
    fn test_surface_area_positive() {
        let t = build_trapezohedron(6, 1.0, 0.3, 1.5);
        assert!(trapezohedron_surface_area(&t) > 0.0);
    }

    #[test]
    fn test_sides_stored() {
        let t = build_trapezohedron(7, 1.0, 0.5, 1.0);
        assert_eq!(t.sides, 7);
    }

    #[test]
    fn test_cube_trapezohedron() {
        /* 3-sided trapezohedron is the dual of triangular antiprism */
        let t = build_trapezohedron(3, 1.0, 0.4, 1.0);
        assert_eq!(trapezohedron_tri_count(&t), 12);
    }

    #[test]
    fn test_ten_sided_trapezohedron() {
        /* 5-sided trapezohedron (rhombohedron family) */
        let t = build_trapezohedron(5, 1.0, 0.5, 1.5);
        assert_eq!(trapezohedron_tri_count(&t), 20);
    }
}
