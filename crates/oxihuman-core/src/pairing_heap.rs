// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pairing heap stub — min-heap with simple merge-based structure.

/// A node in the pairing heap.
#[derive(Debug, Clone)]
struct PairingNode<K: Ord + Clone, V: Clone> {
    key: K,
    val: V,
    children: Vec<PairingNode<K, V>>,
}

impl<K: Ord + Clone, V: Clone> PairingNode<K, V> {
    fn new(key: K, val: V) -> Self {
        PairingNode {
            key,
            val,
            children: vec![],
        }
    }

    /// Merge two heaps — smaller key becomes root.
    fn merge(self, other: PairingNode<K, V>) -> PairingNode<K, V> {
        if self.key <= other.key {
            let mut root = self;
            root.children.push(other);
            root
        } else {
            let mut root = other;
            root.children.push(self);
            root
        }
    }

    /// Two-pass pairing to re-merge children after extraction.
    fn two_pass_merge(mut children: Vec<PairingNode<K, V>>) -> Option<PairingNode<K, V>> {
        if children.is_empty() {
            return None;
        }
        /* first pass: pair up adjacent children */
        let mut pairs: Vec<PairingNode<K, V>> = Vec::new();
        while children.len() >= 2 {
            let b = children.remove(0);
            let a = children.remove(0);
            pairs.push(a.merge(b));
        }
        if let Some(rem) = children.into_iter().next() {
            pairs.push(rem);
        }
        /* second pass: fold from right */
        pairs.into_iter().reduce(|acc, p| p.merge(acc))
    }
}

/// Pairing min-heap.
pub struct PairingHeap<K: Ord + Clone, V: Clone> {
    root: Option<PairingNode<K, V>>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> PairingHeap<K, V> {
    /// Create a new empty pairing heap.
    pub fn new() -> Self {
        PairingHeap {
            root: None,
            count: 0,
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let node = PairingNode::new(key, val);
        self.root = match self.root.take() {
            None => Some(node),
            Some(r) => Some(r.merge(node)),
        };
        self.count += 1;
    }

    /// Peek at the minimum.
    pub fn peek_min(&self) -> Option<(&K, &V)> {
        self.root.as_ref().map(|r| (&r.key, &r.val))
    }

    /// Extract the minimum element.
    pub fn extract_min(&mut self) -> Option<(K, V)> {
        let root = self.root.take()?;
        self.root = PairingNode::two_pass_merge(root.children);
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
    pub fn merge(&mut self, other: PairingHeap<K, V>) {
        self.count += other.count;
        self.root = match (self.root.take(), other.root) {
            (None, r) => r,
            (l, None) => l,
            (Some(l), Some(r)) => Some(l.merge(r)),
        };
    }
}

impl<K: Ord + Clone, V: Clone> Default for PairingHeap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_peek() {
        let mut h: PairingHeap<u32, u32> = PairingHeap::new();
        h.insert(5, 50);
        h.insert(2, 20);
        let (k, _) = h.peek_min().unwrap();
        assert_eq!(*k, 2 /* min key */);
    }

    #[test]
    fn test_extract_min() {
        let mut h: PairingHeap<u32, u32> = PairingHeap::new();
        h.insert(4, 40);
        h.insert(1, 10);
        let (k, v) = h.extract_min().unwrap();
        assert_eq!(k, 1 /* min extracted */);
        assert_eq!(v, 10);
    }

    #[test]
    fn test_len_tracks() {
        let mut h: PairingHeap<u32, u32> = PairingHeap::new();
        h.insert(1, 1);
        h.insert(2, 2);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let h: PairingHeap<u32, u32> = PairingHeap::new();
        assert!(h.is_empty());
    }

    #[test]
    fn test_extract_all_sorted() {
        let mut h: PairingHeap<u32, u32> = PairingHeap::new();
        for v in [3u32, 1, 4, 1, 5, 9, 2] {
            h.insert(v, v);
        }
        let mut last = 0u32;
        while let Some((k, _)) = h.extract_min() {
            assert!(k >= last /* non-decreasing order */);
            last = k;
        }
    }

    #[test]
    fn test_merge_heaps() {
        let mut a: PairingHeap<u32, u32> = PairingHeap::new();
        let mut b: PairingHeap<u32, u32> = PairingHeap::new();
        a.insert(10, 10);
        b.insert(5, 5);
        a.merge(b);
        let (k, _) = a.peek_min().unwrap();
        assert_eq!(*k, 5 /* min from merged heap */);
    }

    #[test]
    fn test_empty_extract() {
        let mut h: PairingHeap<u32, u32> = PairingHeap::new();
        assert!(h.extract_min().is_none() /* empty returns None */);
    }

    #[test]
    fn test_default() {
        let h: PairingHeap<u32, u32> = PairingHeap::default();
        assert!(h.is_empty());
    }

    #[test]
    fn test_single_element() {
        let mut h: PairingHeap<u32, u32> = PairingHeap::new();
        h.insert(42, 420);
        assert_eq!(h.extract_min(), Some((42, 420)) /* single element */);
        assert!(h.is_empty());
    }
}
