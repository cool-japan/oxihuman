// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Circle rendering utilities for the 3D viewer.

use std::f32::consts::PI;

/// A circle to render.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Circle {
    pub center: [f32; 3],
    pub radius: f32,
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub segments: u32,
}

/// Circle batch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CircleBatch {
    pub circles: Vec<Circle>,
}

/// Create a new circle.
#[allow(dead_code)]
pub fn new_circle(center: [f32; 3], radius: f32) -> Circle {
    Circle {
        center,
        radius,
        normal: [0.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        segments: 32,
    }
}

/// Generate circle vertices on the XZ plane.
#[allow(dead_code)]
pub fn generate_circle_vertices(circle: &Circle) -> Vec<[f32; 3]> {
    let n = circle.segments.max(3);
    let mut verts = Vec::with_capacity(n as usize);
    for i in 0..n {
        let angle = 2.0 * PI * (i as f32) / (n as f32);
        verts.push([
            circle.center[0] + circle.radius * angle.cos(),
            circle.center[1],
            circle.center[2] + circle.radius * angle.sin(),
        ]);
    }
    verts
}

/// Create empty batch.
#[allow(dead_code)]
pub fn new_circle_batch() -> CircleBatch {
    CircleBatch { circles: Vec::new() }
}

/// Add circle to batch.
#[allow(dead_code)]
pub fn add_circle(batch: &mut CircleBatch, circle: Circle) {
    batch.circles.push(circle);
}

/// Clear batch.
#[allow(dead_code)]
pub fn clear_circle_batch(batch: &mut CircleBatch) {
    batch.circles.clear();
}

/// Circle count.
#[allow(dead_code)]
pub fn circle_count(batch: &CircleBatch) -> usize {
    batch.circles.len()
}

/// Circle circumference.
#[allow(dead_code)]
pub fn circle_circumference(c: &Circle) -> f32 {
    2.0 * PI * c.radius
}

/// Circle area.
#[allow(dead_code)]
pub fn circle_area(c: &Circle) -> f32 {
    PI * c.radius * c.radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_circle() {
        let c = new_circle([0.0, 0.0, 0.0], 1.0);
        assert!((c.radius - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_vertices() {
        let c = new_circle([0.0, 0.0, 0.0], 1.0);
        let verts = generate_circle_vertices(&c);
        assert_eq!(verts.len(), 32);
    }

    #[test]
    fn test_vertex_on_circle() {
        let c = new_circle([0.0, 0.0, 0.0], 1.0);
        let verts = generate_circle_vertices(&c);
        let dist = (verts[0][0].powi(2) + verts[0][2].powi(2)).sqrt();
        assert!((dist - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_new_batch() {
        let b = new_circle_batch();
        assert!(b.circles.is_empty());
    }

    #[test]
    fn test_add_circle() {
        let mut b = new_circle_batch();
        add_circle(&mut b, new_circle([0.0, 0.0, 0.0], 1.0));
        assert_eq!(circle_count(&b), 1);
    }

    #[test]
    fn test_clear() {
        let mut b = new_circle_batch();
        add_circle(&mut b, new_circle([0.0, 0.0, 0.0], 1.0));
        clear_circle_batch(&mut b);
        assert_eq!(circle_count(&b), 0);
    }

    #[test]
    fn test_circumference() {
        let c = new_circle([0.0, 0.0, 0.0], 1.0);
        assert!((circle_circumference(&c) - 2.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_area() {
        let c = new_circle([0.0, 0.0, 0.0], 1.0);
        assert!((circle_area(&c) - PI).abs() < 1e-4);
    }

    #[test]
    fn test_offset_center() {
        let c = new_circle([1.0, 2.0, 3.0], 1.0);
        let verts = generate_circle_vertices(&c);
        // All verts should be distance 1 from center in XZ
        for v in &verts {
            let dx = v[0] - 1.0;
            let dz = v[2] - 3.0;
            let dist = (dx * dx + dz * dz).sqrt();
            assert!((dist - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn test_min_segments() {
        let mut c = new_circle([0.0, 0.0, 0.0], 1.0);
        c.segments = 1;
        let verts = generate_circle_vertices(&c);
        assert_eq!(verts.len(), 3);
    }
}
