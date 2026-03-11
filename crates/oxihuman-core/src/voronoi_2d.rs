// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D Voronoi diagram cells using brute-force nearest-seed computation.

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[allow(dead_code)]
    pub fn dist_sq(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

/// A Voronoi cell: seed point + index of nearest seed for queries.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VoronoiCell2D {
    pub seed: Point2,
    pub id: usize,
}

/// A 2D Voronoi diagram.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Voronoi2D {
    pub seeds: Vec<Point2>,
}

impl Voronoi2D {
    /// Create an empty diagram.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a seed point.
    #[allow(dead_code)]
    pub fn add_seed(&mut self, p: Point2) {
        self.seeds.push(p);
    }

    /// Find the index of the nearest seed to query point `q`.
    /// Returns `None` if there are no seeds.
    #[allow(dead_code)]
    pub fn nearest_seed(&self, q: Point2) -> Option<usize> {
        if self.seeds.is_empty() {
            return None;
        }
        let mut best = 0;
        let mut best_d = q.dist_sq(self.seeds[0]);
        for (i, &s) in self.seeds.iter().enumerate().skip(1) {
            let d = q.dist_sq(s);
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        Some(best)
    }

    /// Rasterize the Voronoi diagram into a `width x height` grid.
    /// Each cell value is the index of the nearest seed (or 0 if no seeds).
    #[allow(dead_code)]
    pub fn rasterize(&self, width: usize, height: usize) -> Vec<usize> {
        let mut grid = vec![0usize; width * height];
        if self.seeds.is_empty() {
            return grid;
        }
        for row in 0..height {
            for col in 0..width {
                let q = Point2::new(col as f32, row as f32);
                let idx = self.nearest_seed(q).unwrap_or(0);
                grid[row * width + col] = idx;
            }
        }
        grid
    }

    /// Number of seeds.
    #[allow(dead_code)]
    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }

    /// Clear all seeds.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.seeds.clear();
    }

    /// Build a list of `VoronoiCell2D` from current seeds.
    #[allow(dead_code)]
    pub fn cells(&self) -> Vec<VoronoiCell2D> {
        self.seeds
            .iter()
            .enumerate()
            .map(|(id, &seed)| VoronoiCell2D { seed, id })
            .collect()
    }
}

/// Compute the Voronoi cell assignment for a set of query points.
#[allow(dead_code)]
pub fn voronoi_assign(diagram: &Voronoi2D, queries: &[Point2]) -> Vec<usize> {
    queries
        .iter()
        .map(|&q| diagram.nearest_seed(q).unwrap_or(0))
        .collect()
}

/// Build a `Voronoi2D` from a slice of seed points.
#[allow(dead_code)]
pub fn build_voronoi(seeds: &[Point2]) -> Voronoi2D {
    let mut v = Voronoi2D::new();
    for &s in seeds {
        v.add_seed(s);
    }
    v
}

/// Compute the centroid of seed points assigned to a given cell.
#[allow(dead_code)]
pub fn cell_centroid(diagram: &Voronoi2D, cell_id: usize, samples: &[Point2]) -> Option<Point2> {
    let pts: Vec<Point2> = samples
        .iter()
        .copied()
        .filter(|&q| diagram.nearest_seed(q) == Some(cell_id))
        .collect();
    if pts.is_empty() {
        return None;
    }
    let n = pts.len() as f32;
    let sx: f32 = pts.iter().map(|p| p.x).sum();
    let sy: f32 = pts.iter().map(|p| p.y).sum();
    Some(Point2::new(sx / n, sy / n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_seeds() -> Voronoi2D {
        let mut v = Voronoi2D::new();
        v.add_seed(Point2::new(0.0, 0.0));
        v.add_seed(Point2::new(10.0, 0.0));
        v.add_seed(Point2::new(5.0, 10.0));
        v
    }

    #[test]
    fn nearest_seed_returns_none_for_empty() {
        let v = Voronoi2D::new();
        assert!(v.nearest_seed(Point2::new(1.0, 1.0)).is_none());
    }

    #[test]
    fn nearest_seed_basic() {
        let v = make_seeds();
        assert_eq!(v.nearest_seed(Point2::new(0.5, 0.5)), Some(0));
        assert_eq!(v.nearest_seed(Point2::new(9.5, 0.5)), Some(1));
        assert_eq!(v.nearest_seed(Point2::new(5.0, 9.5)), Some(2));
    }

    #[test]
    fn seed_count_matches() {
        let v = make_seeds();
        assert_eq!(v.seed_count(), 3);
    }

    #[test]
    fn clear_removes_all_seeds() {
        let mut v = make_seeds();
        v.clear();
        assert_eq!(v.seed_count(), 0);
    }

    #[test]
    fn cells_returns_correct_ids() {
        let v = make_seeds();
        let cells = v.cells();
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].id, 0);
        assert_eq!(cells[2].id, 2);
    }

    #[test]
    fn rasterize_size_matches() {
        let v = make_seeds();
        let grid = v.rasterize(4, 3);
        assert_eq!(grid.len(), 12);
    }

    #[test]
    fn rasterize_empty_returns_zeros() {
        let v = Voronoi2D::new();
        let grid = v.rasterize(3, 3);
        assert!(grid.iter().all(|&x| x == 0));
    }

    #[test]
    fn voronoi_assign_empty_seeds() {
        let v = Voronoi2D::new();
        let pts = vec![Point2::new(0.0, 0.0)];
        let out = voronoi_assign(&v, &pts);
        assert_eq!(out, vec![0]);
    }

    #[test]
    fn build_voronoi_seed_count() {
        let seeds = vec![Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)];
        let v = build_voronoi(&seeds);
        assert_eq!(v.seed_count(), 2);
    }

    #[test]
    fn cell_centroid_none_when_no_samples() {
        let v = make_seeds();
        let c = cell_centroid(&v, 0, &[]);
        assert!(c.is_none());
    }
}
