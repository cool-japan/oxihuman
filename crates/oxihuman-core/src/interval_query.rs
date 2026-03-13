// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Interval query tree — stores intervals and answers overlap queries.

/// A closed interval [lo, hi] with an associated value.
#[derive(Debug, Clone)]
pub struct Interval<V: Clone> {
    pub lo: f64,
    pub hi: f64,
    pub val: V,
}

impl<V: Clone> Interval<V> {
    /// Create a new interval.
    pub fn new(lo: f64, hi: f64, val: V) -> Self {
        Interval {
            lo: lo.min(hi),
            hi: lo.max(hi),
            val,
        }
    }

    /// True if this interval overlaps with [qlo, qhi].
    pub fn overlaps(&self, qlo: f64, qhi: f64) -> bool {
        self.lo <= qhi && self.hi >= qlo
    }
}

/// Node in the interval tree (augmented BST on interval midpoint).
#[derive(Debug, Clone)]
struct INode<V: Clone> {
    interval: Interval<V>,
    max_hi: f64,
    left: Option<Box<INode<V>>>,
    right: Option<Box<INode<V>>>,
}

impl<V: Clone> INode<V> {
    fn new(interval: Interval<V>) -> Box<Self> {
        let hi = interval.hi;
        Box::new(INode {
            interval,
            max_hi: hi,
            left: None,
            right: None,
        })
    }

    fn update_max(&mut self) {
        self.max_hi = self.interval.hi;
        if let Some(l) = &self.left {
            if l.max_hi > self.max_hi {
                self.max_hi = l.max_hi;
            }
        }
        if let Some(r) = &self.right {
            if r.max_hi > self.max_hi {
                self.max_hi = r.max_hi;
            }
        }
    }

    fn insert(node: Option<Box<INode<V>>>, interval: Interval<V>) -> Box<INode<V>> {
        let Some(mut n) = node else {
            return INode::new(interval);
        };
        let mid = (n.interval.lo + n.interval.hi) / 2.0;
        let new_mid = (interval.lo + interval.hi) / 2.0;
        if new_mid <= mid {
            n.left = Some(Self::insert(n.left.take(), interval));
        } else {
            n.right = Some(Self::insert(n.right.take(), interval));
        }
        n.update_max();
        n
    }

    fn query<'a>(&'a self, qlo: f64, qhi: f64, result: &mut Vec<&'a Interval<V>>) {
        if self.max_hi < qlo {
            return;
        }
        if let Some(l) = &self.left {
            l.query(qlo, qhi, result);
        }
        if self.interval.overlaps(qlo, qhi) {
            result.push(&self.interval);
        }
        if self.interval.lo <= qhi {
            if let Some(r) = &self.right {
                r.query(qlo, qhi, result);
            }
        }
    }
}

/// Interval overlap query tree.
pub struct IntervalQueryTree<V: Clone> {
    root: Option<Box<INode<V>>>,
    count: usize,
}

impl<V: Clone> IntervalQueryTree<V> {
    /// Create a new empty interval query tree.
    pub fn new() -> Self {
        IntervalQueryTree {
            root: None,
            count: 0,
        }
    }

    /// Insert an interval.
    pub fn insert(&mut self, lo: f64, hi: f64, val: V) {
        let iv = Interval::new(lo, hi, val);
        self.root = Some(INode::insert(self.root.take(), iv));
        self.count += 1;
    }

    /// Query all intervals overlapping [qlo, qhi].
    pub fn query(&self, qlo: f64, qhi: f64) -> Vec<&Interval<V>> {
        let mut result = Vec::new();
        if let Some(r) = &self.root {
            r.query(qlo, qhi, &mut result);
        }
        result
    }

    /// Number of intervals stored.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl<V: Clone> Default for IntervalQueryTree<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query_overlap() {
        let mut t: IntervalQueryTree<u32> = IntervalQueryTree::new();
        t.insert(1.0, 5.0, 1);
        let res = t.query(3.0, 4.0);
        assert!(!res.is_empty() /* [1,5] overlaps [3,4] */);
    }

    #[test]
    fn test_no_overlap() {
        let mut t: IntervalQueryTree<u32> = IntervalQueryTree::new();
        t.insert(1.0, 2.0, 1);
        let res = t.query(5.0, 6.0);
        assert!(res.is_empty() /* [1,2] does not overlap [5,6] */);
    }

    #[test]
    fn test_len() {
        let mut t: IntervalQueryTree<u32> = IntervalQueryTree::new();
        t.insert(0.0, 1.0, 0);
        t.insert(2.0, 3.0, 1);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let t: IntervalQueryTree<u32> = IntervalQueryTree::new();
        assert!(t.is_empty());
    }

    #[test]
    fn test_multiple_overlaps() {
        let mut t: IntervalQueryTree<u32> = IntervalQueryTree::new();
        t.insert(0.0, 10.0, 0);
        t.insert(5.0, 15.0, 1);
        t.insert(12.0, 20.0, 2);
        let res = t.query(6.0, 13.0);
        /* [0,10] and [5,15] and [12,20] all overlap [6,13] */
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn test_point_query() {
        let mut t: IntervalQueryTree<u32> = IntervalQueryTree::new();
        t.insert(1.0, 5.0, 10);
        t.insert(6.0, 9.0, 20);
        let res = t.query(3.0, 3.0);
        assert_eq!(res.len(), 1 /* only [1,5] contains point 3 */);
    }

    #[test]
    fn test_interval_overlap_method() {
        let iv = Interval::new(1.0, 5.0, ());
        assert!(iv.overlaps(4.0, 6.0) /* partial overlap right */);
        assert!(!iv.overlaps(6.0, 8.0) /* no overlap */);
    }

    #[test]
    fn test_reversed_lo_hi_normalized() {
        let iv = Interval::new(5.0, 1.0, 42u32);
        assert!(iv.lo < iv.hi /* lo and hi are normalized */);
    }

    #[test]
    fn test_default() {
        let t: IntervalQueryTree<u32> = IntervalQueryTree::default();
        assert!(t.is_empty());
    }
}
