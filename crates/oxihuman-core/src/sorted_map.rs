// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A sorted map using a balanced binary search tree (AVL-like) stored in a flat Vec.
//! Supports O(log n) insert, lookup, and ordered iteration.

/// A node-index in the flat storage.
#[allow(dead_code)]
const NIL: usize = usize::MAX;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct BstNode<K, V> {
    key: K,
    value: V,
    left: usize,
    right: usize,
    height: i32,
}

/// A sorted map backed by an AVL tree in flat storage.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SortedMap<K, V> {
    nodes: Vec<BstNode<K, V>>,
    root: usize,
}

#[allow(dead_code)]
impl<K: Ord + Clone, V: Clone> SortedMap<K, V> {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), root: NIL }
    }

    fn height(&self, idx: usize) -> i32 {
        if idx == NIL { 0 } else { self.nodes[idx].height }
    }

    fn update_height(&mut self, idx: usize) {
        let lh = self.height(self.nodes[idx].left);
        let rh = self.height(self.nodes[idx].right);
        self.nodes[idx].height = 1 + lh.max(rh);
    }

    fn balance_factor(&self, idx: usize) -> i32 {
        self.height(self.nodes[idx].left) - self.height(self.nodes[idx].right)
    }

    fn rotate_right(&mut self, y: usize) -> usize {
        let x = self.nodes[y].left;
        let t2 = self.nodes[x].right;
        self.nodes[x].right = y;
        self.nodes[y].left = t2;
        self.update_height(y);
        self.update_height(x);
        x
    }

    fn rotate_left(&mut self, x: usize) -> usize {
        let y = self.nodes[x].right;
        let t2 = self.nodes[y].left;
        self.nodes[y].left = x;
        self.nodes[x].right = t2;
        self.update_height(x);
        self.update_height(y);
        y
    }

    fn rebalance(&mut self, idx: usize) -> usize {
        self.update_height(idx);
        let bf = self.balance_factor(idx);
        if bf > 1 {
            if self.balance_factor(self.nodes[idx].left) < 0 {
                let l = self.nodes[idx].left;
                self.nodes[idx].left = self.rotate_left(l);
            }
            return self.rotate_right(idx);
        }
        if bf < -1 {
            if self.balance_factor(self.nodes[idx].right) > 0 {
                let r = self.nodes[idx].right;
                self.nodes[idx].right = self.rotate_right(r);
            }
            return self.rotate_left(idx);
        }
        idx
    }

    fn insert_at(&mut self, idx: usize, key: K, value: V) -> (usize, Option<V>) {
        if idx == NIL {
            let new_idx = self.nodes.len();
            self.nodes.push(BstNode { key, value, left: NIL, right: NIL, height: 1 });
            return (new_idx, None);
        }
        let old;
        if key < self.nodes[idx].key {
            let l = self.nodes[idx].left;
            let (nl, o) = self.insert_at(l, key, value);
            self.nodes[idx].left = nl;
            old = o;
        } else if key > self.nodes[idx].key {
            let r = self.nodes[idx].right;
            let (nr, o) = self.insert_at(r, key, value);
            self.nodes[idx].right = nr;
            old = o;
        } else {
            let prev = self.nodes[idx].value.clone();
            self.nodes[idx].value = value;
            return (idx, Some(prev));
        }
        let balanced = self.rebalance(idx);
        (balanced, old)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let r = self.root;
        let (new_root, old) = self.insert_at(r, key, value);
        self.root = new_root;
        old
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut idx = self.root;
        while idx != NIL {
            if *key < self.nodes[idx].key {
                idx = self.nodes[idx].left;
            } else if *key > self.nodes[idx].key {
                idx = self.nodes[idx].right;
            } else {
                return Some(&self.nodes[idx].value);
            }
        }
        None
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.count_nodes(self.root)
    }

    fn count_nodes(&self, idx: usize) -> usize {
        if idx == NIL { return 0; }
        1 + self.count_nodes(self.nodes[idx].left) + self.count_nodes(self.nodes[idx].right)
    }

    pub fn is_empty(&self) -> bool {
        self.root == NIL
    }

    /// In-order traversal returning sorted keys.
    pub fn keys_sorted(&self) -> Vec<K> {
        let mut result = Vec::new();
        self.inorder(self.root, &mut result);
        result
    }

    fn inorder(&self, idx: usize, out: &mut Vec<K>) {
        if idx == NIL { return; }
        self.inorder(self.nodes[idx].left, out);
        out.push(self.nodes[idx].key.clone());
        self.inorder(self.nodes[idx].right, out);
    }

    pub fn min_key(&self) -> Option<&K> {
        if self.root == NIL { return None; }
        let mut idx = self.root;
        while self.nodes[idx].left != NIL { idx = self.nodes[idx].left; }
        Some(&self.nodes[idx].key)
    }

    pub fn max_key(&self) -> Option<&K> {
        if self.root == NIL { return None; }
        let mut idx = self.root;
        while self.nodes[idx].right != NIL { idx = self.nodes[idx].right; }
        Some(&self.nodes[idx].key)
    }
}

impl<K: Ord + Clone, V: Clone> Default for SortedMap<K, V> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut m = SortedMap::new();
        m.insert(3, "c");
        m.insert(1, "a");
        m.insert(2, "b");
        assert_eq!(m.get(&1), Some(&"a"));
        assert_eq!(m.get(&2), Some(&"b"));
        assert_eq!(m.get(&3), Some(&"c"));
    }

    #[test]
    fn test_overwrite() {
        let mut m = SortedMap::new();
        m.insert(1, 10);
        assert_eq!(m.insert(1, 20), Some(10));
        assert_eq!(m.get(&1), Some(&20));
    }

    #[test]
    fn test_keys_sorted() {
        let mut m = SortedMap::new();
        m.insert(5, 'e');
        m.insert(3, 'c');
        m.insert(7, 'g');
        m.insert(1, 'a');
        assert_eq!(m.keys_sorted(), vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_min_max() {
        let mut m = SortedMap::new();
        m.insert(10, 0);
        m.insert(5, 0);
        m.insert(20, 0);
        assert_eq!(m.min_key(), Some(&5));
        assert_eq!(m.max_key(), Some(&20));
    }

    #[test]
    fn test_empty() {
        let m: SortedMap<i32, i32> = SortedMap::new();
        assert!(m.is_empty());
        assert_eq!(m.min_key(), None);
    }

    #[test]
    fn test_len() {
        let mut m = SortedMap::new();
        m.insert(1, 1);
        m.insert(2, 2);
        m.insert(3, 3);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_balance() {
        let mut m = SortedMap::new();
        // Insert in sorted order - AVL should still be balanced
        for i in 0..20 {
            m.insert(i, i);
        }
        assert_eq!(m.len(), 20);
        // Tree height should be reasonable for AVL
        let root_h = m.nodes[m.root].height;
        assert!(root_h <= 6); // log2(20) ≈ 4.3, AVL allows up to 1.44*log2
    }

    #[test]
    fn test_contains() {
        let mut m = SortedMap::new();
        m.insert(42, "x");
        assert!(m.contains_key(&42));
        assert!(!m.contains_key(&43));
    }

    #[test]
    fn test_missing() {
        let m: SortedMap<i32, i32> = SortedMap::new();
        assert_eq!(m.get(&999), None);
    }

    #[test]
    fn test_reverse_insert() {
        let mut m = SortedMap::new();
        for i in (0..10).rev() {
            m.insert(i, i);
        }
        assert_eq!(m.keys_sorted(), (0..10).collect::<Vec<_>>());
    }
}
