// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Contact manifold with warm-starting impulse cache.

#![allow(dead_code)]

/// A single contact point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContactPoint {
    /// Contact position in world space.
    pub position: [f32; 3],
    /// Contact normal (pointing from B to A).
    pub normal: [f32; 3],
    /// Penetration depth (positive = overlapping).
    pub depth: f32,
    /// Cached normal impulse (for warm starting).
    pub lambda_n: f32,
    /// Cached tangential impulse.
    pub lambda_t: [f32; 2],
}

impl ContactPoint {
    pub fn new(position: [f32; 3], normal: [f32; 3], depth: f32) -> Self {
        Self {
            position,
            normal,
            depth,
            lambda_n: 0.0,
            lambda_t: [0.0, 0.0],
        }
    }
}

/// A contact manifold between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactManifoldV2 {
    pub body_a: usize,
    pub body_b: usize,
    pub points: Vec<ContactPoint>,
    /// Max points to keep (typically 4 for stable contacts).
    pub max_points: usize,
}

#[allow(dead_code)]
impl ContactManifoldV2 {
    pub fn new(body_a: usize, body_b: usize) -> Self {
        Self {
            body_a,
            body_b,
            points: Vec::new(),
            max_points: 4,
        }
    }

    /// Add a contact point, trimming to max_points by removing shallowest.
    pub fn add_point(&mut self, cp: ContactPoint) {
        self.points.push(cp);
        if self.points.len() > self.max_points {
            // Remove the point with smallest penetration depth
            let mut min_idx = 0;
            let mut min_depth = self.points[0].depth;
            for (i, p) in self.points.iter().enumerate() {
                if p.depth < min_depth {
                    min_depth = p.depth;
                    min_idx = i;
                }
            }
            self.points.remove(min_idx);
        }
    }

    /// Remove contacts with non-positive depth (no longer penetrating).
    pub fn prune_stale(&mut self) {
        self.points.retain(|p| p.depth > -1e-3);
    }

    /// Number of contact points.
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// Maximum penetration depth.
    pub fn max_depth(&self) -> f32 {
        self.points
            .iter()
            .map(|p| p.depth)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// Average contact position.
    pub fn average_position(&self) -> [f32; 3] {
        if self.points.is_empty() {
            return [0.0; 3];
        }
        let n = self.points.len() as f32;
        let mut sum = [0.0f32; 3];
        for p in &self.points {
            for (s, pos) in sum.iter_mut().zip(p.position.iter()) {
                *s += pos;
            }
        }
        [sum[0] / n, sum[1] / n, sum[2] / n]
    }

    /// Apply warm-starting: scale cached impulses by a factor.
    pub fn warm_start_scale(&mut self, scale: f32) {
        for p in &mut self.points {
            p.lambda_n *= scale;
            p.lambda_t[0] *= scale;
            p.lambda_t[1] *= scale;
        }
    }

    /// Update cached normal impulse for contact i.
    pub fn update_lambda_n(&mut self, i: usize, delta: f32) {
        if i < self.points.len() {
            self.points[i].lambda_n = (self.points[i].lambda_n + delta).max(0.0);
        }
    }

    /// Total cached normal impulse.
    pub fn total_lambda_n(&self) -> f32 {
        self.points.iter().map(|p| p.lambda_n).sum()
    }

    /// True if manifold has any contact.
    pub fn is_active(&self) -> bool {
        !self.points.is_empty()
    }

    /// Clear all contact points.
    pub fn clear(&mut self) {
        self.points.clear();
    }
}

/// A collection of contact manifolds.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ManifoldCache {
    pub manifolds: Vec<ContactManifoldV2>,
}

#[allow(dead_code)]
impl ManifoldCache {
    pub fn new() -> Self {
        Self {
            manifolds: Vec::new(),
        }
    }

    /// Find or create manifold for (a, b).
    pub fn get_or_create(&mut self, body_a: usize, body_b: usize) -> &mut ContactManifoldV2 {
        let key_a = body_a.min(body_b);
        let key_b = body_a.max(body_b);
        if let Some(i) = self
            .manifolds
            .iter()
            .position(|m| m.body_a.min(m.body_b) == key_a && m.body_a.max(m.body_b) == key_b)
        {
            return &mut self.manifolds[i];
        }
        self.manifolds.push(ContactManifoldV2::new(key_a, key_b));
        let len = self.manifolds.len();
        &mut self.manifolds[len - 1]
    }

    pub fn manifold_count(&self) -> usize {
        self.manifolds.len()
    }

    pub fn total_contacts(&self) -> usize {
        self.manifolds.iter().map(|m| m.point_count()).sum()
    }

    /// Remove empty manifolds.
    pub fn prune_empty(&mut self) {
        self.manifolds.retain(|m| m.is_active());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(depth: f32) -> ContactPoint {
        ContactPoint::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], depth)
    }

    #[test]
    fn initial_empty() {
        let m = ContactManifoldV2::new(0, 1);
        assert_eq!(m.point_count(), 0);
        assert!(!m.is_active());
    }

    #[test]
    fn add_point() {
        let mut m = ContactManifoldV2::new(0, 1);
        m.add_point(make_contact(0.1));
        assert_eq!(m.point_count(), 1);
    }

    #[test]
    fn trim_to_max_points() {
        let mut m = ContactManifoldV2::new(0, 1);
        for i in 0..6 {
            m.add_point(make_contact(i as f32 * 0.1 + 0.1));
        }
        assert_eq!(m.point_count(), 4);
    }

    #[test]
    fn prune_stale() {
        let mut m = ContactManifoldV2::new(0, 1);
        m.add_point(make_contact(0.1));
        m.add_point(make_contact(-0.5));
        m.prune_stale();
        assert_eq!(m.point_count(), 1);
    }

    #[test]
    fn max_depth() {
        let mut m = ContactManifoldV2::new(0, 1);
        m.add_point(make_contact(0.1));
        m.add_point(make_contact(0.5));
        assert!((m.max_depth() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn warm_start_scale() {
        let mut m = ContactManifoldV2::new(0, 1);
        let mut cp = make_contact(0.1);
        cp.lambda_n = 2.0;
        m.add_point(cp);
        m.warm_start_scale(0.5);
        assert!((m.points[0].lambda_n - 1.0).abs() < 1e-5);
    }

    #[test]
    fn update_lambda_n_clamped() {
        let mut m = ContactManifoldV2::new(0, 1);
        m.add_point(make_contact(0.1));
        m.update_lambda_n(0, -100.0); // Should clamp to 0
        assert!(m.points[0].lambda_n >= 0.0);
    }

    #[test]
    fn manifold_cache_get_or_create() {
        let mut cache = ManifoldCache::new();
        let _m = cache.get_or_create(0, 1);
        let _m2 = cache.get_or_create(0, 1); // should reuse
        assert_eq!(cache.manifold_count(), 1);
    }

    #[test]
    fn manifold_cache_total_contacts() {
        let mut cache = ManifoldCache::new();
        {
            let m = cache.get_or_create(0, 1);
            m.add_point(make_contact(0.1));
            m.add_point(make_contact(0.2));
        }
        assert_eq!(cache.total_contacts(), 2);
    }

    #[test]
    fn average_position_midpoint() {
        let mut m = ContactManifoldV2::new(0, 1);
        m.add_point(ContactPoint::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1));
        m.add_point(ContactPoint::new([2.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1));
        let avg = m.average_position();
        assert!((avg[0] - 1.0).abs() < 1e-5);
    }
}
