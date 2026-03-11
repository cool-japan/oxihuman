// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

pub struct AnnulusParams {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub segments: u32,
    pub height: f32,
}

pub fn new_annulus(inner: f32, outer: f32, segments: u32) -> AnnulusParams {
    AnnulusParams {
        inner_radius: inner,
        outer_radius: outer,
        segments,
        height: 0.0,
    }
}

pub fn annulus_vertex(p: &AnnulusParams, ring: u8, seg: u32) -> [f32; 3] {
    let angle = (seg as f32 / p.segments as f32) * TAU;
    let r = if ring == 0 {
        p.inner_radius
    } else {
        p.outer_radius
    };
    [r * angle.cos(), r * angle.sin(), 0.0]
}

pub fn annulus_vertex_count(p: &AnnulusParams) -> usize {
    (p.segments * 2) as usize
}

pub fn annulus_face_count(p: &AnnulusParams) -> usize {
    (p.segments * 2) as usize
}

pub fn annulus_area(p: &AnnulusParams) -> f32 {
    PI * (p.outer_radius * p.outer_radius - p.inner_radius * p.inner_radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_annulus() {
        /* construction */
        let a = new_annulus(0.5, 1.0, 16);
        assert!((a.inner_radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_annulus_area() {
        /* pi*(R^2 - r^2) */
        let a = new_annulus(0.0, 1.0, 16);
        let area = annulus_area(&a);
        assert!((area - PI).abs() < 1e-4);
    }

    #[test]
    fn test_annulus_vertex_inner_outer() {
        /* inner radius at ring=0 */
        let a = new_annulus(0.5, 1.0, 16);
        let vi = annulus_vertex(&a, 0, 0);
        assert!((vi[0] - 0.5).abs() < 1e-5);
        let vo = annulus_vertex(&a, 1, 0);
        assert!((vo[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_annulus_vertex_count() {
        /* 2*segments */
        let a = new_annulus(0.5, 1.0, 8);
        assert_eq!(annulus_vertex_count(&a), 16);
    }

    #[test]
    fn test_annulus_face_count() {
        /* 2*segments */
        let a = new_annulus(0.5, 1.0, 8);
        assert_eq!(annulus_face_count(&a), 16);
    }

    #[test]
    fn test_annulus_area_positive() {
        /* area > 0 */
        let a = new_annulus(0.3, 0.8, 16);
        assert!(annulus_area(&a) > 0.0);
    }
}
