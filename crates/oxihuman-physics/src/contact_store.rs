// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A stored contact point for persistent contact tracking.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredContact {
    pub body_a: u32,
    pub body_b: u32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub age: u32,
    pub warm_impulse: f32,
}

#[allow(dead_code)]
impl StoredContact {
    pub fn new(body_a: u32, body_b: u32, point: [f32; 3], normal: [f32; 3], depth: f32) -> Self {
        Self {
            body_a,
            body_b,
            point,
            normal,
            depth,
            age: 0,
            warm_impulse: 0.0,
        }
    }

    pub fn is_separating(&self) -> bool {
        self.depth < 0.0
    }

    pub fn involves(&self, body_id: u32) -> bool {
        self.body_a == body_id || self.body_b == body_id
    }
}

/// A contact store that maintains contacts across frames for warm-starting.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ContactStore {
    contacts: Vec<StoredContact>,
    max_age: u32,
}

#[allow(dead_code)]
impl ContactStore {
    pub fn new(max_age: u32) -> Self {
        Self {
            contacts: Vec::new(),
            max_age,
        }
    }

    pub fn add(&mut self, contact: StoredContact) {
        // Check for existing contact between same pair at similar point
        let existing = self.contacts.iter_mut().find(|c| {
            c.body_a == contact.body_a && c.body_b == contact.body_b && point_near(c.point, contact.point, 0.01)
        });
        if let Some(c) = existing {
            c.point = contact.point;
            c.normal = contact.normal;
            c.depth = contact.depth;
            c.age = 0;
        } else {
            self.contacts.push(contact);
        }
    }

    pub fn age_and_prune(&mut self) {
        for c in &mut self.contacts {
            c.age += 1;
        }
        let max = self.max_age;
        self.contacts.retain(|c| c.age <= max);
    }

    pub fn find(&self, body_a: u32, body_b: u32) -> Vec<&StoredContact> {
        self.contacts
            .iter()
            .filter(|c| {
                (c.body_a == body_a && c.body_b == body_b)
                    || (c.body_a == body_b && c.body_b == body_a)
            })
            .collect()
    }

    pub fn contacts_for_body(&self, body_id: u32) -> Vec<&StoredContact> {
        self.contacts.iter().filter(|c| c.involves(body_id)).collect()
    }

    pub fn count(&self) -> usize {
        self.contacts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contacts.is_empty()
    }

    pub fn clear(&mut self) {
        self.contacts.clear();
    }

    pub fn total_penetration(&self) -> f32 {
        self.contacts.iter().map(|c| c.depth.max(0.0)).sum()
    }

    pub fn deepest_contact(&self) -> Option<&StoredContact> {
        self.contacts.iter().max_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn all_contacts(&self) -> &[StoredContact] {
        &self.contacts
    }
}

#[allow(dead_code)]
fn point_near(a: [f32; 3], b: [f32; 3], threshold: f32) -> bool {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz) < threshold * threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(a: u32, b: u32, depth: f32) -> StoredContact {
        StoredContact::new(a, b, [0.0; 3], [0.0, 1.0, 0.0], depth)
    }

    #[test]
    fn test_add_contact() {
        let mut store = ContactStore::new(3);
        store.add(make_contact(0, 1, 0.1));
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_age_and_prune() {
        let mut store = ContactStore::new(1);
        store.add(make_contact(0, 1, 0.1));
        store.age_and_prune();
        assert_eq!(store.count(), 1);
        store.age_and_prune();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_find() {
        let mut store = ContactStore::new(10);
        store.add(make_contact(0, 1, 0.1));
        store.add(make_contact(2, 3, 0.2));
        assert_eq!(store.find(0, 1).len(), 1);
    }

    #[test]
    fn test_contacts_for_body() {
        let mut store = ContactStore::new(10);
        store.add(make_contact(0, 1, 0.1));
        store.add(make_contact(0, 2, 0.1));
        store.add(make_contact(3, 4, 0.1));
        assert_eq!(store.contacts_for_body(0).len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut store = ContactStore::new(10);
        store.add(make_contact(0, 1, 0.1));
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_total_penetration() {
        let mut store = ContactStore::new(10);
        store.add(make_contact(0, 1, 0.1));
        store.add(StoredContact::new(2, 3, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.2));
        assert!((store.total_penetration() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_deepest_contact() {
        let mut store = ContactStore::new(10);
        store.add(make_contact(0, 1, 0.1));
        store.add(StoredContact::new(2, 3, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5));
        let deepest = store.deepest_contact().expect("should succeed");
        assert!((deepest.depth - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_update_existing() {
        let mut store = ContactStore::new(10);
        store.add(make_contact(0, 1, 0.1));
        store.add(make_contact(0, 1, 0.3));
        assert_eq!(store.count(), 1);
        assert!((store.all_contacts()[0].depth - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_separating() {
        let c = make_contact(0, 1, -0.1);
        assert!(c.is_separating());
    }

    #[test]
    fn test_involves() {
        let c = make_contact(5, 10, 0.1);
        assert!(c.involves(5));
        assert!(c.involves(10));
        assert!(!c.involves(7));
    }
}
