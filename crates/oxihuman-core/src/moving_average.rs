// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple, exponential, and weighted moving averages.

use std::collections::VecDeque;

/// Simple Moving Average (SMA) over a sliding window.
pub struct SimpleMovingAverage {
    window: VecDeque<f64>,
    size: usize,
    sum: f64,
}

/// Construct a new SimpleMovingAverage.
pub fn new_sma(window_size: usize) -> SimpleMovingAverage {
    SimpleMovingAverage {
        window: VecDeque::new(),
        size: window_size.max(1),
        sum: 0.0,
    }
}

impl SimpleMovingAverage {
    /// Push a value and return the current SMA.
    pub fn push(&mut self, x: f64) -> f64 {
        self.window.push_back(x);
        self.sum += x;
        if self.window.len() > self.size {
            self.sum -= self.window.pop_front().unwrap_or(0.0);
        }
        self.current()
    }

    /// Current SMA value.
    pub fn current(&self) -> f64 {
        if self.window.is_empty() {
            0.0
        } else {
            self.sum / self.window.len() as f64
        }
    }

    /// Number of values in the window.
    pub fn len(&self) -> usize {
        self.window.len()
    }

    /// Whether the window is empty.
    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    /// Whether the window is full.
    pub fn is_full(&self) -> bool {
        self.window.len() == self.size
    }

    /// Reset the averager.
    pub fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
    }
}

/// Exponential Moving Average (EMA).
pub struct ExponentialMovingAverage {
    alpha: f64,
    value: Option<f64>,
}

/// Construct a new EMA with smoothing factor `alpha` in (0, 1].
pub fn new_ema(alpha: f64) -> ExponentialMovingAverage {
    let alpha = alpha.clamp(1e-6, 1.0);
    ExponentialMovingAverage { alpha, value: None }
}

impl ExponentialMovingAverage {
    /// Push a value and return the current EMA.
    pub fn push(&mut self, x: f64) -> f64 {
        self.value = Some(match self.value {
            None => x,
            Some(prev) => self.alpha * x + (1.0 - self.alpha) * prev,
        });
        self.value.unwrap_or_default()
    }

    /// Current EMA (None if no values pushed).
    pub fn current(&self) -> Option<f64> {
        self.value
    }

    /// Smoothing factor.
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Reset.
    pub fn reset(&mut self) {
        self.value = None;
    }
}

/// Weighted Moving Average (WMA).
pub struct WeightedMovingAverage {
    window: VecDeque<f64>,
    size: usize,
}

/// Construct a new WMA.
pub fn new_wma(window_size: usize) -> WeightedMovingAverage {
    WeightedMovingAverage {
        window: VecDeque::new(),
        size: window_size.max(1),
    }
}

impl WeightedMovingAverage {
    /// Push a value and return the current WMA.
    pub fn push(&mut self, x: f64) -> f64 {
        self.window.push_back(x);
        if self.window.len() > self.size {
            self.window.pop_front();
        }
        self.current()
    }

    /// Compute the WMA (more recent values have higher weight).
    pub fn current(&self) -> f64 {
        let n = self.window.len();
        if n == 0 {
            return 0.0;
        }
        let denom = (n * (n + 1)) as f64 / 2.0;
        self.window
            .iter()
            .enumerate()
            .map(|(i, &v)| v * (i + 1) as f64)
            .sum::<f64>()
            / denom
    }

    /// Reset.
    pub fn reset(&mut self) {
        self.window.clear();
    }
}

/// Apply SMA to a slice, returning a vector of the same length.
pub fn apply_sma(data: &[f64], window_size: usize) -> Vec<f64> {
    let mut sma = new_sma(window_size);
    data.iter().map(|&x| sma.push(x)).collect()
}

/// Apply EMA to a slice.
pub fn apply_ema(data: &[f64], alpha: f64) -> Vec<f64> {
    let mut ema = new_ema(alpha);
    data.iter().map(|&x| ema.push(x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_constant() {
        /* SMA of constant values equals the constant */
        let mut sma = new_sma(3);
        sma.push(5.0);
        sma.push(5.0);
        let v = sma.push(5.0);
        assert!((v - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_sma_window_full() {
        /* is_full returns true after window_size values */
        let mut sma = new_sma(3);
        sma.push(1.0);
        sma.push(2.0);
        assert!(!sma.is_full());
        sma.push(3.0);
        assert!(sma.is_full());
    }

    #[test]
    fn test_sma_sliding() {
        /* SMA slides: pushing beyond window size drops oldest */
        let mut sma = new_sma(2);
        sma.push(10.0);
        sma.push(20.0);
        let v = sma.push(30.0);
        assert!((v - 25.0).abs() < 1e-12, "v={v}");
    }

    #[test]
    fn test_ema_first_value() {
        /* EMA first value equals the pushed value */
        let mut ema = new_ema(0.5);
        let v = ema.push(10.0);
        assert!((v - 10.0).abs() < 1e-12);
    }

    #[test]
    fn test_ema_smoothing() {
        /* EMA damps large spikes */
        let mut ema = new_ema(0.1);
        for _ in 0..100 {
            ema.push(0.0);
        }
        let v = ema.push(100.0);
        assert!(v < 20.0, "v={v}");
    }

    #[test]
    fn test_wma_current() {
        /* WMA weights recent values more heavily */
        let mut wma = new_wma(3);
        wma.push(1.0);
        wma.push(2.0);
        let v = wma.push(3.0);
        /* weights: 1,2,3 => (1*1 + 2*2 + 3*3)/(1+2+3) = 14/6 ~ 2.33 */
        assert!((v - 14.0 / 6.0).abs() < 1e-10, "v={v}");
    }

    #[test]
    fn test_apply_sma_length() {
        /* apply_sma returns same length as input */
        let data: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let out = apply_sma(&data, 3);
        assert_eq!(out.len(), 10);
    }

    #[test]
    fn test_apply_ema_length() {
        /* apply_ema returns same length as input */
        let data = vec![1.0f64; 5];
        assert_eq!(apply_ema(&data, 0.3).len(), 5);
    }

    #[test]
    fn test_sma_reset() {
        /* reset clears sma state */
        let mut sma = new_sma(3);
        sma.push(1.0);
        sma.reset();
        assert_eq!(sma.len(), 0);
    }
}
