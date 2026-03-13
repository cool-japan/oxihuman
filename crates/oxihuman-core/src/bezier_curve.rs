// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bezier curve evaluation utilities.

/// Evaluate a quadratic Bezier curve at parameter t in [0, 1].
/// p0, p1, p2 are the three control points (f32 scalars).
pub fn bezier_quadratic(p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

/// Evaluate a cubic Bezier curve at parameter t in [0, 1].
pub fn bezier_cubic(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// Evaluate a quadratic Bezier curve at t for 2D control points.
pub fn bezier_quadratic_2d(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], t: f32) -> [f32; 2] {
    [
        bezier_quadratic(p0[0], p1[0], p2[0], t),
        bezier_quadratic(p0[1], p1[1], p2[1], t),
    ]
}

/// Evaluate a cubic Bezier curve at t for 2D control points.
pub fn bezier_cubic_2d(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    [
        bezier_cubic(p0[0], p1[0], p2[0], p3[0], t),
        bezier_cubic(p0[1], p1[1], p2[1], p3[1], t),
    ]
}

/// Tangent (derivative) of a cubic Bezier curve at parameter t.
pub fn bezier_cubic_tangent(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * (p1 - p0) + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

/// Sample a cubic Bezier curve into `n` evenly-spaced points in t ∈ [0, 1].
pub fn bezier_cubic_sample(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    n: usize,
) -> Vec<[f32; 2]> {
    if n == 0 {
        return vec![];
    }
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1).max(1) as f32;
            bezier_cubic_2d(p0, p1, p2, p3, t)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_endpoints() {
        /* t=0 -> p0, t=1 -> p2 */
        assert!((bezier_quadratic(1.0, 5.0, 3.0, 0.0) - 1.0).abs() < 1e-6);
        assert!((bezier_quadratic(1.0, 5.0, 3.0, 1.0) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_endpoints() {
        /* t=0 -> p0, t=1 -> p3 */
        assert!((bezier_cubic(0.0, 1.0, 2.0, 3.0, 0.0) - 0.0).abs() < 1e-6);
        assert!((bezier_cubic(0.0, 1.0, 2.0, 3.0, 1.0) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_midpoint_linear() {
        /* for collinear control points bezier_cubic mid = mid value */
        let mid = bezier_cubic(0.0, 1.0, 2.0, 3.0, 0.5);
        assert!((mid - 1.5).abs() < 1e-4);
    }

    #[test]
    fn test_quadratic_2d_endpoints() {
        /* start and end match control points */
        let p = bezier_quadratic_2d([0.0, 0.0], [1.0, 2.0], [2.0, 0.0], 0.0);
        assert!((p[0] - 0.0).abs() < 1e-6);
        let p = bezier_quadratic_2d([0.0, 0.0], [1.0, 2.0], [2.0, 0.0], 1.0);
        assert!((p[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_2d_endpoints() {
        /* first and last sample match p0 and p3 */
        let pts = bezier_cubic_sample([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0], 3);
        assert!((pts[0][0]).abs() < 1e-5);
        assert!((pts[2][0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_cubic_tangent_at_endpoints() {
        /* tangent at t=0 is proportional to p1-p0 */
        let tan = bezier_cubic_tangent(0.0, 1.0, 2.0, 3.0, 0.0);
        assert!(tan > 0.0);
    }

    #[test]
    fn test_sample_count() {
        /* returns exactly n points */
        let pts = bezier_cubic_sample([0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], 10);
        assert_eq!(pts.len(), 10);
    }
}
