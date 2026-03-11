// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! IQR-based outlier filter.

#[derive(Debug, Clone)]
pub struct IqrFilter {
    pub k: f64,
}

impl IqrFilter {
    /// Standard Tukey fence: k=1.5 for mild, k=3.0 for extreme outliers.
    pub fn new(k: f64) -> Self {
        IqrFilter { k }
    }

    pub fn standard() -> Self {
        IqrFilter::new(1.5)
    }

    pub fn extreme() -> Self {
        IqrFilter::new(3.0)
    }

    pub fn bounds(&self, data: &[f64]) -> Option<(f64, f64)> {
        iqr_bounds(data, self.k)
    }

    pub fn filter(&self, data: &[f64]) -> Vec<f64> {
        filter_outliers(data, self.k)
    }

    pub fn flag(&self, data: &[f64]) -> Vec<bool> {
        flag_outliers(data, self.k)
    }
}

/// Compute percentile by linear interpolation on a sorted slice.
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    assert!(!sorted.is_empty(), "data must not be empty");
    let idx = p / 100.0 * (sorted.len() as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let frac = idx - lo as f64;
    sorted[lo] + frac * (sorted[hi] - sorted[lo])
}

pub fn iqr_bounds(data: &[f64], k: f64) -> Option<(f64, f64)> {
    if data.is_empty() {
        return None;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = percentile(&sorted, 25.0);
    let q3 = percentile(&sorted, 75.0);
    let iqr = q3 - q1;
    Some((q1 - k * iqr, q3 + k * iqr))
}

pub fn filter_outliers(data: &[f64], k: f64) -> Vec<f64> {
    match iqr_bounds(data, k) {
        None => Vec::new(),
        Some((lo, hi)) => data
            .iter()
            .filter(|&&x| x >= lo && x <= hi)
            .copied()
            .collect(),
    }
}

pub fn flag_outliers(data: &[f64], k: f64) -> Vec<bool> {
    match iqr_bounds(data, k) {
        None => vec![false; data.len()],
        Some((lo, hi)) => data.iter().map(|&x| x < lo || x > hi).collect(),
    }
}

pub fn outlier_count(data: &[f64], k: f64) -> usize {
    flag_outliers(data, k).iter().filter(|&&b| b).count()
}

pub fn winsorize(data: &[f64], k: f64) -> Vec<f64> {
    match iqr_bounds(data, k) {
        None => data.to_vec(),
        Some((lo, hi)) => data.iter().map(|&x| x.clamp(lo, hi)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&data, 50.0) - 3.0).abs() < 1e-10, /* median of 1..5 = 3 */);
    }

    #[test]
    fn test_iqr_bounds_symmetric() {
        let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let (lo, hi) = iqr_bounds(&data, 1.5).unwrap();
        assert!(lo < 1.0 /* lower fence below minimum */,);
        assert!(hi > 100.0 /* upper fence above maximum */,);
    }

    #[test]
    fn test_filter_removes_outlier() {
        let mut data: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        data.push(1000.0);
        let filtered = filter_outliers(&data, 1.5);
        assert!(!filtered.contains(&1000.0) /* outlier removed */,);
    }

    #[test]
    fn test_flag_outlier() {
        let mut data: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        data.push(1000.0);
        let flags = flag_outliers(&data, 1.5);
        assert!(*flags.last().unwrap() /* last value flagged */,);
    }

    #[test]
    fn test_no_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(
            outlier_count(&data, 1.5),
            0, /* no outliers in tight range */
        );
    }

    #[test]
    fn test_winsorize_clamps() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 1000.0];
        let w = winsorize(&data, 1.5);
        assert!(*w.last().unwrap() < 1000.0 /* outlier clamped */,);
    }

    #[test]
    fn test_iqr_filter_standard() {
        let filter = IqrFilter::standard();
        assert!((filter.k - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_empty_data() {
        assert!(iqr_bounds(&[], 1.5).is_none(), /* no bounds for empty data */);
    }

    #[test]
    fn test_outlier_count() {
        let mut data: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        data.push(1000.0);
        data.push(-1000.0);
        let count = outlier_count(&data, 1.5);
        assert_eq!(count, 2 /* two outliers */,);
    }
}
