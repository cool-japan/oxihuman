// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Voronoi cell mesh from seed points, clipped to AABB.

/// Axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Aabb3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb3 {
    #[allow(dead_code)]
    pub fn contains(&self, p: [f32; 3]) -> bool {
        (self.min[0]..=self.max[0]).contains(&p[0])
            && (self.min[1]..=self.max[1]).contains(&p[1])
            && (self.min[2]..=self.max[2]).contains(&p[2])
    }
}

/// A Voronoi cell: the set of grid points closest to a given seed.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoronoiCell {
    pub seed_index: usize,
    pub points: Vec<[f32; 3]>,
}

/// Nearest seed index for a point (brute-force).
#[allow(dead_code)]
pub fn nearest_seed(p: [f32; 3], seeds: &[[f32; 3]]) -> usize {
    let mut best = 0;
    let mut best_d = f32::MAX;
    for (i, &s) in seeds.iter().enumerate() {
        let d = dist3sq(p, s);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

/// Build Voronoi cells from seeds by sampling the AABB on a grid.
#[allow(dead_code)]
pub fn build_voronoi_cells(seeds: &[[f32; 3]], aabb: Aabb3, resolution: usize) -> Vec<VoronoiCell> {
    if seeds.is_empty() || resolution == 0 {
        return Vec::new();
    }
    let mut cells: Vec<VoronoiCell> = (0..seeds.len())
        .map(|i| VoronoiCell {
            seed_index: i,
            points: Vec::new(),
        })
        .collect();

    let rx = resolution;
    let ry = resolution;
    let rz = resolution;
    let dx = (aabb.max[0] - aabb.min[0]) / rx as f32;
    let dy = (aabb.max[1] - aabb.min[1]) / ry as f32;
    let dz = (aabb.max[2] - aabb.min[2]) / rz as f32;

    for iz in 0..rz {
        for iy in 0..ry {
            for ix in 0..rx {
                let p = [
                    aabb.min[0] + (ix as f32 + 0.5) * dx,
                    aabb.min[1] + (iy as f32 + 0.5) * dy,
                    aabb.min[2] + (iz as f32 + 0.5) * dz,
                ];
                let idx = nearest_seed(p, seeds);
                cells[idx].points.push(p);
            }
        }
    }
    cells
}

/// Total grid points assigned.
#[allow(dead_code)]
pub fn total_assigned_points(cells: &[VoronoiCell]) -> usize {
    cells.iter().map(|c| c.points.len()).sum()
}

fn dist3sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_aabb() -> Aabb3 {
        Aabb3 {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        }
    }

    #[test]
    fn aabb_contains_center() {
        let a = unit_aabb();
        assert!(a.contains([0.5, 0.5, 0.5]));
    }

    #[test]
    fn aabb_excludes_outside() {
        let a = unit_aabb();
        assert!(!a.contains([2.0, 0.5, 0.5]));
    }

    #[test]
    fn nearest_seed_basic() {
        let seeds = vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        assert_eq!(nearest_seed([1.0, 0.0, 0.0], &seeds), 0);
        assert_eq!(nearest_seed([9.0, 0.0, 0.0], &seeds), 1);
    }

    #[test]
    fn cell_count_equals_seed_count() {
        let seeds = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let cells = build_voronoi_cells(&seeds, unit_aabb(), 4);
        assert_eq!(cells.len(), 2);
    }

    #[test]
    fn total_points_equals_grid_size() {
        let seeds = vec![[0.1, 0.5, 0.5], [0.9, 0.5, 0.5]];
        let cells = build_voronoi_cells(&seeds, unit_aabb(), 4);
        assert_eq!(total_assigned_points(&cells), 64);
    }

    #[test]
    fn empty_seeds_returns_empty() {
        let cells = build_voronoi_cells(&[], unit_aabb(), 4);
        assert!(cells.is_empty());
    }

    #[test]
    fn zero_resolution_returns_empty() {
        let seeds = vec![[0.5, 0.5, 0.5]];
        let cells = build_voronoi_cells(&seeds, unit_aabb(), 0);
        assert!(cells.is_empty());
    }

    #[test]
    fn single_seed_gets_all_points() {
        let seeds = vec![[0.5, 0.5, 0.5]];
        let cells = build_voronoi_cells(&seeds, unit_aabb(), 3);
        assert_eq!(cells[0].points.len(), 27);
    }

    #[test]
    fn seed_index_stored() {
        let seeds = vec![[0.0, 0.5, 0.5], [1.0, 0.5, 0.5]];
        let cells = build_voronoi_cells(&seeds, unit_aabb(), 2);
        assert_eq!(cells[0].seed_index, 0);
        assert_eq!(cells[1].seed_index, 1);
    }

    #[test]
    fn cells_partition_all_points() {
        let seeds = vec![[0.2, 0.5, 0.5], [0.8, 0.5, 0.5]];
        let cells = build_voronoi_cells(&seeds, unit_aabb(), 4);
        let total = total_assigned_points(&cells);
        assert_eq!(total, 64);
    }
}
