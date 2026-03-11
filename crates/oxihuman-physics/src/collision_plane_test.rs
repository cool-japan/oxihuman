// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Plane collision tests: plane-point, plane-sphere, plane-AABB, plane-ray.

/// A plane defined by normal and distance from origin.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestPlane {
    pub normal: [f32; 3],
    pub d: f32,
}

#[allow(dead_code)]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[allow(dead_code)]
impl TestPlane {
    pub fn new(normal: [f32; 3], d: f32) -> Self {
        let len = vec3_len(normal);
        if len < 1e-10 {
            return Self { normal: [0.0, 1.0, 0.0], d };
        }
        Self {
            normal: [normal[0] / len, normal[1] / len, normal[2] / len],
            d: d / len,
        }
    }

    pub fn from_point_normal(point: [f32; 3], normal: [f32; 3]) -> Self {
        let len = vec3_len(normal);
        let n = if len < 1e-10 {
            [0.0, 1.0, 0.0]
        } else {
            [normal[0] / len, normal[1] / len, normal[2] / len]
        };
        let d = -dot3(n, point);
        Self { normal: n, d }
    }

    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        dot3(self.normal, point) + self.d
    }

    pub fn distance(&self, point: [f32; 3]) -> f32 {
        self.signed_distance(point).abs()
    }

    pub fn is_front(&self, point: [f32; 3]) -> bool {
        self.signed_distance(point) > 0.0
    }

    pub fn project_point(&self, point: [f32; 3]) -> [f32; 3] {
        let sd = self.signed_distance(point);
        [
            point[0] - self.normal[0] * sd,
            point[1] - self.normal[1] * sd,
            point[2] - self.normal[2] * sd,
        ]
    }

    /// Sphere-plane test. Returns penetration depth (>0 if intersecting).
    pub fn sphere_test(&self, center: [f32; 3], radius: f32) -> f32 {
        let sd = self.signed_distance(center);
        radius - sd.abs()
    }

    pub fn sphere_intersects(&self, center: [f32; 3], radius: f32) -> bool {
        self.sphere_test(center, radius) > 0.0
    }

    /// Ray-plane intersection: returns t parameter or None.
    pub fn ray_test(&self, origin: [f32; 3], dir: [f32; 3]) -> Option<f32> {
        let denom = dot3(self.normal, dir);
        if denom.abs() < 1e-10 { return None; }
        let t = -(dot3(self.normal, origin) + self.d) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }

    /// AABB-plane test: returns true if AABB intersects the plane.
    pub fn aabb_test(&self, min: [f32; 3], max: [f32; 3]) -> bool {
        // Project AABB extents onto plane normal
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let half = [
            (max[0] - min[0]) * 0.5,
            (max[1] - min[1]) * 0.5,
            (max[2] - min[2]) * 0.5,
        ];
        let r = half[0] * self.normal[0].abs()
              + half[1] * self.normal[1].abs()
              + half[2] * self.normal[2].abs();
        let sd = self.signed_distance(center);
        sd.abs() <= r
    }

    pub fn flip(&self) -> TestPlane {
        TestPlane {
            normal: [-self.normal[0], -self.normal[1], -self.normal[2]],
            d: -self.d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_distance() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        assert!((plane.signed_distance([0.0, 5.0, 0.0]) - 5.0).abs() < 0.01);
        assert!((plane.signed_distance([0.0, -3.0, 0.0]) - (-3.0)).abs() < 0.01);
    }

    #[test]
    fn test_is_front() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        assert!(plane.is_front([0.0, 1.0, 0.0]));
        assert!(!plane.is_front([0.0, -1.0, 0.0]));
    }

    #[test]
    fn test_project_point() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        let p = plane.project_point([3.0, 5.0, 7.0]);
        assert!((p[1]).abs() < 0.01);
        assert!((p[0] - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_sphere_intersect() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        assert!(plane.sphere_intersects([0.0, 0.5, 0.0], 1.0));
        assert!(!plane.sphere_intersects([0.0, 5.0, 0.0], 1.0));
    }

    #[test]
    fn test_ray() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        let t = plane.ray_test([0.0, 5.0, 0.0], [0.0, -1.0, 0.0]);
        assert!((t.unwrap() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_ray_parallel() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        assert!(plane.ray_test([0.0, 5.0, 0.0], [1.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_aabb_intersect() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        assert!(plane.aabb_test([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
        assert!(!plane.aabb_test([0.0, 5.0, 0.0], [1.0, 6.0, 1.0]));
    }

    #[test]
    fn test_from_point_normal() {
        let plane = TestPlane::from_point_normal([0.0, 2.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((plane.signed_distance([0.0, 2.0, 0.0])).abs() < 0.01);
    }

    #[test]
    fn test_flip() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], -5.0);
        let f = plane.flip();
        assert!((f.normal[1] - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_distance() {
        let plane = TestPlane::new([0.0, 1.0, 0.0], 0.0);
        assert!((plane.distance([0.0, -3.0, 0.0]) - 3.0).abs() < 0.01);
    }
}
