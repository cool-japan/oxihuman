// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Box-counting fractal dimension.

/// Box-counting fractal dimension estimator.
#[derive(Debug, Clone)]
pub struct FractalDimension {
    /// Points in 2D space.
    pub points: Vec<(f64, f64)>,
}

impl FractalDimension {
    pub fn new(points: Vec<(f64, f64)>) -> Self {
        Self { points }
    }

    /// Count occupied boxes at a given box size.
    pub fn box_count(&self, box_size: f64) -> u64 {
        use std::collections::HashSet;
        if self.points.is_empty() || box_size <= 0.0 {
            return 0;
        }
        let mut boxes = HashSet::new();
        for &(x, y) in &self.points {
            let bx = (x / box_size).floor() as i64;
            let by = (y / box_size).floor() as i64;
            boxes.insert((bx, by));
        }
        boxes.len() as u64
    }

    /// Estimate box-counting dimension from a range of scales.
    pub fn estimate_dimension(&self, scales: &[f64]) -> f64 {
        if scales.len() < 2 {
            return 0.0;
        }
        let mut log_n = Vec::with_capacity(scales.len());
        let mut log_r = Vec::with_capacity(scales.len());
        for &s in scales {
            let n = self.box_count(s);
            if n == 0 {
                continue;
            }
            log_n.push((n as f64).ln());
            log_r.push((1.0 / s).ln());
        }
        if log_r.len() < 2 {
            return 0.0;
        }
        linear_slope(&log_r, &log_n)
    }
}

/// Linear regression slope (least-squares).
fn linear_slope(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sxx: f64 = x.iter().map(|xi| xi * xi).sum();
    let denom = n * sxx - sx * sx;
    if denom.abs() < f64::EPSILON {
        return 0.0;
    }
    (n * sxy - sx * sy) / denom
}

pub fn new_fractal_dimension(points: Vec<(f64, f64)>) -> FractalDimension {
    FractalDimension::new(points)
}

pub fn fd_box_count(fd: &FractalDimension, box_size: f64) -> u64 {
    fd.box_count(box_size)
}

pub fn fd_estimate(fd: &FractalDimension, scales: &[f64]) -> f64 {
    fd.estimate_dimension(scales)
}

pub fn fd_point_count(fd: &FractalDimension) -> usize {
    fd.points.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_points(n: usize) -> Vec<(f64, f64)> {
        (0..n).map(|i| (i as f64 * 0.01, 0.0)).collect()
    }

    fn square_points(n: usize) -> Vec<(f64, f64)> {
        let mut pts = Vec::new();
        let side = (n as f64).sqrt() as usize;
        for i in 0..side {
            for j in 0..side {
                pts.push((i as f64 * 0.01, j as f64 * 0.01));
            }
        }
        pts
    }

    #[test]
    fn test_empty_box_count_zero() {
        let fd = new_fractal_dimension(vec![]);
        assert_eq!(fd_box_count(&fd, 1.0), 0);
    }

    #[test]
    fn test_single_point_one_box() {
        let fd = new_fractal_dimension(vec![(0.5, 0.5)]);
        assert_eq!(fd_box_count(&fd, 1.0), 1);
    }

    #[test]
    fn test_two_points_same_box() {
        let fd = new_fractal_dimension(vec![(0.1, 0.1), (0.2, 0.2)]);
        assert_eq!(fd_box_count(&fd, 1.0), 1);
    }

    #[test]
    fn test_two_points_different_boxes() {
        let fd = new_fractal_dimension(vec![(0.0, 0.0), (2.0, 0.0)]);
        assert_eq!(fd_box_count(&fd, 1.0), 2);
    }

    #[test]
    fn test_line_dimension_near_1() {
        /* 1D line should have dimension close to 1 */
        let pts = line_points(100);
        let fd = new_fractal_dimension(pts);
        let scales = vec![0.1, 0.05, 0.02, 0.01];
        let d = fd_estimate(&fd, &scales);
        assert!(d > 0.5 && d < 1.5);
    }

    #[test]
    fn test_square_dimension_near_2() {
        /* 2D filled square should have dimension close to 2 */
        let pts = square_points(400);
        let fd = new_fractal_dimension(pts);
        let scales = vec![0.1, 0.05, 0.02];
        let d = fd_estimate(&fd, &scales);
        assert!(d > 1.5 && d < 2.5);
    }

    #[test]
    fn test_point_count() {
        let pts = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)];
        let fd = new_fractal_dimension(pts);
        assert_eq!(fd_point_count(&fd), 3);
    }

    #[test]
    fn test_zero_box_size_returns_zero() {
        let fd = new_fractal_dimension(vec![(0.0, 0.0)]);
        assert_eq!(fd_box_count(&fd, 0.0), 0);
    }

    #[test]
    fn test_estimate_insufficient_scales() {
        let fd = new_fractal_dimension(vec![(0.0, 0.0)]);
        let d = fd_estimate(&fd, &[0.1]);
        assert_eq!(d, 0.0);
    }
}
