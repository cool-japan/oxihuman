// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Quad renderer for screen-space quads and sprites.

/// A screen-space quad.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ScreenQuad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

/// Quad batch state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuadBatch {
    pub quads: Vec<ScreenQuad>,
    pub blend_enabled: bool,
}

#[allow(dead_code)]
pub fn new_screen_quad(x: f32, y: f32, width: f32, height: f32) -> ScreenQuad {
    ScreenQuad { x, y, width, height, uv_min: [0.0, 0.0], uv_max: [1.0, 1.0] }
}

#[allow(dead_code)]
pub fn fullscreen_quad() -> ScreenQuad {
    ScreenQuad { x: -1.0, y: -1.0, width: 2.0, height: 2.0, uv_min: [0.0, 0.0], uv_max: [1.0, 1.0] }
}

#[allow(dead_code)]
pub fn new_quad_batch() -> QuadBatch {
    QuadBatch { quads: Vec::new(), blend_enabled: true }
}

#[allow(dead_code)]
pub fn add_quad(batch: &mut QuadBatch, quad: ScreenQuad) {
    batch.quads.push(quad);
}

#[allow(dead_code)]
pub fn quad_count(batch: &QuadBatch) -> usize {
    batch.quads.len()
}

#[allow(dead_code)]
pub fn clear_quads(batch: &mut QuadBatch) {
    batch.quads.clear();
}

#[allow(dead_code)]
pub fn quad_area(quad: &ScreenQuad) -> f32 {
    quad.width * quad.height
}

#[allow(dead_code)]
pub fn set_quad_uv(quad: &mut ScreenQuad, u_min: f32, v_min: f32, u_max: f32, v_max: f32) {
    quad.uv_min = [u_min, v_min];
    quad.uv_max = [u_max, v_max];
}

#[allow(dead_code)]
pub fn quad_center(quad: &ScreenQuad) -> [f32; 2] {
    [quad.x + quad.width * 0.5, quad.y + quad.height * 0.5]
}

#[allow(dead_code)]
pub fn is_batch_empty(batch: &QuadBatch) -> bool {
    batch.quads.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_screen_quad() {
        let q = new_screen_quad(0.0, 0.0, 100.0, 50.0);
        assert!((q.width - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_fullscreen_quad() {
        let q = fullscreen_quad();
        assert!((q.width - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_batch() {
        let b = new_quad_batch();
        assert!(is_batch_empty(&b));
    }

    #[test]
    fn test_add_quad() {
        let mut b = new_quad_batch();
        add_quad(&mut b, new_screen_quad(0.0, 0.0, 10.0, 10.0));
        assert_eq!(quad_count(&b), 1);
    }

    #[test]
    fn test_clear_quads() {
        let mut b = new_quad_batch();
        add_quad(&mut b, new_screen_quad(0.0, 0.0, 10.0, 10.0));
        clear_quads(&mut b);
        assert!(is_batch_empty(&b));
    }

    #[test]
    fn test_quad_area() {
        let q = new_screen_quad(0.0, 0.0, 10.0, 20.0);
        assert!((quad_area(&q) - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_quad_uv() {
        let mut q = new_screen_quad(0.0, 0.0, 10.0, 10.0);
        set_quad_uv(&mut q, 0.25, 0.25, 0.75, 0.75);
        assert!((q.uv_min[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_quad_center() {
        let q = new_screen_quad(10.0, 20.0, 100.0, 50.0);
        let c = quad_center(&q);
        assert!((c[0] - 60.0).abs() < 1e-6);
        assert!((c[1] - 45.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_quads() {
        let mut b = new_quad_batch();
        add_quad(&mut b, new_screen_quad(0.0, 0.0, 10.0, 10.0));
        add_quad(&mut b, new_screen_quad(20.0, 20.0, 10.0, 10.0));
        assert_eq!(quad_count(&b), 2);
    }

    #[test]
    fn test_blend_enabled_default() {
        let b = new_quad_batch();
        assert!(b.blend_enabled);
    }
}
