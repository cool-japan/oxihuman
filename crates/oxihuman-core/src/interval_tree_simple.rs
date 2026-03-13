// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple interval overlap query structure.

pub struct IntervalSimple {
    pub lo: f32,
    pub hi: f32,
    pub id: usize,
}

pub struct IntervalTreeSimple {
    pub intervals: Vec<IntervalSimple>,
}

pub fn new_interval_tree_simple() -> IntervalTreeSimple {
    IntervalTreeSimple {
        intervals: Vec::new(),
    }
}

pub fn itree_simple_insert(t: &mut IntervalTreeSimple, lo: f32, hi: f32, id: usize) {
    t.intervals.push(IntervalSimple { lo, hi, id });
}

pub fn itree_simple_query_overlaps(t: &IntervalTreeSimple, lo: f32, hi: f32) -> Vec<usize> {
    t.intervals
        .iter()
        .filter(|i| i.lo <= hi && i.hi >= lo)
        .map(|i| i.id)
        .collect()
}

pub fn itree_simple_count_overlaps(t: &IntervalTreeSimple, lo: f32, hi: f32) -> usize {
    t.intervals
        .iter()
        .filter(|i| i.lo <= hi && i.hi >= lo)
        .count()
}

pub fn itree_simple_size(t: &IntervalTreeSimple) -> usize {
    t.intervals.len()
}

pub fn itree_simple_contains_point(t: &IntervalTreeSimple, pt: f32) -> Vec<usize> {
    t.intervals
        .iter()
        .filter(|i| i.lo <= pt && i.hi >= pt)
        .map(|i| i.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_interval_tree_simple() {
        /* empty tree has size 0 */
        let t = new_interval_tree_simple();
        assert_eq!(itree_simple_size(&t), 0);
    }

    #[test]
    fn test_insert_and_size() {
        /* inserting intervals increases size */
        let mut t = new_interval_tree_simple();
        itree_simple_insert(&mut t, 0.0, 5.0, 1);
        itree_simple_insert(&mut t, 3.0, 8.0, 2);
        assert_eq!(itree_simple_size(&t), 2);
    }

    #[test]
    fn test_query_overlaps() {
        /* overlapping interval query returns correct ids */
        let mut t = new_interval_tree_simple();
        itree_simple_insert(&mut t, 0.0, 5.0, 1);
        itree_simple_insert(&mut t, 6.0, 10.0, 2);
        let overlaps = itree_simple_query_overlaps(&t, 4.0, 7.0);
        assert!(overlaps.contains(&1));
        assert!(overlaps.contains(&2));
    }

    #[test]
    fn test_no_overlap() {
        /* non-overlapping intervals not returned */
        let mut t = new_interval_tree_simple();
        itree_simple_insert(&mut t, 0.0, 3.0, 1);
        itree_simple_insert(&mut t, 7.0, 10.0, 2);
        let overlaps = itree_simple_query_overlaps(&t, 4.0, 5.0);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn test_count_overlaps() {
        /* count returns correct number */
        let mut t = new_interval_tree_simple();
        itree_simple_insert(&mut t, 0.0, 10.0, 1);
        itree_simple_insert(&mut t, 5.0, 15.0, 2);
        itree_simple_insert(&mut t, 20.0, 30.0, 3);
        assert_eq!(itree_simple_count_overlaps(&t, 6.0, 7.0), 2);
    }

    #[test]
    fn test_contains_point() {
        /* point query returns intervals that contain the point */
        let mut t = new_interval_tree_simple();
        itree_simple_insert(&mut t, 0.0, 10.0, 1);
        itree_simple_insert(&mut t, 5.0, 15.0, 2);
        itree_simple_insert(&mut t, 20.0, 30.0, 3);
        let ids = itree_simple_contains_point(&t, 7.0);
        assert!(ids.contains(&1) && ids.contains(&2));
        assert!(!ids.contains(&3));
    }

    #[test]
    fn test_point_boundary() {
        /* boundary points are included */
        let mut t = new_interval_tree_simple();
        itree_simple_insert(&mut t, 0.0, 5.0, 1);
        let ids = itree_simple_contains_point(&t, 5.0);
        assert!(ids.contains(&1));
    }
}
