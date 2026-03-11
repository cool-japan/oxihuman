// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fit primitive shapes to point clouds.

#![allow(dead_code)]

/// Computes the centroid of a point cloud.
#[allow(dead_code)]
pub fn centroid(points: &[[f32; 3]]) -> [f32; 3] {
    if points.is_empty() {
        return [0.0; 3];
    }
    let n = points.len() as f32;
    let mut sum = [0.0f32; 3];
    for p in points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Computes the maximum distance from any point to the given center.
#[allow(dead_code)]
pub fn point_cloud_radius(points: &[[f32; 3]], center: [f32; 3]) -> f32 {
    points.iter().fold(0.0f32, |max_r, p| {
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        max_r.max(r)
    })
}

/// Fits a bounding sphere to a point cloud. Returns `(center, radius)`.
#[allow(dead_code)]
pub fn fit_sphere(points: &[[f32; 3]]) -> ([f32; 3], f32) {
    if points.is_empty() {
        return ([0.0; 3], 0.0);
    }
    let c = centroid(points);
    let r = point_cloud_radius(points, c);
    (c, r)
}

/// Fits an axis-aligned bounding box (AABB) to a point cloud.
/// Returns `(min_corner, max_corner)`.
#[allow(dead_code)]
pub fn fit_aabb(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = [f32::INFINITY; 3];
    let mut mx = [f32::NEG_INFINITY; 3];
    for p in points {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    (mn, mx)
}

/// Approximate oriented bounding box fit via PCA-like diagonal.
/// Returns `(center, half_extents, axes_diagonal)`.
#[allow(dead_code)]
pub fn fit_obb_approx(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let (mn, mx) = fit_aabb(points);
    let center = [
        (mn[0] + mx[0]) * 0.5,
        (mn[1] + mx[1]) * 0.5,
        (mn[2] + mx[2]) * 0.5,
    ];
    let half = [
        (mx[0] - mn[0]) * 0.5,
        (mx[1] - mn[1]) * 0.5,
        (mx[2] - mn[2]) * 0.5,
    ];
    let diag_len = (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt();
    let axis = if diag_len > 1e-9 {
        [half[0] / diag_len, half[1] / diag_len, half[2] / diag_len]
    } else {
        [1.0, 0.0, 0.0]
    };
    (center, half, axis)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    #[test]
    fn test_centroid_empty() {
        let c = centroid(&[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_centroid_single() {
        let c = centroid(&[[1.0, 2.0, 3.0]]);
        assert!((c[0] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_centroid_symmetric() {
        let pts = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let c = centroid(&pts);
        assert!(c[0].abs() < EPS);
    }

    #[test]
    fn test_fit_sphere_empty() {
        let (c, r) = fit_sphere(&[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
        assert!(r.abs() < EPS);
    }

    #[test]
    fn test_fit_sphere_unit() {
        let pts = [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0]];
        let (c, r) = fit_sphere(&pts);
        assert!(c[0].abs() < EPS);
        assert!((r - 1.0).abs() < EPS);
    }

    #[test]
    fn test_fit_aabb_empty() {
        let (mn, mx) = fit_aabb(&[]);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_fit_aabb_basic() {
        let pts = [[0.0, 0.0, 0.0], [2.0, 3.0, 4.0]];
        let (mn, mx) = fit_aabb(&pts);
        assert!((mn[0]).abs() < EPS);
        assert!((mx[2] - 4.0).abs() < EPS);
    }

    #[test]
    fn test_fit_obb_approx_basic() {
        let pts = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let (center, half, _) = fit_obb_approx(&pts);
        assert!((center[0] - 1.0).abs() < EPS);
        assert!((half[0] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_point_cloud_radius_unit() {
        let pts = [[1.0, 0.0, 0.0]];
        let r = point_cloud_radius(&pts, [0.0, 0.0, 0.0]);
        assert!((r - 1.0).abs() < EPS);
    }
}
