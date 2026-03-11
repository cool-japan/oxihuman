// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Uniform grid broadphase for spatial partitioning of physics bodies.

use std::collections::HashMap;

/// A body entry in the broadphase grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridBody {
    pub id: u32,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// 3D cell key.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellKey(i32, i32, i32);

/// Uniform grid broadphase collision detection.
#[allow(dead_code)]
#[derive(Debug)]
pub struct BroadphaseGrid {
    cell_size: f32,
    cells: HashMap<CellKey, Vec<u32>>,
    bodies: Vec<GridBody>,
}

#[allow(dead_code)]
impl BroadphaseGrid {
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0);
        Self { cell_size, cells: HashMap::new(), bodies: Vec::new() }
    }

    fn to_cell(&self, x: f32, y: f32, z: f32) -> CellKey {
        CellKey(
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
            (z / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, body: GridBody) {
        let min_cell = self.to_cell(body.min[0], body.min[1], body.min[2]);
        let max_cell = self.to_cell(body.max[0], body.max[1], body.max[2]);
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    self.cells.entry(CellKey(x, y, z)).or_default().push(body.id);
                }
            }
        }
        self.bodies.push(body);
    }

    /// Find all potential collision pairs.
    pub fn find_pairs(&self) -> Vec<(u32, u32)> {
        let mut pairs = std::collections::HashSet::new();
        for ids in self.cells.values() {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = ids[i].min(ids[j]);
                    let b = ids[i].max(ids[j]);
                    pairs.insert((a, b));
                }
            }
        }
        pairs.into_iter().collect()
    }

    /// Query all bodies overlapping a point.
    pub fn query_point(&self, point: [f32; 3]) -> Vec<u32> {
        let cell = self.to_cell(point[0], point[1], point[2]);
        self.cells.get(&cell).cloned().unwrap_or_default()
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.bodies.clear();
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn bodies_in_cell(&self, key: CellKey) -> Vec<u32> {
        self.cells.get(&key).cloned().unwrap_or_default()
    }

    /// Average bodies per cell.
    pub fn average_cell_occupancy(&self) -> f32 {
        if self.cells.is_empty() { return 0.0; }
        let total: usize = self.cells.values().map(|v| v.len()).sum();
        total as f32 / self.cells.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(id: u32, min: [f32; 3], max: [f32; 3]) -> GridBody {
        GridBody { id, min, max }
    }

    #[test]
    fn test_insert() {
        let mut g = BroadphaseGrid::new(1.0);
        g.insert(body(0, [0.0;3], [0.5;3]));
        assert_eq!(g.body_count(), 1);
    }

    #[test]
    fn test_find_pairs_overlap() {
        let mut g = BroadphaseGrid::new(2.0);
        g.insert(body(0, [0.0;3], [1.0;3]));
        g.insert(body(1, [0.5;3], [1.5;3]));
        let pairs = g.find_pairs();
        assert!(!pairs.is_empty());
        assert!(pairs.contains(&(0, 1)));
    }

    #[test]
    fn test_no_pair() {
        let mut g = BroadphaseGrid::new(1.0);
        g.insert(body(0, [0.0;3], [0.5;3]));
        g.insert(body(1, [10.0;3], [10.5;3]));
        let pairs = g.find_pairs();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_query_point() {
        let mut g = BroadphaseGrid::new(2.0);
        g.insert(body(0, [0.0;3], [1.0;3]));
        let r = g.query_point([0.5, 0.5, 0.5]);
        assert!(r.contains(&0));
    }

    #[test]
    fn test_clear() {
        let mut g = BroadphaseGrid::new(1.0);
        g.insert(body(0, [0.0;3], [1.0;3]));
        g.clear();
        assert_eq!(g.body_count(), 0);
        assert_eq!(g.cell_count(), 0);
    }

    #[test]
    fn test_cell_count() {
        let mut g = BroadphaseGrid::new(1.0);
        g.insert(body(0, [0.0;3], [2.5;3]));
        assert!(g.cell_count() > 1);
    }

    #[test]
    fn test_cell_size() {
        let g = BroadphaseGrid::new(3.0);
        assert!((g.cell_size() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_occupancy() {
        let mut g = BroadphaseGrid::new(10.0);
        g.insert(body(0, [0.0;3], [1.0;3]));
        g.insert(body(1, [0.0;3], [1.0;3]));
        assert!(g.average_cell_occupancy() > 1.0);
    }

    #[test]
    fn test_many_bodies() {
        let mut g = BroadphaseGrid::new(5.0);
        for i in 0..20u32 {
            let x = i as f32;
            g.insert(body(i, [x, 0.0, 0.0], [x+1.0, 1.0, 1.0]));
        }
        assert_eq!(g.body_count(), 20);
    }

    #[test]
    fn test_negative_coords() {
        let mut g = BroadphaseGrid::new(1.0);
        g.insert(body(0, [-2.0;3], [-1.0;3]));
        let r = g.query_point([-1.5, -1.5, -1.5]);
        assert!(r.contains(&0));
    }
}
