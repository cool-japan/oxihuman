// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cylinder mesh generation (with optional caps).

#![allow(dead_code)]

use std::f32::consts::PI;

/// Return the number of vertices for a cylinder.
/// Each ring has `segs` vertices, and there are 2 rings + optional cap centers.
#[allow(dead_code)]
pub fn cylinder_vert_count(segs: u32, capped: bool) -> usize {
    if capped {
        (2 * segs + 2) as usize
    } else {
        (2 * segs) as usize
    }
}

/// Lateral surface area of a cylinder: 2 * pi * r * h
/// If capped, add 2 * pi * r^2
#[allow(dead_code)]
pub fn cylinder_surface_area(radius: f32, height: f32) -> f32 {
    2.0 * PI * radius * (height + radius)
}

/// Volume of a cylinder: pi * r^2 * h
#[allow(dead_code)]
pub fn cylinder_volume(radius: f32, height: f32) -> f32 {
    PI * radius * radius * height
}

/// Generate a cylinder mesh.
/// `capped` controls whether top/bottom caps are generated.
/// Returns (vertices, triangles).
#[allow(dead_code)]
pub fn make_cylinder(
    radius: f32,
    height: f32,
    segments: u32,
    capped: bool,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if segments < 3 {
        return (vec![], vec![]);
    }
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let half_h = height * 0.5;
    // Bottom ring
    for i in 0..segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        verts.push([radius * angle.cos(), -half_h, radius * angle.sin()]);
    }
    // Top ring
    for i in 0..segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        verts.push([radius * angle.cos(), half_h, radius * angle.sin()]);
    }
    let mut tris: Vec<[u32; 3]> = Vec::new();
    // Side faces
    for i in 0..segments {
        let next_i = (i + 1) % segments;
        let a = i;
        let b = next_i;
        let c = segments + i;
        let d = segments + next_i;
        tris.push([a, b, c]);
        tris.push([b, d, c]);
    }
    // Caps
    if capped {
        let bottom_center_idx = verts.len() as u32;
        verts.push([0.0, -half_h, 0.0]);
        let top_center_idx = verts.len() as u32;
        verts.push([0.0, half_h, 0.0]);
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            // Bottom cap (winding: clockwise from below)
            tris.push([bottom_center_idx, next_i, i]);
            // Top cap
            tris.push([top_center_idx, segments + i, segments + next_i]);
        }
    }
    (verts, tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vert_count_uncapped() {
        assert_eq!(cylinder_vert_count(8, false), 16);
    }

    #[test]
    fn test_vert_count_capped() {
        assert_eq!(cylinder_vert_count(8, true), 18);
    }

    #[test]
    fn test_make_cylinder_uncapped_verts() {
        let (verts, _) = make_cylinder(1.0, 2.0, 8, false);
        assert_eq!(verts.len(), 16);
    }

    #[test]
    fn test_make_cylinder_capped_verts() {
        let (verts, _) = make_cylinder(1.0, 2.0, 8, true);
        assert_eq!(verts.len(), 18);
    }

    #[test]
    fn test_make_cylinder_few_segs() {
        let (verts, tris) = make_cylinder(1.0, 2.0, 2, false);
        assert!(verts.is_empty());
        assert!(tris.is_empty());
    }

    #[test]
    fn test_surface_area_positive() {
        let area = cylinder_surface_area(1.0, 2.0);
        assert!(area > 0.0);
    }

    #[test]
    fn test_volume_formula() {
        let vol = cylinder_volume(1.0, 1.0);
        assert!((vol - PI).abs() < 1e-4);
    }

    #[test]
    fn test_indices_in_range() {
        let (verts, tris) = make_cylinder(1.0, 2.0, 10, true);
        let nv = verts.len() as u32;
        for tri in &tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_uncapped_tri_count() {
        let (_, tris) = make_cylinder(1.0, 2.0, 8, false);
        // 8 quads * 2 tris
        assert_eq!(tris.len(), 16);
    }

    #[test]
    fn test_capped_tri_count() {
        let (_, tris) = make_cylinder(1.0, 2.0, 8, true);
        // 8 side quads * 2 + 8 bottom + 8 top
        assert_eq!(tris.len(), 32);
    }
}
