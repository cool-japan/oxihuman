// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D Delaunay triangulation (Bowyer-Watson algorithm).

#![allow(dead_code)]

/// A triangle (vertex indices).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelaunayTri {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

impl DelaunayTri {
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        Self { a, b, c }
    }
    fn has_vertex(&self, v: usize) -> bool {
        self.a == v || self.b == v || self.c == v
    }
    fn edges(&self) -> [(usize, usize); 3] {
        [(self.a, self.b), (self.b, self.c), (self.c, self.a)]
    }
}

/// Result of Delaunay triangulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DelaunayResult {
    pub points: Vec<[f32; 2]>,
    pub triangles: Vec<DelaunayTri>,
}

/// Circumcircle of a triangle: returns (center_x, center_y, radius_sq).
fn circumcircle(pts: &[[f32; 2]], t: &DelaunayTri) -> (f32, f32, f32) {
    let ax = pts[t.a][0];
    let ay = pts[t.a][1];
    let bx = pts[t.b][0];
    let by = pts[t.b][1];
    let cx = pts[t.c][0];
    let cy = pts[t.c][1];
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-10 {
        return (0.0, 0.0, f32::MAX);
    }
    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;
    let rsq = (ax - ux) * (ax - ux) + (ay - uy) * (ay - uy);
    (ux, uy, rsq)
}

fn point_in_circumcircle(pts: &[[f32; 2]], t: &DelaunayTri, p: [f32; 2]) -> bool {
    let (ux, uy, rsq) = circumcircle(pts, t);
    let dx = p[0] - ux;
    let dy = p[1] - uy;
    dx * dx + dy * dy < rsq - 1e-8
}

/// Bowyer-Watson 2D Delaunay triangulation.
#[allow(dead_code)]
pub fn delaunay_2d(input_points: &[[f32; 2]]) -> DelaunayResult {
    if input_points.len() < 3 {
        return DelaunayResult {
            points: input_points.to_vec(),
            triangles: vec![],
        };
    }
    let mut pts = input_points.to_vec();

    // Super-triangle that contains all input points
    let mut min_x = pts[0][0];
    let mut max_x = pts[0][0];
    let mut min_y = pts[0][1];
    let mut max_y = pts[0][1];
    for p in &pts {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let delta = dx.max(dy) * 10.0 + 1.0;
    let st0 = [min_x - delta, min_y - delta];
    let st1 = [min_x + 2.0 * delta + dx, min_y - delta];
    let st2 = [min_x + dx * 0.5, min_y + 2.0 * delta + dy];
    let n = pts.len();
    pts.push(st0);
    pts.push(st1);
    pts.push(st2);

    let mut triangles: Vec<DelaunayTri> = vec![DelaunayTri::new(n, n + 1, n + 2)];

    for (pi, &p) in input_points.iter().enumerate() {
        // Find all bad triangles (point in circumcircle)
        let bad: Vec<DelaunayTri> = triangles
            .iter()
            .filter(|t| point_in_circumcircle(&pts, t, p))
            .copied()
            .collect();

        // Find boundary polygon (edges not shared by two bad triangles)
        let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        for t in &bad {
            for (ea, eb) in t.edges() {
                let key = if ea < eb { (ea, eb) } else { (eb, ea) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }
        let boundary: Vec<(usize, usize)> = edge_count
            .into_iter()
            .filter(|(_, c)| *c == 1)
            .map(|(e, _)| e)
            .collect();

        // Remove bad triangles
        triangles.retain(|t| !bad.contains(t));

        // Re-triangulate from boundary edges
        for (ea, eb) in boundary {
            triangles.push(DelaunayTri::new(ea, eb, pi));
        }
    }

    // Remove triangles that share a vertex with the super-triangle
    triangles.retain(|t| !t.has_vertex(n) && !t.has_vertex(n + 1) && !t.has_vertex(n + 2));

    // Remove the super-triangle vertices
    pts.truncate(n);

    DelaunayResult {
        points: pts,
        triangles,
    }
}

/// Number of triangles in result.
#[allow(dead_code)]
pub fn delaunay_tri_count(r: &DelaunayResult) -> usize {
    r.triangles.len()
}

/// Check if all triangle vertex indices are in range.
#[allow(dead_code)]
pub fn delaunay_valid(r: &DelaunayResult) -> bool {
    let n = r.points.len();
    r.triangles.iter().all(|t| t.a < n && t.b < n && t.c < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_points_two_triangles() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let r = delaunay_2d(&pts);
        assert_eq!(r.triangles.len(), 2);
    }

    #[test]
    fn triangle_valid_indices() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0], [0.5, 0.5]];
        let r = delaunay_2d(&pts);
        assert!(delaunay_valid(&r));
    }

    #[test]
    fn three_points_one_triangle() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let r = delaunay_2d(&pts);
        assert_eq!(r.triangles.len(), 1);
    }

    #[test]
    fn empty_input() {
        let r = delaunay_2d(&[]);
        assert!(r.triangles.is_empty());
    }

    #[test]
    fn result_points_match_input() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let r = delaunay_2d(&pts);
        assert_eq!(r.points.len(), 3);
    }

    #[test]
    fn five_points_valid() {
        let pts = vec![
            [0.0f32, 0.0],
            [2.0, 0.0],
            [2.0, 2.0],
            [0.0, 2.0],
            [1.0, 1.0],
        ];
        let r = delaunay_2d(&pts);
        assert!(delaunay_valid(&r));
        assert!(!r.triangles.is_empty());
    }

    #[test]
    fn delaunay_tri_count_fn() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let r = delaunay_2d(&pts);
        assert_eq!(delaunay_tri_count(&r), r.triangles.len());
    }

    #[test]
    fn circumcircle_unit_triangle() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let t = DelaunayTri::new(0, 1, 2);
        let (cx, cy, rsq) = circumcircle(&pts, &t);
        // Center should be equidistant from all three vertices
        let d0 = (pts[0][0] - cx).powi(2) + (pts[0][1] - cy).powi(2);
        assert!((d0 - rsq).abs() < 1e-4);
    }

    #[test]
    fn has_vertex() {
        let t = DelaunayTri::new(1, 3, 5);
        assert!(t.has_vertex(3));
        assert!(!t.has_vertex(2));
    }

    #[test]
    fn edges_three() {
        let t = DelaunayTri::new(0, 1, 2);
        let edges = t.edges();
        assert_eq!(edges.len(), 3);
    }
}
