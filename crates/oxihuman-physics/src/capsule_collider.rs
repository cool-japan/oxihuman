// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A capsule collider primitive and point/sphere queries against it.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapsuleCollider {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub radius: f32,
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

#[allow(dead_code)]
impl CapsuleCollider {
    pub fn new(a: [f32; 3], b: [f32; 3], radius: f32) -> Self {
        Self { a, b, radius: radius.abs() }
    }

    pub fn height(&self) -> f32 {
        vec3_len(vec3_sub(self.b, self.a))
    }

    pub fn total_length(&self) -> f32 {
        self.height() + 2.0 * self.radius
    }

    pub fn volume(&self) -> f32 {
        let h = self.height();
        let r = self.radius;
        PI * r * r * h + (4.0 / 3.0) * PI * r * r * r
    }

    pub fn surface_area(&self) -> f32 {
        let h = self.height();
        let r = self.radius;
        2.0 * PI * r * h + 4.0 * PI * r * r
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.a[0] + self.b[0]) * 0.5,
            (self.a[1] + self.b[1]) * 0.5,
            (self.a[2] + self.b[2]) * 0.5,
        ]
    }

    pub fn closest_point_on_axis(&self, point: [f32; 3]) -> [f32; 3] {
        let ab = vec3_sub(self.b, self.a);
        let ap = vec3_sub(point, self.a);
        let ab_sq = vec3_dot(ab, ab);
        if ab_sq < 1e-12 {
            return self.a;
        }
        let t = (vec3_dot(ap, ab) / ab_sq).clamp(0.0, 1.0);
        [
            self.a[0] + ab[0] * t,
            self.a[1] + ab[1] * t,
            self.a[2] + ab[2] * t,
        ]
    }

    pub fn distance_to_point(&self, point: [f32; 3]) -> f32 {
        let closest = self.closest_point_on_axis(point);
        let d = vec3_len(vec3_sub(point, closest));
        (d - self.radius).max(0.0)
    }

    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        let closest = self.closest_point_on_axis(point);
        let d = vec3_len(vec3_sub(point, closest));
        d <= self.radius
    }

    pub fn intersects_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        let closest = self.closest_point_on_axis(center);
        let d = vec3_len(vec3_sub(center, closest));
        d <= self.radius + radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_height() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5);
        assert!((c.height() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_total_length() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5);
        assert!((c.total_length() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_volume() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        let sphere_vol = (4.0 / 3.0) * PI;
        assert!((c.volume() - sphere_vol).abs() < 1e-3);
    }

    #[test]
    fn test_center() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 4.0, 0.0], 1.0);
        let ctr = c.center();
        assert!((ctr[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_contains_point_inside() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 1.0);
        assert!(c.contains_point([0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_contains_point_outside() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5);
        assert!(!c.contains_point([5.0, 1.0, 0.0]));
    }

    #[test]
    fn test_distance_to_point() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 1.0);
        let d = c.distance_to_point([3.0, 1.0, 0.0]);
        assert!((d - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersects_sphere() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 1.0);
        assert!(c.intersects_sphere([2.0, 1.0, 0.0], 1.5));
        assert!(!c.intersects_sphere([10.0, 1.0, 0.0], 0.1));
    }

    #[test]
    fn test_surface_area() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        let sphere_sa = 4.0 * PI;
        assert!((c.surface_area() - sphere_sa).abs() < 1e-3);
    }

    #[test]
    fn test_closest_on_axis() {
        let c = CapsuleCollider::new([0.0, 0.0, 0.0], [0.0, 4.0, 0.0], 1.0);
        let p = c.closest_point_on_axis([0.0, 2.0, 5.0]);
        assert!((p[1] - 2.0).abs() < 1e-5);
    }
}
