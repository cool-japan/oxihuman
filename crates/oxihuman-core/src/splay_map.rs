// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Splay tree map stub — self-adjusting BST that splays accessed nodes to root.

/// A node in the splay tree.
#[derive(Debug)]
struct SplayNode<K, V> {
    key: K,
    val: V,
    left: Option<Box<SplayNode<K, V>>>,
    right: Option<Box<SplayNode<K, V>>>,
}

impl<K: Ord, V> SplayNode<K, V> {
    fn new(key: K, val: V) -> Box<Self> {
        Box::new(SplayNode {
            key,
            val,
            left: None,
            right: None,
        })
    }
}

fn rotate_right<K: Ord, V>(mut y: Box<SplayNode<K, V>>) -> Box<SplayNode<K, V>> {
    let Some(mut x) = y.left.take() else { return y };
    y.left = x.right.take();
    x.right = Some(y);
    x
}

fn rotate_left<K: Ord, V>(mut x: Box<SplayNode<K, V>>) -> Box<SplayNode<K, V>> {
    let Some(mut y) = x.right.take() else {
        return x;
    };
    x.right = y.left.take();
    y.left = Some(x);
    y
}

/// Splay the node with `key` to the root.
fn splay<K: Ord, V>(node: Option<Box<SplayNode<K, V>>>, key: &K) -> Option<Box<SplayNode<K, V>>> {
    let mut root = node?;
    match key.cmp(&root.key) {
        std::cmp::Ordering::Equal => Some(root),
        std::cmp::Ordering::Less => {
            let Some(left) = root.left.as_mut() else {
                return Some(root);
            };
            if key < &left.key {
                let sub = left.left.take();
                left.left = splay(sub, key);
                root = rotate_right(root);
            } else if key > &left.key {
                let sub = left.right.take();
                left.right = splay(sub, key);
                if root.left.as_ref().is_some_and(|l| l.right.is_some()) {
                    if let Some(l) = root.left.take() {
                        root.left = Some(rotate_left(l));
                    }
                }
            }
            if root.left.is_some() {
                rotate_right(root)
            } else {
                root
            }
            .into()
        }
        std::cmp::Ordering::Greater => {
            let Some(right) = root.right.as_mut() else {
                return Some(root);
            };
            if key > &right.key {
                let sub = right.right.take();
                right.right = splay(sub, key);
                root = rotate_left(root);
            } else if key < &right.key {
                let sub = right.left.take();
                right.left = splay(sub, key);
                if root.right.as_ref().is_some_and(|r| r.left.is_some()) {
                    if let Some(r) = root.right.take() {
                        root.right = Some(rotate_right(r));
                    }
                }
            }
            if root.right.is_some() {
                rotate_left(root)
            } else {
                root
            }
            .into()
        }
    }
}

/// Splay tree map.
pub struct SplayMap<K: Ord, V> {
    root: Option<Box<SplayNode<K, V>>>,
    count: usize,
}

impl<K: Ord, V> SplayMap<K, V> {
    /// Create a new empty splay map.
    pub fn new() -> Self {
        SplayMap {
            root: None,
            count: 0,
        }
    }

    /// Insert or update a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        self.root = splay(self.root.take(), &key);
        match &self.root {
            Some(r) if r.key == key => {
                if let Some(r) = self.root.as_mut() {
                    r.val = val;
                }
            }
            _ => {
                let mut new_node = SplayNode::new(key, val);
                match &self.root {
                    None => {}
                    Some(r) if new_node.key < r.key => {
                        new_node.right = self.root.take();
                        if let Some(right) = new_node.right.as_mut() {
                            new_node.left = right.left.take();
                        }
                    }
                    _ => {
                        new_node.left = self.root.take();
                        if let Some(left) = new_node.left.as_mut() {
                            new_node.right = left.right.take();
                        }
                    }
                }
                self.root = Some(new_node);
                self.count += 1;
            }
        }
    }

    /// Look up a key.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.root = splay(self.root.take(), key);
        self.root.as_ref().filter(|r| &r.key == key).map(|r| &r.val)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if key is present.
    pub fn contains_key(&mut self, key: &K) -> bool {
        self.get(key).is_some()
    }
}

impl<K: Ord, V> Default for SplayMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        m.insert(10, 100);
        assert_eq!(m.get(&10), Some(&100) /* basic get after insert */);
    }

    #[test]
    fn test_get_missing() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        assert_eq!(m.get(&5), None /* key absent */);
    }

    #[test]
    fn test_update() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        m.insert(3, 30);
        m.insert(3, 99);
        assert_eq!(m.get(&3), Some(&99) /* updated value */);
    }

    #[test]
    fn test_len() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        m.insert(1, 1);
        m.insert(2, 2);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let m: SplayMap<u32, u32> = SplayMap::new();
        assert!(m.is_empty());
    }

    #[test]
    fn test_multiple_inserts() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        for i in 0u32..20 {
            m.insert(i, i * 10);
        }
        for i in 0u32..20 {
            assert_eq!(m.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_contains_key() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        m.insert(7, 70);
        assert!(m.contains_key(&7));
        assert!(!m.contains_key(&99));
    }

    #[test]
    fn test_default() {
        let m: SplayMap<u32, u32> = SplayMap::default();
        assert!(m.is_empty());
    }

    #[test]
    fn test_sequential_access_splays_to_root() {
        let mut m: SplayMap<u32, u32> = SplayMap::new();
        for i in 0u32..10 {
            m.insert(i, i);
        }
        /* accessing 5 should splay it to root */
        m.get(&5);
        assert_eq!(m.root.as_ref().map(|r| r.key), Some(5));
    }
}
