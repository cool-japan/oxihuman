// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vector clock for distributed state tracking.

#![allow(dead_code)]

use std::collections::HashMap;

/// A vector clock mapping node ID to logical timestamp.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VersionVector {
    pub clocks: HashMap<String, u64>,
}

/// Create an empty version vector.
#[allow(dead_code)]
pub fn new_version_vector() -> VersionVector {
    VersionVector {
        clocks: HashMap::new(),
    }
}

/// Increment the clock for a given node.
#[allow(dead_code)]
pub fn vv_increment(vv: &mut VersionVector, node: &str) {
    let entry = vv.clocks.entry(node.to_string()).or_insert(0);
    *entry += 1;
}

/// Get the clock value for a node (0 if not present).
#[allow(dead_code)]
pub fn vv_get(vv: &VersionVector, node: &str) -> u64 {
    *vv.clocks.get(node).unwrap_or(&0)
}

/// Merge two version vectors, taking the max of each component.
#[allow(dead_code)]
pub fn vv_merge(a: &VersionVector, b: &VersionVector) -> VersionVector {
    let mut result = a.clone();
    for (node, &val) in &b.clocks {
        let entry = result.clocks.entry(node.clone()).or_insert(0);
        if val > *entry {
            *entry = val;
        }
    }
    result
}

/// Check if `a` happens-before `b` (all a's components <= b's, and at least one is strictly less).
#[allow(dead_code)]
pub fn vv_happens_before(a: &VersionVector, b: &VersionVector) -> bool {
    // Every component of a must be <= corresponding component in b
    let all_le = a.clocks.iter().all(|(k, &v)| vv_get(b, k) >= v);
    // At least one component of a must be strictly less, OR b has extra keys
    let any_lt = a.clocks.iter().any(|(k, &v)| vv_get(b, k) > v)
        || b.clocks.keys().any(|k| !a.clocks.contains_key(k.as_str()));
    all_le && any_lt
}

/// Check if two version vectors are concurrent (neither happens-before the other).
#[allow(dead_code)]
pub fn vv_concurrent(a: &VersionVector, b: &VersionVector) -> bool {
    !vv_happens_before(a, b) && !vv_happens_before(b, a) && a != b
}

/// Compare two version vectors: -1 (a<b), 0 (equal), 1 (a>b), 2 (concurrent).
#[allow(dead_code)]
pub fn vv_compare(a: &VersionVector, b: &VersionVector) -> i32 {
    if a == b {
        0
    } else if vv_happens_before(a, b) {
        -1
    } else if vv_happens_before(b, a) {
        1
    } else {
        2
    }
}

/// Number of nodes in the vector.
#[allow(dead_code)]
pub fn vv_node_count(vv: &VersionVector) -> usize {
    vv.clocks.len()
}

/// Reset a node's clock to zero.
#[allow(dead_code)]
pub fn vv_reset_node(vv: &mut VersionVector, node: &str) {
    vv.clocks.insert(node.to_string(), 0);
}

/// Return all node names.
#[allow(dead_code)]
pub fn vv_nodes(vv: &VersionVector) -> Vec<&str> {
    vv.clocks.keys().map(|s| s.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let vv = new_version_vector();
        assert_eq!(vv_node_count(&vv), 0);
    }

    #[test]
    fn increment_and_get() {
        let mut vv = new_version_vector();
        vv_increment(&mut vv, "A");
        vv_increment(&mut vv, "A");
        assert_eq!(vv_get(&vv, "A"), 2);
    }

    #[test]
    fn get_missing_returns_zero() {
        let vv = new_version_vector();
        assert_eq!(vv_get(&vv, "X"), 0);
    }

    #[test]
    fn merge_takes_max() {
        let mut a = new_version_vector();
        let mut b = new_version_vector();
        vv_increment(&mut a, "A");
        vv_increment(&mut b, "A");
        vv_increment(&mut b, "A");
        vv_increment(&mut b, "B");
        let merged = vv_merge(&a, &b);
        assert_eq!(vv_get(&merged, "A"), 2);
        assert_eq!(vv_get(&merged, "B"), 1);
    }

    #[test]
    fn happens_before_basic() {
        let mut a = new_version_vector();
        let mut b = new_version_vector();
        vv_increment(&mut a, "A");
        vv_increment(&mut b, "A");
        vv_increment(&mut b, "A");
        assert!(vv_happens_before(&a, &b));
        assert!(!vv_happens_before(&b, &a));
    }

    #[test]
    fn equal_vectors_not_happens_before() {
        let mut a = new_version_vector();
        let mut b = new_version_vector();
        vv_increment(&mut a, "A");
        vv_increment(&mut b, "A");
        assert!(!vv_happens_before(&a, &b));
    }

    #[test]
    fn concurrent_vectors() {
        let mut a = new_version_vector();
        let mut b = new_version_vector();
        vv_increment(&mut a, "A");
        vv_increment(&mut b, "B");
        assert!(vv_concurrent(&a, &b));
    }

    #[test]
    fn compare_equal() {
        let mut a = new_version_vector();
        let mut b = new_version_vector();
        vv_increment(&mut a, "X");
        vv_increment(&mut b, "X");
        assert_eq!(vv_compare(&a, &b), 0);
    }

    #[test]
    fn reset_node() {
        let mut vv = new_version_vector();
        vv_increment(&mut vv, "N");
        vv_increment(&mut vv, "N");
        vv_reset_node(&mut vv, "N");
        assert_eq!(vv_get(&vv, "N"), 0);
    }

    #[test]
    fn node_count_grows() {
        let mut vv = new_version_vector();
        vv_increment(&mut vv, "A");
        vv_increment(&mut vv, "B");
        assert_eq!(vv_node_count(&vv), 2);
    }
}
