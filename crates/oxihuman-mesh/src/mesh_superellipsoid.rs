// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Superellipsoid parametric mesh generation.

use std::f32::consts::PI;

/// Parameters for a superellipsoid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SuperellipsoidParams {
    /// Exponent controlling north-south shape (e1).
    pub e1: f32,
    /// Exponent controlling east-west shape (e2).
    pub e2: f32,
    /// Scale along each axis.
    pub scale: [f32; 3],
    /// Latitude segments.
    pub lat_segs: usize,
    /// Longitude segments.
    pub lon_segs: usize,
}

/// A superellipsoid mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SuperellipsoidMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Signed power function: sign(x)*|x|^p.
#[allow(dead_code)]
pub fn spow(x: f32, p: f32) -> f32 {
    let s = x.signum();
    s * x.abs().powf(p)
}

/// Evaluate the superellipsoid surface at (lat, lon) angles.
/// lat ∈ [-π/2, π/2], lon ∈ [-π, π].
#[allow(dead_code)]
pub fn superellipsoid_point(lat: f32, lon: f32, e1: f32, e2: f32, scale: [f32; 3]) -> [f32; 3] {
    let (slat, clat) = lat.sin_cos();
    let (slon, clon) = lon.sin_cos();
    [
        scale[0] * spow(clat, e1) * spow(clon, e2),
        scale[1] * spow(clat, e1) * spow(slon, e2),
        scale[2] * spow(slat, e1),
    ]
}

/// Build a superellipsoid mesh.
#[allow(dead_code)]
pub fn build_superellipsoid_mesh(params: &SuperellipsoidParams) -> SuperellipsoidMesh {
    let nlat = params.lat_segs.max(2);
    let nlon = params.lon_segs.max(3);

    let mut positions = Vec::with_capacity((nlat + 1) * (nlon + 1));
    for ilat in 0..=nlat {
        let lat = -PI * 0.5 + PI * ilat as f32 / nlat as f32;
        for ilon in 0..=nlon {
            let lon = -PI + 2.0 * PI * ilon as f32 / nlon as f32;
            positions.push(superellipsoid_point(
                lat,
                lon,
                params.e1,
                params.e2,
                params.scale,
            ));
        }
    }

    let mut indices = Vec::new();
    for ilat in 0..nlat {
        for ilon in 0..nlon {
            let a = (ilat * (nlon + 1) + ilon) as u32;
            let b = (ilat * (nlon + 1) + ilon + 1) as u32;
            let c = ((ilat + 1) * (nlon + 1) + ilon) as u32;
            let d = ((ilat + 1) * (nlon + 1) + ilon + 1) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    SuperellipsoidMesh { positions, indices }
}

/// Vertex count.
#[allow(dead_code)]
pub fn superellipsoid_vertex_count(m: &SuperellipsoidMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn superellipsoid_triangle_count(m: &SuperellipsoidMesh) -> usize {
    m.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_params() -> SuperellipsoidParams {
        SuperellipsoidParams {
            e1: 1.0,
            e2: 1.0,
            scale: [1.0, 1.0, 1.0],
            lat_segs: 8,
            lon_segs: 8,
        }
    }

    fn cube_params() -> SuperellipsoidParams {
        SuperellipsoidParams {
            e1: 0.1,
            e2: 0.1,
            scale: [1.0, 1.0, 1.0],
            lat_segs: 8,
            lon_segs: 8,
        }
    }

    #[test]
    fn sphere_vertex_count() {
        let p = sphere_params();
        let m = build_superellipsoid_mesh(&p);
        assert_eq!(m.positions.len(), (p.lat_segs + 1) * (p.lon_segs + 1));
    }

    #[test]
    fn indices_multiple_of_three() {
        let m = build_superellipsoid_mesh(&sphere_params());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn all_positions_finite() {
        let m = build_superellipsoid_mesh(&sphere_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn cube_positions_finite() {
        let m = build_superellipsoid_mesh(&cube_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn spow_preserves_sign() {
        assert!(spow(-2.0, 2.0) < 0.0);
        assert!(spow(2.0, 2.0) > 0.0);
    }

    #[test]
    fn spow_zero() {
        assert!((spow(0.0, 2.0)).abs() < 1e-6);
    }

    #[test]
    fn vertex_count_helper() {
        let m = build_superellipsoid_mesh(&sphere_params());
        assert_eq!(superellipsoid_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn triangle_count_helper() {
        let m = build_superellipsoid_mesh(&sphere_params());
        assert_eq!(superellipsoid_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn index_max_within_bounds() {
        let m = build_superellipsoid_mesh(&sphere_params());
        let max_idx = m.indices.iter().copied().max().unwrap_or(0) as usize;
        assert!(max_idx < m.positions.len());
    }

    #[test]
    fn scale_affects_positions() {
        let mut p = sphere_params();
        p.scale = [2.0, 1.0, 1.0];
        let m = build_superellipsoid_mesh(&p);
        let max_x = m
            .positions
            .iter()
            .map(|p| p[0].abs())
            .fold(0.0_f32, f32::max);
        assert!(max_x > 1.5);
    }
}
