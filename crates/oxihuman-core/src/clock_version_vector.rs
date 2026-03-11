// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A distributed version vector (vector clock).
pub struct ClockVersionVector {
    pub clocks: HashMap<String, u64>,
}

impl ClockVersionVector {
    pub fn new() -> Self {
        ClockVersionVector {
            clocks: HashMap::new(),
        }
    }
}

impl Default for ClockVersionVector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_clock_version_vector() -> ClockVersionVector {
    ClockVersionVector::new()
}

pub fn cvv_increment(v: &mut ClockVersionVector, node: &str) {
    let counter = v.clocks.entry(node.to_string()).or_insert(0);
    *counter += 1;
}

pub fn cvv_get(v: &ClockVersionVector, node: &str) -> u64 {
    *v.clocks.get(node).unwrap_or(&0)
}

pub fn cvv_merge(a: &ClockVersionVector, b: &ClockVersionVector) -> ClockVersionVector {
    let mut merged = ClockVersionVector::new();
    for (k, &va) in &a.clocks {
        let vb = *b.clocks.get(k).unwrap_or(&0);
        merged.clocks.insert(k.clone(), va.max(vb));
    }
    for (k, &vb) in &b.clocks {
        if !merged.clocks.contains_key(k) {
            merged.clocks.insert(k.clone(), vb);
        }
    }
    merged
}

/// Returns true if `a` dominates (happens-after or equals) `b`.
pub fn cvv_dominates(a: &ClockVersionVector, b: &ClockVersionVector) -> bool {
    for (k, &vb) in &b.clocks {
        if cvv_get(a, k) < vb {
            return false;
        }
    }
    true
}

pub fn cvv_concurrent(a: &ClockVersionVector, b: &ClockVersionVector) -> bool {
    !cvv_dominates(a, b) && !cvv_dominates(b, a)
}

pub fn cvv_node_count(v: &ClockVersionVector) -> usize {
    v.clocks.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new vector has no nodes */
        let v = new_clock_version_vector();
        assert_eq!(cvv_node_count(&v), 0);
    }

    #[test]
    fn test_increment_and_get() {
        /* increment raises counter for node */
        let mut v = new_clock_version_vector();
        cvv_increment(&mut v, "A");
        cvv_increment(&mut v, "A");
        assert_eq!(cvv_get(&v, "A"), 2);
    }

    #[test]
    fn test_get_unknown() {
        /* get for unknown node returns 0 */
        let v = new_clock_version_vector();
        assert_eq!(cvv_get(&v, "X"), 0);
    }

    #[test]
    fn test_merge() {
        /* merge takes element-wise max */
        let mut a = new_clock_version_vector();
        let mut b = new_clock_version_vector();
        cvv_increment(&mut a, "A");
        cvv_increment(&mut a, "A");
        cvv_increment(&mut b, "A");
        cvv_increment(&mut b, "B");
        let m = cvv_merge(&a, &b);
        assert_eq!(cvv_get(&m, "A"), 2);
        assert_eq!(cvv_get(&m, "B"), 1);
    }

    #[test]
    fn test_dominates() {
        /* strictly advanced vector dominates the other */
        let mut a = new_clock_version_vector();
        let mut b = new_clock_version_vector();
        cvv_increment(&mut a, "A");
        cvv_increment(&mut a, "A");
        cvv_increment(&mut b, "A");
        assert!(cvv_dominates(&a, &b));
        assert!(!cvv_dominates(&b, &a));
    }

    #[test]
    fn test_concurrent() {
        /* concurrent when neither dominates */
        let mut a = new_clock_version_vector();
        let mut b = new_clock_version_vector();
        cvv_increment(&mut a, "A");
        cvv_increment(&mut b, "B");
        assert!(cvv_concurrent(&a, &b));
    }

    #[test]
    fn test_node_count() {
        /* node count reflects unique nodes */
        let mut v = new_clock_version_vector();
        cvv_increment(&mut v, "X");
        cvv_increment(&mut v, "Y");
        cvv_increment(&mut v, "X");
        assert_eq!(cvv_node_count(&v), 2);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let v = ClockVersionVector::default();
        assert_eq!(v.clocks.len(), 0);
    }
}
