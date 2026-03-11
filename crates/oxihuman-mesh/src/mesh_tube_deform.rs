// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A path with per-point radii for tube deformation.
pub struct TubePath {
    pub points: Vec<[f32; 3]>,
    pub radii: Vec<f32>,
}

pub fn new_tube_path(points: Vec<[f32; 3]>) -> TubePath {
    let n = points.len();
    TubePath {
        points,
        radii: vec![1.0; n],
    }
}

pub fn tube_path_length(p: &TubePath) -> f32 {
    let n = p.points.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n - 1 {
        let a = p.points[i];
        let b = p.points[i + 1];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

pub fn tube_path_at(p: &TubePath, t: f32) -> [f32; 3] {
    let n = p.points.len();
    if n == 0 {
        return [0.0; 3];
    }
    if n == 1 {
        return p.points[0];
    }
    let t = t.clamp(0.0, 1.0);
    let seg_t = t * (n - 1) as f32;
    let i = (seg_t as usize).min(n - 2);
    let local = seg_t - i as f32;
    let a = p.points[i];
    let b = p.points[i + 1];
    [
        a[0] + (b[0] - a[0]) * local,
        a[1] + (b[1] - a[1]) * local,
        a[2] + (b[2] - a[2]) * local,
    ]
}

pub fn tube_radius_at(p: &TubePath, t: f32) -> f32 {
    let n = p.radii.len();
    if n == 0 {
        return 1.0;
    }
    if n == 1 {
        return p.radii[0];
    }
    let t = t.clamp(0.0, 1.0);
    let seg_t = t * (n - 1) as f32;
    let i = (seg_t as usize).min(n - 2);
    let local = seg_t - i as f32;
    p.radii[i] + (p.radii[i + 1] - p.radii[i]) * local
}

pub fn tube_point_count(p: &TubePath) -> usize {
    p.points.len()
}

pub fn tube_deform_vertex(p: &TubePath, v: [f32; 3], axis_blend: f32) -> [f32; 3] {
    let t = (v[1] * 0.5 + 0.5).clamp(0.0, 1.0);
    let center = tube_path_at(p, t);
    let r = tube_radius_at(p, t);
    let blend = axis_blend.clamp(0.0, 1.0);
    [
        v[0] * (1.0 - blend) + (center[0] + v[0] * r) * blend,
        v[1] * (1.0 - blend) + center[1] * blend,
        v[2] * (1.0 - blend) + (center[2] + v[2] * r) * blend,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tube_path_radius_default() {
        let p = new_tube_path(vec![[0.0; 3]; 3]);
        assert_eq!(p.radii.len(), 3);
        assert!((p.radii[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tube_path_length_two_points() {
        let p = new_tube_path(vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]]);
        assert!((tube_path_length(&p) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_tube_path_at_endpoints() {
        let p = new_tube_path(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let start = tube_path_at(&p, 0.0);
        let end = tube_path_at(&p, 1.0);
        assert!((start[0] - 0.0).abs() < 1e-5);
        assert!((end[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_tube_path_at_midpoint() {
        let p = new_tube_path(vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
        let mid = tube_path_at(&p, 0.5);
        assert!((mid[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_tube_radius_at_interpolation() {
        let mut p = new_tube_path(vec![[0.0; 3]; 2]);
        p.radii[0] = 1.0;
        p.radii[1] = 3.0;
        let r = tube_radius_at(&p, 0.5);
        assert!((r - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_tube_point_count() {
        let p = new_tube_path(vec![[0.0; 3]; 7]);
        assert_eq!(tube_point_count(&p), 7);
    }

    #[test]
    fn test_tube_deform_vertex_no_blend() {
        let p = new_tube_path(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        let v = [1.0, 0.5, 0.0];
        let out = tube_deform_vertex(&p, v, 0.0);
        /* with blend=0, vertex unchanged */
        assert!((out[0] - v[0]).abs() < 1e-5);
    }
}
