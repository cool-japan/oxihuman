// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

pub struct TorusParams {
    pub major_radius: f32,
    pub minor_radius: f32,
    pub major_segments: u32,
    pub minor_segments: u32,
}

pub fn new_torus(major: f32, minor: f32, major_segs: u32, minor_segs: u32) -> TorusParams {
    TorusParams {
        major_radius: major,
        minor_radius: minor,
        major_segments: major_segs,
        minor_segments: minor_segs,
    }
}

pub fn torus_vertex(p: &TorusParams, i: u32, j: u32) -> [f32; 3] {
    let theta = (i as f32 / p.major_segments as f32) * TAU;
    let phi = (j as f32 / p.minor_segments as f32) * TAU;
    let x = (p.major_radius + p.minor_radius * phi.cos()) * theta.cos();
    let y = (p.major_radius + p.minor_radius * phi.cos()) * theta.sin();
    let z = p.minor_radius * phi.sin();
    [x, y, z]
}

pub fn torus_vertex_count(p: &TorusParams) -> usize {
    (p.major_segments * p.minor_segments) as usize
}

pub fn torus_face_count(p: &TorusParams) -> usize {
    (p.major_segments * p.minor_segments * 2) as usize
}

pub fn torus_surface_area(p: &TorusParams) -> f32 {
    4.0 * PI * PI * p.major_radius * p.minor_radius
}

pub fn torus_volume(p: &TorusParams) -> f32 {
    2.0 * PI * PI * p.major_radius * p.minor_radius * p.minor_radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_torus() {
        /* construction */
        let t = new_torus(2.0, 0.5, 32, 16);
        assert!((t.major_radius - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_torus_vertex_count() {
        /* vertex count = major*minor */
        let t = new_torus(1.0, 0.3, 8, 6);
        assert_eq!(torus_vertex_count(&t), 48);
    }

    #[test]
    fn test_torus_face_count() {
        /* face count = 2*major*minor */
        let t = new_torus(1.0, 0.3, 8, 6);
        assert_eq!(torus_face_count(&t), 96);
    }

    #[test]
    fn test_torus_surface_area() {
        /* 4pi^2*R*r */
        let t = new_torus(2.0, 1.0, 32, 16);
        let area = torus_surface_area(&t);
        let expected = 4.0 * PI * PI * 2.0 * 1.0;
        assert!((area - expected).abs() < 1e-3);
    }

    #[test]
    fn test_torus_volume() {
        /* 2pi^2*R*r^2 */
        let t = new_torus(3.0, 1.0, 32, 16);
        let vol = torus_volume(&t);
        let expected = 2.0 * PI * PI * 3.0 * 1.0;
        assert!((vol - expected).abs() < 1e-3);
    }

    #[test]
    fn test_torus_vertex_radius() {
        /* vertex at i=0,j=0 has x=R+r, y=0, z=0 */
        let t = new_torus(2.0, 0.5, 32, 16);
        let v = torus_vertex(&t, 0, 0);
        assert!((v[0] - 2.5).abs() < 1e-5);
        assert!(v[1].abs() < 1e-5);
    }

    #[test]
    fn test_torus_vertex_count_different() {
        /* different segment counts */
        let t = new_torus(1.0, 0.2, 4, 4);
        assert_eq!(torus_vertex_count(&t), 16);
    }
}
