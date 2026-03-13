// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub fn stat_mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    Some(xs.iter().sum::<f64>() / xs.len() as f64)
}

#[allow(dead_code)]
pub fn stat_median(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let mut sorted = xs.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n.is_multiple_of(2) {
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    } else {
        Some(sorted[n / 2])
    }
}

#[allow(dead_code)]
pub fn stat_mode(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let mut counts: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for &x in xs {
        let key = (x * 100.0).round() as u64;
        *counts.entry(key).or_insert(0) += 1;
    }
    let max_key = counts.iter().max_by_key(|&(_, &v)| v).map(|(&k, _)| k)?;
    Some(max_key as f64 / 100.0)
}

#[allow(dead_code)]
pub fn stat_std_dev(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let mean = stat_mean(xs)?;
    let variance = xs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / xs.len() as f64;
    Some(variance.sqrt())
}

#[allow(dead_code)]
pub fn stat_skewness(xs: &[f64]) -> Option<f64> {
    if xs.len() < 3 {
        return None;
    }
    let mean = stat_mean(xs)?;
    let std = stat_std_dev(xs)?;
    if std < 1e-10 {
        return Some(0.0);
    }
    let n = xs.len() as f64;
    let sum3 = xs.iter().map(|&x| ((x - mean) / std).powi(3)).sum::<f64>();
    Some(sum3 / n)
}

#[allow(dead_code)]
pub fn stat_kurtosis(xs: &[f64]) -> Option<f64> {
    if xs.len() < 4 {
        return None;
    }
    let mean = stat_mean(xs)?;
    let std = stat_std_dev(xs)?;
    if std < 1e-10 {
        return Some(0.0);
    }
    let n = xs.len() as f64;
    let sum4 = xs.iter().map(|&x| ((x - mean) / std).powi(4)).sum::<f64>();
    Some(sum4 / n - 3.0)
}

#[allow(dead_code)]
pub fn stat_percentile(xs: &[f64], p: f64) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let mut sorted = xs.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len() as f64;
    let idx = (p / 100.0 * n - 1.0).max(0.0);
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if hi >= sorted.len() {
        return sorted.last().copied();
    }
    let frac = idx - lo as f64;
    Some(sorted[lo] * (1.0 - frac) + sorted[hi] * frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((stat_mean(&xs).expect("should succeed") - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_mean_empty() {
        assert!(stat_mean(&[]).is_none());
    }

    #[test]
    fn test_median_odd() {
        let xs = [3.0, 1.0, 5.0, 2.0, 4.0];
        assert!((stat_median(&xs).expect("should succeed") - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_median_even() {
        let xs = [1.0, 2.0, 3.0, 4.0];
        assert!((stat_median(&xs).expect("should succeed") - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev() {
        let xs = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std = stat_std_dev(&xs).expect("should succeed");
        assert!((std - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_empty() {
        assert!(stat_std_dev(&[]).is_none());
    }

    #[test]
    fn test_percentile_50th_equals_median() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let p50 = stat_percentile(&xs, 50.0).expect("should succeed");
        let med = stat_median(&xs).expect("should succeed");
        assert!((p50 - med).abs() < 1e-9);
    }

    #[test]
    fn test_percentile_100() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let p100 = stat_percentile(&xs, 100.0).expect("should succeed");
        assert!((p100 - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_mode() {
        let xs = [1.0, 2.0, 2.0, 3.0, 3.0, 3.0];
        let m = stat_mode(&xs).expect("should succeed");
        assert!((m - 3.0).abs() < 0.01);
    }
}
