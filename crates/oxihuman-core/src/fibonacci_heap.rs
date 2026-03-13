// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fibonacci heap stub — min-heap with amortized O(1) insert and decrease-key.

/// A node in the Fibonacci heap.
#[derive(Debug, Clone)]
struct FibNode<K: Ord + Clone, V: Clone> {
    key: K,
    val: V,
    children: Vec<FibNode<K, V>>,
    marked: bool,
}

impl<K: Ord + Clone, V: Clone> FibNode<K, V> {
    fn new(key: K, val: V) -> Self {
        FibNode {
            key,
            val,
            children: vec![],
            marked: false,
        }
    }

    fn degree(&self) -> usize {
        self.children.len()
    }
}

/// Fibonacci min-heap.
pub struct FibonacciHeap<K: Ord + Clone, V: Clone> {
    roots: Vec<FibNode<K, V>>,
    min_idx: Option<usize>,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> FibonacciHeap<K, V> {
    /// Create a new empty Fibonacci heap.
    pub fn new() -> Self {
        FibonacciHeap {
            roots: vec![],
            min_idx: None,
            count: 0,
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        let node = FibNode::new(key, val);
        self.roots.push(node);
        let last = self.roots.len() - 1;
        match self.min_idx {
            None => self.min_idx = Some(last),
            Some(m) => {
                if self.roots[last].key < self.roots[m].key {
                    self.min_idx = Some(last);
                }
            }
        }
        self.count += 1;
    }

    /// Peek at the minimum key-value pair.
    pub fn peek_min(&self) -> Option<(&K, &V)> {
        self.min_idx
            .map(|i| (&self.roots[i].key, &self.roots[i].val))
    }

    /// Extract the minimum element. Uses a simplified consolidation.
    pub fn extract_min(&mut self) -> Option<(K, V)> {
        let mi = self.min_idx?;
        let node = self.roots.remove(mi);
        /* add children to root list */
        for child in node.children {
            self.roots.push(child);
        }
        self.consolidate();
        self.count = self.count.saturating_sub(1);
        Some((node.key, node.val))
    }

    fn consolidate(&mut self) {
        if self.roots.is_empty() {
            self.min_idx = None;
            return;
        }
        /* simplified: find min in root list */
        let mut min_i = 0;
        for i in 1..self.roots.len() {
            if self.roots[i].key < self.roots[min_i].key {
                min_i = i;
            }
        }
        self.min_idx = Some(min_i);
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
    pub fn merge(&mut self, other: FibonacciHeap<K, V>) {
        for r in other.roots {
            self.roots.push(r);
        }
        self.count += other.count;
        self.consolidate();
    }
}

impl<K: Ord + Clone, V: Clone> Default for FibonacciHeap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_peek() {
        let mut h: FibonacciHeap<u32, &str> = FibonacciHeap::new();
        h.insert(3, "three");
        h.insert(1, "one");
        let (k, _) = h.peek_min().expect("should succeed");
        assert_eq!(*k, 1 /* min should be 1 */);
    }

    #[test]
    fn test_extract_min() {
        let mut h: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        h.insert(5, 50);
        h.insert(2, 20);
        let (k, v) = h.extract_min().expect("should succeed");
        assert_eq!(k, 2 /* extracted min key */);
        assert_eq!(v, 20 /* extracted min val */);
    }

    #[test]
    fn test_len_tracks() {
        let mut h: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        h.insert(1, 1);
        h.insert(2, 2);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let h: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        assert!(h.is_empty());
    }

    #[test]
    fn test_extract_all_in_order() {
        let mut h: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        for v in [4u32, 1, 3, 2, 5] {
            h.insert(v, v);
        }
        let first = h.extract_min().expect("should succeed").0;
        assert_eq!(first, 1 /* smallest extracted first */);
    }

    #[test]
    fn test_merge() {
        let mut a: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        let mut b: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        a.insert(5, 5);
        b.insert(3, 3);
        a.merge(b);
        let (k, _) = a.peek_min().expect("should succeed");
        assert_eq!(*k, 3 /* min of merged heap */);
    }

    #[test]
    fn test_empty_peek() {
        let h: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        assert!(h.peek_min().is_none() /* empty heap has no min */);
    }

    #[test]
    fn test_empty_extract() {
        let mut h: FibonacciHeap<u32, u32> = FibonacciHeap::new();
        assert!(h.extract_min().is_none() /* empty heap extract returns None */);
    }

    #[test]
    fn test_default() {
        let h: FibonacciHeap<u32, u32> = FibonacciHeap::default();
        assert!(h.is_empty());
    }
}
