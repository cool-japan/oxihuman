// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A capsule collision shape defined by two endpoints and a radius.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CapsuleShape {
    start: [f32; 3],
    end: [f32; 3],
    radius: f32,
}

#[allow(dead_code)]
impl CapsuleShape {
    pub fn new(start: [f32; 3], end: [f32; 3], radius: f32) -> Self {
        Self { start, end, radius }
    }

    pub fn from_height(center: [f32; 3], height: f32, radius: f32) -> Self {
        let half = height * 0.5;
        Self {
            start: [center[0], center[1] - half, center[2]],
            end: [center[0], center[1] + half, center[2]],
            radius,
        }
    }

    pub fn start(&self) -> [f32; 3] {
        self.start
    }

    pub fn end(&self) -> [f32; 3] {
        self.end
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn height(&self) -> f32 {
        let dx = self.end[0] - self.start[0];
        let dy = self.end[1] - self.start[1];
        let dz = self.end[2] - self.start[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.start[0] + self.end[0]) * 0.5,
            (self.start[1] + self.end[1]) * 0.5,
            (self.start[2] + self.end[2]) * 0.5,
        ]
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

    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        self.distance_to_point(point) <= 0.0
    }

    pub fn distance_to_point(&self, point: [f32; 3]) -> f32 {
        let seg = [
            self.end[0] - self.start[0],
            self.end[1] - self.start[1],
            self.end[2] - self.start[2],
        ];
        let to_p = [
            point[0] - self.start[0],
            point[1] - self.start[1],
            point[2] - self.start[2],
        ];
        let seg_len_sq = seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2];
        let t = if seg_len_sq < 1e-12 {
            0.0
        } else {
            let dot = to_p[0] * seg[0] + to_p[1] * seg[1] + to_p[2] * seg[2];
            (dot / seg_len_sq).clamp(0.0, 1.0)
        };
        let closest = [
            self.start[0] + seg[0] * t,
            self.start[1] + seg[1] * t,
            self.start[2] + seg[2] * t,
        ];
        let dx = point[0] - closest[0];
        let dy = point[1] - closest[1];
        let dz = point[2] - closest[2];
        (dx * dx + dy * dy + dz * dz).sqrt() - self.radius
    }

    pub fn aabb_min(&self) -> [f32; 3] {
        [
            self.start[0].min(self.end[0]) - self.radius,
            self.start[1].min(self.end[1]) - self.radius,
            self.start[2].min(self.end[2]) - self.radius,
        ]
    }

    pub fn aabb_max(&self) -> [f32; 3] {
        [
            self.start[0].max(self.end[0]) + self.radius,
            self.start[1].max(self.end[1]) + self.radius,
            self.start[2].max(self.end[2]) + self.radius,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 1.0, 0.0], 0.5);
        assert!((c.radius() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_height() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 2.0, 0.0], 0.5);
        assert!((c.height() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_center() {
        let c = CapsuleShape::new([0.0, 0.0, 0.0], [0.0, 4.0, 0.0], 1.0);
        let ctr = c.center();
        assert!((ctr[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_height() {
        let c = CapsuleShape::from_height([0.0, 0.0, 0.0], 2.0, 0.5);
        assert!((c.start()[1] - (-1.0)).abs() < 1e-6);
        assert!((c.end()[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_volume() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 1.0, 0.0], 1.0);
        let v = c.volume();
        assert!(v > 0.0);
    }

    #[test]
    fn test_surface_area() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 1.0, 0.0], 1.0);
        let sa = c.surface_area();
        assert!(sa > 0.0);
    }

    #[test]
    fn test_contains_point_inside() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 2.0, 0.0], 1.0);
        assert!(c.contains_point([0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_contains_point_outside() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 2.0, 0.0], 0.5);
        assert!(!c.contains_point([10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_aabb() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 2.0, 0.0], 0.5);
        let mn = c.aabb_min();
        let mx = c.aabb_max();
        assert!((mn[0] - (-0.5)).abs() < 1e-6);
        assert!((mx[1] - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_distance_to_point() {
        let c = CapsuleShape::new([0.0; 3], [0.0, 1.0, 0.0], 0.5);
        let d = c.distance_to_point([1.0, 0.5, 0.0]);
        assert!((d - 0.5).abs() < 1e-5);
    }
}
