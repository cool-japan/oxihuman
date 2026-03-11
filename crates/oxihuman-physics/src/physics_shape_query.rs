// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shape query utilities: overlap tests, closest point, distance queries.

/// Supported query shapes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum QueryShape {
    Sphere { center: [f32; 3], radius: f32 },
    Box { min: [f32; 3], max: [f32; 3] },
    Point([f32; 3]),
}

/// Result of a shape query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub hit: bool,
    pub distance: f32,
    pub closest_point: [f32; 3],
    pub normal: [f32; 3],
}

#[allow(dead_code)]
impl QueryResult {
    pub fn miss() -> Self {
        Self { hit: false, distance: f32::MAX, closest_point: [0.0;3], normal: [0.0;3] }
    }

    pub fn hit_result(dist: f32, point: [f32; 3], normal: [f32; 3]) -> Self {
        Self { hit: true, distance: dist, closest_point: point, normal }
    }
}

#[allow(dead_code)]
fn v3_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0]-a[0]; let dy = b[1]-a[1]; let dz = b[2]-a[2];
    (dx*dx + dy*dy + dz*dz).sqrt()
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[allow(dead_code)]
fn v3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();
    if l < 1e-10 { [0.0,1.0,0.0] } else { [v[0]/l, v[1]/l, v[2]/l] }
}

/// Point to sphere distance query.
#[allow(dead_code)]
pub fn point_sphere_query(point: [f32; 3], center: [f32; 3], radius: f32) -> QueryResult {
    let d = v3_dist(point, center);
    if d <= radius {
        let n = v3_normalize(v3_sub(point, center));
        QueryResult::hit_result(d - radius, point, n)
    } else {
        QueryResult::miss()
    }
}

/// Point to AABB closest point.
#[allow(dead_code)]
pub fn point_aabb_closest(point: [f32; 3], min: [f32; 3], max: [f32; 3]) -> [f32; 3] {
    [
        point[0].clamp(min[0], max[0]),
        point[1].clamp(min[1], max[1]),
        point[2].clamp(min[2], max[2]),
    ]
}

/// Point to AABB distance query.
#[allow(dead_code)]
pub fn point_aabb_query(point: [f32; 3], min: [f32; 3], max: [f32; 3]) -> QueryResult {
    let closest = point_aabb_closest(point, min, max);
    let d = v3_dist(point, closest);
    let inside = point[0] >= min[0] && point[0] <= max[0]
        && point[1] >= min[1] && point[1] <= max[1]
        && point[2] >= min[2] && point[2] <= max[2];
    if inside || d < 1e-6 {
        QueryResult::hit_result(0.0, closest, [0.0, 1.0, 0.0])
    } else {
        QueryResult { hit: false, distance: d, closest_point: closest, normal: v3_normalize(v3_sub(point, closest)) }
    }
}

/// Sphere to sphere overlap test.
#[allow(dead_code)]
pub fn sphere_sphere_overlap(c1: [f32; 3], r1: f32, c2: [f32; 3], r2: f32) -> QueryResult {
    let d = v3_dist(c1, c2);
    let pen = r1 + r2 - d;
    if pen > 0.0 {
        let n = v3_normalize(v3_sub(c2, c1));
        let cp = [c1[0] + n[0] * r1, c1[1] + n[1] * r1, c1[2] + n[2] * r1];
        QueryResult::hit_result(pen, cp, n)
    } else {
        QueryResult::miss()
    }
}

/// Sphere to AABB overlap test.
#[allow(dead_code)]
pub fn sphere_aabb_overlap(center: [f32; 3], radius: f32, min: [f32; 3], max: [f32; 3]) -> QueryResult {
    let closest = point_aabb_closest(center, min, max);
    let d = v3_dist(center, closest);
    if d <= radius {
        let n = v3_normalize(v3_sub(center, closest));
        QueryResult::hit_result(radius - d, closest, n)
    } else {
        QueryResult::miss()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_sphere() {
        let r = point_sphere_query([0.0;3], [0.0;3], 1.0);
        assert!(r.hit);
    }

    #[test]
    fn test_point_outside_sphere() {
        let r = point_sphere_query([5.0, 0.0, 0.0], [0.0;3], 1.0);
        assert!(!r.hit);
    }

    #[test]
    fn test_point_aabb_inside() {
        let r = point_aabb_query([0.5;3], [0.0;3], [1.0;3]);
        assert!(r.hit);
    }

    #[test]
    fn test_point_aabb_outside() {
        let r = point_aabb_query([5.0;3], [0.0;3], [1.0;3]);
        assert!(!r.hit);
        assert!(r.distance > 0.0);
    }

    #[test]
    fn test_sphere_sphere_overlap() {
        let r = sphere_sphere_overlap([0.0;3], 1.0, [1.5, 0.0, 0.0], 1.0);
        assert!(r.hit);
        assert!(r.distance > 0.0);
    }

    #[test]
    fn test_sphere_sphere_no_overlap() {
        let r = sphere_sphere_overlap([0.0;3], 1.0, [5.0, 0.0, 0.0], 1.0);
        assert!(!r.hit);
    }

    #[test]
    fn test_sphere_aabb_overlap() {
        let r = sphere_aabb_overlap([1.5, 0.5, 0.5], 1.0, [0.0;3], [1.0;3]);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_aabb_no_overlap() {
        let r = sphere_aabb_overlap([10.0;3], 0.5, [0.0;3], [1.0;3]);
        assert!(!r.hit);
    }

    #[test]
    fn test_closest_point_clamped() {
        let c = point_aabb_closest([5.0, 0.5, 0.5], [0.0;3], [1.0;3]);
        assert!((c[0] - 1.0).abs() < 0.01);
        assert!((c[1] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_miss_result() {
        let m = QueryResult::miss();
        assert!(!m.hit);
        assert_eq!(m.distance, f32::MAX);
    }
}
