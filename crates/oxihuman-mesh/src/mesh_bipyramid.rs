// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bipyramid mesh generator.

use std::f32::consts::TAU;

/// A bipyramid mesh: an equatorial polygon ring with top and bottom apex vertices.
#[derive(Debug, Clone)]
pub struct Bipyramid {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub sides: usize,
    pub apex_height: f32,
}

/// Build a bipyramid with `sides` sides, equatorial radius `radius`,
/// and apex distance `apex_height` above/below the equatorial plane.
pub fn build_bipyramid(sides: usize, radius: f32, apex_height: f32) -> Bipyramid {
    if sides < 3 {
        return Bipyramid {
            verts: vec![],
            tris: vec![],
            sides: 0,
            apex_height: 0.0,
        };
    }
    let n = sides;
    let mut verts = Vec::with_capacity(n + 2);
    /* equatorial ring */
    for i in 0..n {
        let a = TAU * i as f32 / n as f32;
        verts.push([radius * a.cos(), 0.0, radius * a.sin()]);
    }
    let top_idx = n as u32;
    let bot_idx = (n + 1) as u32;
    verts.push([0.0, apex_height, 0.0]);
    verts.push([0.0, -apex_height, 0.0]);

    let mut tris = Vec::new();
    for i in 0..n {
        let next = (i + 1) % n;
        /* top pyramid */
        tris.push([i as u32, next as u32, top_idx]);
        /* bottom pyramid */
        tris.push([next as u32, i as u32, bot_idx]);
    }
    Bipyramid {
        verts,
        tris,
        sides: n,
        apex_height,
    }
}

/// Return vertex count.
pub fn bipyramid_vertex_count(b: &Bipyramid) -> usize {
    b.verts.len()
}

/// Return triangle count.
pub fn bipyramid_tri_count(b: &Bipyramid) -> usize {
    b.tris.len()
}

/// Expected triangle count for a bipyramid with n sides.
pub fn bipyramid_expected_tris(sides: usize) -> usize {
    2 * sides
}

/// Validate all triangle indices.
pub fn validate_bipyramid(b: &Bipyramid) -> bool {
    let n = b.verts.len() as u32;
    b.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute the approximate surface area.
pub fn bipyramid_surface_area(b: &Bipyramid) -> f32 {
    let mut area = 0.0f32;
    for tri in &b.tris {
        let a = b.verts[tri[0] as usize];
        let v = b.verts[tri[1] as usize];
        let c = b.verts[tri[2] as usize];
        let ab = [v[0] - a[0], v[1] - a[1], v[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        area += (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt() * 0.5;
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bipyramid_vertex_count() {
        /* 6 equatorial + 2 apices = 8 */
        let b = build_bipyramid(6, 1.0, 1.0);
        assert_eq!(bipyramid_vertex_count(&b), 8);
    }

    #[test]
    fn test_bipyramid_tri_count() {
        /* 2*6 = 12 */
        let b = build_bipyramid(6, 1.0, 1.0);
        assert_eq!(bipyramid_tri_count(&b), 12);
    }

    #[test]
    fn test_bipyramid_expected_tris() {
        assert_eq!(bipyramid_expected_tris(6), 12);
    }

    #[test]
    fn test_validate_bipyramid() {
        let b = build_bipyramid(5, 1.5, 2.0);
        assert!(validate_bipyramid(&b));
    }

    #[test]
    fn test_bipyramid_empty_on_too_few_sides() {
        let b = build_bipyramid(2, 1.0, 1.0);
        assert_eq!(bipyramid_vertex_count(&b), 0);
    }

    #[test]
    fn test_bipyramid_surface_area_positive() {
        let b = build_bipyramid(8, 1.0, 1.0);
        assert!(bipyramid_surface_area(&b) > 0.0);
    }

    #[test]
    fn test_bipyramid_sides_stored() {
        let b = build_bipyramid(7, 1.0, 2.0);
        assert_eq!(b.sides, 7);
    }

    #[test]
    fn test_bipyramid_apex_height_stored() {
        let b = build_bipyramid(4, 1.0, 3.5);
        assert!((b.apex_height - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_triangular_bipyramid() {
        /* triangular bipyramid = regular octahedron? No, different. 3 sides → 5 verts, 6 tris */
        let b = build_bipyramid(3, 1.0, 1.0);
        assert_eq!(bipyramid_vertex_count(&b), 5);
        assert_eq!(bipyramid_tri_count(&b), 6);
    }
}
