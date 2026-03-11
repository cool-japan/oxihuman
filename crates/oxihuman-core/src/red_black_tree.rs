// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Red-black tree stub (color invariant maintained via basic insertion).

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

type RbLink<K, V> = Option<Box<RbNode<K, V>>>;

#[derive(Debug, Clone)]
struct RbNode<K: Ord + Clone, V: Clone> {
    key: K,
    value: V,
    color: Color,
    left: RbLink<K, V>,
    right: RbLink<K, V>,
}

impl<K: Ord + Clone, V: Clone> RbNode<K, V> {
    fn new_red(key: K, value: V) -> Box<Self> {
        Box::new(Self {
            key,
            value,
            color: Color::Red,
            left: None,
            right: None,
        })
    }
}

fn is_red<K: Ord + Clone, V: Clone>(node: &RbLink<K, V>) -> bool {
    node.as_ref().is_some_and(|n| n.color == Color::Red)
}

fn rotate_left<K: Ord + Clone, V: Clone>(mut h: Box<RbNode<K, V>>) -> Box<RbNode<K, V>> {
    let Some(mut x) = h.right.take() else {
        return h;
    };
    h.right = x.left.take();
    x.color = h.color;
    h.color = Color::Red;
    x.left = Some(h);
    x
}

fn rotate_right<K: Ord + Clone, V: Clone>(mut h: Box<RbNode<K, V>>) -> Box<RbNode<K, V>> {
    let Some(mut x) = h.left.take() else { return h };
    h.left = x.right.take();
    x.color = h.color;
    h.color = Color::Red;
    x.right = Some(h);
    x
}

fn flip_colors<K: Ord + Clone, V: Clone>(h: &mut Box<RbNode<K, V>>) {
    h.color = Color::Red;
    if let Some(l) = h.left.as_mut() {
        l.color = Color::Black;
    }
    if let Some(r) = h.right.as_mut() {
        r.color = Color::Black;
    }
}

fn insert_rb<K: Ord + Clone, V: Clone>(node: RbLink<K, V>, key: K, value: V) -> Box<RbNode<K, V>> {
    let mut h = match node {
        None => return RbNode::new_red(key, value),
        Some(n) => n,
    };
    match key.cmp(&h.key) {
        Ordering::Less => h.left = Some(insert_rb(h.left.take(), key, value)),
        Ordering::Greater => h.right = Some(insert_rb(h.right.take(), key, value)),
        Ordering::Equal => {
            h.value = value;
            return h;
        }
    }
    // Fix-up
    if is_red(&h.right) && !is_red(&h.left) {
        h = rotate_left(h);
    }
    if is_red(&h.left) && h.left.as_ref().is_some_and(|l| is_red(&l.left)) {
        h = rotate_right(h);
    }
    if is_red(&h.left) && is_red(&h.right) {
        flip_colors(&mut h);
    }
    h
}

fn get_rb<'a, K: Ord + Clone, V: Clone>(node: &'a RbLink<K, V>, key: &K) -> Option<&'a V> {
    let n = node.as_ref()?;
    match key.cmp(&n.key) {
        Ordering::Less => get_rb(&n.left, key),
        Ordering::Greater => get_rb(&n.right, key),
        Ordering::Equal => Some(&n.value),
    }
}

fn tree_height<K: Ord + Clone, V: Clone>(node: &RbLink<K, V>) -> usize {
    match node {
        None => 0,
        Some(n) => 1 + tree_height(&n.left).max(tree_height(&n.right)),
    }
}

/// A red-black tree (left-leaning LLRB variant).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RedBlackTree<K: Ord + Clone, V: Clone> {
    root: RbLink<K, V>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> Default for RedBlackTree<K, V> {
    fn default() -> Self {
        Self {
            root: None,
            count: 0,
        }
    }
}

impl<K: Ord + Clone, V: Clone> RedBlackTree<K, V> {
    /// Create a new empty red-black tree.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: K, value: V) {
        let had = self.contains(&key);
        let root = self.root.take();
        let mut new_root = insert_rb(root, key, value);
        new_root.color = Color::Black;
        self.root = Some(new_root);
        if !had {
            self.count += 1;
        }
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: &K) -> Option<&V> {
        get_rb(&self.root, key)
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
    pub fn height(&self) -> usize {
        tree_height(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        t.insert(5, 50);
        assert_eq!(t.get(&5), Some(&50));
    }

    #[test]
    fn get_missing_is_none() {
        let t: RedBlackTree<i32, i32> = RedBlackTree::new();
        assert!(t.get(&1).is_none());
    }

    #[test]
    fn contains_after_insert() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        t.insert(3, 0);
        assert!(t.contains(&3));
    }

    #[test]
    fn overwrite_same_key() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        t.insert(1, 10);
        t.insert(1, 20);
        assert_eq!(t.get(&1), Some(&20));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn len_correct() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        for i in 0..5i32 {
            t.insert(i, i);
        }
        assert_eq!(t.len(), 5);
    }

    #[test]
    fn is_empty_initially() {
        let t: RedBlackTree<i32, i32> = RedBlackTree::new();
        assert!(t.is_empty());
    }

    #[test]
    fn height_is_bounded() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        for i in 0..32i32 {
            t.insert(i, i);
        }
        assert!(t.height() <= 12);
    }

    #[test]
    fn root_is_black_after_insert() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        t.insert(1, 1);
        assert_eq!(t.root.as_ref().unwrap().color, Color::Black);
    }

    #[test]
    fn insert_string_keys() {
        let mut t: RedBlackTree<String, usize> = RedBlackTree::new();
        t.insert("alpha".to_string(), 1);
        t.insert("beta".to_string(), 2);
        assert_eq!(t.get(&"alpha".to_string()), Some(&1));
    }

    #[test]
    fn is_empty_false_after_insert() {
        let mut t: RedBlackTree<i32, i32> = RedBlackTree::new();
        t.insert(42, 42);
        assert!(!t.is_empty());
    }
}
