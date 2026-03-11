// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2D spatial hashing for broad-phase point queries.

#![allow(dead_code)]

use std::collections::HashMap;

/// A 2D spatial hash grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpatialHash2D {
    pub cell_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
    positions: Vec<[f32; 2]>,
}

#[allow(dead_code)]
impl SpatialHash2D {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(1e-6),
            cells: HashMap::new(),
            positions: Vec::new(),
        }
    }

    fn cell_key(&self, x: f32, y: f32) -> (i32, i32) {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        (cx, cy)
    }

    /// Insert a point and return its index.
    pub fn insert(&mut self, pos: [f32; 2]) -> usize {
        let id = self.positions.len();
        self.positions.push(pos);
        let key = self.cell_key(pos[0], pos[1]);
        self.cells.entry(key).or_default().push(id);
        id
    }

    /// Query all point indices in the same or neighboring cells (within radius).
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> Vec<usize> {
        let r_cells = (radius / self.cell_size).ceil() as i32 + 1;
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        let mut result = Vec::new();
        for dx in -r_cells..=r_cells {
            for dy in -r_cells..=r_cells {
                if let Some(ids) = self.cells.get(&(cx + dx, cy + dy)) {
                    for &id in ids {
                        let p = self.positions[id];
                        let ddx = p[0] - x;
                        let ddy = p[1] - y;
                        if ddx * ddx + ddy * ddy <= radius * radius {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// Query all points in a rectangle.
    pub fn query_aabb(&self, min: [f32; 2], max: [f32; 2]) -> Vec<usize> {
        let c_min = self.cell_key(min[0], min[1]);
        let c_max = self.cell_key(max[0], max[1]);
        let mut result = Vec::new();
        for cx in c_min.0..=c_max.0 {
            for cy in c_min.1..=c_max.1 {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for &id in ids {
                        let p = self.positions[id];
                        if p[0] >= min[0] && p[0] <= max[0] && p[1] >= min[1] && p[1] <= max[1] {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// Number of inserted points.
    pub fn point_count(&self) -> usize {
        self.positions.len()
    }

    /// Number of non-empty cells.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.positions.clear();
    }

    /// Get position by index.
    pub fn get(&self, id: usize) -> Option<[f32; 2]> {
        self.positions.get(id).copied()
    }

    /// Rebuild the grid from existing positions.
    pub fn rebuild(&mut self) {
        self.cells.clear();
        for (id, pos) in self.positions.iter().enumerate() {
            let key = self.cell_key(pos[0], pos[1]);
            self.cells.entry(key).or_default().push(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_count() {
        let mut h = SpatialHash2D::new(1.0);
        h.insert([0.5, 0.5]);
        h.insert([1.5, 1.5]);
        assert_eq!(h.point_count(), 2);
    }

    #[test]
    fn query_radius_finds_nearby() {
        let mut h = SpatialHash2D::new(1.0);
        let id0 = h.insert([0.0, 0.0]);
        let _id1 = h.insert([10.0, 10.0]);
        let results = h.query_radius(0.0, 0.0, 0.5);
        assert!(results.contains(&id0));
    }

    #[test]
    fn query_radius_excludes_far() {
        let mut h = SpatialHash2D::new(1.0);
        let _id0 = h.insert([0.0, 0.0]);
        let id1 = h.insert([10.0, 10.0]);
        let results = h.query_radius(0.0, 0.0, 0.5);
        assert!(!results.contains(&id1));
    }

    #[test]
    fn query_aabb_finds_inside() {
        let mut h = SpatialHash2D::new(1.0);
        let id = h.insert([1.5, 1.5]);
        let results = h.query_aabb([0.0, 0.0], [2.0, 2.0]);
        assert!(results.contains(&id));
    }

    #[test]
    fn query_aabb_excludes_outside() {
        let mut h = SpatialHash2D::new(1.0);
        let _id = h.insert([5.0, 5.0]);
        let results = h.query_aabb([0.0, 0.0], [2.0, 2.0]);
        assert!(results.is_empty());
    }

    #[test]
    fn cell_count_nonzero() {
        let mut h = SpatialHash2D::new(1.0);
        h.insert([0.0, 0.0]);
        h.insert([5.0, 5.0]);
        assert!(h.cell_count() >= 2);
    }

    #[test]
    fn clear_resets_count() {
        let mut h = SpatialHash2D::new(1.0);
        h.insert([1.0, 1.0]);
        h.clear();
        assert_eq!(h.point_count(), 0);
        assert_eq!(h.cell_count(), 0);
    }

    #[test]
    fn get_by_id() {
        let mut h = SpatialHash2D::new(1.0);
        use std::f32::consts::PI;
        let id = h.insert([PI, 2.71]);
        let p = h.get(id).unwrap();
        assert!((p[0] - PI).abs() < 1e-5);
    }

    #[test]
    fn rebuild_preserves_query() {
        let mut h = SpatialHash2D::new(1.0);
        let id = h.insert([0.5, 0.5]);
        h.rebuild();
        let results = h.query_radius(0.5, 0.5, 0.1);
        assert!(results.contains(&id));
    }

    #[test]
    fn all_in_radius_one() {
        let mut h = SpatialHash2D::new(0.5);
        let ids: Vec<usize> = (0..5).map(|i| h.insert([i as f32 * 0.1, 0.0])).collect();
        let results = h.query_radius(0.2, 0.0, 0.5);
        // should find all 5
        for id in &ids {
            assert!(results.contains(id), "missing id {id}");
        }
    }
}
