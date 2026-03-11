// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Static-dispatch filter pipeline using named filter kinds.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FilterKind {
    GreaterThan(f64),
    LessThan(f64),
    EqualTo(f64),
    Between(f64, f64),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FilterPipeline {
    pub filters: Vec<FilterKind>,
}

#[allow(dead_code)]
pub fn new_filter_pipeline() -> FilterPipeline {
    FilterPipeline {
        filters: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_gt(fp: &mut FilterPipeline, v: f64) {
    fp.filters.push(FilterKind::GreaterThan(v));
}

#[allow(dead_code)]
pub fn add_lt(fp: &mut FilterPipeline, v: f64) {
    fp.filters.push(FilterKind::LessThan(v));
}

#[allow(dead_code)]
pub fn add_between(fp: &mut FilterPipeline, lo: f64, hi: f64) {
    fp.filters.push(FilterKind::Between(lo, hi));
}

fn filter_passes(fk: &FilterKind, x: f64) -> bool {
    match fk {
        FilterKind::GreaterThan(v) => x > *v,
        FilterKind::LessThan(v) => x < *v,
        FilterKind::EqualTo(v) => (x - v).abs() < f64::EPSILON,
        FilterKind::Between(lo, hi) => x >= *lo && x <= *hi,
    }
}

#[allow(dead_code)]
pub fn apply(fp: &FilterPipeline, x: f64) -> bool {
    fp.filters.iter().all(|f| filter_passes(f, x))
}

#[allow(dead_code)]
pub fn filter_slice(fp: &FilterPipeline, xs: &[f64]) -> Vec<f64> {
    xs.iter().copied().filter(|&x| apply(fp, x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gt_filter() {
        let mut fp = new_filter_pipeline();
        add_gt(&mut fp, 5.0);
        assert!(apply(&fp, 6.0));
        assert!(!apply(&fp, 5.0));
    }

    #[test]
    fn test_lt_filter() {
        let mut fp = new_filter_pipeline();
        add_lt(&mut fp, 5.0);
        assert!(apply(&fp, 4.0));
        assert!(!apply(&fp, 5.0));
    }

    #[test]
    fn test_between() {
        let mut fp = new_filter_pipeline();
        add_between(&mut fp, 2.0, 8.0);
        assert!(apply(&fp, 5.0));
        assert!(!apply(&fp, 10.0));
    }

    #[test]
    fn test_combined() {
        let mut fp = new_filter_pipeline();
        add_gt(&mut fp, 0.0);
        add_lt(&mut fp, 10.0);
        assert!(apply(&fp, 5.0));
        assert!(!apply(&fp, -1.0));
    }

    #[test]
    fn test_empty_pipeline_passes_all() {
        let fp = new_filter_pipeline();
        assert!(apply(&fp, 999.0));
    }

    #[test]
    fn test_filter_slice() {
        let mut fp = new_filter_pipeline();
        add_gt(&mut fp, 0.0);
        let result = filter_slice(&fp, &[-1.0, 0.0, 1.0, 2.0]);
        assert_eq!(result, vec![1.0, 2.0]);
    }

    #[test]
    fn test_equal_to() {
        let fp = FilterPipeline {
            filters: vec![FilterKind::EqualTo(3.0)],
        };
        assert!(apply(&fp, 3.0));
        assert!(!apply(&fp, 3.1));
    }

    #[test]
    fn test_filter_slice_empty_input() {
        let fp = new_filter_pipeline();
        let result = filter_slice(&fp, &[]);
        assert!(result.is_empty());
    }
}
