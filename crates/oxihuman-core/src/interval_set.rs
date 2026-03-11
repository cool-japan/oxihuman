// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A set of non-overlapping intervals with merge-on-insert semantics.

/// A closed interval [lo, hi].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

#[allow(dead_code)]
impl Interval {
    pub fn new(lo: f64, hi: f64) -> Self {
        let (a, b) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        Self { lo: a, hi: b }
    }

    pub fn length(&self) -> f64 {
        self.hi - self.lo
    }

    pub fn contains(&self, x: f64) -> bool {
        x >= self.lo && x <= self.hi
    }

    pub fn overlaps(&self, other: &Interval) -> bool {
        self.lo <= other.hi && other.lo <= self.hi
    }

    pub fn merge(&self, other: &Interval) -> Interval {
        Interval {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }

    pub fn midpoint(&self) -> f64 {
        (self.lo + self.hi) * 0.5
    }
}

/// A collection of non-overlapping intervals, automatically merged on insert.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntervalSet {
    intervals: Vec<Interval>,
}

#[allow(dead_code)]
impl IntervalSet {
    pub fn new() -> Self {
        Self { intervals: Vec::new() }
    }

    pub fn insert(&mut self, interval: Interval) {
        let mut merged = interval;
        let mut remaining = Vec::new();
        for iv in &self.intervals {
            if iv.overlaps(&merged) || (iv.hi == merged.lo) || (merged.hi == iv.lo) {
                merged = merged.merge(iv);
            } else {
                remaining.push(*iv);
            }
        }
        remaining.push(merged);
        remaining.sort_by(|a, b| a.lo.partial_cmp(&b.lo).unwrap_or(std::cmp::Ordering::Equal));
        self.intervals = remaining;
    }

    pub fn contains(&self, x: f64) -> bool {
        self.intervals.iter().any(|iv| iv.contains(x))
    }

    pub fn interval_count(&self) -> usize {
        self.intervals.len()
    }

    pub fn total_length(&self) -> f64 {
        self.intervals.iter().map(|iv| iv.length()).sum()
    }

    pub fn intervals(&self) -> &[Interval] {
        &self.intervals
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn span(&self) -> Option<Interval> {
        if self.intervals.is_empty() {
            return None;
        }
        let lo = self.intervals.first()?.lo;
        let hi = self.intervals.last()?.hi;
        Some(Interval::new(lo, hi))
    }

    pub fn gap_count(&self) -> usize {
        if self.intervals.len() <= 1 { return 0; }
        self.intervals.len() - 1
    }
}

impl Default for IntervalSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_insert() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(1.0, 3.0));
        assert_eq!(s.interval_count(), 1);
    }

    #[test]
    fn test_merge_overlapping() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(1.0, 3.0));
        s.insert(Interval::new(2.0, 5.0));
        assert_eq!(s.interval_count(), 1);
        assert!((s.intervals()[0].lo - 1.0).abs() < f64::EPSILON);
        assert!((s.intervals()[0].hi - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_non_overlapping() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(1.0, 2.0));
        s.insert(Interval::new(4.0, 5.0));
        assert_eq!(s.interval_count(), 2);
    }

    #[test]
    fn test_contains() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(0.0, 10.0));
        assert!(s.contains(5.0));
        assert!(!s.contains(15.0));
    }

    #[test]
    fn test_total_length() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(0.0, 3.0));
        s.insert(Interval::new(5.0, 8.0));
        assert!((s.total_length() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adjacent_merge() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(0.0, 1.0));
        s.insert(Interval::new(1.0, 2.0));
        assert_eq!(s.interval_count(), 1);
    }

    #[test]
    fn test_span() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(2.0, 4.0));
        s.insert(Interval::new(8.0, 10.0));
        let span = s.span().unwrap();
        assert!((span.lo - 2.0).abs() < f64::EPSILON);
        assert!((span.hi - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gap_count() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(0.0, 1.0));
        s.insert(Interval::new(3.0, 4.0));
        s.insert(Interval::new(6.0, 7.0));
        assert_eq!(s.gap_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut s = IntervalSet::new();
        s.insert(Interval::new(0.0, 1.0));
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_interval_midpoint() {
        let iv = Interval::new(2.0, 8.0);
        assert!((iv.midpoint() - 5.0).abs() < f64::EPSILON);
    }
}
