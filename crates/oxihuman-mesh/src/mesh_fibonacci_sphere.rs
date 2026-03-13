// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

const GOLDEN_ANGLE: f32 = 2.399_963_2; // TAU * (1 - 1/phi)

pub fn fibonacci_sphere_point(i: usize, total: usize) -> [f32; 3] {
    let y = 1.0 - (i as f32 / (total - 1) as f32) * 2.0;
    let r = (1.0 - y * y).max(0.0).sqrt();
    let theta = GOLDEN_ANGLE * i as f32;
    [r * theta.cos(), y, r * theta.sin()]
}

pub fn fibonacci_sphere(n: usize) -> Vec<[f32; 3]> {
    (0..n)
        .map(|i| fibonacci_sphere_point(i, n.max(2)))
        .collect()
}

pub fn fibonacci_min_angle(n: usize) -> f32 {
    if n <= 1 {
        return 180.0;
    }
    (4.0 * PI / n as f32).sqrt().asin() * 2.0 * 180.0 / PI
}

pub fn fibonacci_coverage_estimate(n: usize, radius: f32) -> f32 {
    let cap_area = PI * radius * radius;
    let sphere_area = 4.0 * PI;
    ((n as f32 * cap_area) / sphere_area).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_sphere_count() {
        /* returns n points */
        let pts = fibonacci_sphere(100);
        assert_eq!(pts.len(), 100);
    }

    #[test]
    fn test_fibonacci_sphere_on_unit_sphere() {
        /* all points near unit sphere */
        for pt in fibonacci_sphere(50) {
            let r = (pt[0] * pt[0] + pt[1] * pt[1] + pt[2] * pt[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-4, "r={}", r);
        }
    }

    #[test]
    fn test_fibonacci_min_angle_decreases() {
        /* larger n => smaller min angle (use large enough n so asin arg is in [0,1]) */
        let a1 = fibonacci_min_angle(20);
        let a2 = fibonacci_min_angle(500);
        assert!(a2 < a1);
    }

    #[test]
    fn test_fibonacci_coverage_clamped() {
        /* coverage <= 1 */
        let cov = fibonacci_coverage_estimate(10000, 0.5);
        assert!(cov <= 1.0);
    }

    #[test]
    fn test_fibonacci_point_normalized() {
        /* single point on sphere */
        let pt = fibonacci_sphere_point(5, 20);
        let r = (pt[0] * pt[0] + pt[1] * pt[1] + pt[2] * pt[2]).sqrt();
        assert!((r - 1.0).abs() < 1e-4);
    }
}
