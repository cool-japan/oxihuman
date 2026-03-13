// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A pair of body IDs in contact, plus associated contact data.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactPair {
    pub body_a: u32,
    pub body_b: u32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub friction: f32,
    pub restitution: f32,
}

/// A set of contact pairs for a simulation frame.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPairSet {
    pairs: Vec<ContactPair>,
}

#[allow(dead_code)]
impl ContactPair {
    pub fn new(body_a: u32, body_b: u32) -> Self {
        Self {
            body_a,
            body_b,
            point: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            depth: 0.0,
            friction: 0.5,
            restitution: 0.3,
        }
    }

    pub fn with_point(mut self, point: [f32; 3]) -> Self {
        self.point = point;
        self
    }

    pub fn with_normal(mut self, normal: [f32; 3]) -> Self {
        self.normal = normal;
        self
    }

    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    pub fn involves(&self, body: u32) -> bool {
        self.body_a == body || self.body_b == body
    }

    pub fn other_body(&self, body: u32) -> Option<u32> {
        if self.body_a == body {
            Some(self.body_b)
        } else if self.body_b == body {
            Some(self.body_a)
        } else {
            None
        }
    }

    pub fn is_deep(&self, threshold: f32) -> bool {
        self.depth > threshold
    }

    pub fn impulse_magnitude(&self, relative_velocity: f32) -> f32 {
        -(1.0 + self.restitution) * relative_velocity
    }
}

impl Default for ContactPairSet {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ContactPairSet {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn add(&mut self, pair: ContactPair) {
        self.pairs.push(pair);
    }

    pub fn count(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn pairs(&self) -> &[ContactPair] {
        &self.pairs
    }

    pub fn contacts_for(&self, body: u32) -> Vec<&ContactPair> {
        self.pairs.iter().filter(|p| p.involves(body)).collect()
    }

    pub fn deepest(&self) -> Option<&ContactPair> {
        self.pairs.iter().max_by(|a, b| {
            a.depth
                .partial_cmp(&b.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn total_depth(&self) -> f32 {
        self.pairs.iter().map(|p| p.depth).sum()
    }

    pub fn average_depth(&self) -> f32 {
        if self.pairs.is_empty() {
            return 0.0;
        }
        self.total_depth() / self.pairs.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pair() {
        let cp = ContactPair::new(0, 1);
        assert_eq!(cp.body_a, 0);
        assert_eq!(cp.body_b, 1);
    }

    #[test]
    fn test_involves() {
        let cp = ContactPair::new(2, 5);
        assert!(cp.involves(2));
        assert!(cp.involves(5));
        assert!(!cp.involves(3));
    }

    #[test]
    fn test_other_body() {
        let cp = ContactPair::new(0, 1);
        assert_eq!(cp.other_body(0), Some(1));
        assert_eq!(cp.other_body(1), Some(0));
        assert!(cp.other_body(9).is_none());
    }

    #[test]
    fn test_with_depth() {
        let cp = ContactPair::new(0, 1).with_depth(0.5);
        assert!((cp.depth - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_deep() {
        let cp = ContactPair::new(0, 1).with_depth(0.5);
        assert!(cp.is_deep(0.1));
        assert!(!cp.is_deep(1.0));
    }

    #[test]
    fn test_set_add_and_count() {
        let mut set = ContactPairSet::new();
        set.add(ContactPair::new(0, 1));
        set.add(ContactPair::new(1, 2));
        assert_eq!(set.count(), 2);
    }

    #[test]
    fn test_contacts_for() {
        let mut set = ContactPairSet::new();
        set.add(ContactPair::new(0, 1));
        set.add(ContactPair::new(1, 2));
        set.add(ContactPair::new(3, 4));
        assert_eq!(set.contacts_for(1).len(), 2);
    }

    #[test]
    fn test_deepest() {
        let mut set = ContactPairSet::new();
        set.add(ContactPair::new(0, 1).with_depth(0.1));
        set.add(ContactPair::new(1, 2).with_depth(0.5));
        assert!((set.deepest().expect("should succeed").depth - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear() {
        let mut set = ContactPairSet::new();
        set.add(ContactPair::new(0, 1));
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn test_average_depth() {
        let mut set = ContactPairSet::new();
        set.add(ContactPair::new(0, 1).with_depth(1.0));
        set.add(ContactPair::new(1, 2).with_depth(3.0));
        assert!((set.average_depth() - 2.0).abs() < f32::EPSILON);
    }
}
