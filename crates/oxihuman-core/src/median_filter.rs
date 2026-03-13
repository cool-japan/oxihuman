// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sliding window median filter.

/// Apply a sliding window median filter of `window_size` to `data`.
/// Edge values are handled by clamping window at boundaries.
pub fn median_filter_1d(data: &[f32], window_size: usize) -> Vec<f32> {
    if data.is_empty() || window_size == 0 {
        return data.to_vec();
    }
    let half = window_size / 2;
    let n = data.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let lo = i.saturating_sub(half);
        let hi = (i + half + 1).min(n);
        let mut window: Vec<f32> = data[lo..hi].to_vec();
        window.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = window.len() / 2;
        out.push(if window.len().is_multiple_of(2) {
            (window[mid - 1] + window[mid]) / 2.0
        } else {
            window[mid]
        });
    }
    out
}

/// Sliding window median filter state for streaming data.
pub struct MedianFilter {
    window: std::collections::VecDeque<f32>,
    size: usize,
}

/// Construct a new MedianFilter with given window size.
pub fn new_median_filter(window_size: usize) -> MedianFilter {
    MedianFilter {
        window: std::collections::VecDeque::new(),
        size: window_size.max(1),
    }
}

impl MedianFilter {
    /// Push a new value and return the current median.
    pub fn push(&mut self, x: f32) -> f32 {
        self.window.push_back(x);
        if self.window.len() > self.size {
            self.window.pop_front();
        }
        self.current_median()
    }

    /// Compute current median of the window.
    pub fn current_median(&self) -> f32 {
        if self.window.is_empty() {
            return 0.0;
        }
        let mut v: Vec<f32> = self.window.iter().copied().collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = v.len() / 2;
        if v.len().is_multiple_of(2) {
            (v[mid - 1] + v[mid]) / 2.0
        } else {
            v[mid]
        }
    }

    /// Window fill fraction.
    pub fn fill_fraction(&self) -> f32 {
        self.window.len() as f32 / self.size as f32
    }

    /// Number of values in the window.
    pub fn window_len(&self) -> usize {
        self.window.len()
    }

    /// Reset the filter.
    pub fn reset(&mut self) {
        self.window.clear();
    }
}

/// Compute the median of a slice.
pub fn slice_median(data: &[f32]) -> Option<f32> {
    if data.is_empty() {
        return None;
    }
    let mut v = data.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    Some(if v.len().is_multiple_of(2) {
        (v[mid - 1] + v[mid]) / 2.0
    } else {
        v[mid]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_filter_odd_window() {
        /* median filter with window 3 on [1,2,3,4,5] passes constant ramp */
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let out = median_filter_1d(&data, 3);
        assert_eq!(out.len(), 5);
        /* middle element of sorted ramp is 3 */
        assert!((out[2] - 3.0).abs() < 1e-5, "out[2]={}", out[2]);
    }

    #[test]
    fn test_median_filter_empty() {
        /* empty input returns empty */
        assert!(median_filter_1d(&[], 3).is_empty());
    }

    #[test]
    fn test_median_filter_preserves_length() {
        /* output has same length as input */
        let data = vec![1.0f32; 10];
        assert_eq!(median_filter_1d(&data, 3).len(), 10);
    }

    #[test]
    fn test_streaming_push() {
        /* streaming push returns reasonable median */
        let mut mf = new_median_filter(3);
        mf.push(1.0);
        mf.push(5.0);
        let med = mf.push(3.0);
        assert!((med - 3.0).abs() < 1e-5, "med={med}");
    }

    #[test]
    fn test_slice_median_odd() {
        /* slice_median of [1,2,3,4,5] = 3 */
        assert!(
            (slice_median(&[1.0, 2.0, 3.0, 4.0, 5.0]).expect("should succeed") - 3.0).abs() < 1e-5
        );
    }

    #[test]
    fn test_slice_median_even() {
        /* slice_median of [1,2,3,4] = 2.5 */
        assert!((slice_median(&[1.0, 2.0, 3.0, 4.0]).expect("should succeed") - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_slice_median_none_empty() {
        /* slice_median of empty is None */
        assert!(slice_median(&[]).is_none());
    }

    #[test]
    fn test_reset() {
        /* reset clears the window */
        let mut mf = new_median_filter(5);
        mf.push(1.0);
        mf.push(2.0);
        mf.reset();
        assert_eq!(mf.window_len(), 0);
    }

    #[test]
    fn test_window_len_capped() {
        /* window len does not exceed size */
        let mut mf = new_median_filter(3);
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
            mf.push(v);
        }
        assert_eq!(mf.window_len(), 3);
    }
}
