// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Axis-aligned bounding box collision tests: AABB-AABB, AABB-point, AABB-ray, AABB-sphere.

/// An axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Result of a box collision test.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BoxTestResult {
    pub hit: bool,
    pub overlap: f32,
    pub normal: [f32; 3],
}

#[allow(dead_code)]
impl TestAabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub fn from_center_half(center: [f32; 3], half: [f32; 3]) -> Self {
        Self {
            min: [center[0] - half[0], center[1] - half[1], center[2] - half[2]],
            max: [center[0] + half[0], center[1] + half[1], center[2] + half[2]],
        }
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn half_extents(&self) -> [f32; 3] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
            (self.max[2] - self.min[2]) * 0.5,
        ]
    }

    pub fn volume(&self) -> f32 {
        let d = self.half_extents();
        8.0 * d[0] * d[1] * d[2]
    }

    pub fn surface_area(&self) -> f32 {
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        p[0] >= self.min[0] && p[0] <= self.max[0]
            && p[1] >= self.min[1] && p[1] <= self.max[1]
            && p[2] >= self.min[2] && p[2] <= self.max[2]
    }

    pub fn intersects(&self, other: &TestAabb) -> bool {
        self.min[0] <= other.max[0] && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1] && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2] && self.max[2] >= other.min[2]
    }

    pub fn overlap_depth(&self, other: &TestAabb) -> f32 {
        if !self.intersects(other) { return 0.0; }
        let mut min_overlap = f32::MAX;
        for i in 0..3 {
            let o1 = self.max[i] - other.min[i];
            let o2 = other.max[i] - self.min[i];
            let o = o1.min(o2);
            if o < min_overlap { min_overlap = o; }
        }
        min_overlap
    }

    pub fn merge(&self, other: &TestAabb) -> TestAabb {
        TestAabb {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn expand(&self, margin: f32) -> TestAabb {
        TestAabb {
            min: [self.min[0] - margin, self.min[1] - margin, self.min[2] - margin],
            max: [self.max[0] + margin, self.max[1] + margin, self.max[2] + margin],
        }
    }

    /// Ray-AABB intersection returning t parameter or None.
    pub fn ray_test(&self, origin: [f32; 3], dir: [f32; 3]) -> Option<f32> {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;
        for i in 0..3 {
            if dir[i].abs() < 1e-10 {
                if origin[i] < self.min[i] || origin[i] > self.max[i] {
                    return None;
                }
            } else {
                let inv = 1.0 / dir[i];
                let mut t1 = (self.min[i] - origin[i]) * inv;
                let mut t2 = (self.max[i] - origin[i]) * inv;
                if t1 > t2 { std::mem::swap(&mut t1, &mut t2); }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax { return None; }
            }
        }
        if tmax < 0.0 { None } else { Some(tmin.max(0.0)) }
    }

    /// Sphere-AABB test.
    #[allow(clippy::needless_range_loop)]
    pub fn intersects_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        let mut sq_dist = 0.0f32;
        for i in 0..3 {
            if center[i] < self.min[i] {
                let d = self.min[i] - center[i];
                sq_dist += d * d;
            } else if center[i] > self.max[i] {
                let d = center[i] - self.max[i];
                sq_dist += d * d;
            }
        }
        sq_dist <= radius * radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersects() {
        let a = TestAabb::new([0.0; 3], [2.0; 3]);
        let b = TestAabb::new([1.0; 3], [3.0; 3]);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_no_intersect() {
        let a = TestAabb::new([0.0; 3], [1.0; 3]);
        let b = TestAabb::new([5.0; 3], [6.0; 3]);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_contains_point() {
        let a = TestAabb::new([0.0; 3], [2.0; 3]);
        assert!(a.contains_point([1.0, 1.0, 1.0]));
        assert!(!a.contains_point([3.0, 0.0, 0.0]));
    }

    #[test]
    fn test_overlap_depth() {
        let a = TestAabb::new([0.0; 3], [2.0; 3]);
        let b = TestAabb::new([1.5, 0.0, 0.0], [3.0, 2.0, 2.0]);
        let d = a.overlap_depth(&b);
        assert!((d - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_volume() {
        let a = TestAabb::new([0.0; 3], [2.0, 3.0, 4.0]);
        assert!((a.volume() - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_area() {
        let a = TestAabb::new([0.0; 3], [1.0, 1.0, 1.0]);
        assert!((a.surface_area() - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ray_hit() {
        let a = TestAabb::new([1.0; 3], [3.0; 3]);
        let t = a.ray_test([0.0, 2.0, 2.0], [1.0, 0.0, 0.0]);
        assert!(t.is_some());
        assert!((t.unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_ray_miss() {
        let a = TestAabb::new([1.0; 3], [3.0; 3]);
        assert!(a.ray_test([0.0, 10.0, 0.0], [1.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_sphere_intersect() {
        let a = TestAabb::new([0.0; 3], [2.0; 3]);
        assert!(a.intersects_sphere([3.0, 1.0, 1.0], 1.5));
        assert!(!a.intersects_sphere([10.0, 10.0, 10.0], 0.5));
    }

    #[test]
    fn test_merge() {
        let a = TestAabb::new([0.0; 3], [1.0; 3]);
        let b = TestAabb::new([2.0; 3], [3.0; 3]);
        let m = a.merge(&b);
        assert_eq!(m.min, [0.0; 3]);
        assert_eq!(m.max, [3.0; 3]);
    }
}
