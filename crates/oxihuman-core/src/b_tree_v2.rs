// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#![allow(clippy::vec_box)]

//! B-tree v2 — simple order-4 B-tree (2-3-4 tree) map stub.

const ORDER: usize = 4; /* max keys per node = ORDER - 1 */
const MAX_KEYS: usize = ORDER - 1;
const MIN_KEYS: usize = ORDER / 2 - 1;

/// A node in the B-tree.
#[derive(Debug, Clone)]
struct BTreeNodeV2<K, V> {
    keys: Vec<K>,
    vals: Vec<V>,
    children: Vec<Box<BTreeNodeV2<K, V>>>,
    is_leaf: bool,
}

impl<K: Ord + Clone, V: Clone> BTreeNodeV2<K, V> {
    fn new_leaf() -> Self {
        BTreeNodeV2 {
            keys: vec![],
            vals: vec![],
            children: vec![],
            is_leaf: true,
        }
    }

    fn new_internal() -> Self {
        BTreeNodeV2 {
            keys: vec![],
            vals: vec![],
            children: vec![],
            is_leaf: false,
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        let pos = self.keys.partition_point(|k| k < key);
        if pos < self.keys.len() && &self.keys[pos] == key {
            return Some(&self.vals[pos]);
        }
        if self.is_leaf {
            return None;
        }
        self.children[pos].get(key)
    }

    fn insert_non_full(&mut self, key: K, val: V) {
        let pos = self.keys.partition_point(|k| k < &key);
        if self.is_leaf {
            if pos < self.keys.len() && self.keys[pos] == key {
                self.vals[pos] = val;
            } else {
                self.keys.insert(pos, key);
                self.vals.insert(pos, val);
            }
        } else {
            let child = &mut self.children[pos];
            if child.keys.len() == MAX_KEYS {
                self.split_child(pos);
                let new_pos = if key > self.keys[pos] { pos + 1 } else { pos };
                self.children[new_pos].insert_non_full(key, val);
            } else {
                child.insert_non_full(key, val);
            }
        }
    }

    fn split_child(&mut self, i: usize) {
        let mid = MAX_KEYS / 2;
        let mut child = self.children.remove(i);
        let mut right = if child.is_leaf {
            BTreeNodeV2::new_leaf()
        } else {
            BTreeNodeV2::new_internal()
        };
        right.keys = child.keys.split_off(mid + 1);
        right.vals = child.vals.split_off(mid + 1);
        if !child.is_leaf {
            right.children = child.children.split_off(mid + 1);
        }
        let Some(mid_key) = child.keys.pop() else {
            return;
        };
        let Some(mid_val) = child.vals.pop() else {
            return;
        };
        self.keys.insert(i, mid_key);
        self.vals.insert(i, mid_val);
        self.children.insert(i, child);
        self.children.insert(i + 1, Box::new(right));
    }
}

/// Simple B-tree map (order 4).
pub struct BTreeMapV2<K: Ord + Clone, V: Clone> {
    root: Box<BTreeNodeV2<K, V>>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> BTreeMapV2<K, V> {
    /// Create a new empty B-tree map.
    pub fn new() -> Self {
        BTreeMapV2 {
            root: Box::new(BTreeNodeV2::new_leaf()),
            count: 0,
        }
    }

    /// Insert or update a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let exists = self.root.get(&key).is_some();
        if self.root.keys.len() == MAX_KEYS {
            let old_root = std::mem::replace(&mut self.root, Box::new(BTreeNodeV2::new_internal()));
            self.root.children.push(old_root);
            self.root.split_child(0);
        }
        self.root.insert_non_full(key, val);
        if !exists {
            self.count += 1;
        }
    }

    /// Look up a key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.root.get(key)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if a key is present.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}

impl<K: Ord + Clone, V: Clone> Default for BTreeMapV2<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        bt.insert(5, 50);
        assert_eq!(bt.get(&5), Some(&50) /* basic insert/get */);
    }

    #[test]
    fn test_get_missing() {
        let bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        assert_eq!(bt.get(&1), None /* key not present */);
    }

    #[test]
    fn test_update() {
        let mut bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        bt.insert(3, 30);
        bt.insert(3, 99);
        assert_eq!(bt.get(&3), Some(&99) /* updated value */);
    }

    #[test]
    fn test_len() {
        let mut bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        bt.insert(1, 1);
        bt.insert(2, 2);
        assert_eq!(bt.len(), 2 /* two distinct keys */);
    }

    #[test]
    fn test_is_empty() {
        let bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        assert!(bt.is_empty() /* fresh map is empty */);
    }

    #[test]
    fn test_many_inserts() {
        let mut bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        for i in 0u32..100 {
            bt.insert(i, i * 3);
        }
        for i in 0u32..100 {
            assert_eq!(bt.get(&i), Some(&(i * 3)) /* each value matches */);
        }
    }

    #[test]
    fn test_contains_key() {
        let mut bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        bt.insert(7, 70);
        assert!(bt.contains_key(&7) /* key 7 present */);
        assert!(!bt.contains_key(&8) /* key 8 absent */);
    }

    #[test]
    fn test_reverse_order_inserts() {
        let mut bt: BTreeMapV2<u32, u32> = BTreeMapV2::new();
        for i in (0u32..20).rev() {
            bt.insert(i, i);
        }
        assert_eq!(bt.len(), 20 /* 20 distinct keys */);
    }

    #[test]
    fn test_default() {
        let bt: BTreeMapV2<u32, u32> = BTreeMapV2::default();
        assert!(bt.is_empty() /* default is empty */);
    }
}
