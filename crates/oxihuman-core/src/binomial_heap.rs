// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Binomial heap stub — min-heap composed of binomial trees.

/// A binomial tree node.
#[derive(Debug, Clone)]
struct BinomialNode<K: Ord + Clone, V: Clone> {
    key: K,
    val: V,
    children: Vec<BinomialNode<K, V>>,
}

impl<K: Ord + Clone, V: Clone> BinomialNode<K, V> {
    fn new(key: K, val: V) -> Self {
        BinomialNode {
            key,
            val,
            children: vec![],
        }
    }

    fn degree(&self) -> usize {
        self.children.len()
    }

    /// Link two binomial trees of same degree: smaller key becomes root.
    fn link(mut a: BinomialNode<K, V>, b: BinomialNode<K, V>) -> BinomialNode<K, V> {
        if a.key <= b.key {
            a.children.push(b);
            a
        } else {
            let mut root = b;
            root.children.push(a);
            root
        }
    }
}

/// Merge two root lists of binomial trees.
#[allow(clippy::needless_range_loop)]
fn merge_lists<K: Ord + Clone, V: Clone>(
    a: Vec<BinomialNode<K, V>>,
    b: Vec<BinomialNode<K, V>>,
) -> Vec<BinomialNode<K, V>> {
    let mut result: Vec<Option<BinomialNode<K, V>>> = Vec::new();
    let max_len = a.len().max(b.len()) + 1;
    result.resize_with(max_len, || None);
    let mut carry: Option<BinomialNode<K, V>> = None;

    let get =
        |v: &[BinomialNode<K, V>], i: usize| -> Option<BinomialNode<K, V>> { v.get(i).cloned() };

    for i in 0..max_len {
        let ai = get(&a, i);
        let bi = get(&b, i);
        let (res, new_carry) = match (ai, bi, carry.take()) {
            (None, None, c) => (c, None),
            (Some(x), None, None) | (None, Some(x), None) => (Some(x), None),
            (Some(x), Some(y), None) => (None, Some(BinomialNode::link(x, y))),
            (Some(x), None, Some(c)) | (None, Some(x), Some(c)) => {
                (None, Some(BinomialNode::link(x, c)))
            }
            (Some(x), Some(y), Some(c)) => (Some(c), Some(BinomialNode::link(x, y))),
        };
        result[i] = res;
        carry = new_carry;
    }
    result.into_iter().flatten().collect()
}

/// Binomial min-heap.
pub struct BinomialHeap<K: Ord + Clone, V: Clone> {
    trees: Vec<BinomialNode<K, V>>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> BinomialHeap<K, V> {
    /// Create a new empty binomial heap.
    pub fn new() -> Self {
        BinomialHeap {
            trees: vec![],
            count: 0,
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let node = BinomialNode::new(key, val);
        let single = vec![node];
        self.trees = merge_lists(std::mem::take(&mut self.trees), single);
        self.count += 1;
    }

    /// Peek at the minimum.
    pub fn peek_min(&self) -> Option<(&K, &V)> {
        self.trees
            .iter()
            .min_by(|a, b| a.key.cmp(&b.key))
            .map(|n| (&n.key, &n.val))
    }

    /// Extract the minimum element.
    pub fn extract_min(&mut self) -> Option<(K, V)> {
        if self.trees.is_empty() {
            return None;
        }
        let min_i = self
            .trees
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.key.cmp(&b.key))
            .map(|(i, _)| i)?;
        let min_node = self.trees.remove(min_i);
        let children = min_node.children.clone();
        self.trees = merge_lists(std::mem::take(&mut self.trees), children);
        self.count = self.count.saturating_sub(1);
        Some((min_node.key, min_node.val))
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Merge another heap into this one (consuming it).
    pub fn merge_with(&mut self, other: BinomialHeap<K, V>) {
        self.count += other.count;
        self.trees = merge_lists(std::mem::take(&mut self.trees), other.trees);
    }
}

impl<K: Ord + Clone, V: Clone> Default for BinomialHeap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_peek() {
        let mut h: BinomialHeap<u32, u32> = BinomialHeap::new();
        h.insert(5, 50);
        h.insert(2, 20);
        let (k, _) = h.peek_min().expect("should succeed");
        assert_eq!(*k, 2 /* min key */);
    }

    #[test]
    fn test_extract_min() {
        let mut h: BinomialHeap<u32, u32> = BinomialHeap::new();
        h.insert(8, 80);
        h.insert(3, 30);
        let (k, v) = h.extract_min().expect("should succeed");
        assert_eq!(k, 3 /* min extracted */);
        assert_eq!(v, 30);
    }

    #[test]
    fn test_len() {
        let mut h: BinomialHeap<u32, u32> = BinomialHeap::new();
        h.insert(1, 1);
        h.insert(2, 2);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let h: BinomialHeap<u32, u32> = BinomialHeap::new();
        assert!(h.is_empty());
    }

    #[test]
    fn test_sorted_extraction() {
        let mut h: BinomialHeap<u32, u32> = BinomialHeap::new();
        for v in [7u32, 3, 9, 1, 4] {
            h.insert(v, v);
        }
        let mut prev = 0u32;
        while let Some((k, _)) = h.extract_min() {
            assert!(k >= prev /* non-decreasing */);
            prev = k;
        }
    }

    #[test]
    fn test_merge_with() {
        let mut a: BinomialHeap<u32, u32> = BinomialHeap::new();
        let mut b: BinomialHeap<u32, u32> = BinomialHeap::new();
        a.insert(10, 10);
        b.insert(4, 4);
        a.merge_with(b);
        let (k, _) = a.peek_min().expect("should succeed");
        assert_eq!(*k, 4 /* merged min */);
    }

    #[test]
    fn test_empty_extract() {
        let mut h: BinomialHeap<u32, u32> = BinomialHeap::new();
        assert!(h.extract_min().is_none());
    }

    #[test]
    fn test_default() {
        let h: BinomialHeap<u32, u32> = BinomialHeap::default();
        assert!(h.is_empty());
    }

    #[test]
    fn test_many_inserts() {
        let mut h: BinomialHeap<u32, u32> = BinomialHeap::new();
        for i in (0u32..32).rev() {
            h.insert(i, i);
        }
        assert_eq!(
            h.extract_min().expect("should succeed").0,
            0 /* global minimum */
        );
    }
}
