// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Segment tree for range min/max queries with point updates.

pub struct SegmentTreeRange {
    pub data: Vec<i64>,
    pub n: usize,
}

pub fn new_segment_tree_range(values: &[i64]) -> SegmentTreeRange {
    let n = values.len();
    let mut data = vec![i64::MAX; 4 * n.max(1)];
    if n > 0 {
        build(&mut data, values, 1, 0, n - 1);
    }
    SegmentTreeRange { data, n }
}

fn build(data: &mut [i64], values: &[i64], node: usize, lo: usize, hi: usize) {
    if lo == hi {
        data[node] = values[lo];
        return;
    }
    let mid = (lo + hi) / 2;
    build(data, values, node * 2, lo, mid);
    build(data, values, node * 2 + 1, mid + 1, hi);
    data[node] = data[node * 2].min(data[node * 2 + 1]);
}

fn query_min_inner(data: &[i64], node: usize, lo: usize, hi: usize, l: usize, r: usize) -> i64 {
    if r < lo || hi < l {
        return i64::MAX;
    }
    if l <= lo && hi <= r {
        return data[node];
    }
    let mid = (lo + hi) / 2;
    query_min_inner(data, node * 2, lo, mid, l, r).min(query_min_inner(
        data,
        node * 2 + 1,
        mid + 1,
        hi,
        l,
        r,
    ))
}

fn query_max_inner(data: &[i64], node: usize, lo: usize, hi: usize, l: usize, r: usize) -> i64 {
    if r < lo || hi < l {
        return i64::MIN;
    }
    if l <= lo && hi <= r {
        // rebuild max lazily from stored min tree by using separate max pass
        // Since the tree stores min, we walk leaves for max query
        if lo == hi {
            return data[node];
        }
        let mid = (lo + hi) / 2;
        return query_max_inner(data, node * 2, lo, mid, l, r).max(query_max_inner(
            data,
            node * 2 + 1,
            mid + 1,
            hi,
            l,
            r,
        ));
    }
    let mid = (lo + hi) / 2;
    query_max_inner(data, node * 2, lo, mid, l, r).max(query_max_inner(
        data,
        node * 2 + 1,
        mid + 1,
        hi,
        l,
        r,
    ))
}

fn update_inner(data: &mut [i64], node: usize, lo: usize, hi: usize, i: usize, val: i64) {
    if lo == hi {
        data[node] = val;
        return;
    }
    let mid = (lo + hi) / 2;
    if i <= mid {
        update_inner(data, node * 2, lo, mid, i, val);
    } else {
        update_inner(data, node * 2 + 1, mid + 1, hi, i, val);
    }
    data[node] = data[node * 2].min(data[node * 2 + 1]);
}

pub fn seg_range_query_min(t: &SegmentTreeRange, l: usize, r: usize) -> i64 {
    if t.n == 0 {
        return i64::MAX;
    }
    query_min_inner(&t.data, 1, 0, t.n - 1, l, r)
}

pub fn seg_range_query_max(t: &SegmentTreeRange, l: usize, r: usize) -> i64 {
    if t.n == 0 {
        return i64::MIN;
    }
    query_max_inner(&t.data, 1, 0, t.n - 1, l, r)
}

pub fn seg_range_update(t: &mut SegmentTreeRange, i: usize, val: i64) {
    if t.n == 0 {
        return;
    }
    update_inner(&mut t.data, 1, 0, t.n - 1, i, val);
}

pub fn seg_range_size(t: &SegmentTreeRange) -> usize {
    t.n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_min_basic() {
        /* min over full range */
        let t = new_segment_tree_range(&[3, 1, 4, 1, 5, 9, 2, 6]);
        assert_eq!(seg_range_query_min(&t, 0, 7), 1);
    }

    #[test]
    fn test_range_max_basic() {
        /* max over full range */
        let t = new_segment_tree_range(&[3, 1, 4, 1, 5, 9, 2, 6]);
        assert_eq!(seg_range_query_max(&t, 0, 7), 9);
    }

    #[test]
    fn test_range_min_subrange() {
        /* min over subrange */
        let t = new_segment_tree_range(&[5, 3, 8, 2, 7]);
        assert_eq!(seg_range_query_min(&t, 1, 3), 2);
    }

    #[test]
    fn test_range_max_subrange() {
        /* max over subrange */
        let t = new_segment_tree_range(&[5, 3, 8, 2, 7]);
        assert_eq!(seg_range_query_max(&t, 0, 2), 8);
    }

    #[test]
    fn test_update_changes_result() {
        /* point update changes min query */
        let mut t = new_segment_tree_range(&[5, 3, 8, 2, 7]);
        seg_range_update(&mut t, 3, 100);
        assert_eq!(seg_range_query_min(&t, 0, 4), 3);
    }

    #[test]
    fn test_size() {
        /* size equals input length */
        let t = new_segment_tree_range(&[1, 2, 3]);
        assert_eq!(seg_range_size(&t), 3);
    }

    #[test]
    fn test_single_element() {
        /* single element tree works correctly */
        let t = new_segment_tree_range(&[42]);
        assert_eq!(seg_range_query_min(&t, 0, 0), 42);
        assert_eq!(seg_range_query_max(&t, 0, 0), 42);
    }
}
