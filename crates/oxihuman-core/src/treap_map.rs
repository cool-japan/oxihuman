// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Treap map stub — randomized BST (BST + heap on random priorities).

/// A node in the treap.
#[derive(Debug)]
struct TreapNode<K, V> {
    key: K,
    val: V,
    priority: u64,
    left: Option<Box<TreapNode<K, V>>>,
    right: Option<Box<TreapNode<K, V>>>,
}

impl<K: Ord, V> TreapNode<K, V> {
    fn new(key: K, val: V, priority: u64) -> Box<Self> {
        Box::new(TreapNode {
            key,
            val,
            priority,
            left: None,
            right: None,
        })
    }

    fn rotate_right(mut y: Box<TreapNode<K, V>>) -> Box<TreapNode<K, V>> {
        let Some(mut x) = y.left.take() else { return y };
        y.left = x.right.take();
        x.right = Some(y);
        x
    }

    fn rotate_left(mut x: Box<TreapNode<K, V>>) -> Box<TreapNode<K, V>> {
        let Some(mut y) = x.right.take() else {
            return x;
        };
        x.right = y.left.take();
        y.left = Some(x);
        y
    }

    fn insert(
        node: Option<Box<TreapNode<K, V>>>,
        key: K,
        val: V,
        prio: u64,
    ) -> (Box<TreapNode<K, V>>, bool) {
        let Some(mut n) = node else {
            return (Self::new(key, val, prio), true);
        };
        let added;
        match key.cmp(&n.key) {
            std::cmp::Ordering::Equal => {
                n.val = val;
                return (n, false);
            }
            std::cmp::Ordering::Less => {
                let (l, a) = Self::insert(n.left.take(), key, val, prio);
                n.left = Some(l);
                added = a;
                if n.left.as_ref().is_some_and(|l| l.priority > n.priority) {
                    n = Self::rotate_right(n);
                }
            }
            std::cmp::Ordering::Greater => {
                let (r, a) = Self::insert(n.right.take(), key, val, prio);
                n.right = Some(r);
                added = a;
                if n.right.as_ref().is_some_and(|r| r.priority > n.priority) {
                    n = Self::rotate_left(n);
                }
            }
        }
        (n, added)
    }

    fn get(&self, key: &K) -> Option<&V> {
        match key.cmp(&self.key) {
            std::cmp::Ordering::Equal => Some(&self.val),
            std::cmp::Ordering::Less => self.left.as_ref()?.get(key),
            std::cmp::Ordering::Greater => self.right.as_ref()?.get(key),
        }
    }
}

/// Ordered map backed by a treap.
pub struct TreapMap<K: Ord, V> {
    root: Option<Box<TreapNode<K, V>>>,
    count: usize,
    rng: u64,
}

impl<K: Ord, V> TreapMap<K, V> {
    /// Create a new empty treap map.
    pub fn new() -> Self {
        TreapMap {
            root: None,
            count: 0,
            rng: 0x853c_49e6_748f_ea9b,
        }
    }

    fn next_prio(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    /// Insert or update a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let prio = self.next_prio();
        let (node, added) = TreapNode::insert(self.root.take(), key, val, prio);
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
}

impl<K: Ord, V> Default for TreapMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut t: TreapMap<u32, u32> = TreapMap::new();
        t.insert(5, 50);
        assert_eq!(t.get(&5), Some(&50) /* basic get */);
    }

    #[test]
    fn test_missing_key() {
        let t: TreapMap<u32, u32> = TreapMap::new();
        assert_eq!(t.get(&1), None /* key absent */);
    }

    #[test]
    fn test_update() {
        let mut t: TreapMap<u32, u32> = TreapMap::new();
        t.insert(2, 20);
        t.insert(2, 99);
        assert_eq!(t.get(&2), Some(&99) /* updated value */);
    }

    #[test]
    fn test_len() {
        let mut t: TreapMap<u32, u32> = TreapMap::new();
        t.insert(1, 1);
        t.insert(2, 2);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let t: TreapMap<u32, u32> = TreapMap::new();
        assert!(t.is_empty());
    }

    #[test]
    fn test_many_inserts() {
        let mut t: TreapMap<u32, u32> = TreapMap::new();
        for i in 0u32..50 {
            t.insert(i, i * 3);
        }
        for i in 0u32..50 {
            assert_eq!(t.get(&i), Some(&(i * 3)));
        }
    }

    #[test]
    fn test_contains_key() {
        let mut t: TreapMap<u32, u32> = TreapMap::new();
        t.insert(9, 90);
        assert!(t.contains_key(&9));
        assert!(!t.contains_key(&10));
    }

    #[test]
    fn test_reverse_inserts() {
        let mut t: TreapMap<u32, u32> = TreapMap::new();
        for i in (0u32..30).rev() {
            t.insert(i, i);
        }
        assert_eq!(t.len(), 30);
    }

    #[test]
    fn test_default() {
        let t: TreapMap<u32, u32> = TreapMap::default();
        assert!(t.is_empty());
    }
}
