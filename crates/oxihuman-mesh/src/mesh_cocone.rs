// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Co-cone algorithm stub for surface reconstruction.

use std::f32::consts::PI;

/// Config for the co-cone algorithm.
#[allow(dead_code)]
pub struct CoconeConfig {
    pub cocone_angle: f32,
    pub flatness_ratio: f32,
}

#[allow(dead_code)]
impl Default for CoconeConfig {
    fn default() -> Self {
        Self { cocone_angle: PI / 8.0, flatness_ratio: 0.12 }
    }
}

/// Result of co-cone surface reconstruction.
#[allow(dead_code)]
pub struct CoconeResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub is_watertight: bool,
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v,v).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 { v } else { [v[0]/l, v[1]/l, v[2]/l] }
}

/// Check if a direction is inside the co-cone of a vertex.
#[allow(dead_code)]
pub fn in_cocone(dir: [f32; 3], normal: [f32; 3], cocone_angle: f32) -> bool {
    let d = normalize3(dir);
    let cos_a = dot3(d, normal).abs();
    cos_a >= cocone_angle.cos()
}

/// Estimate the normal at a vertex from its neighbours (average direction stub).
#[allow(dead_code)]
pub fn estimate_normal_from_neighbours(p: [f32; 3], neighbours: &[[f32; 3]]) -> [f32; 3] {
    if neighbours.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let mut avg = [0.0f32; 3];
    for &nb in neighbours {
        let d = sub3(nb, p);
        avg[0] += d[0];
        avg[1] += d[1];
        avg[2] += d[2];
    }
    normalize3(avg)
}

/// Check flatness: ratio of smallest to largest principal curvature.
#[allow(dead_code)]
pub fn is_flat_region(variance_ratios: [f32; 2], threshold: f32) -> bool {
    if variance_ratios[1] < 1e-10 {
        return false;
    }
    variance_ratios[0] / variance_ratios[1] < threshold
}

/// Co-cone reconstruction stub.
#[allow(dead_code)]
pub fn cocone_reconstruct_stub(
    points: &[[f32; 3]],
    normals: &[[f32; 3]],
    _config: &CoconeConfig,
) -> CoconeResult {
    let n = points.len();
    let mut triangles = Vec::new();
    if n >= 3 {
        for i in 1..(n - 1) {
            let dir_ij = sub3(points[i], points[0]);
            let dir_ik = sub3(points[i + 1], points[0]);
            let same_side = dot3(dir_ij, normals[0]) * dot3(dir_ik, normals[0]) >= 0.0;
            if same_side {
                triangles.push([0, i, i + 1]);
            }
        }
    }
    CoconeResult {
        positions: points.to_vec(),
        normals: normals.to_vec(),
        triangles,
        is_watertight: false,
    }
}

/// Count triangles.
#[allow(dead_code)]
pub fn cocone_triangle_count(result: &CoconeResult) -> usize {
    result.triangles.len()
}

/// Compute the cocone condition angle.
#[allow(dead_code)]
pub fn cocone_condition_angle(config: &CoconeConfig) -> f32 {
    PI / 2.0 - config.cocone_angle
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,0.5]]
    }

    #[test]
    fn in_cocone_normal_direction() {
        let n = [0.0, 0.0, 1.0];
        assert!(in_cocone([0.0, 0.0, 1.0], n, PI / 4.0));
    }

    #[test]
    fn in_cocone_orthogonal_direction() {
        let n = [0.0, 0.0, 1.0];
        assert!(!in_cocone([1.0, 0.0, 0.0], n, PI / 4.0));
    }

    #[test]
    fn estimate_normal_nonzero() {
        let p = [0.0, 0.0, 0.0];
        let nb = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let n = estimate_normal_from_neighbours(p, &nb);
        let l = n[0]*n[0]+n[1]*n[1]+n[2]*n[2];
        assert!((l - 1.0).abs() < 1e-5);
    }

    #[test]
    fn estimate_normal_empty_neighbours() {
        let n = estimate_normal_from_neighbours([0.0,0.0,0.0], &[]);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn is_flat_threshold() {
        assert!(is_flat_region([0.01, 1.0], 0.1));
        assert!(!is_flat_region([0.5, 1.0], 0.1));
    }

    #[test]
    fn cocone_stub_returns_points() {
        let pts = sample();
        let normals = vec![[0.0,0.0,1.0f32]; pts.len()];
        let cfg = CoconeConfig::default();
        let r = cocone_reconstruct_stub(&pts, &normals, &cfg);
        assert_eq!(r.positions.len(), pts.len());
    }

    #[test]
    fn triangle_count_accessible() {
        let pts = sample();
        let normals = vec![[0.0,0.0,1.0f32]; pts.len()];
        let cfg = CoconeConfig::default();
        let r = cocone_reconstruct_stub(&pts, &normals, &cfg);
        let _ = cocone_triangle_count(&r);
    }

    #[test]
    fn condition_angle_positive() {
        let cfg = CoconeConfig::default();
        let a = cocone_condition_angle(&cfg);
        assert!(a > 0.0);
    }

    #[test]
    fn pi_is_used() {
        let _ = PI;
        let cfg = CoconeConfig::default();
        assert!(cfg.cocone_angle > 0.0 && cfg.cocone_angle < PI);
    }
}
