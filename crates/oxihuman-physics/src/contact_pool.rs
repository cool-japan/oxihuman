// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A pool of contact points recycled each physics step.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub body_a: u32,
    pub body_b: u32,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub friction: f32,
}

#[allow(dead_code)]
impl ContactPoint {
    pub fn new(body_a: u32, body_b: u32, position: [f32; 3], normal: [f32; 3], depth: f32) -> Self {
        Self { body_a, body_b, position, normal, depth, friction: 0.5 }
    }

    pub fn is_separating(&self) -> bool {
        self.depth < 0.0
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ContactPool {
    contacts: Vec<ContactPoint>,
    capacity: usize,
}

#[allow(dead_code)]
impl ContactPool {
    pub fn new(capacity: usize) -> Self {
        Self { contacts: Vec::with_capacity(capacity), capacity }
    }

    pub fn add(&mut self, contact: ContactPoint) -> bool {
        if self.contacts.len() >= self.capacity {
            return false;
        }
        self.contacts.push(contact);
        true
    }

    pub fn clear(&mut self) {
        self.contacts.clear();
    }

    pub fn len(&self) -> usize {
        self.contacts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contacts.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.contacts.len() >= self.capacity
    }

    pub fn get(&self, idx: usize) -> Option<&ContactPoint> {
        self.contacts.get(idx)
    }

    pub fn contacts(&self) -> &[ContactPoint] {
        &self.contacts
    }

    pub fn contacts_for_body(&self, body_id: u32) -> Vec<&ContactPoint> {
        self.contacts.iter().filter(|c| c.body_a == body_id || c.body_b == body_id).collect()
    }

    pub fn deepest_contact(&self) -> Option<&ContactPoint> {
        self.contacts.iter().max_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn total_depth(&self) -> f32 {
        self.contacts.iter().map(|c| c.depth.max(0.0)).sum()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(a: u32, b: u32, depth: f32) -> ContactPoint {
        ContactPoint::new(a, b, [0.0; 3], [0.0, 1.0, 0.0], depth)
    }

    #[test]
    fn test_add_and_len() {
        let mut pool = ContactPool::new(10);
        pool.add(make_contact(0, 1, 0.5));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_capacity_limit() {
        let mut pool = ContactPool::new(2);
        assert!(pool.add(make_contact(0, 1, 0.1)));
        assert!(pool.add(make_contact(1, 2, 0.2)));
        assert!(!pool.add(make_contact(2, 3, 0.3)));
    }

    #[test]
    fn test_clear() {
        let mut pool = ContactPool::new(10);
        pool.add(make_contact(0, 1, 0.5));
        pool.clear();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_contacts_for_body() {
        let mut pool = ContactPool::new(10);
        pool.add(make_contact(0, 1, 0.1));
        pool.add(make_contact(0, 2, 0.2));
        pool.add(make_contact(3, 4, 0.3));
        assert_eq!(pool.contacts_for_body(0).len(), 2);
    }

    #[test]
    fn test_deepest() {
        let mut pool = ContactPool::new(10);
        pool.add(make_contact(0, 1, 0.1));
        pool.add(make_contact(1, 2, 0.5));
        pool.add(make_contact(2, 3, 0.3));
        let d = pool.deepest_contact().expect("should succeed");
        assert!((d.depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_total_depth() {
        let mut pool = ContactPool::new(10);
        pool.add(make_contact(0, 1, 0.2));
        pool.add(make_contact(1, 2, 0.3));
        assert!((pool.total_depth() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_is_separating() {
        let c = make_contact(0, 1, -0.1);
        assert!(c.is_separating());
    }

    #[test]
    fn test_is_full() {
        let mut pool = ContactPool::new(1);
        assert!(!pool.is_full());
        pool.add(make_contact(0, 1, 0.1));
        assert!(pool.is_full());
    }

    #[test]
    fn test_get() {
        let mut pool = ContactPool::new(10);
        pool.add(make_contact(5, 6, 0.7));
        let c = pool.get(0).expect("should succeed");
        assert_eq!(c.body_a, 5);
    }

    #[test]
    fn test_empty_pool() {
        let pool = ContactPool::new(10);
        assert!(pool.deepest_contact().is_none());
        assert!((pool.total_depth()).abs() < 1e-10);
    }
}
