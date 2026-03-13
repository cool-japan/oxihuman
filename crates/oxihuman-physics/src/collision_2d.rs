// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D AABB/circle collision stub.

#[derive(Debug, Clone, PartialEq)]
pub struct Aabb2d {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Aabb2d {
    pub fn new(min: [f32; 2], max: [f32; 2]) -> Self {
        Aabb2d { min, max }
    }

    pub fn from_center_half(cx: f32, cy: f32, hx: f32, hy: f32) -> Self {
        Aabb2d {
            min: [cx - hx, cy - hy],
            max: [cx + hx, cy + hy],
        }
    }

    pub fn center(&self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
        ]
    }

    pub fn half_extents(&self) -> [f32; 2] {
        [
            (self.max[0] - self.min[0]) / 2.0,
            (self.max[1] - self.min[1]) / 2.0,
        ]
    }

    pub fn area(&self) -> f32 {
        let w = (self.max[0] - self.min[0]).max(0.0);
        let h = (self.max[1] - self.min[1]).max(0.0);
        w * h
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle2d {
    pub center: [f32; 2],
    pub radius: f32,
}

impl Circle2d {
    pub fn new(cx: f32, cy: f32, radius: f32) -> Self {
        Circle2d {
            center: [cx, cy],
            radius,
        }
    }

    pub fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }
}

pub fn aabb_aabb_2d(a: &Aabb2d, b: &Aabb2d) -> bool {
    a.min[0] <= b.max[0] && a.max[0] >= b.min[0] && a.min[1] <= b.max[1] && a.max[1] >= b.min[1]
}

pub fn circle_circle_2d(a: &Circle2d, b: &Circle2d) -> bool {
    let dx = a.center[0] - b.center[0];
    let dy = a.center[1] - b.center[1];
    let dist_sq = dx * dx + dy * dy;
    let r_sum = a.radius + b.radius;
    dist_sq <= r_sum * r_sum
}

pub fn aabb_circle_2d(aabb: &Aabb2d, circle: &Circle2d) -> bool {
    let cx = circle.center[0].clamp(aabb.min[0], aabb.max[0]);
    let cy = circle.center[1].clamp(aabb.min[1], aabb.max[1]);
    let dx = cx - circle.center[0];
    let dy = cy - circle.center[1];
    dx * dx + dy * dy <= circle.radius * circle.radius
}

pub fn point_in_aabb(point: [f32; 2], aabb: &Aabb2d) -> bool {
    (aabb.min[0]..=aabb.max[0]).contains(&point[0])
        && (aabb.min[1]..=aabb.max[1]).contains(&point[1])
}

pub fn point_in_circle(point: [f32; 2], circle: &Circle2d) -> bool {
    let dx = point[0] - circle.center[0];
    let dy = point[1] - circle.center[1];
    dx * dx + dy * dy <= circle.radius * circle.radius
}

pub fn aabb_overlap_area(a: &Aabb2d, b: &Aabb2d) -> f32 {
    let ox = (a.max[0].min(b.max[0]) - a.min[0].max(b.min[0])).max(0.0);
    let oy = (a.max[1].min(b.max[1]) - a.min[1].max(b.min[1])).max(0.0);
    ox * oy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_aabb_overlap() {
        let a = Aabb2d::new([0.0, 0.0], [2.0, 2.0]);
        let b = Aabb2d::new([1.0, 1.0], [3.0, 3.0]);
        assert!(aabb_aabb_2d(&a, &b) /* overlapping AABBs */,);
    }

    #[test]
    fn test_aabb_aabb_no_overlap() {
        let a = Aabb2d::new([0.0, 0.0], [1.0, 1.0]);
        let b = Aabb2d::new([2.0, 2.0], [3.0, 3.0]);
        assert!(!aabb_aabb_2d(&a, &b) /* separate AABBs */,);
    }

    #[test]
    fn test_circle_circle_overlap() {
        let a = Circle2d::new(0.0, 0.0, 1.0);
        let b = Circle2d::new(1.0, 0.0, 1.0);
        assert!(circle_circle_2d(&a, &b) /* circles touch */,);
    }

    #[test]
    fn test_circle_circle_no_overlap() {
        let a = Circle2d::new(0.0, 0.0, 1.0);
        let b = Circle2d::new(3.0, 0.0, 1.0);
        assert!(!circle_circle_2d(&a, &b) /* circles far apart */,);
    }

    #[test]
    fn test_aabb_circle_overlap() {
        let aabb = Aabb2d::new([0.0, 0.0], [2.0, 2.0]);
        let c = Circle2d::new(1.0, 1.0, 0.5);
        assert!(aabb_circle_2d(&aabb, &c) /* circle inside aabb */,);
    }

    #[test]
    fn test_point_in_aabb() {
        let aabb = Aabb2d::new([0.0, 0.0], [10.0, 10.0]);
        assert!(point_in_aabb([5.0, 5.0], &aabb) /* center point */,);
        assert!(!point_in_aabb([11.0, 5.0], &aabb) /* outside */,);
    }

    #[test]
    fn test_point_in_circle() {
        let c = Circle2d::new(0.0, 0.0, 5.0);
        assert!(point_in_circle([3.0, 4.0], &c) /* 3-4-5 on boundary */,);
        assert!(!point_in_circle([3.1, 4.0], &c) /* slightly outside */,);
    }

    #[test]
    fn test_overlap_area() {
        let a = Aabb2d::new([0.0, 0.0], [2.0, 2.0]);
        let b = Aabb2d::new([1.0, 1.0], [3.0, 3.0]);
        assert!((aabb_overlap_area(&a, &b) - 1.0).abs() < 1e-5, /* 1x1 overlap */);
    }

    #[test]
    fn test_aabb_area() {
        let a = Aabb2d::new([0.0, 0.0], [3.0, 4.0]);
        assert!((a.area() - 12.0).abs() < 1e-5 /* 3*4=12 */,);
    }
}
