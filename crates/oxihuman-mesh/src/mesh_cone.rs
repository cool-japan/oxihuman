// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cone mesh generation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Return the number of vertices for a cone.
/// Base ring (segs) + apex vertex + optional base center.
#[allow(dead_code)]
pub fn cone_vert_count(segs: u32) -> usize {
    // base ring + apex + base center
    (segs + 2) as usize
}

/// Lateral surface area of a cone: pi * r * slant_height
/// Total with base: pi * r * (slant + r)
#[allow(dead_code)]
pub fn cone_surface_area(radius: f32, height: f32) -> f32 {
    let slant = (radius * radius + height * height).sqrt();
    PI * radius * (slant + radius)
}

/// Volume of a cone: (1/3) * pi * r^2 * h
#[allow(dead_code)]
pub fn cone_volume(radius: f32, height: f32) -> f32 {
    PI * radius * radius * height / 3.0
}

/// Generate a cone mesh.
/// Returns (vertices, triangles).
#[allow(dead_code)]
pub fn make_cone(radius: f32, height: f32, segments: u32) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if segments < 3 {
        return (vec![], vec![]);
    }
    let mut verts: Vec<[f32; 3]> = Vec::new();
    // Base ring
    for i in 0..segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        verts.push([radius * angle.cos(), 0.0, radius * angle.sin()]);
    }
    // Apex
    let apex_idx = segments;
    verts.push([0.0, height, 0.0]);
    // Base center
    let base_center_idx = segments + 1;
    verts.push([0.0, 0.0, 0.0]);

    let mut tris: Vec<[u32; 3]> = Vec::new();
    for i in 0..segments {
        let next_i = (i + 1) % segments;
        // Side face
        tris.push([i, next_i, apex_idx]);
        // Base cap
        tris.push([base_center_idx, next_i, i]);
    }
    (verts, tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cone_vert_count() {
        assert_eq!(cone_vert_count(8), 10);
    }

    #[test]
    fn test_make_cone_vert_count() {
        let (verts, _) = make_cone(1.0, 2.0, 8);
        assert_eq!(verts.len(), cone_vert_count(8));
    }

    #[test]
    fn test_make_cone_too_few_segs() {
        let (verts, tris) = make_cone(1.0, 2.0, 2);
        assert!(verts.is_empty());
        assert!(tris.is_empty());
    }

    #[test]
    fn test_cone_surface_area_positive() {
        let area = cone_surface_area(1.0, 1.0);
        assert!(area > 0.0);
    }

    #[test]
    fn test_cone_volume_formula() {
        let vol = cone_volume(1.0, 3.0);
        let expected = PI;
        assert!((vol - expected).abs() < 1e-4);
    }

    #[test]
    fn test_cone_volume_positive() {
        let vol = cone_volume(2.0, 3.0);
        assert!(vol > 0.0);
    }

    #[test]
    fn test_apex_at_height() {
        let (verts, _) = make_cone(1.0, 5.0, 8);
        let apex = verts[8];
        assert!((apex[1] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_indices_in_range() {
        let (verts, tris) = make_cone(1.0, 2.0, 12);
        let nv = verts.len() as u32;
        for tri in &tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_tri_count() {
        let (_, tris) = make_cone(1.0, 2.0, 8);
        // 8 side tris + 8 base tris
        assert_eq!(tris.len(), 16);
    }
}
