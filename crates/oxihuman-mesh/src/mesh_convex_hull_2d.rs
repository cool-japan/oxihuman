#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D convex hull on XZ plane (for footprint computation).

#[allow(dead_code)]
pub fn cross_2d(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

#[allow(dead_code)]
pub fn convex_hull_2d(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    let mut pts = points.to_vec();
    pts.sort_by(|a, b| {
        a[0].partial_cmp(&b[0])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal))
    });
    let mut hull: Vec<[f32; 2]> = Vec::with_capacity(2 * n);
    // lower hull
    for &p in &pts {
        while hull.len() >= 2 && cross_2d(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0 {
            hull.pop();
        }
        hull.push(p);
    }
    // upper hull
    let lower_len = hull.len() + 1;
    for &p in pts.iter().rev() {
        while hull.len() >= lower_len
            && cross_2d(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
        {
            hull.pop();
        }
        hull.push(p);
    }
    hull.pop();
    hull
}

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
    area.abs() * 0.5
}

#[allow(dead_code)]
pub fn hull_perimeter(hull: &[[f32; 2]]) -> f32 {
    let n = hull.len();
    if n < 2 {
        return 0.0;
    }
    let mut perim = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = hull[j][0] - hull[i][0];
        let dy = hull[j][1] - hull[i][1];
        perim += (dx * dx + dy * dy).sqrt();
    }
    perim
}

#[allow(dead_code)]
pub fn point_in_hull(pt: [f32; 2], hull: &[[f32; 2]]) -> bool {
    let n = hull.len();
    if n < 3 {
        return false;
    }
    for i in 0..n {
        let j = (i + 1) % n;
        if cross_2d(hull[i], hull[j], pt) < 0.0 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_pts() -> Vec<[f32; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    #[test]
    fn hull_of_square_has_4_points() {
        let pts = square_pts();
        let h = convex_hull_2d(&pts);
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn hull_empty() {
        let h = convex_hull_2d(&[]);
        assert!(h.is_empty());
    }

    #[test]
    fn hull_area_square() {
        let pts = square_pts();
        let h = convex_hull_2d(&pts);
        let area = hull_area(&h);
        assert!((area - 1.0).abs() < 1e-5);
    }

    #[test]
    fn hull_perimeter_square() {
        let pts = square_pts();
        let h = convex_hull_2d(&pts);
        let perim = hull_perimeter(&h);
        assert!((perim - 4.0).abs() < 1e-5);
    }

    #[test]
    fn point_inside_hull() {
        let pts = square_pts();
        let h = convex_hull_2d(&pts);
        assert!(point_in_hull([0.5, 0.5], &h));
    }

    #[test]
    fn point_outside_hull() {
        let pts = square_pts();
        let h = convex_hull_2d(&pts);
        assert!(!point_in_hull([2.0, 2.0], &h));
    }

    #[test]
    fn cross_2d_ccw_positive() {
        let c = cross_2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!(c > 0.0);
    }

    #[test]
    fn hull_with_interior_point_excluded() {
        let mut pts = square_pts();
        pts.push([0.5, 0.5]); // interior point
        let h = convex_hull_2d(&pts);
        assert_eq!(h.len(), 4);
    }
}
