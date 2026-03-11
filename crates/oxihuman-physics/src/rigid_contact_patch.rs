// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rigid body contact patch — computes and manages a set of contact points.

/// A single contact point in a contact patch.
#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub position: [f64; 3],
    pub normal: [f64; 3], /* pointing from B to A */
    pub depth: f64,       /* penetration depth (positive = overlap) */
    pub impulse: f64,     /* accumulated normal impulse */
}

impl ContactPoint {
    pub fn new(position: [f64; 3], normal: [f64; 3], depth: f64) -> Self {
        ContactPoint {
            position,
            normal,
            impulse: 0.0,
            depth,
        }
    }

    /// Normalize the contact normal.
    pub fn normalized_normal(&self) -> [f64; 3] {
        let len = (self.normal[0] * self.normal[0]
            + self.normal[1] * self.normal[1]
            + self.normal[2] * self.normal[2])
            .sqrt();
        if len < 1e-12 {
            return [0.0, 1.0, 0.0];
        }
        [
            self.normal[0] / len,
            self.normal[1] / len,
            self.normal[2] / len,
        ]
    }
}

/// A contact patch between two rigid bodies.
#[derive(Debug, Default, Clone)]
pub struct ContactPatch {
    pub points: Vec<ContactPoint>,
    pub body_a: usize,
    pub body_b: usize,
}

impl ContactPatch {
    /// Create a new empty contact patch.
    pub fn new(body_a: usize, body_b: usize) -> Self {
        ContactPatch {
            points: vec![],
            body_a,
            body_b,
        }
    }

    /// Add a contact point to the patch.
    pub fn add_point(&mut self, position: [f64; 3], normal: [f64; 3], depth: f64) {
        if depth >= 0.0 {
            self.points.push(ContactPoint::new(position, normal, depth));
        }
    }

    /// Average contact position (centroid).
    pub fn centroid(&self) -> [f64; 3] {
        if self.points.is_empty() {
            return [0.0; 3];
        }
        let n = self.points.len() as f64;
        let mut c = [0.0f64; 3];
        for p in &self.points {
            c[0] += p.position[0];
            c[1] += p.position[1];
            c[2] += p.position[2];
        }
        [c[0] / n, c[1] / n, c[2] / n]
    }

    /// Maximum penetration depth in the patch.
    pub fn max_depth(&self) -> f64 {
        self.points.iter().map(|p| p.depth).fold(0.0f64, f64::max)
    }

    /// Average normal (sum of normalized normals, then normalized again).
    pub fn average_normal(&self) -> [f64; 3] {
        if self.points.is_empty() {
            return [0.0, 1.0, 0.0];
        }
        let mut sum = [0.0f64; 3];
        for p in &self.points {
            let n = p.normalized_normal();
            sum[0] += n[0];
            sum[1] += n[1];
            sum[2] += n[2];
        }
        let len = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
        if len < 1e-12 {
            [0.0, 1.0, 0.0]
        } else {
            [sum[0] / len, sum[1] / len, sum[2] / len]
        }
    }

    /// Total accumulated impulse.
    pub fn total_impulse(&self) -> f64 {
        self.points.iter().map(|p| p.impulse).sum()
    }

    /// Number of contact points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// True if no contact points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Prune contact points with depth below threshold.
pub fn prune_contacts(patch: &mut ContactPatch, min_depth: f64) {
    patch.points.retain(|p| p.depth >= min_depth);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_point() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.01);
        assert_eq!(patch.len(), 1 /* one contact point added */);
    }

    #[test]
    fn test_negative_depth_not_added() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], -0.01);
        assert!(patch.is_empty() /* negative depth not added */);
    }

    #[test]
    fn test_centroid_single_point() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([1.0, 2.0, 3.0], [0.0, 1.0, 0.0], 0.01);
        let c = patch.centroid();
        assert_eq!(c, [1.0, 2.0, 3.0] /* centroid equals single point */);
    }

    #[test]
    fn test_max_depth() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([0.0; 3], [0.0, 1.0, 0.0], 0.01);
        patch.add_point([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.05);
        assert!((patch.max_depth() - 0.05).abs() < 1e-10 /* max depth is 0.05 */);
    }

    #[test]
    fn test_average_normal_up() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([0.0; 3], [0.0, 1.0, 0.0], 0.01);
        patch.add_point([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.01);
        let n = patch.average_normal();
        assert!((n[1] - 1.0).abs() < 1e-6 /* average normal points up */);
    }

    #[test]
    fn test_total_impulse() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([0.0; 3], [0.0, 1.0, 0.0], 0.01);
        patch.points[0].impulse = 5.0;
        assert!((patch.total_impulse() - 5.0).abs() < 1e-10 /* impulse sum */);
    }

    #[test]
    fn test_prune_contacts() {
        let mut patch = ContactPatch::new(0, 1);
        patch.add_point([0.0; 3], [0.0, 1.0, 0.0], 0.001);
        patch.add_point([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.05);
        prune_contacts(&mut patch, 0.01);
        assert_eq!(patch.len(), 1 /* shallow contact pruned */);
    }

    #[test]
    fn test_normalized_normal() {
        let cp = ContactPoint::new([0.0; 3], [3.0, 4.0, 0.0], 0.1);
        let n = cp.normalized_normal();
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-10 /* normalized to unit length */);
    }

    #[test]
    fn test_empty_centroid() {
        let patch = ContactPatch::new(0, 1);
        let c = patch.centroid();
        assert_eq!(c, [0.0; 3] /* empty patch centroid is origin */);
    }
}
