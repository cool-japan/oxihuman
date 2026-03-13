// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Triangular grid with barycentric coordinate lookup.

#![allow(dead_code)]

/// A triangle defined by three 2D vertices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Triangle2D {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub c: [f32; 2],
}

#[allow(dead_code)]
impl Triangle2D {
    pub fn new(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> Self {
        Self { a, b, c }
    }

    /// Compute barycentric coordinates (u, v, w) for point p.
    /// Returns None if the triangle is degenerate.
    pub fn barycentric(&self, p: [f32; 2]) -> Option<[f32; 3]> {
        let v0 = [self.b[0] - self.a[0], self.b[1] - self.a[1]];
        let v1 = [self.c[0] - self.a[0], self.c[1] - self.a[1]];
        let v2 = [p[0] - self.a[0], p[1] - self.a[1]];
        let d00 = dot2(v0, v0);
        let d01 = dot2(v0, v1);
        let d11 = dot2(v1, v1);
        let d20 = dot2(v2, v0);
        let d21 = dot2(v2, v1);
        let denom = d00 * d11 - d01 * d01;
        if denom.abs() < 1e-10 {
            return None;
        }
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;
        Some([u, v, w])
    }

    /// Test if point p is inside the triangle.
    pub fn contains(&self, p: [f32; 2]) -> bool {
        match self.barycentric(p) {
            None => false,
            Some([u, v, w]) => u >= 0.0 && v >= 0.0 && w >= 0.0,
        }
    }

    /// Interpolate a scalar value at barycentric coords (u, v, w).
    pub fn interpolate(&self, values: [f32; 3], bary: [f32; 3]) -> f32 {
        values[0] * bary[0] + values[1] * bary[1] + values[2] * bary[2]
    }

    /// Area of the triangle (positive for CCW).
    pub fn signed_area(&self) -> f32 {
        let v0 = [self.b[0] - self.a[0], self.b[1] - self.a[1]];
        let v1 = [self.c[0] - self.a[0], self.c[1] - self.a[1]];
        0.5 * (v0[0] * v1[1] - v0[1] * v1[0])
    }

    pub fn area(&self) -> f32 {
        self.signed_area().abs()
    }

    /// Centroid of the triangle.
    pub fn centroid(&self) -> [f32; 2] {
        [
            (self.a[0] + self.b[0] + self.c[0]) / 3.0,
            (self.a[1] + self.b[1] + self.c[1]) / 3.0,
        ]
    }
}

fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// A uniform triangular grid covering a rectangle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriGrid {
    pub origin: [f32; 2],
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
}

#[allow(dead_code)]
impl TriGrid {
    pub fn new(origin: [f32; 2], cell_size: f32, cols: usize, rows: usize) -> Self {
        Self {
            origin,
            cell_size,
            cols,
            rows,
        }
    }

    /// Get the two triangles covering cell (col, row).
    pub fn cell_triangles(&self, col: usize, row: usize) -> [Triangle2D; 2] {
        let s = self.cell_size;
        let ox = self.origin[0] + col as f32 * s;
        let oy = self.origin[1] + row as f32 * s;
        let a = [ox, oy];
        let b = [ox + s, oy];
        let c = [ox + s, oy + s];
        let d = [ox, oy + s];
        [Triangle2D::new(a, b, d), Triangle2D::new(b, c, d)]
    }

    /// Find which triangle in the grid contains point p.
    pub fn find_triangle(&self, p: [f32; 2]) -> Option<(usize, usize, usize)> {
        let s = self.cell_size;
        let col = ((p[0] - self.origin[0]) / s).floor() as i32;
        let row = ((p[1] - self.origin[1]) / s).floor() as i32;
        if col < 0 || row < 0 || col as usize >= self.cols || row as usize >= self.rows {
            return None;
        }
        let tris = self.cell_triangles(col as usize, row as usize);
        for (i, tri) in tris.iter().enumerate() {
            if tri.contains(p) {
                return Some((col as usize, row as usize, i));
            }
        }
        None
    }

    /// Total number of triangles in the grid.
    pub fn triangle_count(&self) -> usize {
        self.cols * self.rows * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri() -> Triangle2D {
        Triangle2D::new([0.0, 0.0], [1.0, 0.0], [0.0, 1.0])
    }

    #[test]
    fn centroid_inside() {
        let t = unit_tri();
        let c = t.centroid();
        assert!(t.contains(c));
    }

    #[test]
    fn barycentric_vertex_a() {
        let t = unit_tri();
        let [u, v, w] = t.barycentric([0.0, 0.0]).expect("should succeed");
        assert!((u - 1.0).abs() < 1e-5);
        assert!(v.abs() < 1e-5);
        assert!(w.abs() < 1e-5);
    }

    #[test]
    fn point_outside_not_contained() {
        let t = unit_tri();
        assert!(!t.contains([1.0, 1.0]));
    }

    #[test]
    fn area_unit_tri() {
        let t = unit_tri();
        assert!((t.area() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn interpolate_at_vertex_a() {
        let t = unit_tri();
        let bary = t.barycentric([0.0, 0.0]).expect("should succeed");
        let val = t.interpolate([3.0, 0.0, 0.0], bary);
        assert!((val - 3.0).abs() < 1e-4);
    }

    #[test]
    fn grid_triangle_count() {
        let g = TriGrid::new([0.0, 0.0], 1.0, 4, 3);
        assert_eq!(g.triangle_count(), 24);
    }

    #[test]
    fn find_triangle_origin() {
        let g = TriGrid::new([0.0, 0.0], 1.0, 4, 4);
        assert!(g.find_triangle([0.1, 0.1]).is_some());
    }

    #[test]
    fn find_triangle_outside_returns_none() {
        let g = TriGrid::new([0.0, 0.0], 1.0, 4, 4);
        assert!(g.find_triangle([-0.1, 0.5]).is_none());
    }

    #[test]
    fn degenerate_barycentric_is_none() {
        let t = Triangle2D::new([0.0, 0.0], [1.0, 0.0], [2.0, 0.0]);
        assert!(t.barycentric([0.5, 0.0]).is_none());
    }

    #[test]
    fn signed_area_ccw_positive() {
        let t = unit_tri();
        assert!(t.signed_area() > 0.0);
    }
}
