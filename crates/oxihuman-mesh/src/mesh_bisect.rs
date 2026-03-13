// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bisect mesh by plane.

/// A bisecting plane defined by normal and origin.
#[derive(Debug, Clone)]
pub struct BisectPlane {
    pub normal: [f32; 3],
    pub origin: [f32; 3],
}

/// Build a `BisectPlane`.
pub fn new_bisect_plane(normal: [f32; 3], origin: [f32; 3]) -> BisectPlane {
    BisectPlane { normal, origin }
}

/// Classify a vertex: positive means above, negative means below.
pub fn bisect_classify_vertex(plane: &BisectPlane, vertex: [f32; 3]) -> f32 {
    let d = vertex[0] - plane.origin[0];
    let e = vertex[1] - plane.origin[1];
    let f = vertex[2] - plane.origin[2];
    d * plane.normal[0] + e * plane.normal[1] + f * plane.normal[2]
}

/// Find the parametric intersection of an edge with the plane (0..1).
/// Returns None if the edge does not cross the plane.
pub fn bisect_edge_intersection(plane: &BisectPlane, a: [f32; 3], b: [f32; 3]) -> Option<f32> {
    let da = bisect_classify_vertex(plane, a);
    let db = bisect_classify_vertex(plane, b);
    if (da > 0.0) == (db > 0.0) {
        return None;
    }
    let denom = da - db;
    if denom.abs() < f32::EPSILON {
        return None;
    }
    Some(da / denom)
}

/// Count vertices above (or on) the plane.
pub fn bisect_count_above(plane: &BisectPlane, positions: &[[f32; 3]]) -> usize {
    positions
        .iter()
        .filter(|&&p| bisect_classify_vertex(plane, p) >= 0.0)
        .count()
}

/// Count vertices below the plane.
pub fn bisect_count_below(plane: &BisectPlane, positions: &[[f32; 3]]) -> usize {
    positions
        .iter()
        .filter(|&&p| bisect_classify_vertex(plane, p) < 0.0)
        .count()
}

/// Interpolated point on an edge at parameter t.
pub fn bisect_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xz_plane() -> BisectPlane {
        /* Y=0 plane, normal pointing up */
        new_bisect_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0])
    }

    #[test]
    fn test_classify_above() {
        let p = xz_plane();
        assert!(bisect_classify_vertex(&p, [0.0, 1.0, 0.0]) > 0.0);
    }

    #[test]
    fn test_classify_below() {
        let p = xz_plane();
        assert!(bisect_classify_vertex(&p, [0.0, -1.0, 0.0]) < 0.0);
    }

    #[test]
    fn test_edge_intersection_crossing() {
        let p = xz_plane();
        let t = bisect_edge_intersection(&p, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(t.is_some());
        let t = t.expect("should succeed");
        assert!((t - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_edge_no_intersection_same_side() {
        let p = xz_plane();
        let t = bisect_edge_intersection(&p, [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]);
        assert!(t.is_none());
    }

    #[test]
    fn test_count_above() {
        let p = xz_plane();
        let pts = vec![[0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 2.0, 0.0]];
        assert_eq!(bisect_count_above(&p, &pts), 2);
    }

    #[test]
    fn test_count_below() {
        let p = xz_plane();
        let pts = vec![[0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -2.0, 0.0]];
        assert_eq!(bisect_count_below(&p, &pts), 2);
    }
}
