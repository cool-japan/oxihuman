// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! d-ary heap — generalized heap where each node has d children.

/// d-ary min-heap backed by a Vec.
pub struct DAryHeap<K: Ord + Clone, V: Clone> {
    data: Vec<(K, V)>,
    d: usize,
}

impl<K: Ord + Clone, V: Clone> DAryHeap<K, V> {
    /// Create a new d-ary min-heap. `d` is clamped to `[2, 16]`.
    pub fn new(d: usize) -> Self {
        DAryHeap {
            data: vec![],
            d: d.clamp(2, 16),
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, val: V) {
        self.data.push((key, val));
        let last = self.data.len() - 1;
        self.sift_up(last);
    }

    fn parent(&self, i: usize) -> usize {
        if i == 0 {
            0
        } else {
            (i - 1) / self.d
        }
    }

    fn first_child(&self, i: usize) -> usize {
        i * self.d + 1
    }

    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let p = self.parent(i);
            if self.data[i].0 < self.data[p].0 {
                self.data.swap(i, p);
                i = p;
            } else {
                break;
            }
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        let n = self.data.len();
        loop {
            let fc = self.first_child(i);
            if fc >= n {
                break;
            }
            let lc = (fc + self.d).min(n);
            let Some(min_child) = (fc..lc).min_by(|&a, &b| self.data[a].0.cmp(&self.data[b].0))
            else {
                break;
            };
            if self.data[min_child].0 < self.data[i].0 {
                self.data.swap(i, min_child);
                i = min_child;
            } else {
                break;
            }
        }
    }

    /// Peek at the minimum element.
    pub fn peek_min(&self) -> Option<(&K, &V)> {
        self.data.first().map(|(k, v)| (k, v))
    }

    /// Extract the minimum element.
    pub fn extract_min(&mut self) -> Option<(K, V)> {
        if self.data.is_empty() {
            return None;
        }
        let last = self.data.len() - 1;
        self.data.swap(0, last);
        let min = self.data.pop()?;
        if !self.data.is_empty() {
            self.sift_down(0);
        }
        Some(min)
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The branching factor d.
    pub fn branching_factor(&self) -> usize {
        self.d
    }
}

impl<K: Ord + Clone, V: Clone> Default for DAryHeap<K, V> {
    fn default() -> Self {
        Self::new(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_peek_binary() {
        let mut h: DAryHeap<u32, u32> = DAryHeap::new(2);
        h.insert(5, 50);
        h.insert(2, 20);
        let (k, _) = h.peek_min().expect("should succeed");
        assert_eq!(*k, 2 /* min with d=2 */);
    }

    #[test]
    fn test_insert_and_peek_4ary() {
        let mut h: DAryHeap<u32, u32> = DAryHeap::new(4);
        h.insert(7, 70);
        h.insert(1, 10);
        let (k, _) = h.peek_min().expect("should succeed");
        assert_eq!(*k, 1 /* min with d=4 */);
    }

    #[test]
    fn test_extract_min() {
        let mut h: DAryHeap<u32, u32> = DAryHeap::new(3);
        h.insert(9, 9);
        h.insert(3, 3);
        h.insert(6, 6);
        let (k, _) = h.extract_min().expect("should succeed");
        assert_eq!(k, 3 /* min extracted */);
    }

    #[test]
    fn test_sorted_extraction() {
        let mut h: DAryHeap<u32, u32> = DAryHeap::new(4);
        for v in [5u32, 2, 8, 1, 4, 9, 3] {
            h.insert(v, v);
        }
        let mut prev = 0u32;
        while let Some((k, _)) = h.extract_min() {
            assert!(k >= prev /* non-decreasing */);
            prev = k;
        }
    }

    #[test]
    fn test_len() {
        let mut h: DAryHeap<u32, u32> = DAryHeap::new(4);
        h.insert(1, 1);
        h.insert(2, 2);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let h: DAryHeap<u32, u32> = DAryHeap::new(4);
        assert!(h.is_empty());
    }

    #[test]
    fn test_branching_factor() {
        let h: DAryHeap<u32, u32> = DAryHeap::new(8);
        assert_eq!(h.branching_factor(), 8 /* d stored correctly */);
    }

    #[test]
    fn test_d_clamp() {
        let h: DAryHeap<u32, u32> = DAryHeap::new(0);
        assert_eq!(h.branching_factor(), 2 /* clamped to 2 */);
    }

    #[test]
    fn test_default() {
        let h: DAryHeap<u32, u32> = DAryHeap::default();
        assert_eq!(h.branching_factor(), 4 /* default d=4 */);
    }
}
