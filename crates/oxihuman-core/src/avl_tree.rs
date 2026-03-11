// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! AVL tree (height-balanced BST) with insert and lookup.

use std::cmp::Ordering;

type Link<K, V> = Option<Box<AvlNode<K, V>>>;

#[derive(Debug, Clone)]
struct AvlNode<K: Ord + Clone, V: Clone> {
    key: K,
    value: V,
    height: i32,
    left: Link<K, V>,
    right: Link<K, V>,
}

impl<K: Ord + Clone, V: Clone> AvlNode<K, V> {
    fn new(key: K, value: V) -> Box<Self> {
        Box::new(Self {
            key,
            value,
            height: 1,
            left: None,
            right: None,
        })
    }
}

fn height<K: Ord + Clone, V: Clone>(node: &Link<K, V>) -> i32 {
    node.as_ref().map(|n| n.height).unwrap_or(0)
}

fn update_height<K: Ord + Clone, V: Clone>(node: &mut Box<AvlNode<K, V>>) {
    node.height = 1 + height(&node.left).max(height(&node.right));
}

fn balance_factor<K: Ord + Clone, V: Clone>(node: &AvlNode<K, V>) -> i32 {
    height(&node.left) - height(&node.right)
}

fn rotate_right<K: Ord + Clone, V: Clone>(mut y: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
    let Some(mut x) = y.left.take() else { return y };
    y.left = x.right.take();
    update_height(&mut y);
    x.right = Some(y);
    update_height(&mut x);
    x
}

fn rotate_left<K: Ord + Clone, V: Clone>(mut x: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
    let Some(mut y) = x.right.take() else {
        return x;
    };
    x.right = y.left.take();
    update_height(&mut x);
    y.left = Some(x);
    update_height(&mut y);
    y
}

fn balance<K: Ord + Clone, V: Clone>(mut node: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
    update_height(&mut node);
    let bf = balance_factor(&node);
    if bf > 1 {
        if node.left.as_ref().is_some_and(|l| balance_factor(l) < 0) {
            if let Some(left) = node.left.take() {
                node.left = Some(rotate_left(left));
            }
        }
        return rotate_right(node);
    }
    if bf < -1 {
        if node.right.as_ref().is_some_and(|r| balance_factor(r) > 0) {
            if let Some(right) = node.right.take() {
                node.right = Some(rotate_right(right));
            }
        }
        return rotate_left(node);
    }
    node
}

fn insert<K: Ord + Clone, V: Clone>(node: Link<K, V>, key: K, value: V) -> Box<AvlNode<K, V>> {
    match node {
        None => AvlNode::new(key, value),
        Some(mut n) => {
            match key.cmp(&n.key) {
                Ordering::Less => {
                    n.left = Some(insert(n.left.take(), key, value));
                }
                Ordering::Greater => {
                    n.right = Some(insert(n.right.take(), key, value));
                }
                Ordering::Equal => {
                    n.value = value;
                    return n;
                }
            }
            balance(n)
        }
    }
}

fn get<'a, K: Ord + Clone, V: Clone>(node: &'a Link<K, V>, key: &K) -> Option<&'a V> {
    let n = node.as_ref()?;
    match key.cmp(&n.key) {
        Ordering::Less => get(&n.left, key),
        Ordering::Greater => get(&n.right, key),
        Ordering::Equal => Some(&n.value),
    }
}

/// An AVL tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AvlTree<K: Ord + Clone, V: Clone> {
    root: Link<K, V>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> Default for AvlTree<K, V> {
    fn default() -> Self {
        Self {
            root: None,
            count: 0,
        }
    }
}

impl<K: Ord + Clone, V: Clone> AvlTree<K, V> {
    /// Create a new empty AVL tree.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: K, value: V) {
        let had = self.contains(&key);
        let root = self.root.take();
        self.root = Some(insert(root, key, value));
        if !had {
            self.count += 1;
        }
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: &K) -> Option<&V> {
        get(&self.root, key)
    }

    /// Check if a key exists.
    #[allow(dead_code)]
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Height of the tree.
    #[allow(dead_code)]
    pub fn height(&self) -> i32 {
        height(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        t.insert(5, 50);
        assert_eq!(t.get(&5), Some(&50));
    }

    #[test]
    fn get_missing_is_none() {
        let t: AvlTree<i32, i32> = AvlTree::new();
        assert!(t.get(&1).is_none());
    }

    #[test]
    fn contains_after_insert() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        t.insert(3, 0);
        assert!(t.contains(&3));
    }

    #[test]
    fn overwrite_keeps_count() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        t.insert(1, 10);
        t.insert(1, 20);
        assert_eq!(t.len(), 1);
        assert_eq!(t.get(&1), Some(&20));
    }

    #[test]
    fn len_correct() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        t.insert(1, 1);
        t.insert(2, 2);
        t.insert(3, 3);
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn is_empty_initially() {
        let t: AvlTree<i32, i32> = AvlTree::new();
        assert!(t.is_empty());
    }

    #[test]
    fn height_is_balanced() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        for i in 0..16i32 {
            t.insert(i, i);
        }
        assert!(t.height() <= 6);
    }

    #[test]
    fn string_keys_work() {
        let mut t: AvlTree<String, usize> = AvlTree::new();
        t.insert("hello".to_string(), 1);
        t.insert("world".to_string(), 2);
        assert_eq!(t.get(&"hello".to_string()), Some(&1));
    }

    #[test]
    fn insert_descending_stays_balanced() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        for i in (0..8i32).rev() {
            t.insert(i, i);
        }
        assert_eq!(t.len(), 8);
        assert!(t.height() <= 5);
    }

    #[test]
    fn is_empty_false_after_insert() {
        let mut t: AvlTree<i32, i32> = AvlTree::new();
        t.insert(1, 1);
        assert!(!t.is_empty());
    }
}
