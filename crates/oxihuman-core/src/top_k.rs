// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Top-K element tracking with min-heap.

#![allow(dead_code)]

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

/// An item with a frequency count.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopKItem {
    pub key: String,
    pub count: u64,
}

impl PartialOrd for TopKItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopKItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.count.cmp(&other.count).then(self.key.cmp(&other.key))
    }
}

/// Top-K tracker backed by a frequency map and min-heap eviction.
#[allow(dead_code)]
pub struct TopK {
    k: usize,
    counts: HashMap<String, u64>,
}

impl TopK {
    #[allow(dead_code)]
    pub fn new(k: usize) -> Self {
        Self {
            k: k.max(1),
            counts: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, key: &str) {
        self.add_n(key, 1);
    }

    #[allow(dead_code)]
    pub fn add_n(&mut self, key: &str, n: u64) {
        let entry = self.counts.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_add(n);
    }

    /// Return top-K items in descending order of count.
    #[allow(dead_code)]
    pub fn top(&self) -> Vec<TopKItem> {
        let mut heap: BinaryHeap<Reverse<TopKItem>> = BinaryHeap::new();
        for (key, &count) in &self.counts {
            let item = TopKItem { key: key.clone(), count };
            if heap.len() < self.k {
                heap.push(Reverse(item));
            } else if let Some(Reverse(min)) = heap.peek() {
                if count > min.count {
                    heap.pop();
                    heap.push(Reverse(item));
                }
            }
        }
        let mut result: Vec<TopKItem> = heap.into_iter().map(|Reverse(x)| x).collect();
        result.sort_by(|a, b| b.count.cmp(&a.count).then(a.key.cmp(&b.key)));
        result
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> u64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn k(&self) -> usize {
        self.k
    }

    #[allow(dead_code)]
    pub fn total_unique(&self) -> usize {
        self.counts.len()
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.counts.clear();
    }

    #[allow(dead_code)]
    pub fn contains(&self, key: &str) -> bool {
        self.counts.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top1() {
        let mut tk = TopK::new(1);
        tk.add_n("a", 5);
        tk.add_n("b", 3);
        let top = tk.top();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].key, "a");
    }

    #[test]
    fn test_top3_ordering() {
        let mut tk = TopK::new(3);
        tk.add_n("c", 1);
        tk.add_n("a", 10);
        tk.add_n("b", 5);
        tk.add_n("d", 8);
        let top = tk.top();
        assert_eq!(top[0].count, 10);
        assert_eq!(top[1].count, 8);
        assert_eq!(top[2].count, 5);
    }

    #[test]
    fn test_get() {
        let mut tk = TopK::new(5);
        tk.add_n("x", 7);
        assert_eq!(tk.get("x"), 7);
        assert_eq!(tk.get("y"), 0);
    }

    #[test]
    fn test_add_increments() {
        let mut tk = TopK::new(5);
        tk.add("item");
        tk.add("item");
        assert_eq!(tk.get("item"), 2);
    }

    #[test]
    fn test_reset() {
        let mut tk = TopK::new(3);
        tk.add_n("a", 5);
        tk.reset();
        assert_eq!(tk.total_unique(), 0);
    }

    #[test]
    fn test_contains() {
        let mut tk = TopK::new(3);
        tk.add("present");
        assert!(tk.contains("present"));
        assert!(!tk.contains("absent"));
    }

    #[test]
    fn test_total_unique() {
        let mut tk = TopK::new(10);
        tk.add("p");
        tk.add("q");
        tk.add("p");
        assert_eq!(tk.total_unique(), 2);
    }

    #[test]
    fn test_k_getter() {
        let tk = TopK::new(7);
        assert_eq!(tk.k(), 7);
    }

    #[test]
    fn test_k_min_one() {
        let tk = TopK::new(0);
        assert_eq!(tk.k(), 1);
    }

    #[test]
    fn test_top_fewer_than_k() {
        let mut tk = TopK::new(10);
        tk.add_n("only", 1);
        assert_eq!(tk.top().len(), 1);
    }
}
