// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! AVL tree map stub — self-balancing BST with height-balance invariant.

/// A node in the AVL tree.
#[derive(Debug)]
struct AvlNode<K, V> {
    key: K,
    val: V,
    height: i32,
    left: Option<Box<AvlNode<K, V>>>,
    right: Option<Box<AvlNode<K, V>>>,
}

impl<K: Ord, V> AvlNode<K, V> {
    fn new(key: K, val: V) -> Box<Self> {
        Box::new(AvlNode {
            key,
            val,
            height: 1,
            left: None,
            right: None,
        })
    }

    fn height(n: &Option<Box<AvlNode<K, V>>>) -> i32 {
        n.as_ref().map_or(0, |x| x.height)
    }

    fn update_height(n: &mut Box<AvlNode<K, V>>) {
        n.height = 1 + Self::height(&n.left).max(Self::height(&n.right));
    }

    fn balance_factor(n: &AvlNode<K, V>) -> i32 {
        Self::height(&n.left) - Self::height(&n.right)
    }

    fn rotate_right(mut y: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
        let Some(mut x) = y.left.take() else { return y };
        y.left = x.right.take();
        Self::update_height(&mut y);
        x.right = Some(y);
        Self::update_height(&mut x);
        x
    }

    fn rotate_left(mut x: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
        let Some(mut y) = x.right.take() else {
            return x;
        };
        x.right = y.left.take();
        Self::update_height(&mut x);
        y.left = Some(x);
        Self::update_height(&mut y);
        y
    }

    fn rebalance(mut n: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
        Self::update_height(&mut n);
        let bf = Self::balance_factor(&n);
        if bf > 1 {
            if n.left.as_ref().is_some_and(|l| Self::balance_factor(l) < 0) {
                if let Some(left) = n.left.take() {
                    n.left = Some(Self::rotate_left(left));
                }
            }
            return Self::rotate_right(n);
        }
        if bf < -1 {
            if n.right
                .as_ref()
                .is_some_and(|r| Self::balance_factor(r) > 0)
            {
                if let Some(right) = n.right.take() {
                    n.right = Some(Self::rotate_right(right));
                }
            }
            return Self::rotate_left(n);
        }
        n
    }

    fn insert(node: Option<Box<AvlNode<K, V>>>, key: K, val: V) -> (Box<AvlNode<K, V>>, bool) {
        let Some(mut n) = node else {
            return (Self::new(key, val), true);
        };
        let added = match key.cmp(&n.key) {
            std::cmp::Ordering::Equal => {
                n.val = val;
                false
            }
            std::cmp::Ordering::Less => {
                let (l, a) = Self::insert(n.left.take(), key, val);
                n.left = Some(l);
                a
            }
            std::cmp::Ordering::Greater => {
                let (r, a) = Self::insert(n.right.take(), key, val);
                n.right = Some(r);
                a
            }
        };
        (Self::rebalance(n), added)
    }

    fn get(&self, key: &K) -> Option<&V> {
        match key.cmp(&self.key) {
            std::cmp::Ordering::Equal => Some(&self.val),
            std::cmp::Ordering::Less => self.left.as_ref()?.get(key),
            std::cmp::Ordering::Greater => self.right.as_ref()?.get(key),
        }
    }
}

/// Ordered map backed by an AVL tree.
pub struct AvlMap<K: Ord, V> {
    root: Option<Box<AvlNode<K, V>>>,
    count: usize,
}

impl<K: Ord, V> AvlMap<K, V> {
    /// Create a new empty AVL map.
    pub fn new() -> Self {
        AvlMap {
            root: None,
            count: 0,
        }
    }

    /// Insert or update a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let (node, added) = AvlNode::insert(self.root.take(), key, val);
        self.root = Some(node);
        if added {
            self.count += 1;
        }
    }

    /// Look up a key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.root.as_ref()?.get(key)
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

    /// Height of the tree (for testing balance).
    pub fn height(&self) -> i32 {
        AvlNode::height(&self.root)
    }
}

impl<K: Ord, V> Default for AvlMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut m: AvlMap<u32, u32> = AvlMap::new();
        m.insert(5, 50);
        assert_eq!(m.get(&5), Some(&50) /* basic insert/get */);
    }

    #[test]
    fn test_get_missing() {
        let m: AvlMap<u32, u32> = AvlMap::new();
        assert_eq!(m.get(&3), None /* key absent */);
    }

    #[test]
    fn test_update() {
        let mut m: AvlMap<u32, u32> = AvlMap::new();
        m.insert(1, 10);
        m.insert(1, 99);
        assert_eq!(m.get(&1), Some(&99) /* updated value */);
    }

    #[test]
    fn test_len() {
        let mut m: AvlMap<u32, u32> = AvlMap::new();
        m.insert(1, 1);
        m.insert(2, 2);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let m: AvlMap<u32, u32> = AvlMap::new();
        assert!(m.is_empty());
    }

    #[test]
    fn test_balance_maintained() {
        let mut m: AvlMap<u32, u32> = AvlMap::new();
        for i in 0u32..64 {
            m.insert(i, i);
        }
        /* AVL height must be O(log n); for 64 nodes height <= 12 */
        assert!(m.height() <= 12);
    }

    #[test]
    fn test_many_inserts() {
        let mut m: AvlMap<u32, u32> = AvlMap::new();
        for i in 0u32..100 {
            m.insert(i, i * 5);
        }
        for i in 0u32..100 {
            assert_eq!(m.get(&i), Some(&(i * 5)));
        }
    }

    #[test]
    fn test_contains_key() {
        let mut m: AvlMap<u32, u32> = AvlMap::new();
        m.insert(9, 90);
        assert!(m.contains_key(&9));
        assert!(!m.contains_key(&10));
    }

    #[test]
    fn test_default() {
        let m: AvlMap<u32, u32> = AvlMap::default();
        assert!(m.is_empty());
    }
}
