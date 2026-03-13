// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Frustum (tapered prism) mesh generator.

use std::f32::consts::TAU;

/// A frustum / tapered prism mesh.
#[derive(Debug, Clone)]
pub struct PrismFrustum {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub sides: usize,
    pub r_bottom: f32,
    pub r_top: f32,
    pub height: f32,
}

/// Build a frustum with `sides` sides, bottom radius `r_bottom`, top radius `r_top`,
/// and the given `height`. When `r_top == r_bottom` this is a prism; when `r_top == 0`
/// it is a cone.
pub fn build_prism_frustum(sides: usize, r_bottom: f32, r_top: f32, height: f32) -> PrismFrustum {
    if sides < 3 {
        return PrismFrustum {
            verts: vec![],
            tris: vec![],
            sides: 0,
            r_bottom: 0.0,
            r_top: 0.0,
            height: 0.0,
        };
    }
    let n = sides;
    let mut verts = Vec::with_capacity(2 * n + 2);
    /* bottom ring */
    for i in 0..n {
        let angle = TAU * i as f32 / n as f32;
        verts.push([r_bottom * angle.cos(), 0.0, r_bottom * angle.sin()]);
    }
    /* top ring */
    for i in 0..n {
        let angle = TAU * i as f32 / n as f32;
        verts.push([r_top * angle.cos(), height, r_top * angle.sin()]);
    }
    let bot_center = (2 * n) as u32;
    let top_center = (2 * n + 1) as u32;
    verts.push([0.0, 0.0, 0.0]);
    verts.push([0.0, height, 0.0]);

    let mut tris = Vec::new();
    /* lateral faces */
    for i in 0..n {
        let next = (i + 1) % n;
        let a = i as u32;
        let b = next as u32;
        let c = (n + i) as u32;
        let d = (n + next) as u32;
        tris.push([a, b, d]);
        tris.push([a, d, c]);
    }
    /* bottom cap */
    for i in 0..n {
        tris.push([bot_center, ((i + 1) % n) as u32, i as u32]);
    }
    /* top cap */
    for i in 0..n {
        tris.push([top_center, (n + i) as u32, (n + (i + 1) % n) as u32]);
    }
    PrismFrustum {
        verts,
        tris,
        sides: n,
        r_bottom,
        r_top,
        height,
    }
}

/// Return vertex count.
pub fn frustum_vertex_count(f: &PrismFrustum) -> usize {
    f.verts.len()
}

/// Return triangle count.
pub fn frustum_tri_count(f: &PrismFrustum) -> usize {
    f.tris.len()
}

/// Validate all triangle indices.
pub fn validate_prism_frustum(f: &PrismFrustum) -> bool {
    let n = f.verts.len() as u32;
    f.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute the approximate lateral surface area of the frustum.
pub fn frustum_lateral_area(f: &PrismFrustum) -> f32 {
    let slant = (f.height.powi(2) + (f.r_top - f.r_bottom).powi(2)).sqrt();
    std::f32::consts::PI * (f.r_bottom + f.r_top) * slant
}

/// Compute the volume of the frustum.
pub fn frustum_volume(f: &PrismFrustum) -> f32 {
    let rb = f.r_bottom;
    let rt = f.r_top;
    std::f32::consts::PI * f.height / 3.0 * (rb * rb + rb * rt + rt * rt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_vertex_count() {
        /* 6-sided: 6+6+2 = 14 verts */
        let f = build_prism_frustum(6, 1.0, 0.5, 2.0);
        assert_eq!(frustum_vertex_count(&f), 14);
    }

    #[test]
    fn test_frustum_tri_count() {
        /* 6*2 lateral + 6 bottom + 6 top = 24 */
        let f = build_prism_frustum(6, 1.0, 0.5, 2.0);
        assert_eq!(frustum_tri_count(&f), 24);
    }

    #[test]
    fn test_frustum_empty_on_few_sides() {
        let f = build_prism_frustum(2, 1.0, 1.0, 1.0);
        assert_eq!(frustum_vertex_count(&f), 0);
    }

    #[test]
    fn test_validate_prism_frustum() {
        let f = build_prism_frustum(5, 1.0, 0.7, 3.0);
        assert!(validate_prism_frustum(&f));
    }

    #[test]
    fn test_frustum_lateral_area_cylinder() {
        /* cylinder: r_bottom == r_top → area = 2*pi*r*h */
        let f = build_prism_frustum(32, 1.0, 1.0, 1.0);
        let expected = 2.0 * std::f32::consts::PI * 1.0 * 1.0;
        assert!((frustum_lateral_area(&f) - expected).abs() < 0.01);
    }

    #[test]
    fn test_frustum_volume_cylinder() {
        /* cylinder volume = pi*r^2*h */
        let f = build_prism_frustum(32, 1.0, 1.0, 1.0);
        let expected = std::f32::consts::PI * 1.0 * 1.0 * 1.0;
        assert!((frustum_volume(&f) - expected).abs() < 0.01);
    }

    #[test]
    fn test_frustum_cone_volume() {
        /* cone: r_top=0 → volume = pi*r^2*h/3 */
        let f = build_prism_frustum(32, 1.0, 0.0, 3.0);
        let expected = std::f32::consts::PI * 1.0 * 3.0 / 3.0;
        assert!((frustum_volume(&f) - expected).abs() < 0.01);
    }

    #[test]
    fn test_frustum_sides_stored() {
        let f = build_prism_frustum(8, 2.0, 1.0, 4.0);
        assert_eq!(f.sides, 8);
        assert!((f.r_bottom - 2.0).abs() < 1e-6);
        assert!((f.r_top - 1.0).abs() < 1e-6);
    }
}
