// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! UV sphere mesh generation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Return the number of vertices for a UV sphere.
/// Includes 2 pole vertices plus (lat_segs-1) * lon_segs band vertices.
#[allow(dead_code)]
pub fn sphere_vert_count(lat: u32, lon: u32) -> usize {
    2 + ((lat - 1) * lon) as usize
}

/// Surface area of a sphere: 4 * pi * r^2
#[allow(dead_code)]
pub fn sphere_surface_area(radius: f32) -> f32 {
    4.0 * PI * radius * radius
}

/// Compute spherical UV coordinates for a list of vertices.
/// Uses atan2/asin mapping to [0,1] x [0,1].
#[allow(dead_code)]
pub fn sphere_uv_coords(verts: &[[f32; 3]]) -> Vec<[f32; 2]> {
    verts
        .iter()
        .map(|v| {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-8);
            let u = (v[2].atan2(v[0]) / (2.0 * PI) + 0.5).clamp(0.0, 1.0);
            let lat = (v[1] / len).clamp(-1.0, 1.0).asin();
            let v_coord = (lat / PI + 0.5).clamp(0.0, 1.0);
            [u, v_coord]
        })
        .collect()
}

/// Generate a UV sphere mesh.
/// lat_segs: number of latitude bands, lon_segs: number of longitude segments.
/// Returns (vertices, triangles).
#[allow(dead_code)]
pub fn make_uv_sphere(
    radius: f32,
    lat_segs: u32,
    lon_segs: u32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if lat_segs < 2 || lon_segs < 3 {
        return (vec![], vec![]);
    }
    let mut verts: Vec<[f32; 3]> = Vec::new();
    // North pole
    verts.push([0.0, radius, 0.0]);
    for lat in 1..lat_segs {
        let phi = PI * lat as f32 / lat_segs as f32;
        let y = radius * phi.cos();
        let r = radius * phi.sin();
        for lon in 0..lon_segs {
            let theta = 2.0 * PI * lon as f32 / lon_segs as f32;
            verts.push([r * theta.cos(), y, r * theta.sin()]);
        }
    }
    // South pole
    verts.push([0.0, -radius, 0.0]);
    let south_pole_idx = (verts.len() - 1) as u32;

    let mut tris: Vec<[u32; 3]> = Vec::new();
    // Top cap
    for lon in 0..lon_segs {
        let next_lon = (lon + 1) % lon_segs;
        tris.push([0, 1 + lon, 1 + next_lon]);
    }
    // Body
    for lat in 0..(lat_segs - 2) {
        for lon in 0..lon_segs {
            let next_lon = (lon + 1) % lon_segs;
            let a = 1 + lat * lon_segs + lon;
            let b = 1 + lat * lon_segs + next_lon;
            let c = 1 + (lat + 1) * lon_segs + lon;
            let d = 1 + (lat + 1) * lon_segs + next_lon;
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    // Bottom cap
    let last_ring_start = 1 + (lat_segs - 2) * lon_segs;
    for lon in 0..lon_segs {
        let next_lon = (lon + 1) % lon_segs;
        tris.push([last_ring_start + lon, south_pole_idx, last_ring_start + next_lon]);
    }
    (verts, tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_vert_count() {
        assert_eq!(sphere_vert_count(4, 8), 2 + 3 * 8);
    }

    #[test]
    fn test_make_uv_sphere_vert_count() {
        let (verts, _) = make_uv_sphere(1.0, 4, 8);
        assert_eq!(verts.len(), sphere_vert_count(4, 8));
    }

    #[test]
    fn test_make_uv_sphere_empty() {
        let (verts, tris) = make_uv_sphere(1.0, 1, 8);
        assert!(verts.is_empty());
        assert!(tris.is_empty());
    }

    #[test]
    fn test_surface_area_unit_sphere() {
        let area = sphere_surface_area(1.0);
        let expected = 4.0 * PI;
        assert!((area - expected).abs() < 1e-4);
    }

    #[test]
    fn test_surface_area_scales() {
        let r = 2.0f32;
        let area = sphere_surface_area(r);
        let expected = 4.0 * PI * r * r;
        assert!((area - expected).abs() < 1e-4);
    }

    #[test]
    fn test_sphere_uv_coords_length() {
        let (verts, _) = make_uv_sphere(1.0, 6, 12);
        let uvs = sphere_uv_coords(&verts);
        assert_eq!(uvs.len(), verts.len());
    }

    #[test]
    fn test_sphere_uv_range() {
        let (verts, _) = make_uv_sphere(1.0, 6, 12);
        let uvs = sphere_uv_coords(&verts);
        for uv in &uvs {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }

    #[test]
    fn test_indices_in_range() {
        let (verts, tris) = make_uv_sphere(1.0, 6, 8);
        let nv = verts.len() as u32;
        for tri in &tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_poles_on_axis() {
        let (verts, _) = make_uv_sphere(1.0, 4, 8);
        // North pole
        assert!((verts[0][1] - 1.0).abs() < 1e-5);
        // South pole
        let last = verts.last().unwrap();
        assert!((last[1] + 1.0).abs() < 1e-5);
    }
}
