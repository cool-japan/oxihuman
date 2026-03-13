// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Curvature-adaptive sizing for remeshing.

use std::f32::consts::FRAC_2_PI;

#[allow(dead_code)]
pub fn ca_principal_curvatures(k1: f32, k2: f32) -> (f32, f32) {
    if k1.abs() >= k2.abs() { (k1, k2) } else { (k2, k1) }
}

#[allow(dead_code)]
pub fn ca_gaussian_curvature(k1: f32, k2: f32) -> f32 {
    k1 * k2
}

#[allow(dead_code)]
pub fn ca_mean_curvature(k1: f32, k2: f32) -> f32 {
    (k1 + k2) / 2.0
}

#[allow(dead_code)]
pub fn ca_adaptive_size(k1: f32, k2: f32, base_size: f32) -> f32 {
    let mean = ca_mean_curvature(k1, k2).abs();
    base_size / (1.0 + mean)
}

#[allow(dead_code)]
pub fn ca_shape_index(k1: f32, k2: f32) -> f32 {
    FRAC_2_PI * ((k1 + k2) / (k1 - k2 + 1e-9)).atan()
}

#[allow(dead_code)]
pub struct CurvatureStats {
    pub k1: f32,
    pub k2: f32,
    pub gaussian: f32,
    pub mean: f32,
}

#[allow(dead_code)]
pub fn ca_stats(k1: f32, k2: f32) -> CurvatureStats {
    CurvatureStats {
        k1,
        k2,
        gaussian: ca_gaussian_curvature(k1, k2),
        mean: ca_mean_curvature(k1, k2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_gaussian_curvature() {
        assert!((ca_gaussian_curvature(2.0, 3.0) - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_mean_curvature() {
        assert!((ca_mean_curvature(2.0, 4.0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_adaptive_size_flat() {
        let s = ca_adaptive_size(0.0, 0.0, 1.0);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_adaptive_size_high_curvature_smaller() {
        let s_low = ca_adaptive_size(0.0, 0.0, 1.0);
        let s_high = ca_adaptive_size(5.0, 5.0, 1.0);
        assert!(s_high < s_low);
    }

    #[test]
    fn test_principal_curvatures_sorted() {
        let (big, small) = ca_principal_curvatures(1.0, 3.0);
        assert!(big.abs() >= small.abs());
    }

    #[test]
    fn test_shape_index_sphere() {
        let si = ca_shape_index(1.0, 1.0);
        assert!((si - 1.0).abs() < 0.01, "Shape index for sphere should be 1, got {si}");
    }

    #[test]
    fn test_stats_values() {
        let s = ca_stats(2.0, 4.0);
        assert!((s.gaussian - 8.0).abs() < 1e-4);
        assert!((s.mean - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_shape_index_saddle() {
        /* For a saddle with k1=1, k2=-1, shape_index should be 0 */
        let si = ca_shape_index(1.0, -1.0);
        let _ = PI; /* suppress unused import */
        assert!(si.abs() < 0.01, "Shape index for saddle should be ~0, got {si}");
    }
}
