// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-body contact list tracking active contacts with other bodies.

/// A single contact record between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyContact {
    pub body_a: u32,
    pub body_b: u32,
    pub normal: [f32; 3],
    pub depth: f32,
    pub point: [f32; 3],
    pub impulse: f32,
}

#[allow(dead_code)]
impl BodyContact {
    pub fn new(body_a: u32, body_b: u32, normal: [f32; 3], depth: f32, point: [f32; 3]) -> Self {
        Self { body_a, body_b, normal, depth, point, impulse: 0.0 }
    }

    pub fn other_body(&self, me: u32) -> u32 {
        if self.body_a == me { self.body_b } else { self.body_a }
    }

    pub fn is_separating(&self) -> bool {
        self.depth <= 0.0
    }
}

/// Maintains a list of active contacts for bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyContactList {
    contacts: Vec<BodyContact>,
    max_contacts: usize,
}

#[allow(dead_code)]
impl BodyContactList {
    pub fn new(max_contacts: usize) -> Self {
        Self { contacts: Vec::new(), max_contacts }
    }

    pub fn add(&mut self, contact: BodyContact) -> bool {
        if self.contacts.len() >= self.max_contacts {
            // Replace shallowest if new is deeper
            if let Some(pos) = self.contacts.iter().enumerate()
                .min_by(|(_, a), (_, b)| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
            {
                if self.contacts[pos].depth < contact.depth {
                    self.contacts[pos] = contact;
                    return true;
                }
            }
            return false;
        }
        self.contacts.push(contact);
        true
    }

    pub fn remove_by_body(&mut self, body_id: u32) {
        self.contacts.retain(|c| c.body_a != body_id && c.body_b != body_id);
    }

    pub fn contacts_for(&self, body_id: u32) -> Vec<&BodyContact> {
        self.contacts.iter()
            .filter(|c| c.body_a == body_id || c.body_b == body_id)
            .collect()
    }

    pub fn contact_count(&self) -> usize {
        self.contacts.len()
    }

    pub fn deepest_contact(&self) -> Option<&BodyContact> {
        self.contacts.iter().max_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn total_impulse(&self) -> f32 {
        self.contacts.iter().map(|c| c.impulse).sum()
    }

    pub fn clear(&mut self) {
        self.contacts.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.contacts.is_empty()
    }

    pub fn max_depth(&self) -> f32 {
        self.contacts.iter().map(|c| c.depth).fold(0.0f32, f32::max)
    }

    pub fn average_depth(&self) -> f32 {
        if self.contacts.is_empty() { return 0.0; }
        let sum: f32 = self.contacts.iter().map(|c| c.depth).sum();
        sum / self.contacts.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(a: u32, b: u32, depth: f32) -> BodyContact {
        BodyContact::new(a, b, [0.0, 1.0, 0.0], depth, [0.0, 0.0, 0.0])
    }

    #[test]
    fn test_add() {
        let mut cl = BodyContactList::new(10);
        assert!(cl.add(make_contact(0, 1, 0.5)));
        assert_eq!(cl.contact_count(), 1);
    }

    #[test]
    fn test_max_contacts() {
        let mut cl = BodyContactList::new(2);
        cl.add(make_contact(0, 1, 0.1));
        cl.add(make_contact(0, 2, 0.2));
        // Third contact deeper than shallowest should replace
        assert!(cl.add(make_contact(0, 3, 0.5)));
        assert_eq!(cl.contact_count(), 2);
    }

    #[test]
    fn test_contacts_for() {
        let mut cl = BodyContactList::new(10);
        cl.add(make_contact(0, 1, 0.1));
        cl.add(make_contact(0, 2, 0.2));
        cl.add(make_contact(3, 4, 0.3));
        assert_eq!(cl.contacts_for(0).len(), 2);
    }

    #[test]
    fn test_remove_by_body() {
        let mut cl = BodyContactList::new(10);
        cl.add(make_contact(0, 1, 0.1));
        cl.add(make_contact(0, 2, 0.2));
        cl.remove_by_body(0);
        assert!(cl.is_empty());
    }

    #[test]
    fn test_deepest() {
        let mut cl = BodyContactList::new(10);
        cl.add(make_contact(0, 1, 0.1));
        cl.add(make_contact(0, 2, 0.5));
        cl.add(make_contact(0, 3, 0.3));
        assert!((cl.deepest_contact().unwrap().depth - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_other_body() {
        let c = make_contact(5, 10, 0.1);
        assert_eq!(c.other_body(5), 10);
        assert_eq!(c.other_body(10), 5);
    }

    #[test]
    fn test_separating() {
        let c = BodyContact::new(0, 1, [0.0, 1.0, 0.0], -0.1, [0.0, 0.0, 0.0]);
        assert!(c.is_separating());
    }

    #[test]
    fn test_clear() {
        let mut cl = BodyContactList::new(10);
        cl.add(make_contact(0, 1, 0.1));
        cl.clear();
        assert!(cl.is_empty());
    }

    #[test]
    fn test_average_depth() {
        let mut cl = BodyContactList::new(10);
        cl.add(make_contact(0, 1, 1.0));
        cl.add(make_contact(0, 2, 3.0));
        assert!((cl.average_depth() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_depth() {
        let mut cl = BodyContactList::new(10);
        cl.add(make_contact(0, 1, 0.5));
        cl.add(make_contact(0, 2, 1.5));
        assert!((cl.max_depth() - 1.5).abs() < f32::EPSILON);
    }
}
