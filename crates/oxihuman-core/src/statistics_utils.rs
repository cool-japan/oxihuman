// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Basic statistical utilities.

/// Arithmetic mean of a slice. Returns 0.0 for empty input.
pub fn mean(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f32>() / data.len() as f32
}

/// Population variance of a slice. Returns 0.0 for fewer than 2 elements.
pub fn variance(data: &[f32]) -> f32 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    data.iter().map(|&x| (x - m) * (x - m)).sum::<f32>() / data.len() as f32
}

/// Population standard deviation.
pub fn std_dev(data: &[f32]) -> f32 {
    variance(data).sqrt()
}

/// Median of a slice (clones and sorts). Returns 0.0 for empty.
pub fn median(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n.is_multiple_of(2) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

/// Minimum value. Returns f32::MAX for empty.
pub fn min_val(data: &[f32]) -> f32 {
    data.iter().cloned().fold(f32::MAX, f32::min)
}

/// Maximum value. Returns f32::MIN for empty.
pub fn max_val(data: &[f32]) -> f32 {
    data.iter().cloned().fold(f32::MIN, f32::max)
}

/// Pearson correlation coefficient between two equal-length slices.
/// Returns 0.0 if inputs are empty or have zero variance.
pub fn pearson_r(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len().min(y.len());
    if n == 0 {
        return 0.0;
    }
    let mx = mean(&x[..n]);
    let my = mean(&y[..n]);
    let num: f32 = x[..n]
        .iter()
        .zip(y[..n].iter())
        .map(|(&xi, &yi)| (xi - mx) * (yi - my))
        .sum();
    let dx: f32 = x[..n]
        .iter()
        .map(|&xi| (xi - mx) * (xi - mx))
        .sum::<f32>()
        .sqrt();
    let dy: f32 = y[..n]
        .iter()
        .map(|&yi| (yi - my) * (yi - my))
        .sum::<f32>()
        .sqrt();
    if dx < 1e-12 || dy < 1e-12 {
        return 0.0;
    }
    num / (dx * dy)
}

/// p-th percentile (0–100) using linear interpolation. Returns 0.0 for empty data.
pub fn percentile(data: &[f32], p: f32) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (p / 100.0 * (sorted.len() - 1) as f32).clamp(0.0, (sorted.len() - 1) as f32);
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f32;
    sorted[lo] + frac * (sorted[hi] - sorted[lo])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_basic() {
        /* mean of [1,2,3,4,5] = 3 */
        let d = [1.0f32, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&d) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_mean_empty() {
        /* empty returns 0 */
        assert!(mean(&[]).abs() < 1e-9);
    }

    #[test]
    fn test_variance_constant() {
        /* constant array has zero variance */
        let d = [5.0f32; 10];
        assert!(variance(&d).abs() < 1e-9);
    }

    #[test]
    fn test_std_dev_known() {
        /* std dev of [2,4,4,4,5,5,7,9] = 2 */
        let d = [2.0f32, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        assert!((std_dev(&d) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_median_odd() {
        /* median of [1,3,5] = 3 */
        let d = [5.0f32, 1.0, 3.0];
        assert!((median(&d) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_median_even() {
        /* median of [1,2,3,4] = 2.5 */
        let d = [4.0f32, 2.0, 1.0, 3.0];
        assert!((median(&d) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_min_max() {
        /* min and max */
        let d = [3.0f32, 1.0, 4.0, 1.0, 5.0];
        assert!((min_val(&d) - 1.0).abs() < 1e-6);
        assert!((max_val(&d) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_pearson_r_perfect() {
        /* perfectly correlated -> r=1 */
        let x: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let y: Vec<f32> = x.iter().map(|&v| v * 2.0 + 3.0).collect();
        let r = pearson_r(&x, &y);
        assert!((r - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_percentile_50() {
        /* 50th percentile ≈ median */
        let d: Vec<f32> = (1..=11).map(|i| i as f32).collect();
        let p = percentile(&d, 50.0);
        assert!((p - 6.0).abs() < 1e-4);
    }
}
