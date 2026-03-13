// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Map of non-overlapping f32 ranges to values.

/// A half-open interval [lo, hi).
#[derive(Debug, Clone, PartialEq)]
pub struct RangeEntry<V> {
    pub lo: f32,
    pub hi: f32,
    pub value: V,
}

/// Ordered map of non-overlapping ranges.
pub struct RangeMap<V> {
    entries: Vec<RangeEntry<V>>,
}

#[allow(dead_code)]
impl<V: Clone> RangeMap<V> {
    pub fn new() -> Self {
        RangeMap {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, lo: f32, hi: f32, value: V) -> bool {
        if lo >= hi {
            return false;
        }
        if self.entries.iter().any(|e| lo < e.hi && hi > e.lo) {
            return false;
        }
        let pos = self.entries.partition_point(|e| e.lo < lo);
        self.entries.insert(pos, RangeEntry { lo, hi, value });
        true
    }

    pub fn query(&self, point: f32) -> Option<&V> {
        self.entries
            .iter()
            .find(|e| (e.lo..e.hi).contains(&point))
            .map(|e| &e.value)
    }

    pub fn remove_containing(&mut self, point: f32) -> bool {
        if let Some(idx) = self
            .entries
            .iter()
            .position(|e| (e.lo..e.hi).contains(&point))
        {
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

    pub fn overlaps(&self, lo: f32, hi: f32) -> Vec<&RangeEntry<V>> {
        self.entries
            .iter()
            .filter(|e| lo < e.hi && hi > e.lo)
            .collect()
    }

    pub fn total_coverage(&self) -> f32 {
        self.entries.iter().map(|e| e.hi - e.lo).sum()
    }

    pub fn ranges(&self) -> Vec<(f32, f32)> {
        self.entries.iter().map(|e| (e.lo, e.hi)).collect()
    }

    pub fn contains_point(&self, point: f32) -> bool {
        self.query(point).is_some()
    }
}

impl<V: Clone> Default for RangeMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_range_map<V: Clone>() -> RangeMap<V> {
    RangeMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn insert_and_query() {
        let mut m: RangeMap<&str> = new_range_map();
        assert!(m.insert(0.0, 1.0, "first"));
        assert_eq!(m.query(0.5), Some(&"first"));
    }

    #[test]
    fn query_boundary_exclusive_hi() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, 1.0, 42);
        assert!(m.query(1.0).is_none());
    }

    #[test]
    fn no_overlapping_insert() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, 2.0, 1);
        assert!(!m.insert(1.0, 3.0, 2));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn multiple_ranges() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, 1.0, 1);
        m.insert(1.0, 2.0, 2);
        assert_eq!(m.query(0.5), Some(&1));
        assert_eq!(m.query(1.5), Some(&2));
    }

    #[test]
    fn remove_containing() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, PI, 99);
        assert!(m.remove_containing(1.0));
        assert!(m.is_empty());
    }

    #[test]
    fn total_coverage() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, 1.0, 1);
        m.insert(2.0, 5.0, 2);
        assert!((m.total_coverage() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn overlaps_query() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, 2.0, 1);
        m.insert(5.0, 8.0, 2);
        let ov = m.overlaps(1.0, 3.0);
        assert_eq!(ov.len(), 1);
    }

    #[test]
    fn contains_point() {
        let mut m: RangeMap<i32> = new_range_map();
        m.insert(0.0, 1.0, 1);
        assert!(m.contains_point(0.5));
        assert!(!m.contains_point(1.5));
    }

    #[test]
    fn invalid_range_rejected() {
        let mut m: RangeMap<i32> = new_range_map();
        assert!(!m.insert(1.0, 0.5, 1));
        assert!(m.is_empty());
    }
}
