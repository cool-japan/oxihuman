// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Parametric torus mesh (ring torus).

use std::f32::consts::PI;

/// Parameters for a parametric torus.
#[derive(Debug, Clone)]
pub struct TorusRingParams {
    /// Distance from the center of the tube to the center of the torus.
    pub major_radius: f32,
    /// Radius of the tube.
    pub minor_radius: f32,
    /// Segments around the major circle.
    pub major_segments: usize,
    /// Segments around the tube cross-section.
    pub minor_segments: usize,
}

impl Default for TorusRingParams {
    fn default() -> Self {
        Self {
            major_radius: 0.3,
            minor_radius: 0.1,
            major_segments: 32,
            minor_segments: 16,
        }
    }
}

/// Generated torus ring mesh.
#[derive(Debug, Clone)]
pub struct TorusRingMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl TorusRingMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Evaluate a point on the torus at parameters (u, v) in `[0, 2π)`.
pub fn torus_point(params: &TorusRingParams, u: f32, v: f32) -> ([f32; 3], [f32; 3]) {
    let (su, cu) = u.sin_cos();
    let (sv, cv) = v.sin_cos();
    let x = (params.major_radius + params.minor_radius * cv) * cu;
    let y = params.minor_radius * sv;
    let z = (params.major_radius + params.minor_radius * cv) * su;
    /* normal: outward from tube center */
    let nx = cv * cu;
    let ny = sv;
    let nz = cv * su;
    ([x, y, z], [nx, ny, nz])
}

/// Build the torus ring mesh.
pub fn build_torus_ring(params: &TorusRingParams) -> TorusRingMesh {
    let maj = params.major_segments.max(4);
    let min = params.minor_segments.max(3);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for i in 0..=maj {
        let u = 2.0 * PI * i as f32 / maj as f32;
        for j in 0..=min {
            let v = 2.0 * PI * j as f32 / min as f32;
            let (pos, nrm) = torus_point(params, u, v);
            positions.push(pos);
            normals.push(nrm);
            uvs.push([i as f32 / maj as f32, j as f32 / min as f32]);
        }
    }
    let row = (min + 1) as u32;
    let mut indices = Vec::new();
    for i in 0..(maj as u32) {
        for j in 0..(min as u32) {
            let a = i * row + j;
            let b = i * row + j + 1;
            let c = (i + 1) * row + j;
            let d = (i + 1) * row + j + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    TorusRingMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Surface area of the torus (analytical).
pub fn torus_surface_area(params: &TorusRingParams) -> f32 {
    4.0 * PI * PI * params.major_radius * params.minor_radius
}

/// Volume of the torus (analytical).
pub fn torus_volume(params: &TorusRingParams) -> f32 {
    2.0 * PI * PI * params.major_radius * params.minor_radius * params.minor_radius
}

/// Validate torus ring params.
pub fn validate_torus_params(p: &TorusRingParams) -> bool {
    p.major_radius > p.minor_radius
        && p.minor_radius > 0.0
        && p.major_segments >= 4
        && p.minor_segments >= 3
}

/// Expected vertex count.
pub fn expected_vertex_count(maj: usize, min: usize) -> usize {
    (maj + 1) * (min + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torus_has_vertices() {
        let m = build_torus_ring(&TorusRingParams::default());
        assert_eq!(m.vertex_count(), expected_vertex_count(32, 16));
    }

    #[test]
    fn torus_has_triangles() {
        let m = build_torus_ring(&TorusRingParams::default());
        assert_eq!(m.triangle_count(), 32 * 16 * 2);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_torus_ring(&TorusRingParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match() {
        let m = build_torus_ring(&TorusRingParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn surface_area_positive() {
        assert!(torus_surface_area(&TorusRingParams::default()) > 0.0);
    }

    #[test]
    fn volume_positive() {
        assert!(torus_volume(&TorusRingParams::default()) > 0.0);
    }

    #[test]
    fn validate_ok() {
        assert!(validate_torus_params(&TorusRingParams::default()));
    }

    #[test]
    fn validate_bad_radii() {
        let mut p = TorusRingParams::default();
        p.minor_radius = p.major_radius + 0.1;
        assert!(!validate_torus_params(&p));
    }

    #[test]
    fn torus_point_at_u0_v0() {
        /* at u=0,v=0 the point should be at (R+r, 0, 0) */
        let p = TorusRingParams::default();
        let (pos, _) = torus_point(&p, 0.0, 0.0);
        let expected = p.major_radius + p.minor_radius;
        assert!((pos[0] - expected).abs() < 1e-5);
    }
}
