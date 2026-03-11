// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Exponential/simple moving average calculator (enhanced).

/// Simple Moving Average over a sliding window.
#[derive(Debug, Clone)]
pub struct SimpleMaCalc {
    window: Vec<f64>,
    size: usize,
    pos: usize,
    filled: bool,
    sum: f64,
}

impl SimpleMaCalc {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "window size must be positive");
        SimpleMaCalc {
            window: vec![0.0; size],
            size,
            pos: 0,
            filled: false,
            sum: 0.0,
        }
    }

    pub fn update(&mut self, value: f64) -> f64 {
        self.sum -= self.window[self.pos];
        self.window[self.pos] = value;
        self.sum += value;
        self.pos += 1;
        if self.pos >= self.size {
            self.pos = 0;
            self.filled = true;
        }
        self.current()
    }

    pub fn current(&self) -> f64 {
        let count = if self.filled {
            self.size
        } else {
            self.pos.max(1)
        };
        self.sum / count as f64
    }

    pub fn is_ready(&self) -> bool {
        self.filled
    }

    pub fn window_size(&self) -> usize {
        self.size
    }
}

/// Exponential Moving Average.
#[derive(Debug, Clone)]
pub struct EmaCalc {
    alpha: f64,
    value: Option<f64>,
}

impl EmaCalc {
    /// `alpha` is the smoothing factor in [0, 1]. Smaller = more smoothing.
    pub fn new(alpha: f64) -> Self {
        let alpha = alpha.clamp(0.0, 1.0);
        EmaCalc { alpha, value: None }
    }

    /// Create EMA from a period: alpha = 2 / (period + 1).
    pub fn from_period(period: usize) -> Self {
        let alpha = 2.0 / (period as f64 + 1.0);
        EmaCalc::new(alpha)
    }

    pub fn update(&mut self, value: f64) -> f64 {
        let ema = match self.value {
            None => value,
            Some(prev) => self.alpha * value + (1.0 - self.alpha) * prev,
        };
        self.value = Some(ema);
        ema
    }

    pub fn current(&self) -> Option<f64> {
        self.value
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }
}

pub fn sma_batch(data: &[f64], window: usize) -> Vec<f64> {
    let mut ma = SimpleMaCalc::new(window);
    data.iter().map(|&v| ma.update(v)).collect()
}

pub fn ema_batch(data: &[f64], period: usize) -> Vec<f64> {
    let mut ema = EmaCalc::from_period(period);
    data.iter().map(|&v| ema.update(v)).collect()
}

pub fn ma_crossover(fast: &[f64], slow: &[f64]) -> Vec<f64> {
    fast.iter().zip(slow.iter()).map(|(f, s)| f - s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_single() {
        let mut ma = SimpleMaCalc::new(3);
        let v = ma.update(10.0);
        assert!((v - 10.0).abs() < 1e-10, /* single value SMA = itself */);
    }

    #[test]
    fn test_sma_window() {
        let mut ma = SimpleMaCalc::new(3);
        ma.update(1.0);
        ma.update(2.0);
        let v = ma.update(3.0);
        assert!((v - 2.0).abs() < 1e-10 /* (1+2+3)/3 = 2 */,);
    }

    #[test]
    fn test_sma_ready() {
        let mut ma = SimpleMaCalc::new(3);
        ma.update(1.0);
        ma.update(2.0);
        assert!(!ma.is_ready() /* not filled yet */,);
        ma.update(3.0);
        assert!(ma.is_ready() /* now filled */,);
    }

    #[test]
    fn test_ema_first_value() {
        let mut ema = EmaCalc::new(0.5);
        let v = ema.update(100.0);
        assert!((v - 100.0).abs() < 1e-10 /* first EMA = first value */,);
    }

    #[test]
    fn test_ema_smoothing() {
        let mut ema = EmaCalc::new(0.5);
        ema.update(100.0);
        let v = ema.update(0.0);
        assert!((v - 50.0).abs() < 1e-10 /* 0.5*0 + 0.5*100 = 50 */,);
    }

    #[test]
    fn test_ema_from_period() {
        let ema = EmaCalc::from_period(9);
        assert!((ema.alpha() - 0.2).abs() < 1e-10, /* alpha = 2/10 = 0.2 */);
    }

    #[test]
    fn test_sma_batch_length() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma_batch(&data, 3);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_ema_batch_length() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = ema_batch(&data, 3);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_crossover_length() {
        let fast = vec![1.0, 2.0, 3.0];
        let slow = vec![1.5, 1.5, 1.5];
        let cross = ma_crossover(&fast, &slow);
        assert_eq!(cross.len(), 3);
        assert!((cross[2] - 1.5).abs() < 1e-10 /* 3 - 1.5 = 1.5 */,);
    }
}
