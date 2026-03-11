// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cylinder mesh with end caps.

use std::f32::consts::PI;

/// Parameters for a capped cylinder mesh.
#[derive(Debug, Clone)]
pub struct CappedCylinderParams {
    /// Radius of the cylinder.
    pub radius: f32,
    /// Height of the cylinder.
    pub height: f32,
    /// Number of radial segments.
    pub radial_segments: usize,
    /// Number of height segments.
    pub height_segments: usize,
    /// Generate top cap.
    pub top_cap: bool,
    /// Generate bottom cap.
    pub bottom_cap: bool,
}

impl Default for CappedCylinderParams {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.0,
            radial_segments: 16,
            height_segments: 4,
            top_cap: true,
            bottom_cap: true,
        }
    }
}

/// Generated capped cylinder mesh.
#[derive(Debug, Clone)]
pub struct CappedCylinderMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl CappedCylinderMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Build a capped cylinder mesh.
pub fn build_capped_cylinder(params: &CappedCylinderParams) -> CappedCylinderMesh {
    let rs = params.radial_segments.max(3);
    let hs = params.height_segments.max(1);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    /* side */
    for i in 0..=hs {
        let y = i as f32 / hs as f32 * params.height - params.height * 0.5;
        let v = i as f32 / hs as f32;
        for j in 0..=rs {
            let angle = 2.0 * PI * j as f32 / rs as f32;
            let (s, c) = angle.sin_cos();
            positions.push([c * params.radius, y, s * params.radius]);
            normals.push([c, 0.0, s]);
            uvs.push([j as f32 / rs as f32, v]);
        }
    }
    let row = (rs + 1) as u32;
    for i in 0..(hs as u32) {
        for j in 0..(rs as u32) {
            let a = i * row + j;
            let b = i * row + j + 1;
            let c = (i + 1) * row + j;
            let d = (i + 1) * row + j + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    /* caps */
    if params.bottom_cap {
        let center_idx = positions.len() as u32;
        positions.push([0.0, -params.height * 0.5, 0.0]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([0.5, 0.5]);
        let ring_start = positions.len() as u32;
        for j in 0..=rs {
            let angle = 2.0 * PI * j as f32 / rs as f32;
            let (s, c) = angle.sin_cos();
            positions.push([c * params.radius, -params.height * 0.5, s * params.radius]);
            normals.push([0.0, -1.0, 0.0]);
            uvs.push([c * 0.5 + 0.5, s * 0.5 + 0.5]);
        }
        for j in 0..(rs as u32) {
            indices.extend_from_slice(&[center_idx, ring_start + j + 1, ring_start + j]);
        }
    }

    if params.top_cap {
        let center_idx = positions.len() as u32;
        positions.push([0.0, params.height * 0.5, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5, 0.5]);
        let ring_start = positions.len() as u32;
        for j in 0..=rs {
            let angle = 2.0 * PI * j as f32 / rs as f32;
            let (s, c) = angle.sin_cos();
            positions.push([c * params.radius, params.height * 0.5, s * params.radius]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([c * 0.5 + 0.5, s * 0.5 + 0.5]);
        }
        for j in 0..(rs as u32) {
            indices.extend_from_slice(&[center_idx, ring_start + j, ring_start + j + 1]);
        }
    }

    CappedCylinderMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Lateral surface area of the cylinder.
pub fn lateral_area(params: &CappedCylinderParams) -> f32 {
    2.0 * PI * params.radius * params.height
}

/// Total surface area including caps.
pub fn total_surface_area(params: &CappedCylinderParams) -> f32 {
    let caps = if params.top_cap { 1 } else { 0 } + if params.bottom_cap { 1 } else { 0 };
    lateral_area(params) + caps as f32 * PI * params.radius * params.radius
}

/// Validate capped cylinder params.
pub fn validate_params(p: &CappedCylinderParams) -> bool {
    p.radius > 0.0 && p.height > 0.0 && p.radial_segments >= 3 && p.height_segments >= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cylinder_has_vertices() {
        let m = build_capped_cylinder(&CappedCylinderParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn cylinder_has_triangles() {
        let m = build_capped_cylinder(&CappedCylinderParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_capped_cylinder(&CappedCylinderParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match() {
        let m = build_capped_cylinder(&CappedCylinderParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn no_caps_fewer_triangles() {
        /* without caps → fewer triangles */
        let with_caps = build_capped_cylinder(&CappedCylinderParams::default());
        let no_caps = build_capped_cylinder(&CappedCylinderParams {
            top_cap: false,
            bottom_cap: false,
            ..CappedCylinderParams::default()
        });
        assert!(with_caps.triangle_count() > no_caps.triangle_count());
    }

    #[test]
    fn lateral_area_positive() {
        assert!(lateral_area(&CappedCylinderParams::default()) > 0.0);
    }

    #[test]
    fn total_area_larger_than_lateral() {
        let p = CappedCylinderParams::default();
        assert!(total_surface_area(&p) > lateral_area(&p));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_params(&CappedCylinderParams::default()));
    }

    #[test]
    fn validate_bad_radius() {
        let p = CappedCylinderParams {
            radius: 0.0,
            ..CappedCylinderParams::default()
        };
        assert!(!validate_params(&p));
    }
}
