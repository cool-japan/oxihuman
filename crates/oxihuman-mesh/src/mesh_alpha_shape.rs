// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Alpha shape (generalized convex hull) from point clouds.

/// Config for alpha shape computation.
#[allow(dead_code)]
pub struct AlphaShapeConfig {
    pub alpha: f32,
}

/// Result of alpha shape extraction.
#[allow(dead_code)]
pub struct AlphaShapeResult {
    pub vertices: Vec<[f32; 3]>,
    pub edges: Vec<[usize; 2]>,
    pub triangles: Vec<[usize; 3]>,
    pub alpha: f32,
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v,v).sqrt()
}

/// Compute circumradius of a triangle.
#[allow(dead_code)]
pub fn circumradius(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let a = len3(sub3(p1, p0));
    let b = len3(sub3(p2, p1));
    let c = len3(sub3(p0, p2));
    let s = (a + b + c) / 2.0;
    let area2 = s * (s - a) * (s - b) * (s - c);
    if area2 <= 0.0 {
        return f32::INFINITY;
    }
    let area = area2.sqrt();
    (a * b * c) / (4.0 * area)
}

/// Check if a triangle is in the alpha shape.
#[allow(dead_code)]
pub fn triangle_in_alpha_shape(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], alpha: f32) -> bool {
    let r = circumradius(p0, p1, p2);
    r < 1.0 / alpha
}

/// Check if an edge's circumscribed circle is within alpha.
#[allow(dead_code)]
pub fn edge_in_alpha_shape(p0: [f32; 3], p1: [f32; 3], alpha: f32) -> bool {
    let half_len = len3(sub3(p1, p0)) * 0.5;
    half_len < 1.0 / alpha
}

/// Simple 2D alpha shape: given points projected to XY, find boundary edges.
#[allow(dead_code)]
pub fn alpha_shape_2d(points: &[[f32; 3]], alpha: f32) -> AlphaShapeResult {
    let n = points.len();
    let mut triangles = Vec::new();
    let mut edges = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                if triangle_in_alpha_shape(points[i], points[j], points[k], alpha) {
                    triangles.push([i, j, k]);
                }
            }
        }
    }

    for i in 0..n {
        for j in (i + 1)..n {
            if edge_in_alpha_shape(points[i], points[j], alpha) {
                edges.push([i, j]);
            }
        }
    }

    AlphaShapeResult {
        vertices: points.to_vec(),
        edges,
        triangles,
        alpha,
    }
}

/// Optimal alpha estimate (1/max edge length as heuristic).
#[allow(dead_code)]
pub fn estimate_alpha(points: &[[f32; 3]]) -> f32 {
    let mut max_dist = 0.0f32;
    let n = points.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let d = len3(sub3(points[i], points[j]));
            if d > max_dist { max_dist = d; }
        }
    }
    if max_dist < 1e-10 { 1.0 } else { 1.0 / max_dist }
}

/// Alpha shape vertex count.
#[allow(dead_code)]
pub fn alpha_shape_vertex_count(result: &AlphaShapeResult) -> usize {
    result.vertices.len()
}

/// Alpha shape triangle count.
#[allow(dead_code)]
pub fn alpha_shape_triangle_count(result: &AlphaShapeResult) -> usize {
    result.triangles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_points() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]]
    }

    #[test]
    fn circumradius_equilateral() {
        let r = circumradius([0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,0.866,0.0]);
        assert!(r > 0.5 && r < 1.0);
    }

    #[test]
    fn triangle_in_alpha_large_alpha() {
        // circumradius ~0.057; alpha=20 => 1/alpha=0.05; use alpha=10 => 1/alpha=0.1 > r
        let p0 = [0.0,0.0,0.0]; let p1 = [0.1,0.0,0.0]; let p2 = [0.05,0.1,0.0];
        assert!(triangle_in_alpha_shape(p0, p1, p2, 10.0));
    }

    #[test]
    fn triangle_not_in_alpha_small() {
        let p0 = [0.0,0.0,0.0]; let p1 = [10.0,0.0,0.0]; let p2 = [5.0,10.0,0.0];
        assert!(!triangle_in_alpha_shape(p0, p1, p2, 1.0));
    }

    #[test]
    fn edge_in_alpha_short() {
        // half_len=0.05; alpha=10 => 1/alpha=0.1 > 0.05
        assert!(edge_in_alpha_shape([0.0,0.0,0.0],[0.1,0.0,0.0], 10.0));
    }

    #[test]
    fn alpha_shape_2d_returns_correct_vertex_count() {
        let pts = square_points();
        let r = alpha_shape_2d(&pts, 2.0);
        assert_eq!(alpha_shape_vertex_count(&r), 4);
    }

    #[test]
    fn estimate_alpha_positive() {
        let pts = square_points();
        let a = estimate_alpha(&pts);
        assert!(a > 0.0);
    }

    #[test]
    fn alpha_shape_triangle_count_nonzero() {
        // circumradius of unit-square triangle ~0.707; alpha=1 => 1/alpha=1.0 > 0.707
        let pts = square_points();
        let r = alpha_shape_2d(&pts, 1.0);
        assert!(alpha_shape_triangle_count(&r) > 0);
    }

    #[test]
    fn circumradius_degenerate_infinite() {
        let r = circumradius([0.0,0.0,0.0],[1.0,0.0,0.0],[2.0,0.0,0.0]);
        assert!(r.is_infinite() || r > 1e6);
    }
}
