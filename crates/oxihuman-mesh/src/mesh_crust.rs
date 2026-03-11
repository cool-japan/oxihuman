// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Crust algorithm for surface reconstruction from point clouds.

/// Result of the crust algorithm.
#[allow(dead_code)]
pub struct CrustResult {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub poles: Vec<[f32; 3]>,
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

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0]*s, v[1]*s, v[2]*s]
}

/// Compute the circumcenter of a triangle (2D approximation, Z=0).
#[allow(dead_code)]
pub fn circumcenter_2d(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let ax = p0[0]; let ay = p0[1];
    let bx = p1[0]; let by = p1[1];
    let cx = p2[0]; let cy = p2[1];
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-10 {
        return scale3(add3(p0, add3(p1, p2)), 1.0 / 3.0);
    }
    let ux = ((ax*ax+ay*ay)*(by-cy)+(bx*bx+by*by)*(cy-ay)+(cx*cx+cy*cy)*(ay-by))/d;
    let uy = ((ax*ax+ay*ay)*(cx-bx)+(bx*bx+by*by)*(ax-cx)+(cx*cx+cy*cy)*(bx-ax))/d;
    [ux, uy, 0.0]
}

/// Compute Voronoi poles from a simple 1D slice (stub).
#[allow(dead_code)]
pub fn compute_poles_stub(points: &[[f32; 3]]) -> Vec<[f32; 3]> {
    points.iter().map(|&p| [p[0], p[1], p[2] + 0.5]).collect()
}

/// Crust reconstruction stub: returns a simple triangulated version of input.
#[allow(dead_code)]
pub fn crust_reconstruct_stub(points: &[[f32; 3]]) -> CrustResult {
    let poles = compute_poles_stub(points);
    let mut all_pts = points.to_vec();
    all_pts.extend_from_slice(&poles);

    let mut triangles = Vec::new();
    let n = points.len();
    if n >= 3 {
        for i in 0..(n - 2) {
            triangles.push([i, i + 1, i + 2]);
        }
    }

    CrustResult {
        positions: all_pts,
        triangles,
        poles,
    }
}

/// Find the farthest point from a query.
#[allow(dead_code)]
pub fn farthest_point(points: &[[f32; 3]], query: [f32; 3]) -> Option<usize> {
    points.iter().enumerate().max_by(|(_, a), (_, b)| {
        let da = len3(sub3(**a, query));
        let db = len3(sub3(**b, query));
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    }).map(|(i, _)| i)
}

/// Compute the local feature size estimate (min distance to medial axis proxy).
#[allow(dead_code)]
pub fn local_feature_size(p: [f32; 3], poles: &[[f32; 3]]) -> f32 {
    poles.iter().map(|&pole| len3(sub3(p, pole))).fold(f32::INFINITY, f32::min)
}

/// Check if three points form a crust triangle (simplified: based on circumradius).
#[allow(dead_code)]
pub fn is_crust_triangle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], alpha: f32) -> bool {
    let a = len3(sub3(p1, p0));
    let b = len3(sub3(p2, p1));
    let c = len3(sub3(p0, p2));
    let s = (a + b + c) / 2.0;
    let area2 = s*(s-a)*(s-b)*(s-c);
    if area2 <= 0.0 { return false; }
    let r = a * b * c / (4.0 * area2.sqrt());
    r < 1.0 / alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,0.5]]
    }

    #[test]
    fn circumcenter_equilateral() {
        let c = circumcenter_2d([0.0,0.0,0.0],[2.0,0.0,0.0],[1.0,1.732,0.0]);
        assert!((c[0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn poles_count_matches_points() {
        let pts = sample();
        let poles = compute_poles_stub(&pts);
        assert_eq!(poles.len(), pts.len());
    }

    #[test]
    fn poles_above_points() {
        let pts = sample();
        let poles = compute_poles_stub(&pts);
        for (p, pole) in pts.iter().zip(poles.iter()) {
            assert!(pole[2] > p[2]);
        }
    }

    #[test]
    fn crust_stub_returns_all_points_plus_poles() {
        let pts = sample();
        let r = crust_reconstruct_stub(&pts);
        assert_eq!(r.positions.len(), 2 * pts.len());
    }

    #[test]
    fn farthest_point_found() {
        let pts = vec![[0.0,0.0,0.0],[5.0,0.0,0.0],[1.0,0.0,0.0]];
        let f = farthest_point(&pts, [0.0,0.0,0.0]);
        assert_eq!(f, Some(1));
    }

    #[test]
    fn local_feature_size_positive() {
        let pts = sample();
        let poles = compute_poles_stub(&pts);
        let lfs = local_feature_size(pts[0], &poles);
        assert!(lfs > 0.0);
    }

    #[test]
    fn crust_triangle_check() {
        // circumradius ~0.0198; alpha=10 => 1/alpha=0.1 > 0.0198
        let p0 = [0.0,0.0,0.0]; let p1 = [0.1,0.0,0.0]; let p2 = [0.05,0.1,0.0];
        assert!(is_crust_triangle(p0, p1, p2, 10.0));
    }

    #[test]
    fn triangles_nonempty_for_enough_points() {
        let pts = sample();
        let r = crust_reconstruct_stub(&pts);
        assert!(!r.triangles.is_empty());
    }
}
