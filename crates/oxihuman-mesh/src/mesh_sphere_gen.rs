// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

pub struct SphereParams {
    pub radius: f32,
    pub subdivisions: u32,
}

pub fn new_sphere_params(radius: f32, subdivisions: u32) -> SphereParams {
    SphereParams {
        radius,
        subdivisions,
    }
}

pub fn uv_sphere_vertex(
    radius: f32,
    lat_idx: u32,
    lon_idx: u32,
    lat_segments: u32,
    lon_segments: u32,
) -> [f32; 3] {
    let lat = (lat_idx as f32 / lat_segments as f32) * PI - PI * 0.5;
    let lon = (lon_idx as f32 / lon_segments as f32) * TAU;
    let x = radius * lat.cos() * lon.cos();
    let y = radius * lat.sin();
    let z = radius * lat.cos() * lon.sin();
    [x, y, z]
}

pub fn uv_sphere_vertex_count(lat_segs: u32, lon_segs: u32) -> usize {
    ((lat_segs + 1) * (lon_segs + 1)) as usize
}

pub fn uv_sphere_face_count(lat_segs: u32, lon_segs: u32) -> usize {
    (lat_segs * lon_segs * 2) as usize
}

pub fn icosphere_vertex_count(subdivisions: u32) -> usize {
    10 * 4_usize.pow(subdivisions) + 2
}

pub fn sphere_surface_area(r: f32) -> f32 {
    4.0 * PI * r * r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sphere_params() {
        /* construction */
        let s = new_sphere_params(1.5, 3);
        assert!((s.radius - 1.5).abs() < 1e-6);
        assert_eq!(s.subdivisions, 3);
    }

    #[test]
    fn test_uv_sphere_vertex_on_surface() {
        /* vertex should be on sphere surface */
        let v = uv_sphere_vertex(2.0, 4, 4, 8, 8);
        let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((r - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_uv_sphere_vertex_count() {
        /* (lat+1)*(lon+1) */
        let count = uv_sphere_vertex_count(8, 16);
        assert_eq!(count, 9 * 17);
    }

    #[test]
    fn test_uv_sphere_face_count() {
        /* lat*lon*2 */
        let count = uv_sphere_face_count(8, 16);
        assert_eq!(count, 8 * 16 * 2);
    }

    #[test]
    fn test_icosphere_vertex_count_0() {
        /* 12 vertices at subdivision 0 */
        assert_eq!(icosphere_vertex_count(0), 12);
    }

    #[test]
    fn test_icosphere_vertex_count_1() {
        /* 42 vertices at subdivision 1 */
        assert_eq!(icosphere_vertex_count(1), 42);
    }

    #[test]
    fn test_sphere_surface_area() {
        /* 4*pi*r^2 */
        let area = sphere_surface_area(1.0);
        assert!((area - 4.0 * PI).abs() < 1e-4);
    }
}
