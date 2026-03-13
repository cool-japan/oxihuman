// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Red-black tree map stub — ordered map with O(log n) operations.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Color {
    Red,
    Black,
}

/// A node in the red-black tree.
#[derive(Debug)]
struct RBNode<K, V> {
    key: K,
    val: V,
    color: Color,
    left: Option<Box<RBNode<K, V>>>,
    right: Option<Box<RBNode<K, V>>>,
}

impl<K: Ord, V> RBNode<K, V> {
    fn new_red(key: K, val: V) -> Box<Self> {
        Box::new(RBNode {
            key,
            val,
            color: Color::Red,
            left: None,
            right: None,
        })
    }

    fn is_red(node: &Option<Box<RBNode<K, V>>>) -> bool {
        node.as_ref().is_some_and(|n| n.color == Color::Red)
    }

    fn get(&self, key: &K) -> Option<&V> {
        match key.cmp(&self.key) {
            std::cmp::Ordering::Equal => Some(&self.val),
            std::cmp::Ordering::Less => self.left.as_ref()?.get(key),
            std::cmp::Ordering::Greater => self.right.as_ref()?.get(key),
        }
    }

    fn rotate_left(mut h: Box<RBNode<K, V>>) -> Box<RBNode<K, V>> {
        let Some(mut x) = h.right.take() else {
            return h;
        };
        h.right = x.left.take();
        x.color = h.color;
        h.color = Color::Red;
        x.left = Some(h);
        x
    }

    fn rotate_right(mut h: Box<RBNode<K, V>>) -> Box<RBNode<K, V>> {
        let Some(mut x) = h.left.take() else { return h };
        h.left = x.right.take();
        x.color = h.color;
        h.color = Color::Red;
        x.right = Some(h);
        x
    }

    fn flip_colors(h: &mut Box<RBNode<K, V>>) {
        h.color = Color::Red;
        if let Some(l) = &mut h.left {
            l.color = Color::Black;
        }
        if let Some(r) = &mut h.right {
            r.color = Color::Black;
        }
    }

    fn insert(node: Option<Box<RBNode<K, V>>>, key: K, val: V) -> (Box<RBNode<K, V>>, bool) {
        let Some(mut h) = node else {
            return (RBNode::new_red(key, val), true);
        };
        let mut new_node = false;
        match key.cmp(&h.key) {
            std::cmp::Ordering::Equal => {
                h.val = val;
            }
            std::cmp::Ordering::Less => {
                let (n, added) = Self::insert(h.left.take(), key, val);
                h.left = Some(n);
                new_node = added;
            }
            std::cmp::Ordering::Greater => {
                let (n, added) = Self::insert(h.right.take(), key, val);
                h.right = Some(n);
                new_node = added;
            }
        }
        if Self::is_red(&h.right) && !Self::is_red(&h.left) {
            h = Self::rotate_left(h);
        }
        if Self::is_red(&h.left) && h.left.as_ref().is_some_and(|l| Self::is_red(&l.left)) {
            h = Self::rotate_right(h);
        }
        if Self::is_red(&h.left) && Self::is_red(&h.right) {
            Self::flip_colors(&mut h);
        }
        (h, new_node)
    }
}

/// Ordered map backed by a left-leaning red-black tree.
pub struct RedBlackMap<K: Ord, V> {
    root: Option<Box<RBNode<K, V>>>,
    count: usize,
}

impl<K: Ord, V> RedBlackMap<K, V> {
    /// Create a new empty red-black map.
    pub fn new() -> Self {
        RedBlackMap {
            root: None,
            count: 0,
        }
    }

    /// Insert or update a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let (mut node, added) = RBNode::insert(self.root.take(), key, val);
        node.color = Color::Black;
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

impl<K: Ord, V> Default for RedBlackMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        rb.insert(10, 100);
        assert_eq!(rb.get(&10), Some(&100) /* basic insert/get */);
    }

    #[test]
    fn test_get_missing() {
        let rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        assert_eq!(rb.get(&5), None /* key absent */);
    }

    #[test]
    fn test_update() {
        let mut rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        rb.insert(1, 10);
        rb.insert(1, 20);
        assert_eq!(rb.get(&1), Some(&20) /* updated value */);
    }

    #[test]
    fn test_len() {
        let mut rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        rb.insert(1, 1);
        rb.insert(2, 2);
        assert_eq!(rb.len(), 2 /* two keys */);
    }

    #[test]
    fn test_is_empty() {
        let rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        assert!(rb.is_empty() /* empty on creation */);
    }

    #[test]
    fn test_many_inserts() {
        let mut rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        for i in 0u32..50 {
            rb.insert(i, i * 2);
        }
        for i in 0u32..50 {
            assert_eq!(rb.get(&i), Some(&(i * 2)) /* each value found */);
        }
    }

    #[test]
    fn test_contains_key() {
        let mut rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        rb.insert(7, 70);
        assert!(rb.contains_key(&7) /* key present */);
        assert!(!rb.contains_key(&8) /* key absent */);
    }

    #[test]
    fn test_reverse_inserts() {
        let mut rb: RedBlackMap<u32, u32> = RedBlackMap::new();
        for i in (0u32..30).rev() {
            rb.insert(i, i);
        }
        assert_eq!(rb.len(), 30 /* 30 keys */);
    }

    #[test]
    fn test_default() {
        let rb: RedBlackMap<u32, u32> = RedBlackMap::default();
        assert!(rb.is_empty() /* default is empty */);
    }
}
