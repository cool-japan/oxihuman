// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Collision pair list management (broad-phase results).

#![allow(dead_code)]

/// A pair of overlapping bodies with their distance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionPair {
    pub a: u32,
    pub b: u32,
    pub distance: f32,
}

/// A list of collision pairs from the broad phase.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionPairList {
    pairs: Vec<CollisionPair>,
}

/// Create a new empty collision pair list.
#[allow(dead_code)]
pub fn new_collision_pair_list() -> CollisionPairList {
    CollisionPairList { pairs: Vec::new() }
}

/// Add a collision pair.
#[allow(dead_code)]
pub fn pair_list_add(list: &mut CollisionPairList, a: u32, b: u32, distance: f32) {
    list.pairs.push(CollisionPair { a, b, distance });
}

/// Remove a pair by body ids. Returns true if found and removed.
#[allow(dead_code)]
pub fn pair_list_remove(list: &mut CollisionPairList, a: u32, b: u32) -> bool {
    let before = list.pairs.len();
    list.pairs.retain(|p| !((p.a == a && p.b == b) || (p.a == b && p.b == a)));
    list.pairs.len() < before
}

/// Return the number of pairs.
#[allow(dead_code)]
pub fn pair_list_count(list: &CollisionPairList) -> usize {
    list.pairs.len()
}

/// Clear all pairs.
#[allow(dead_code)]
pub fn pair_list_clear(list: &mut CollisionPairList) {
    list.pairs.clear();
}

/// Check whether a pair (in either order) is present.
#[allow(dead_code)]
pub fn pair_list_contains(list: &CollisionPairList, a: u32, b: u32) -> bool {
    list.pairs.iter().any(|p| (p.a == a && p.b == b) || (p.a == b && p.b == a))
}

/// Return the pair with the smallest distance, if any.
#[allow(dead_code)]
pub fn pair_list_closest(list: &CollisionPairList) -> Option<&CollisionPair> {
    list.pairs.iter().min_by(|x, y| x.distance.partial_cmp(&y.distance).unwrap_or(std::cmp::Ordering::Equal))
}

/// Return pairs with distance <= max_dist.
#[allow(dead_code)]
pub fn pair_list_filter_by_distance(list: &CollisionPairList, max_dist: f32) -> Vec<&CollisionPair> {
    list.pairs.iter().filter(|p| p.distance <= max_dist).collect()
}

/// Serialize the list to a JSON string.
#[allow(dead_code)]
pub fn pair_list_to_json(list: &CollisionPairList) -> String {
    let parts: Vec<String> = list.pairs.iter()
        .map(|p| format!("{{\"a\":{},\"b\":{},\"dist\":{:.4}}}", p.a, p.b, p.distance))
        .collect();
    format!("[{}]", parts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_list_empty() {
        let list = new_collision_pair_list();
        assert_eq!(pair_list_count(&list), 0);
    }

    #[test]
    fn test_add_pair() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 1, 2, 0.5);
        assert_eq!(pair_list_count(&list), 1);
    }

    #[test]
    fn test_contains() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 3, 4, 1.0);
        assert!(pair_list_contains(&list, 3, 4));
        assert!(pair_list_contains(&list, 4, 3));
        assert!(!pair_list_contains(&list, 3, 5));
    }

    #[test]
    fn test_remove() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 1, 2, 0.5);
        let removed = pair_list_remove(&mut list, 1, 2);
        assert!(removed);
        assert_eq!(pair_list_count(&list), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut list = new_collision_pair_list();
        assert!(!pair_list_remove(&mut list, 9, 10));
    }

    #[test]
    fn test_clear() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 1, 2, 1.0);
        pair_list_add(&mut list, 3, 4, 2.0);
        pair_list_clear(&mut list);
        assert_eq!(pair_list_count(&list), 0);
    }

    #[test]
    fn test_closest() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 1, 2, 3.0);
        pair_list_add(&mut list, 3, 4, 0.5);
        pair_list_add(&mut list, 5, 6, 2.0);
        let closest = pair_list_closest(&list).expect("should succeed");
        assert_eq!(closest.a, 3);
        assert_eq!(closest.b, 4);
    }

    #[test]
    fn test_filter_by_distance() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 1, 2, 1.0);
        pair_list_add(&mut list, 3, 4, 5.0);
        let filtered = pair_list_filter_by_distance(&list, 2.0);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_to_json() {
        let mut list = new_collision_pair_list();
        pair_list_add(&mut list, 1, 2, 0.5);
        let json = pair_list_to_json(&list);
        assert!(json.contains("\"a\":1"));
        assert!(json.contains("\"b\":2"));
    }

    #[test]
    fn test_closest_empty() {
        let list = new_collision_pair_list();
        assert!(pair_list_closest(&list).is_none());
    }
}
