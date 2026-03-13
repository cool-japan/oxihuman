// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct PlaneParams {
    pub width: f32,
    pub height: f32,
    pub subdivisions_x: u32,
    pub subdivisions_y: u32,
}

pub fn new_plane(w: f32, h: f32, sx: u32, sy: u32) -> PlaneParams {
    PlaneParams {
        width: w,
        height: h,
        subdivisions_x: sx,
        subdivisions_y: sy,
    }
}

pub fn plane_vertex(p: &PlaneParams, ix: u32, iy: u32) -> [f32; 3] {
    let x = (ix as f32 / p.subdivisions_x as f32) * p.width - p.width * 0.5;
    let y = 0.0;
    let z = (iy as f32 / p.subdivisions_y as f32) * p.height - p.height * 0.5;
    [x, y, z]
}

pub fn plane_vertex_count(p: &PlaneParams) -> usize {
    ((p.subdivisions_x + 1) * (p.subdivisions_y + 1)) as usize
}

pub fn plane_face_count(p: &PlaneParams) -> usize {
    (p.subdivisions_x * p.subdivisions_y * 2) as usize
}

pub fn plane_uv(p: &PlaneParams, ix: u32, iy: u32) -> [f32; 2] {
    [
        ix as f32 / p.subdivisions_x as f32,
        iy as f32 / p.subdivisions_y as f32,
    ]
}

pub fn plane_area(p: &PlaneParams) -> f32 {
    p.width * p.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_plane() {
        /* construction */
        let pl = new_plane(4.0, 2.0, 8, 4);
        assert!((pl.width - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_plane_vertex_count() {
        /* (sx+1)*(sy+1) */
        let pl = new_plane(1.0, 1.0, 4, 4);
        assert_eq!(plane_vertex_count(&pl), 25);
    }

    #[test]
    fn test_plane_face_count() {
        /* sx*sy*2 */
        let pl = new_plane(1.0, 1.0, 4, 4);
        assert_eq!(plane_face_count(&pl), 32);
    }

    #[test]
    fn test_plane_uv_corners() {
        /* corner UVs are 0,0 and 1,1 */
        let pl = new_plane(1.0, 1.0, 4, 4);
        let uv00 = plane_uv(&pl, 0, 0);
        let uv11 = plane_uv(&pl, 4, 4);
        assert!((uv00[0]).abs() < 1e-6);
        assert!((uv11[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_plane_area() {
        /* width*height */
        let pl = new_plane(3.0, 4.0, 6, 8);
        assert!((plane_area(&pl) - 12.0).abs() < 1e-5);
    }

    #[test]
    fn test_plane_vertex_center() {
        /* center vertex is near origin */
        let pl = new_plane(2.0, 2.0, 2, 2);
        let v = plane_vertex(&pl, 1, 1);
        assert!(v[0].abs() < 1e-5);
        assert!(v[2].abs() < 1e-5);
    }

    #[test]
    fn test_plane_vertex_corner() {
        /* corner vertex */
        let pl = new_plane(2.0, 2.0, 2, 2);
        let v = plane_vertex(&pl, 0, 0);
        assert!((v[0] + 1.0).abs() < 1e-5);
    }
}
