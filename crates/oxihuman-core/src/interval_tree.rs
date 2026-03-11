// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 1D interval tree for range queries.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Interval {
    pub lo: f32,
    pub hi: f32,
    pub id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntervalTree {
    pub intervals: Vec<Interval>,
}

#[allow(dead_code)]
pub fn new_interval_tree() -> IntervalTree {
    IntervalTree {
        intervals: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn it_insert(tree: &mut IntervalTree, lo: f32, hi: f32, id: u32) {
    tree.intervals.push(Interval { lo, hi, id });
}

#[allow(dead_code)]
pub fn it_remove(tree: &mut IntervalTree, id: u32) {
    tree.intervals.retain(|iv| iv.id != id);
}

#[allow(dead_code)]
pub fn it_query_point(tree: &IntervalTree, x: f32) -> Vec<u32> {
    tree.intervals
        .iter()
        .filter(|iv| (iv.lo..=iv.hi).contains(&x))
        .map(|iv| iv.id)
        .collect()
}

#[allow(dead_code)]
pub fn it_query_range(tree: &IntervalTree, lo: f32, hi: f32) -> Vec<u32> {
    tree.intervals
        .iter()
        .filter(|iv| iv.lo <= hi && iv.hi >= lo)
        .map(|iv| iv.id)
        .collect()
}

#[allow(dead_code)]
pub fn it_count(tree: &IntervalTree) -> usize {
    tree.intervals.len()
}

#[allow(dead_code)]
pub fn it_clear(tree: &mut IntervalTree) {
    tree.intervals.clear();
}

#[allow(dead_code)]
pub fn it_contains_id(tree: &IntervalTree, id: u32) -> bool {
    tree.intervals.iter().any(|iv| iv.id == id)
}

#[allow(dead_code)]
pub fn it_to_json(tree: &IntervalTree) -> String {
    let entries: Vec<String> = tree
        .intervals
        .iter()
        .map(|iv| format!("{{\"id\":{},\"lo\":{},\"hi\":{}}}", iv.id, iv.lo, iv.hi))
        .collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let t = new_interval_tree();
        assert_eq!(it_count(&t), 0);
    }

    #[test]
    fn test_insert_and_count() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 1.0, 5.0, 1);
        it_insert(&mut t, 3.0, 8.0, 2);
        assert_eq!(it_count(&t), 2);
    }

    #[test]
    fn test_query_point() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 1.0, 5.0, 1);
        it_insert(&mut t, 6.0, 10.0, 2);
        let res = it_query_point(&t, 3.0);
        assert!(res.contains(&1));
        assert!(!res.contains(&2));
    }

    #[test]
    fn test_query_range_overlap() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 1.0, 5.0, 1);
        it_insert(&mut t, 4.0, 9.0, 2);
        it_insert(&mut t, 10.0, 15.0, 3);
        let res = it_query_range(&t, 3.0, 6.0);
        assert!(res.contains(&1));
        assert!(res.contains(&2));
        assert!(!res.contains(&3));
    }

    #[test]
    fn test_remove() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 1.0, 5.0, 1);
        it_insert(&mut t, 6.0, 10.0, 2);
        it_remove(&mut t, 1);
        assert_eq!(it_count(&t), 1);
        assert!(!it_contains_id(&t, 1));
    }

    #[test]
    fn test_contains_id() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 0.0, 1.0, 42);
        assert!(it_contains_id(&t, 42));
        assert!(!it_contains_id(&t, 99));
    }

    #[test]
    fn test_clear() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 0.0, 1.0, 1);
        it_clear(&mut t);
        assert_eq!(it_count(&t), 0);
    }

    #[test]
    fn test_to_json() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 0.0, 1.0, 1);
        let json = it_to_json(&t);
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_query_point_boundary() {
        let mut t = new_interval_tree();
        it_insert(&mut t, 2.0, 4.0, 7);
        let at_lo = it_query_point(&t, 2.0);
        let at_hi = it_query_point(&t, 4.0);
        assert!(at_lo.contains(&7));
        assert!(at_hi.contains(&7));
    }
}
