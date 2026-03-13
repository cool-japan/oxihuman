// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D convex hull computation using the Graham scan algorithm.

#![allow(dead_code)]

/// Compute the 2D convex hull of a set of points using Graham scan.
/// Returns the hull vertices in counter-clockwise order.
#[allow(dead_code)]
pub fn convex_hull_2d(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    if n == 2 {
        return points.to_vec();
    }
    // Find the lowest (then leftmost) point as pivot.
    let pivot_idx = points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a[1].partial_cmp(&b[1])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal))
        })
        .map(|(i, _)| i)
        .unwrap_or(0);
    // Sort by polar angle w.r.t. pivot, break ties by distance.
    let mut pts: Vec<[f32; 2]> = points.to_vec();
    pts.swap(0, pivot_idx);
    let p0 = pts[0];

    pts[1..].sort_by(|a, b| {
        let ax = a[0] - p0[0];
        let ay = a[1] - p0[1];
        let bx = b[0] - p0[0];
        let by = b[1] - p0[1];
        let cross = ax * by - ay * bx;
        if cross.abs() < 1e-10 {
            let da = ax * ax + ay * ay;
            let db = bx * bx + by * by;
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        } else if cross > 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    // Graham scan.
    let mut hull: Vec<[f32; 2]> = Vec::with_capacity(n);
    for &p in &pts {
        while hull.len() >= 2 {
            let n2 = hull.len();
            let o = hull[n2 - 2];
            let a = hull[n2 - 1];
            let cross = (a[0] - o[0]) * (p[1] - o[1]) - (a[1] - o[1]) * (p[0] - o[0]);
            if cross <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(p);
    }
    hull
}

/// Compute the area of a convex hull (or any polygon) using the shoelace formula.
#[allow(dead_code)]
pub fn hull_area(hull: &[[f32; 2]]) -> f32 {
    let n = hull.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += hull[i][0] * hull[j][1];
        area -= hull[j][0] * hull[i][1];
    }
    (area * 0.5).abs()
}

/// Compute the perimeter of a convex hull.
#[allow(dead_code)]
pub fn hull_perimeter(hull: &[[f32; 2]]) -> f32 {
    let n = hull.len();
    if n < 2 {
        return 0.0;
    }
    let mut peri = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = hull[j][0] - hull[i][0];
        let dy = hull[j][1] - hull[i][1];
        peri += (dx * dx + dy * dy).sqrt();
    }
    peri
}

/// Check if a point is inside a convex hull.
#[allow(dead_code)]
pub fn hull_contains_point(hull: &[[f32; 2]], p: [f32; 2]) -> bool {
    let n = hull.len();
    if n < 3 {
        return false;
    }
    for i in 0..n {
        let a = hull[i];
        let b = hull[(i + 1) % n];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        if cross < 0.0 {
            return false;
        }
    }
    true
}

/// Centroid of a convex hull.
#[allow(dead_code)]
pub fn hull_centroid(hull: &[[f32; 2]]) -> Option<[f32; 2]> {
    let n = hull.len();
    if n == 0 {
        return None;
    }
    let sum = hull
        .iter()
        .fold([0.0f32; 2], |acc, v| [acc[0] + v[0], acc[1] + v[1]]);
    Some([sum[0] / n as f32, sum[1] / n as f32])
}

/// Return the farthest two points of the hull (diameter endpoints).
#[allow(dead_code)]
pub fn hull_diameter(hull: &[[f32; 2]]) -> f32 {
    let n = hull.len();
    let mut max_dist = 0.0f32;
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = hull[j][0] - hull[i][0];
            let dy = hull[j][1] - hull[i][1];
            let d = dx * dx + dy * dy;
            if d > max_dist {
                max_dist = d;
            }
        }
    }
    max_dist.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let hull = convex_hull_2d(&[]);
        assert!(hull.is_empty());
    }

    #[test]
    fn test_single_point() {
        let hull = convex_hull_2d(&[[1.0, 2.0]]);
        assert_eq!(hull.len(), 1);
    }

    #[test]
    fn test_square_hull() {
        let pts = vec![
            [0.0f32, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [0.5, 0.5], // interior point
        ];
        let hull = convex_hull_2d(&pts);
        assert_eq!(hull.len(), 4);
        let area = hull_area(&hull);
        assert!((area - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hull_area_triangle() {
        let hull = vec![[0.0f32, 0.0], [4.0, 0.0], [0.0, 3.0]];
        let area = hull_area(&hull);
        assert!((area - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_hull_perimeter() {
        let hull = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let peri = hull_perimeter(&hull);
        assert!((peri - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_hull_contains_point_inside() {
        let hull = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(hull_contains_point(&hull, [0.5, 0.5]));
    }

    #[test]
    fn test_hull_contains_point_outside() {
        let hull = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(!hull_contains_point(&hull, [2.0, 0.5]));
    }

    #[test]
    fn test_hull_centroid() {
        let hull = vec![[0.0f32, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        let c = hull_centroid(&hull).expect("should succeed");
        assert!((c[0] - 1.0).abs() < 0.01);
        assert!((c[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hull_diameter() {
        let hull = vec![[0.0f32, 0.0], [3.0, 0.0], [3.0, 4.0], [0.0, 4.0]];
        let d = hull_diameter(&hull);
        assert!(d > 4.9 && d < 5.1);
    }

    #[test]
    fn test_collinear_points() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let hull = convex_hull_2d(&pts);
        assert!(!hull.is_empty());
    }
}
