// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Leftist heap stub — min-heap with leftist property (rank of left >= right).

/// A node in the leftist heap.
#[derive(Debug, Clone)]
struct LeftistNode<K: Ord + Clone, V: Clone> {
    key: K,
    val: V,
    rank: usize,
    left: Option<Box<LeftistNode<K, V>>>,
    right: Option<Box<LeftistNode<K, V>>>,
}

impl<K: Ord + Clone, V: Clone> LeftistNode<K, V> {
    fn new(key: K, val: V) -> Box<Self> {
        Box::new(LeftistNode {
            key,
            val,
            rank: 1,
            left: None,
            right: None,
        })
    }

    fn rank(node: &Option<Box<LeftistNode<K, V>>>) -> usize {
        node.as_ref().map_or(0, |n| n.rank)
    }

    fn merge(
        a: Option<Box<LeftistNode<K, V>>>,
        b: Option<Box<LeftistNode<K, V>>>,
    ) -> Option<Box<LeftistNode<K, V>>> {
        match (a, b) {
            (None, x) | (x, None) => x,
            (Some(mut ha), Some(hb)) => {
                if ha.key > hb.key {
                    return Self::merge(Some(hb), Some(ha));
                }
                ha.right = Self::merge(ha.right.take(), Some(hb));
                if Self::rank(&ha.left) < Self::rank(&ha.right) {
                    std::mem::swap(&mut ha.left, &mut ha.right);
                }
                ha.rank = Self::rank(&ha.right) + 1;
                Some(ha)
            }
        }
    }
}

/// Leftist min-heap.
pub struct LeftistHeap<K: Ord + Clone, V: Clone> {
    root: Option<Box<LeftistNode<K, V>>>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> LeftistHeap<K, V> {
    /// Create a new empty leftist heap.
    pub fn new() -> Self {
        LeftistHeap {
            root: None,
            count: 0,
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let node = Some(LeftistNode::new(key, val));
        self.root = LeftistNode::merge(self.root.take(), node);
        self.count += 1;
    }

    /// Peek at the minimum.
    pub fn peek_min(&self) -> Option<(&K, &V)> {
        self.root.as_ref().map(|r| (&r.key, &r.val))
    }

    /// Extract the minimum element.
    pub fn extract_min(&mut self) -> Option<(K, V)> {
        let root = self.root.take()?;
        self.root = LeftistNode::merge(root.left, root.right);
        self.count = self.count.saturating_sub(1);
        Some((root.key, root.val))
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Merge another heap into this one (consuming it).
    pub fn merge_with(&mut self, other: LeftistHeap<K, V>) {
        self.count += other.count;
        self.root = LeftistNode::merge(self.root.take(), other.root);
    }

    /// Rank of the root (leftist property value).
    pub fn root_rank(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.rank)
    }
}

impl<K: Ord + Clone, V: Clone> Default for LeftistHeap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_peek() {
        let mut h: LeftistHeap<u32, u32> = LeftistHeap::new();
        h.insert(7, 70);
        h.insert(3, 30);
        let (k, _) = h.peek_min().unwrap();
        assert_eq!(*k, 3 /* minimum key */);
    }

    #[test]
    fn test_extract_min() {
        let mut h: LeftistHeap<u32, u32> = LeftistHeap::new();
        h.insert(9, 90);
        h.insert(4, 40);
        let (k, v) = h.extract_min().unwrap();
        assert_eq!(k, 4 /* min extracted */);
        assert_eq!(v, 40);
    }

    #[test]
    fn test_len() {
        let mut h: LeftistHeap<u32, u32> = LeftistHeap::new();
        h.insert(1, 1);
        h.insert(2, 2);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let h: LeftistHeap<u32, u32> = LeftistHeap::new();
        assert!(h.is_empty());
    }

    #[test]
    fn test_sorted_extraction() {
        let mut h: LeftistHeap<u32, u32> = LeftistHeap::new();
        for v in [5u32, 2, 8, 1, 4] {
            h.insert(v, v);
        }
        let mut prev = 0u32;
        while let Some((k, _)) = h.extract_min() {
            assert!(k >= prev /* non-decreasing order */);
            prev = k;
        }
    }

    #[test]
    fn test_merge_with() {
        let mut a: LeftistHeap<u32, u32> = LeftistHeap::new();
        let mut b: LeftistHeap<u32, u32> = LeftistHeap::new();
        a.insert(10, 10);
        b.insert(3, 3);
        a.merge_with(b);
        let (k, _) = a.peek_min().unwrap();
        assert_eq!(*k, 3 /* min from merged */);
    }

    #[test]
    fn test_empty_extract() {
        let mut h: LeftistHeap<u32, u32> = LeftistHeap::new();
        assert!(h.extract_min().is_none());
    }

    #[test]
    fn test_root_rank() {
        let mut h: LeftistHeap<u32, u32> = LeftistHeap::new();
        h.insert(1, 1);
        h.insert(2, 2);
        assert!(h.root_rank() >= 1 /* rank must be at least 1 */);
    }

    #[test]
    fn test_default() {
        let h: LeftistHeap<u32, u32> = LeftistHeap::default();
        assert!(h.is_empty());
    }
}
