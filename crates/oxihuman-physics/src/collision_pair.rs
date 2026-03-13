// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Broadphase collision pair management.

#![allow(dead_code)]

/// A pair of body ids that are potentially colliding.
#[allow(dead_code)]
pub struct CollisionPairEntry {
    pub id_a: u32,
    pub id_b: u32,
}

/// A buffer of collision pairs with a maximum capacity.
#[allow(dead_code)]
pub struct PairBuffer {
    pub pairs: Vec<CollisionPairEntry>,
    pub capacity: usize,
}

/// Create a new pair buffer with the given maximum capacity.
#[allow(dead_code)]
pub fn new_pair_buffer(cap: usize) -> PairBuffer {
    PairBuffer {
        pairs: Vec::with_capacity(cap),
        capacity: cap,
    }
}

/// Add a collision pair (a, b). Ignores duplicates and respects capacity.
#[allow(dead_code)]
pub fn add_pair(buf: &mut PairBuffer, a: u32, b: u32) {
    if buf.pairs.len() >= buf.capacity {
        return;
    }
    if !has_pair(buf, a, b) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        buf.pairs.push(CollisionPairEntry { id_a: lo, id_b: hi });
    }
}

/// Return true if the pair (a, b) is in the buffer.
#[allow(dead_code)]
pub fn has_pair(buf: &PairBuffer, a: u32, b: u32) -> bool {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    buf.pairs.iter().any(|p| p.id_a == lo && p.id_b == hi)
}

/// Clear all pairs from the buffer.
#[allow(dead_code)]
pub fn clear_pairs(buf: &mut PairBuffer) {
    buf.pairs.clear();
}

/// Return the number of pairs in the buffer.
#[allow(dead_code)]
pub fn pair_count(buf: &PairBuffer) -> usize {
    buf.pairs.len()
}

/// Return all body ids that body `id` is paired with.
#[allow(dead_code)]
pub fn pairs_for_body(buf: &PairBuffer, id: u32) -> Vec<u32> {
    buf.pairs
        .iter()
        .filter_map(|p| {
            if p.id_a == id {
                Some(p.id_b)
            } else if p.id_b == id {
                Some(p.id_a)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_empty() {
        let buf = new_pair_buffer(10);
        assert_eq!(pair_count(&buf), 0);
    }

    #[test]
    fn add_pair_and_query() {
        let mut buf = new_pair_buffer(10);
        add_pair(&mut buf, 1, 2);
        assert!(has_pair(&buf, 1, 2));
        assert!(has_pair(&buf, 2, 1));
    }

    #[test]
    fn no_duplicate_pairs() {
        let mut buf = new_pair_buffer(10);
        add_pair(&mut buf, 1, 2);
        add_pair(&mut buf, 2, 1);
        assert_eq!(pair_count(&buf), 1);
    }

    #[test]
    fn clear_removes_all() {
        let mut buf = new_pair_buffer(10);
        add_pair(&mut buf, 1, 2);
        add_pair(&mut buf, 3, 4);
        clear_pairs(&mut buf);
        assert_eq!(pair_count(&buf), 0);
    }

    #[test]
    fn capacity_limit_respected() {
        let mut buf = new_pair_buffer(2);
        add_pair(&mut buf, 1, 2);
        add_pair(&mut buf, 3, 4);
        add_pair(&mut buf, 5, 6);
        assert_eq!(pair_count(&buf), 2);
    }

    #[test]
    fn pairs_for_body_correct() {
        let mut buf = new_pair_buffer(10);
        add_pair(&mut buf, 1, 2);
        add_pair(&mut buf, 1, 3);
        add_pair(&mut buf, 4, 5);
        let partners = pairs_for_body(&buf, 1);
        assert!(partners.contains(&2));
        assert!(partners.contains(&3));
        assert_eq!(partners.len(), 2);
    }

    #[test]
    fn has_pair_returns_false_for_missing() {
        let buf = new_pair_buffer(10);
        assert!(!has_pair(&buf, 99, 100));
    }

    #[test]
    fn pair_count_increments_correctly() {
        let mut buf = new_pair_buffer(10);
        add_pair(&mut buf, 0, 1);
        add_pair(&mut buf, 2, 3);
        assert_eq!(pair_count(&buf), 2);
    }
}
