// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Maps non-overlapping intervals [lo, hi) to values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntervalMap<T: Clone> {
    entries: Vec<IntervalEntry<T>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntervalEntry<T: Clone> {
    pub lo: f64,
    pub hi: f64,
    pub value: T,
}

#[allow(dead_code)]
impl<T: Clone> IntervalMap<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, lo: f64, hi: f64, value: T) {
        assert!(hi > lo);
        self.entries.push(IntervalEntry { lo, hi, value });
        self.entries.sort_by(|a, b| a.lo.partial_cmp(&b.lo).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn query(&self, point: f64) -> Option<&T> {
        self.entries
            .iter()
            .find(|e| point >= e.lo && point < e.hi)
            .map(|e| &e.value)
    }

    pub fn query_range(&self, lo: f64, hi: f64) -> Vec<&T> {
        self.entries
            .iter()
            .filter(|e| e.hi > lo && e.lo < hi)
            .map(|e| &e.value)
            .collect()
    }

    pub fn remove_at(&mut self, point: f64) -> bool {
        let pos = self.entries.iter().position(|e| point >= e.lo && point < e.hi);
        if let Some(idx) = pos {
            self.entries.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn span(&self) -> Option<(f64, f64)> {
        if self.entries.is_empty() {
            return None;
        }
        let lo = self.entries.first()?.lo;
        let hi = self.entries.last()?.hi;
        Some((lo, hi))
    }

    pub fn contains_point(&self, point: f64) -> bool {
        self.query(point).is_some()
    }
}

impl<T: Clone> Default for IntervalMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m: IntervalMap<i32> = IntervalMap::new();
        assert!(m.is_empty());
    }

    #[test]
    fn test_insert_query() {
        let mut m = IntervalMap::new();
        m.insert(0.0, 10.0, "first");
        assert_eq!(m.query(5.0), Some(&"first"));
    }

    #[test]
    fn test_query_miss() {
        let mut m = IntervalMap::new();
        m.insert(0.0, 5.0, 1);
        assert!(m.query(5.0).is_none());
        assert!(m.query(-1.0).is_none());
    }

    #[test]
    fn test_query_range() {
        let mut m = IntervalMap::new();
        m.insert(0.0, 5.0, "a");
        m.insert(5.0, 10.0, "b");
        m.insert(10.0, 15.0, "c");
        let results = m.query_range(3.0, 12.0);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_remove_at() {
        let mut m = IntervalMap::new();
        m.insert(0.0, 10.0, 1);
        assert!(m.remove_at(5.0));
        assert!(m.is_empty());
    }

    #[test]
    fn test_remove_miss() {
        let mut m = IntervalMap::new();
        m.insert(0.0, 10.0, 1);
        assert!(!m.remove_at(20.0));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_span() {
        let mut m = IntervalMap::new();
        m.insert(5.0, 10.0, 1);
        m.insert(0.0, 5.0, 2);
        let (lo, hi) = m.span().expect("should succeed");
        assert!((lo - 0.0).abs() < 1e-9);
        assert!((hi - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_contains_point() {
        let mut m = IntervalMap::new();
        m.insert(1.0, 3.0, "x");
        assert!(m.contains_point(2.0));
        assert!(!m.contains_point(0.5));
    }

    #[test]
    fn test_clear() {
        let mut m = IntervalMap::new();
        m.insert(0.0, 1.0, 1);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn test_empty_span() {
        let m: IntervalMap<i32> = IntervalMap::new();
        assert!(m.span().is_none());
    }
}
