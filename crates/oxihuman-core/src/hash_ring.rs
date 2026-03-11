#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Consistent hash ring for distributed key mapping.

use std::collections::BTreeMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HashRing {
    ring: BTreeMap<u64, String>,
    replicas: usize,
}

#[allow(dead_code)]
fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(u64::from(b));
    }
    h
}

#[allow(dead_code)]
pub fn new_hash_ring(replicas: usize) -> HashRing {
    HashRing {
        ring: BTreeMap::new(),
        replicas: if replicas == 0 { 1 } else { replicas },
    }
}

#[allow(dead_code)]
pub fn add_ring_node(hr: &mut HashRing, node: &str) {
    for i in 0..hr.replicas {
        let key = format!("{}#{}", node, i);
        let h = simple_hash(&key);
        hr.ring.insert(h, node.to_string());
    }
}

#[allow(dead_code)]
pub fn remove_ring_node(hr: &mut HashRing, node: &str) {
    hr.ring.retain(|_, v| v != node);
}

#[allow(dead_code)]
pub fn get_ring_node(hr: &HashRing, key: &str) -> Option<String> {
    if hr.ring.is_empty() {
        return None;
    }
    let h = simple_hash(key);
    let node = hr
        .ring
        .range(h..)
        .next()
        .or_else(|| hr.ring.iter().next())
        .map(|(_, v)| v.clone());
    node
}

#[allow(dead_code)]
pub fn ring_node_count(hr: &HashRing) -> usize {
    let mut nodes: Vec<&String> = hr.ring.values().collect();
    nodes.sort();
    nodes.dedup();
    nodes.len()
}

#[allow(dead_code)]
pub fn ring_to_json(hr: &HashRing) -> String {
    format!(
        r#"{{"nodes":{},"entries":{},"replicas":{}}}"#,
        ring_node_count(hr),
        hr.ring.len(),
        hr.replicas
    )
}

#[allow(dead_code)]
pub fn ring_rebalance(hr: &mut HashRing) {
    // Re-insert all nodes to ensure uniform distribution
    let nodes: Vec<String> = {
        let mut n: Vec<String> = hr.ring.values().cloned().collect();
        n.sort();
        n.dedup();
        n
    };
    hr.ring.clear();
    for node in &nodes {
        for i in 0..hr.replicas {
            let key = format!("{}#{}", node, i);
            let h = simple_hash(&key);
            hr.ring.insert(h, node.clone());
        }
    }
}

#[allow(dead_code)]
pub fn ring_distribution(hr: &HashRing) -> Vec<(String, usize)> {
    let mut counts = std::collections::HashMap::new();
    for v in hr.ring.values() {
        *counts.entry(v.clone()).or_insert(0usize) += 1;
    }
    let mut result: Vec<(String, usize)> = counts.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hash_ring() {
        let hr = new_hash_ring(3);
        assert_eq!(ring_node_count(&hr), 0);
    }

    #[test]
    fn test_add_node() {
        let mut hr = new_hash_ring(3);
        add_ring_node(&mut hr, "node_a");
        assert_eq!(ring_node_count(&hr), 1);
    }

    #[test]
    fn test_remove_node() {
        let mut hr = new_hash_ring(3);
        add_ring_node(&mut hr, "node_a");
        remove_ring_node(&mut hr, "node_a");
        assert_eq!(ring_node_count(&hr), 0);
    }

    #[test]
    fn test_get_node() {
        let mut hr = new_hash_ring(3);
        add_ring_node(&mut hr, "node_a");
        let n = get_ring_node(&hr, "key1");
        assert!(n.is_some());
    }

    #[test]
    fn test_get_node_empty() {
        let hr = new_hash_ring(3);
        assert!(get_ring_node(&hr, "key").is_none());
    }

    #[test]
    fn test_ring_to_json() {
        let hr = new_hash_ring(2);
        let json = ring_to_json(&hr);
        assert!(json.contains("\"nodes\":0"));
    }

    #[test]
    fn test_ring_rebalance() {
        let mut hr = new_hash_ring(3);
        add_ring_node(&mut hr, "a");
        ring_rebalance(&mut hr);
        assert_eq!(ring_node_count(&hr), 1);
    }

    #[test]
    fn test_ring_distribution() {
        let mut hr = new_hash_ring(3);
        add_ring_node(&mut hr, "a");
        add_ring_node(&mut hr, "b");
        let dist = ring_distribution(&hr);
        assert_eq!(dist.len(), 2);
    }

    #[test]
    fn test_deterministic_lookup() {
        let mut hr = new_hash_ring(3);
        add_ring_node(&mut hr, "a");
        let n1 = get_ring_node(&hr, "key");
        let n2 = get_ring_node(&hr, "key");
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_zero_replicas() {
        let hr = new_hash_ring(0);
        assert_eq!(hr.replicas, 1);
    }
}
