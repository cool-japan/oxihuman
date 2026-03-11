// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct BoxParams {
    pub size: [f32; 3],
    pub subdivisions: [u32; 3],
}

pub fn new_box_mesh(w: f32, h: f32, d: f32) -> BoxParams {
    BoxParams {
        size: [w, h, d],
        subdivisions: [1, 1, 1],
    }
}

pub fn box_vertex_count(p: &BoxParams) -> usize {
    let sx = p.subdivisions[0] as usize + 1;
    let sy = p.subdivisions[1] as usize + 1;
    let sz = p.subdivisions[2] as usize + 1;
    2 * (sx * sy + sy * sz + sx * sz)
}

pub fn box_face_count(p: &BoxParams) -> usize {
    let sx = p.subdivisions[0] as usize;
    let sy = p.subdivisions[1] as usize;
    let sz = p.subdivisions[2] as usize;
    4 * (sx * sy + sy * sz + sx * sz)
}

pub fn box_volume(p: &BoxParams) -> f32 {
    p.size[0] * p.size[1] * p.size[2]
}

pub fn box_surface_area(p: &BoxParams) -> f32 {
    let w = p.size[0];
    let h = p.size[1];
    let d = p.size[2];
    2.0 * (w * h + h * d + w * d)
}

pub fn box_diagonal(p: &BoxParams) -> f32 {
    let w = p.size[0];
    let h = p.size[1];
    let d = p.size[2];
    (w * w + h * h + d * d).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_box() {
        /* construction */
        let b = new_box_mesh(1.0, 2.0, 3.0);
        assert!((b.size[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_box_volume() {
        /* w*h*d */
        let b = new_box_mesh(2.0, 3.0, 4.0);
        assert!((box_volume(&b) - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_box_surface_area() {
        /* cube: 6*a^2 */
        let b = new_box_mesh(1.0, 1.0, 1.0);
        assert!((box_surface_area(&b) - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_box_diagonal() {
        /* sqrt(3) for unit cube */
        let b = new_box_mesh(1.0, 1.0, 1.0);
        assert!((box_diagonal(&b) - 3.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn test_box_vertex_count() {
        /* > 0 */
        let b = new_box_mesh(1.0, 1.0, 1.0);
        assert!(box_vertex_count(&b) > 0);
    }

    #[test]
    fn test_box_face_count() {
        /* > 0 */
        let b = new_box_mesh(1.0, 1.0, 1.0);
        assert!(box_face_count(&b) > 0);
    }
}
