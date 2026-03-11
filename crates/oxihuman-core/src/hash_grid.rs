// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Spatial hash grid for 2D/3D neighbor queries.

use std::collections::HashMap;

/// A 3D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HgPoint3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl HgPoint3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        HgPoint3 { x, y, z }
    }

    fn cell(&self, cell_size: f32) -> (i32, i32, i32) {
        (
            (self.x / cell_size).floor() as i32,
            (self.y / cell_size).floor() as i32,
            (self.z / cell_size).floor() as i32,
        )
    }

    fn dist_sq(&self, other: &HgPoint3) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
}

/// Spatial hash grid storing point IDs.
pub struct HashGrid {
    cells: HashMap<(i32, i32, i32), Vec<usize>>,
    points: Vec<HgPoint3>,
    cell_size: f32,
}

impl HashGrid {
    /// Create a new hash grid with the given cell size.
    pub fn new(cell_size: f32) -> Self {
        HashGrid {
            cells: HashMap::new(),
            points: Vec::new(),
            cell_size: cell_size.max(1e-6),
        }
    }

    /// Insert a point and return its ID.
    pub fn insert(&mut self, p: HgPoint3) -> usize {
        let id = self.points.len();
        let cell = p.cell(self.cell_size);
        self.cells.entry(cell).or_default().push(id);
        self.points.push(p);
        id
    }

    /// Return IDs of all points within radius `r` of `center`.
    pub fn query_radius(&self, center: &HgPoint3, r: f32) -> Vec<usize> {
        let r_sq = r * r;
        let span = (r / self.cell_size).ceil() as i32 + 1;
        let (cx, cy, cz) = center.cell(self.cell_size);
        let mut result = Vec::new();
        for dx in -span..=span {
            for dy in -span..=span {
                for dz in -span..=span {
                    let key = (cx + dx, cy + dy, cz + dz);
                    if let Some(ids) = self.cells.get(&key) {
                        for &id in ids {
                            if self.points[id].dist_sq(center) <= r_sq {
                                result.push(id);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// Return number of points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get a point by ID.
    pub fn get(&self, id: usize) -> Option<&HgPoint3> {
        self.points.get(id)
    }

    /// Clear all points.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.points.clear();
    }

    /// Return the cell size.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Return the number of occupied cells.
    pub fn occupied_cells(&self) -> usize {
        self.cells.len()
    }

    /// Nearest neighbor (brute force over candidate cells).
    pub fn nearest(&self, center: &HgPoint3) -> Option<(usize, f32)> {
        if self.points.is_empty() {
            return None;
        }
        let mut best_id = 0;
        let mut best_sq = f32::INFINITY;
        for (id, p) in self.points.iter().enumerate() {
            let d = p.dist_sq(center);
            if d < best_sq {
                best_sq = d;
                best_id = id;
            }
        }
        Some((best_id, best_sq.sqrt()))
    }
}

/// Create a new hash grid.
pub fn new_hash_grid(cell_size: f32) -> HashGrid {
    HashGrid::new(cell_size)
}

/// Insert a 2D point (z = 0).
pub fn hg_insert_2d(grid: &mut HashGrid, x: f32, y: f32) -> usize {
    grid.insert(HgPoint3::new(x, y, 0.0))
}

/// Query radius in 2D (z = 0).
pub fn hg_query_2d(grid: &HashGrid, x: f32, y: f32, r: f32) -> Vec<usize> {
    grid.query_radius(&HgPoint3::new(x, y, 0.0), r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut g = new_hash_grid(1.0);
        let id = hg_insert_2d(&mut g, 0.5, 0.5);
        let r = hg_query_2d(&g, 0.5, 0.5, 1.0);
        assert!(r.contains(&id));
    }

    #[test]
    fn test_out_of_range() {
        let mut g = new_hash_grid(1.0);
        hg_insert_2d(&mut g, 0.0, 0.0);
        let r = hg_query_2d(&g, 100.0, 100.0, 1.0);
        assert!(r.is_empty());
    }

    #[test]
    fn test_len() {
        let mut g = new_hash_grid(2.0);
        assert_eq!(g.len(), 0);
        hg_insert_2d(&mut g, 1.0, 2.0);
        hg_insert_2d(&mut g, 3.0, 4.0);
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut g = new_hash_grid(1.0);
        hg_insert_2d(&mut g, 1.0, 1.0);
        g.clear();
        assert!(g.is_empty());
    }

    #[test]
    fn test_nearest() {
        let mut g = new_hash_grid(1.0);
        g.insert(HgPoint3::new(0.0, 0.0, 0.0));
        g.insert(HgPoint3::new(10.0, 0.0, 0.0));
        let (id, dist) = g.nearest(&HgPoint3::new(0.1, 0.0, 0.0)).unwrap();
        assert_eq!(id, 0);
        assert!(dist < 1.0);
    }

    #[test]
    fn test_multiple_in_radius() {
        let mut g = new_hash_grid(1.0);
        for i in 0..5 {
            hg_insert_2d(&mut g, i as f32 * 0.1, 0.0);
        }
        let r = hg_query_2d(&g, 0.2, 0.0, 1.0);
        assert_eq!(r.len(), 5);
    }

    #[test]
    fn test_get_point() {
        let mut g = new_hash_grid(1.0);
        let id = g.insert(HgPoint3::new(3.0, 4.0, 5.0));
        let p = g.get(id).unwrap();
        assert!((p.x - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_occupied_cells() {
        let mut g = new_hash_grid(1.0);
        hg_insert_2d(&mut g, 0.5, 0.5);
        hg_insert_2d(&mut g, 10.5, 10.5);
        assert_eq!(g.occupied_cells(), 2);
    }
}
