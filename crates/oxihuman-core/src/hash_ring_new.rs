// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Consistent hash ring using FNV-1a. Nodes sorted by hash.

pub struct HashRingNew {
    pub nodes: Vec<(u64, String)>,
}

fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub fn new_hash_ring_new() -> HashRingNew {
    HashRingNew { nodes: Vec::new() }
}

pub fn ring_new_add_node(r: &mut HashRingNew, name: &str) {
    let h = fnv1a(name.as_bytes());
    if !r.nodes.iter().any(|(_, n)| n == name) {
        r.nodes.push((h, name.to_string()));
        r.nodes.sort_by_key(|&(h, _)| h);
    }
}

pub fn ring_new_remove_node(r: &mut HashRingNew, name: &str) {
    r.nodes.retain(|(_, n)| n != name);
}

pub fn ring_new_get_node<'a>(r: &'a HashRingNew, key: &str) -> Option<&'a str> {
    if r.nodes.is_empty() {
        return None;
    }
    let h = fnv1a(key.as_bytes());
    // Find first node with hash >= h; wrap around if needed
    let idx = r.nodes.partition_point(|&(nh, _)| nh < h);
    let idx = idx % r.nodes.len();
    Some(&r.nodes[idx].1)
}

pub fn ring_new_node_count(r: &HashRingNew) -> usize {
    r.nodes.len()
}

pub fn ring_new_is_empty(r: &HashRingNew) -> bool {
    r.nodes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ring_is_empty() {
        /* new ring is empty */
        let r = new_hash_ring_new();
        assert!(ring_new_is_empty(&r));
        assert_eq!(ring_new_node_count(&r), 0);
    }

    #[test]
    fn test_add_node() {
        /* adding node increases count */
        let mut r = new_hash_ring_new();
        ring_new_add_node(&mut r, "node1");
        assert_eq!(ring_new_node_count(&r), 1);
        assert!(!ring_new_is_empty(&r));
    }

    #[test]
    fn test_remove_node() {
        /* removing node decreases count */
        let mut r = new_hash_ring_new();
        ring_new_add_node(&mut r, "node1");
        ring_new_remove_node(&mut r, "node1");
        assert_eq!(ring_new_node_count(&r), 0);
    }

    #[test]
    fn test_get_node_returns_some() {
        /* get_node returns a valid node when ring is not empty */
        let mut r = new_hash_ring_new();
        ring_new_add_node(&mut r, "alpha");
        let n = ring_new_get_node(&r, "some_key");
        assert_eq!(n, Some("alpha"));
    }

    #[test]
    fn test_get_node_empty_returns_none() {
        /* get_node on empty ring returns None */
        let r = new_hash_ring_new();
        assert!(ring_new_get_node(&r, "key").is_none());
    }

    #[test]
    fn test_duplicate_add_no_effect() {
        /* adding same node twice does not create duplicates */
        let mut r = new_hash_ring_new();
        ring_new_add_node(&mut r, "node1");
        ring_new_add_node(&mut r, "node1");
        assert_eq!(ring_new_node_count(&r), 1);
    }

    #[test]
    fn test_multiple_nodes() {
        /* multiple distinct nodes are all added */
        let mut r = new_hash_ring_new();
        ring_new_add_node(&mut r, "a");
        ring_new_add_node(&mut r, "b");
        ring_new_add_node(&mut r, "c");
        assert_eq!(ring_new_node_count(&r), 3);
        let n = ring_new_get_node(&r, "test_key");
        assert!(n.is_some());
    }
}
