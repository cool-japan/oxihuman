// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Broad-phase pair: axis-aligned bounding box overlap pairs.

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct BroadAabb {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
    pub id: u32,
}

/// Create a BroadAabb.
#[allow(dead_code)]
pub fn new_broad_aabb(id: u32, cx: f32, cy: f32, cz: f32, hx: f32, hy: f32, hz: f32) -> BroadAabb {
    BroadAabb {
        id,
        min_x: cx - hx,
        min_y: cy - hy,
        min_z: cz - hz,
        max_x: cx + hx,
        max_y: cy + hy,
        max_z: cz + hz,
    }
}

/// Whether two AABBs overlap.
#[allow(dead_code)]
pub fn aabb_overlaps(a: &BroadAabb, b: &BroadAabb) -> bool {
    a.min_x <= b.max_x
        && a.max_x >= b.min_x
        && a.min_y <= b.max_y
        && a.max_y >= b.min_y
        && a.min_z <= b.max_z
        && a.max_z >= b.min_z
}

/// A pair of overlapping body IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct OverlapPair(pub u32, pub u32);

impl OverlapPair {
    /// Canonical form (smaller id first).
    pub fn canonical(self) -> Self {
        if self.0 <= self.1 {
            self
        } else {
            OverlapPair(self.1, self.0)
        }
    }
}

/// Brute-force broad-phase: O(n²) overlap test.
#[allow(dead_code)]
pub fn broad_phase_naive(bodies: &[BroadAabb]) -> Vec<OverlapPair> {
    let mut pairs = Vec::new();
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            if aabb_overlaps(&bodies[i], &bodies[j]) {
                pairs.push(OverlapPair(bodies[i].id, bodies[j].id));
            }
        }
    }
    pairs
}

/// AABB volume.
#[allow(dead_code)]
pub fn aabb_volume(a: &BroadAabb) -> f32 {
    let dx = (a.max_x - a.min_x).max(0.0);
    let dy = (a.max_y - a.min_y).max(0.0);
    let dz = (a.max_z - a.min_z).max(0.0);
    dx * dy * dz
}

/// Expand AABB by margin.
#[allow(dead_code)]
pub fn aabb_expand(a: &BroadAabb, margin: f32) -> BroadAabb {
    BroadAabb {
        id: a.id,
        min_x: a.min_x - margin,
        min_y: a.min_y - margin,
        min_z: a.min_z - margin,
        max_x: a.max_x + margin,
        max_y: a.max_y + margin,
        max_z: a.max_z + margin,
    }
}

/// Center of AABB.
#[allow(dead_code)]
pub fn aabb_center(a: &BroadAabb) -> (f32, f32, f32) {
    (
        (a.min_x + a.max_x) * 0.5,
        (a.min_y + a.max_y) * 0.5,
        (a.min_z + a.max_z) * 0.5,
    )
}

/// Merge two AABBs.
#[allow(dead_code)]
pub fn aabb_merge(a: &BroadAabb, b: &BroadAabb) -> BroadAabb {
    BroadAabb {
        id: a.id,
        min_x: a.min_x.min(b.min_x),
        min_y: a.min_y.min(b.min_y),
        min_z: a.min_z.min(b.min_z),
        max_x: a.max_x.max(b.max_x),
        max_y: a.max_y.max(b.max_y),
        max_z: a.max_z.max(b.max_z),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_true() {
        let a = new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = new_broad_aabb(1, 1.5, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(aabb_overlaps(&a, &b));
    }

    #[test]
    fn test_overlap_false() {
        let a = new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = new_broad_aabb(1, 5.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(!aabb_overlaps(&a, &b));
    }

    #[test]
    fn test_broad_phase_naive() {
        let bodies = vec![
            new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            new_broad_aabb(1, 1.5, 0.0, 0.0, 1.0, 1.0, 1.0),
            new_broad_aabb(2, 10.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ];
        let pairs = broad_phase_naive(&bodies);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].canonical(), OverlapPair(0, 1));
    }

    #[test]
    fn test_volume() {
        let a = new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0);
        assert!((aabb_volume(&a) - 48.0_f32).abs() < 1e-4);
    }

    #[test]
    fn test_expand() {
        let a = new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = aabb_expand(&a, 0.5);
        assert!((b.min_x - (-1.5_f32)).abs() < 1e-5);
    }

    #[test]
    fn test_center() {
        let a = new_broad_aabb(0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0);
        let (cx, cy, cz) = aabb_center(&a);
        assert!((cx - 3.0_f32).abs() < 1e-5);
        assert!((cy - 3.0_f32).abs() < 1e-5);
        assert!((cz - 3.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_merge() {
        let a = new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = new_broad_aabb(1, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0);
        let m = aabb_merge(&a, &b);
        assert!((m.min_x - (-1.0_f32)).abs() < 1e-5);
        assert!((m.max_x - 3.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_canonical_pair() {
        assert_eq!(OverlapPair(3, 1).canonical(), OverlapPair(1, 3));
        assert_eq!(OverlapPair(1, 3).canonical(), OverlapPair(1, 3));
    }

    #[test]
    fn test_no_self_pair() {
        let bodies = vec![new_broad_aabb(0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0)];
        let pairs = broad_phase_naive(&bodies);
        assert!(pairs.is_empty());
    }
}
