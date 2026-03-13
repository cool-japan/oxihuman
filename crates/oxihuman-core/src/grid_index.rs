// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D grid spatial index for fast lookups.

/// A 2D grid spatial index.
pub struct GridIndex {
    cells: Vec<Vec<usize>>,
    width: usize,
    height: usize,
    cell_w: f32,
    cell_h: f32,
    origin_x: f32,
    origin_y: f32,
    points: Vec<[f32; 2]>,
}

impl GridIndex {
    /// Create a new grid index covering [ox, ox+w) x [oy, oy+h) with given cell dimensions.
    pub fn new(
        origin_x: f32,
        origin_y: f32,
        total_w: f32,
        total_h: f32,
        cols: usize,
        rows: usize,
    ) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        GridIndex {
            cells: vec![Vec::new(); cols * rows],
            width: cols,
            height: rows,
            cell_w: total_w / cols as f32,
            cell_h: total_h / rows as f32,
            origin_x,
            origin_y,
            points: Vec::new(),
        }
    }

    fn cell_idx(&self, x: f32, y: f32) -> Option<usize> {
        let ci = ((x - self.origin_x) / self.cell_w).floor() as isize;
        let ri = ((y - self.origin_y) / self.cell_h).floor() as isize;
        if ci < 0 || ri < 0 || ci >= self.width as isize || ri >= self.height as isize {
            return None;
        }
        Some(ri as usize * self.width + ci as usize)
    }

    /// Insert a point and return its ID.
    pub fn insert(&mut self, x: f32, y: f32) -> Option<usize> {
        let idx = self.cell_idx(x, y)?;
        let id = self.points.len();
        self.points.push([x, y]);
        self.cells[idx].push(id);
        Some(id)
    }

    /// Query all point IDs in the cell containing (x, y).
    pub fn query_cell(&self, x: f32, y: f32) -> Vec<usize> {
        if let Some(idx) = self.cell_idx(x, y) {
            self.cells[idx].clone()
        } else {
            vec![]
        }
    }

    /// Query all point IDs within a rectangular region.
    pub fn query_rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<usize> {
        let mut result = Vec::new();
        let ci0 = ((x0 - self.origin_x) / self.cell_w).floor() as isize;
        let ri0 = ((y0 - self.origin_y) / self.cell_h).floor() as isize;
        let ci1 = ((x1 - self.origin_x) / self.cell_w).floor() as isize;
        let ri1 = ((y1 - self.origin_y) / self.cell_h).floor() as isize;
        for ri in ri0.max(0)..=(ri1.min(self.height as isize - 1)) {
            for ci in ci0.max(0)..=(ci1.min(self.width as isize - 1)) {
                let idx = ri as usize * self.width + ci as usize;
                for &pid in &self.cells[idx] {
                    let p = self.points[pid];
                    if p[0] >= x0 && p[0] <= x1 && p[1] >= y0 && p[1] <= y1 {
                        result.push(pid);
                    }
                }
            }
        }
        result
    }

    /// Query all point IDs within radius r of (cx, cy).
    pub fn query_radius(&self, cx: f32, cy: f32, r: f32) -> Vec<usize> {
        let r_sq = r * r;
        self.query_rect(cx - r, cy - r, cx + r, cy + r)
            .into_iter()
            .filter(|&id| {
                let p = self.points[id];
                let dx = p[0] - cx;
                let dy = p[1] - cy;
                dx * dx + dy * dy <= r_sq
            })
            .collect()
    }

    /// Return total number of points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// True if no points inserted.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Clear all points.
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            c.clear();
        }
        self.points.clear();
    }

    /// Get a point by ID.
    pub fn get_point(&self, id: usize) -> Option<[f32; 2]> {
        self.points.get(id).copied()
    }

    /// Number of grid columns.
    pub fn cols(&self) -> usize {
        self.width
    }

    /// Number of grid rows.
    pub fn rows(&self) -> usize {
        self.height
    }

    /// Cell dimensions.
    pub fn cell_size(&self) -> (f32, f32) {
        (self.cell_w, self.cell_h)
    }
}

/// Create a default grid index for the unit square.
pub fn new_grid_index(cols: usize, rows: usize) -> GridIndex {
    GridIndex::new(0.0, 0.0, 1.0, 1.0, cols, rows)
}

/// Create a grid index for a custom region.
pub fn new_grid_index_region(
    ox: f32,
    oy: f32,
    w: f32,
    h: f32,
    cols: usize,
    rows: usize,
) -> GridIndex {
    GridIndex::new(ox, oy, w, h, cols, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query_cell() {
        let mut g = new_grid_index(10, 10);
        let id = g.insert(0.15, 0.15).expect("should succeed");
        let r = g.query_cell(0.15, 0.15);
        assert!(r.contains(&id));
    }

    #[test]
    fn test_out_of_bounds() {
        let mut g = new_grid_index(10, 10);
        let r = g.insert(2.0, 0.5);
        assert!(r.is_none());
    }

    #[test]
    fn test_query_rect() {
        let mut g = new_grid_index(10, 10);
        let id = g.insert(0.5, 0.5).expect("should succeed");
        let r = g.query_rect(0.0, 0.0, 1.0, 1.0);
        assert!(r.contains(&id));
    }

    #[test]
    fn test_query_radius() {
        let mut g = new_grid_index(10, 10);
        let id = g.insert(0.5, 0.5).expect("should succeed");
        g.insert(0.9, 0.9).expect("should succeed");
        let r = g.query_radius(0.5, 0.5, 0.1);
        assert!(r.contains(&id));
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_len_and_clear() {
        let mut g = new_grid_index(5, 5);
        g.insert(0.1, 0.1).expect("should succeed");
        g.insert(0.2, 0.2).expect("should succeed");
        assert_eq!(g.len(), 2);
        g.clear();
        assert!(g.is_empty());
    }

    #[test]
    fn test_get_point() {
        let mut g = new_grid_index(10, 10);
        let id = g.insert(0.3, 0.7).expect("should succeed");
        let p = g.get_point(id).expect("should succeed");
        assert!((p[0] - 0.3).abs() < 1e-5);
        assert!((p[1] - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_cell_size() {
        let g = GridIndex::new(0.0, 0.0, 100.0, 50.0, 10, 5);
        let (cw, ch) = g.cell_size();
        assert!((cw - 10.0).abs() < 1e-4);
        assert!((ch - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_cols_rows() {
        let g = new_grid_index(8, 6);
        assert_eq!(g.cols(), 8);
        assert_eq!(g.rows(), 6);
    }
}
