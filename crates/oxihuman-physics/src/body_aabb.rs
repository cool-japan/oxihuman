// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Axis-aligned bounding box for a physics body.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[allow(dead_code)]
impl BodyAabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            min: [min[0].min(max[0]), min[1].min(max[1]), min[2].min(max[2])],
            max: [min[0].max(max[0]), min[1].max(max[1]), min[2].max(max[2])],
        }
    }

    pub fn from_center_half(center: [f32; 3], half: [f32; 3]) -> Self {
        Self {
            min: [
                center[0] - half[0],
                center[1] - half[1],
                center[2] - half[2],
            ],
            max: [
                center[0] + half[0],
                center[1] + half[1],
                center[2] + half[2],
            ],
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

    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    pub fn volume(&self) -> f32 {
        let s = self.size();
        s[0] * s[1] * s[2]
    }

    pub fn surface_area(&self) -> f32 {
        let s = self.size();
        2.0 * (s[0] * s[1] + s[1] * s[2] + s[2] * s[0])
    }

    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }

    pub fn intersects(&self, other: &BodyAabb) -> bool {
        (0..3).all(|i| self.min[i] <= other.max[i] && self.max[i] >= other.min[i])
    }

    pub fn union(&self, other: &BodyAabb) -> BodyAabb {
        BodyAabb {
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

    pub fn expand(&self, margin: f32) -> BodyAabb {
        BodyAabb {
            min: [
                self.min[0] - margin,
                self.min[1] - margin,
                self.min[2] - margin,
            ],
            max: [
                self.max[0] + margin,
                self.max[1] + margin,
                self.max[2] + margin,
            ],
        }
    }

    pub fn translate(&self, offset: [f32; 3]) -> BodyAabb {
        BodyAabb {
            min: [
                self.min[0] + offset[0],
                self.min[1] + offset[1],
                self.min[2] + offset[2],
            ],
            max: [
                self.max[0] + offset[0],
                self.max[1] + offset[1],
                self.max[2] + offset[2],
            ],
        }
    }

    pub fn longest_axis(&self) -> usize {
        let s = self.size();
        if s[0] >= s[1] && s[0] >= s[2] {
            0
        } else if s[1] >= s[2] {
            1
        } else {
            2
        }
    }

    pub fn from_points(points: &[[f32; 3]]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut min = points[0];
        let mut max = points[0];
        for p in &points[1..] {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        Some(Self { min, max })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let aabb = BodyAabb::new([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]);
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_swapped_min_max() {
        let aabb = BodyAabb::new([1.0, 2.0, 3.0], [0.0, 0.0, 0.0]);
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_center() {
        let aabb = BodyAabb::new([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]);
        let c = aabb.center();
        assert!((c[0] - 1.0).abs() < f32::EPSILON);
        assert!((c[1] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_volume() {
        let aabb = BodyAabb::new([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        assert!((aabb.volume() - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_contains_point() {
        let aabb = BodyAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(aabb.contains_point([0.5, 0.5, 0.5]));
        assert!(!aabb.contains_point([2.0, 0.5, 0.5]));
    }

    #[test]
    fn test_intersects() {
        let a = BodyAabb::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = BodyAabb::new([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_no_intersection() {
        let a = BodyAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = BodyAabb::new([2.0, 2.0, 2.0], [3.0, 3.0, 3.0]);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_union() {
        let a = BodyAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = BodyAabb::new([2.0, 2.0, 2.0], [3.0, 3.0, 3.0]);
        let u = a.union(&b);
        assert_eq!(u.min, [0.0, 0.0, 0.0]);
        assert_eq!(u.max, [3.0, 3.0, 3.0]);
    }

    #[test]
    fn test_expand() {
        let aabb = BodyAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let e = aabb.expand(0.5);
        assert!((e.min[0] + 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_from_points() {
        let pts = [[1.0, 2.0, 3.0], [-1.0, 0.0, 5.0]];
        let aabb = BodyAabb::from_points(&pts).expect("should succeed");
        assert_eq!(aabb.min, [-1.0, 0.0, 3.0]);
        assert_eq!(aabb.max, [1.0, 2.0, 5.0]);
    }
}
