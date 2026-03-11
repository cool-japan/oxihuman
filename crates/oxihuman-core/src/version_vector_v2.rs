// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Version vector for distributed conflict detection.

use std::collections::HashMap;

/// A version vector mapping node IDs to logical clocks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VersionVectorV2 {
    clocks: HashMap<String, u64>,
}

/// Ordering relationship between two version vectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VvRelation {
    /// Self is strictly less than other (other dominates).
    Before,
    /// Self is strictly greater than other (self dominates).
    After,
    /// Self and other are identical.
    Equal,
    /// Neither dominates — concurrent modification.
    Concurrent,
}

impl VersionVectorV2 {
    /// Create a new empty version vector.
    pub fn new() -> Self {
        VersionVectorV2 { clocks: HashMap::new() }
    }

    /// Increment the clock for `node`.
    pub fn increment(&mut self, node: &str) {
        let c = self.clocks.entry(node.to_owned()).or_insert(0);
        *c += 1;
    }

    /// Get the clock value for `node` (0 if absent).
    pub fn get(&self, node: &str) -> u64 {
        *self.clocks.get(node).unwrap_or(&0)
    }

    /// Merge another version vector into self (take component-wise max).
    pub fn merge(&mut self, other: &VersionVectorV2) {
        for (node, &clock) in &other.clocks {
            let entry = self.clocks.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(clock);
        }
    }

    /// Compute the causal relation between self and `other`.
    pub fn relation(&self, other: &VersionVectorV2) -> VvRelation {
        let all_nodes: std::collections::HashSet<&str> = self
            .clocks
            .keys()
            .chain(other.clocks.keys())
            .map(|s| s.as_str())
            .collect();
        let mut self_le = true;
        let mut other_le = true;
        for node in all_nodes {
            let a = self.get(node);
            let b = other.get(node);
            if a > b {
                other_le = false;
            }
            if b > a {
                self_le = false;
            }
        }
        match (self_le, other_le) {
            (true, true) => VvRelation::Equal,
            (true, false) => VvRelation::Before,
            (false, true) => VvRelation::After,
            (false, false) => VvRelation::Concurrent,
        }
    }

    /// Number of distinct nodes tracked.
    pub fn node_count(&self) -> usize {
        self.clocks.len()
    }
}

/// Create a new version vector.
pub fn new_version_vector_v2() -> VersionVectorV2 {
    VersionVectorV2::new()
}

/// Increment a node's clock.
pub fn vv_increment(vv: &mut VersionVectorV2, node: &str) {
    vv.increment(node);
}

/// Merge two version vectors.
pub fn vv_merge(base: &mut VersionVectorV2, other: &VersionVectorV2) {
    base.merge(other);
}

/// Get the causal relation.
pub fn vv_relation(a: &VersionVectorV2, b: &VersionVectorV2) -> VvRelation {
    a.relation(b)
}

/// Get the clock for a node.
pub fn vv_get(vv: &VersionVectorV2, node: &str) -> u64 {
    vv.get(node)
}

/// Node count.
pub fn vv_node_count(vv: &VersionVectorV2) -> usize {
    vv.node_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut vv = new_version_vector_v2();
        vv_increment(&mut vv, "A");
        vv_increment(&mut vv, "A");
        assert_eq!(vv_get(&vv, "A"), 2 /* two increments */);
    }

    #[test]
    fn test_default_zero() {
        let vv = new_version_vector_v2();
        assert_eq!(vv_get(&vv, "X"), 0 /* unknown node is 0 */);
    }

    #[test]
    fn test_equal() {
        let mut a = new_version_vector_v2();
        let mut b = new_version_vector_v2();
        vv_increment(&mut a, "N");
        vv_increment(&mut b, "N");
        assert_eq!(vv_relation(&a, &b), VvRelation::Equal /* same clocks */);
    }

    #[test]
    fn test_before() {
        let a = new_version_vector_v2();
        let mut b = new_version_vector_v2();
        vv_increment(&mut b, "N");
        assert_eq!(vv_relation(&a, &b), VvRelation::Before /* a < b */);
    }

    #[test]
    fn test_after() {
        let mut a = new_version_vector_v2();
        let b = new_version_vector_v2();
        vv_increment(&mut a, "N");
        assert_eq!(vv_relation(&a, &b), VvRelation::After /* a > b */);
    }

    #[test]
    fn test_concurrent() {
        let mut a = new_version_vector_v2();
        let mut b = new_version_vector_v2();
        vv_increment(&mut a, "A");
        vv_increment(&mut b, "B");
        assert_eq!(vv_relation(&a, &b), VvRelation::Concurrent /* concurrent */);
    }

    #[test]
    fn test_merge() {
        let mut a = new_version_vector_v2();
        let mut b = new_version_vector_v2();
        vv_increment(&mut a, "X");
        vv_increment(&mut b, "Y");
        vv_merge(&mut a, &b);
        assert_eq!(vv_get(&a, "Y"), 1 /* merged */);
        assert_eq!(vv_get(&a, "X"), 1);
    }

    #[test]
    fn test_merge_takes_max() {
        let mut a = new_version_vector_v2();
        let mut b = new_version_vector_v2();
        vv_increment(&mut a, "N");
        vv_increment(&mut a, "N");
        vv_increment(&mut b, "N");
        vv_merge(&mut a, &b);
        assert_eq!(vv_get(&a, "N"), 2 /* max is 2 */);
    }

    #[test]
    fn test_node_count() {
        let mut vv = new_version_vector_v2();
        vv_increment(&mut vv, "A");
        vv_increment(&mut vv, "B");
        assert_eq!(vv_node_count(&vv), 2 /* two nodes */);
    }
}
