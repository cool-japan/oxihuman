// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Linear trend slope detector using least-squares regression.

#[derive(Debug, Clone, PartialEq)]
pub struct TrendResult {
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
}

impl TrendResult {
    pub fn predict(&self, x: f64) -> f64 {
        self.slope * x + self.intercept
    }

    pub fn is_uptrend(&self) -> bool {
        self.slope > 0.0
    }

    pub fn is_downtrend(&self) -> bool {
        self.slope < 0.0
    }
}

/// Compute linear regression (slope + intercept) using least squares.
pub fn linear_regression(xs: &[f64], ys: &[f64]) -> Option<TrendResult> {
    let n = xs.len().min(ys.len());
    if n < 2 {
        return None;
    }
    let n_f = n as f64;
    let sum_x: f64 = xs[..n].iter().sum();
    let sum_y: f64 = ys[..n].iter().sum();
    let sum_xx: f64 = xs[..n].iter().map(|&x| x * x).sum();
    let sum_xy: f64 = xs[..n].iter().zip(ys[..n].iter()).map(|(x, y)| x * y).sum();
    let denom = n_f * sum_xx - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return None;
    }
    let slope = (n_f * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n_f;
    let mean_y = sum_y / n_f;
    let ss_tot: f64 = ys[..n].iter().map(|&y| (y - mean_y).powi(2)).sum();
    let ss_res: f64 = xs[..n]
        .iter()
        .zip(ys[..n].iter())
        .map(|(&x, &y)| (y - (slope * x + intercept)).powi(2))
        .sum();
    let r_squared = if ss_tot < f64::EPSILON {
        1.0
    } else {
        1.0 - ss_res / ss_tot
    };
    Some(TrendResult {
        slope,
        intercept,
        r_squared,
    })
}

/// Detect trend from uniformly-spaced samples (x = 0..n-1).
pub fn detect_trend(ys: &[f64]) -> Option<TrendResult> {
    let xs: Vec<f64> = (0..ys.len()).map(|i| i as f64).collect();
    linear_regression(&xs, ys)
}

pub fn trend_direction_label(result: &TrendResult) -> &'static str {
    if result.slope > 0.01 {
        "up"
    } else if result.slope < -0.01 {
        "down"
    } else {
        "flat"
    }
}

pub fn moving_slope(ys: &[f64], window: usize) -> Vec<f64> {
    if window < 2 || ys.len() < window {
        return Vec::new();
    }
    ys.windows(window)
        .map(|w| detect_trend(w).map(|r| r.slope).unwrap_or(0.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_line() {
        let xs = vec![0.0, 1.0, 2.0, 3.0];
        let ys = vec![1.0, 3.0, 5.0, 7.0];
        let r = linear_regression(&xs, &ys).unwrap();
        assert!((r.slope - 2.0).abs() < 1e-10 /* slope = 2 */,);
        assert!((r.intercept - 1.0).abs() < 1e-10 /* intercept = 1 */,);
        assert!((r.r_squared - 1.0).abs() < 1e-10 /* perfect fit */,);
    }

    #[test]
    fn test_single_sample_none() {
        let r = linear_regression(&[0.0], &[1.0]);
        assert!(r.is_none() /* need at least 2 points */,);
    }

    #[test]
    fn test_detect_trend_upward() {
        let ys = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = detect_trend(&ys).unwrap();
        assert!(r.is_uptrend() /* increasing series is uptrend */,);
    }

    #[test]
    fn test_detect_trend_downward() {
        let ys = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let r = detect_trend(&ys).unwrap();
        assert!(r.is_downtrend() /* decreasing series is downtrend */,);
    }

    #[test]
    fn test_trend_direction_label() {
        let up = TrendResult {
            slope: 1.0,
            intercept: 0.0,
            r_squared: 1.0,
        };
        assert_eq!(trend_direction_label(&up), "up");
        let down = TrendResult {
            slope: -1.0,
            intercept: 0.0,
            r_squared: 1.0,
        };
        assert_eq!(trend_direction_label(&down), "down");
        let flat = TrendResult {
            slope: 0.001,
            intercept: 0.0,
            r_squared: 0.0,
        };
        assert_eq!(trend_direction_label(&flat), "flat");
    }

    #[test]
    fn test_predict() {
        let r = TrendResult {
            slope: 2.0,
            intercept: 1.0,
            r_squared: 1.0,
        };
        assert!((r.predict(3.0) - 7.0).abs() < 1e-10 /* 2*3+1=7 */,);
    }

    #[test]
    fn test_moving_slope_length() {
        let ys = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let slopes = moving_slope(&ys, 3);
        assert_eq!(slopes.len(), 4 /* 6-3+1=4 windows */,);
    }

    #[test]
    fn test_moving_slope_constant() {
        let ys = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let slopes = moving_slope(&ys, 3);
        for s in &slopes {
            assert!(s.abs() < 1e-10 /* flat line has zero slope */,);
        }
    }

    #[test]
    fn test_r_squared_flat() {
        /* All same values: r² should be 1 since no residuals from mean */
        let ys = vec![3.0, 3.0, 3.0, 3.0];
        let r = detect_trend(&ys).unwrap();
        assert!((r.r_squared - 1.0).abs() < 1e-6 /* constant series */,);
    }
}
