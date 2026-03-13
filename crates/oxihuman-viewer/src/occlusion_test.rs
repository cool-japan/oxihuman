// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Occlusion query helpers: hardware occlusion tests and software AABB visibility.

/// Result of an occlusion query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OcclusionResult {
    /// Object is definitely visible.
    Visible,
    /// Object is definitely occluded.
    Occluded,
    /// Query result not yet available.
    Pending,
}

/// A hardware occlusion query (stubbed — no GPU).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OcclusionQuery {
    pub id: u32,
    pub result: OcclusionResult,
    pub sample_count: u64,
}

impl OcclusionQuery {
    #[allow(dead_code)]
    pub fn new(id: u32) -> Self {
        Self {
            id,
            result: OcclusionResult::Pending,
            sample_count: 0,
        }
    }
}

/// A conservative AABB for software occlusion testing.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct OcclusionAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl OcclusionAabb {
    #[allow(dead_code)]
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Inflate the AABB by `amount` on each side.
    #[allow(dead_code)]
    pub fn inflate(&self, amount: f32) -> Self {
        Self {
            min: [
                self.min[0] - amount,
                self.min[1] - amount,
                self.min[2] - amount,
            ],
            max: [
                self.max[0] + amount,
                self.max[1] + amount,
                self.max[2] + amount,
            ],
        }
    }

    /// Return true if `point` is inside this AABB.
    #[allow(dead_code)]
    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        p[0] >= self.min[0]
            && p[0] <= self.max[0]
            && p[1] >= self.min[1]
            && p[1] <= self.max[1]
            && p[2] >= self.min[2]
            && p[2] <= self.max[2]
    }

    /// Return true if two AABBs intersect.
    #[allow(dead_code)]
    pub fn intersects(&self, other: &OcclusionAabb) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }

    /// Compute the surface area.
    #[allow(dead_code)]
    pub fn surface_area(&self) -> f32 {
        let dx = (self.max[0] - self.min[0]).max(0.0);
        let dy = (self.max[1] - self.min[1]).max(0.0);
        let dz = (self.max[2] - self.min[2]).max(0.0);
        2.0 * (dx * dy + dy * dz + dz * dx)
    }
}

/// A simple software depth-buffer–based occlusion tester (very conservative stub).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SoftOcclusionBuffer {
    pub width: u32,
    pub height: u32,
    depth: Vec<f32>,
}

impl SoftOcclusionBuffer {
    #[allow(dead_code)]
    pub fn new(width: u32, height: u32) -> Self {
        let count = (width * height) as usize;
        Self {
            width,
            height,
            depth: vec![1.0; count],
        }
    }

    /// Clear depth to far plane (1.0).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for d in self.depth.iter_mut() {
            *d = 1.0;
        }
    }

    /// Write depth at pixel `(x, y)`.
    #[allow(dead_code)]
    pub fn write(&mut self, x: u32, y: u32, depth: f32) {
        let idx = (y * self.width + x) as usize;
        if idx < self.depth.len() {
            let d = &mut self.depth[idx];
            if depth < *d {
                *d = depth;
            }
        }
    }

    /// Test if `depth` is in front of stored depth at `(x, y)`.
    #[allow(dead_code)]
    pub fn test(&self, x: u32, y: u32, depth: f32) -> bool {
        let idx = (y * self.width + x) as usize;
        if idx < self.depth.len() {
            depth < self.depth[idx]
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_starts_pending() {
        let q = OcclusionQuery::new(1);
        assert_eq!(q.result, OcclusionResult::Pending);
    }

    #[test]
    fn aabb_contains_center() {
        let aabb = OcclusionAabb::new([-1.0; 3], [1.0; 3]);
        assert!(aabb.contains_point([0.0, 0.0, 0.0]));
    }

    #[test]
    fn aabb_excludes_outside() {
        let aabb = OcclusionAabb::new([-1.0; 3], [1.0; 3]);
        assert!(!aabb.contains_point([2.0, 0.0, 0.0]));
    }

    #[test]
    fn aabb_inflate_grows() {
        let aabb = OcclusionAabb::new([0.0; 3], [1.0; 3]);
        let inflated = aabb.inflate(1.0);
        assert!((inflated.min[0] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn aabb_intersects_overlapping() {
        let a = OcclusionAabb::new([0.0; 3], [2.0; 3]);
        let b = OcclusionAabb::new([1.0; 3], [3.0; 3]);
        assert!(a.intersects(&b));
    }

    #[test]
    fn aabb_not_intersects_separate() {
        let a = OcclusionAabb::new([0.0; 3], [1.0; 3]);
        let b = OcclusionAabb::new([2.0; 3], [3.0; 3]);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn surface_area_unit_cube() {
        let aabb = OcclusionAabb::new([0.0; 3], [1.0; 3]);
        assert!((aabb.surface_area() - 6.0).abs() < 1e-5);
    }

    #[test]
    fn soft_buffer_write_test() {
        let mut buf = SoftOcclusionBuffer::new(4, 4);
        buf.write(0, 0, 0.5);
        assert!(buf.test(0, 0, 0.3));
        assert!(!buf.test(0, 0, 0.8));
    }

    #[test]
    fn soft_buffer_clear_resets() {
        let mut buf = SoftOcclusionBuffer::new(2, 2);
        buf.write(0, 0, 0.1);
        buf.clear();
        assert!(buf.test(0, 0, 0.99));
    }
}
