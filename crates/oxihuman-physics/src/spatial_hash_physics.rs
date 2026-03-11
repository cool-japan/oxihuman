#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spatial hash for 3D point lookup (physics module).

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpatialHashPhysics {
    pub cells: HashMap<(i32, i32, i32), Vec<u32>>,
    pub cell_size: f32,
}

#[allow(dead_code)]
pub fn new_spatial_hash_physics(cell_size: f32) -> SpatialHashPhysics {
    SpatialHashPhysics {
        cells: HashMap::new(),
        cell_size: cell_size.max(1e-6),
    }
}

#[allow(dead_code)]
pub fn sh_phys_cell(sh: &SpatialHashPhysics, p: [f32; 3]) -> (i32, i32, i32) {
    (
        (p[0] / sh.cell_size).floor() as i32,
        (p[1] / sh.cell_size).floor() as i32,
        (p[2] / sh.cell_size).floor() as i32,
    )
}

#[allow(dead_code)]
pub fn sh_phys_insert(sh: &mut SpatialHashPhysics, id: u32, pos: [f32; 3]) {
    let cell = sh_phys_cell(sh, pos);
    sh.cells.entry(cell).or_default().push(id);
}

#[allow(dead_code)]
pub fn sh_phys_query_radius(sh: &SpatialHashPhysics, center: [f32; 3], radius: f32) -> Vec<u32> {
    let cell_radius = (radius / sh.cell_size).ceil() as i32;
    let cc = sh_phys_cell(sh, center);
    let r_sq = radius * radius;
    let mut result = Vec::new();
    for dx in -cell_radius..=cell_radius {
        for dy in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let key = (cc.0 + dx, cc.1 + dy, cc.2 + dz);
                if let Some(ids) = sh.cells.get(&key) {
                    for &id in ids {
                        // Approximate: accept any point in the cell
                        let cell_cx = (key.0 as f32 + 0.5) * sh.cell_size;
                        let cell_cy = (key.1 as f32 + 0.5) * sh.cell_size;
                        let cell_cz = (key.2 as f32 + 0.5) * sh.cell_size;
                        let dx2 = cell_cx - center[0];
                        let dy2 = cell_cy - center[1];
                        let dz2 = cell_cz - center[2];
                        let dist_sq = dx2 * dx2 + dy2 * dy2 + dz2 * dz2;
                        let cell_diag_sq = sh.cell_size * sh.cell_size * 3.0;
                        if dist_sq <= r_sq + cell_diag_sq {
                            result.push(id);
                        }
                    }
                }
            }
        }
    }
    result.sort_unstable();
    result.dedup();
    result
}

#[allow(dead_code)]
pub fn sh_phys_clear(sh: &mut SpatialHashPhysics) {
    sh.cells.clear();
}

#[allow(dead_code)]
pub fn sh_phys_count(sh: &SpatialHashPhysics) -> usize {
    sh.cells.values().map(|v| v.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let sh = new_spatial_hash_physics(1.0);
        assert_eq!(sh_phys_count(&sh), 0);
    }

    #[test]
    fn insert_increments_count() {
        let mut sh = new_spatial_hash_physics(1.0);
        sh_phys_insert(&mut sh, 1, [0.5, 0.5, 0.5]);
        assert_eq!(sh_phys_count(&sh), 1);
    }

    #[test]
    fn cell_for_origin() {
        let sh = new_spatial_hash_physics(1.0);
        assert_eq!(sh_phys_cell(&sh, [0.0, 0.0, 0.0]), (0, 0, 0));
    }

    #[test]
    fn cell_negative() {
        let sh = new_spatial_hash_physics(1.0);
        let c = sh_phys_cell(&sh, [-0.5, -0.5, -0.5]);
        assert_eq!(c, (-1, -1, -1));
    }

    #[test]
    fn query_finds_nearby() {
        let mut sh = new_spatial_hash_physics(1.0);
        sh_phys_insert(&mut sh, 42, [0.5, 0.5, 0.5]);
        let results = sh_phys_query_radius(&sh, [0.5, 0.5, 0.5], 2.0);
        assert!(results.contains(&42));
    }

    #[test]
    fn query_no_results_far_away() {
        let mut sh = new_spatial_hash_physics(1.0);
        sh_phys_insert(&mut sh, 1, [100.0, 100.0, 100.0]);
        let results = sh_phys_query_radius(&sh, [0.0, 0.0, 0.0], 1.0);
        assert!(!results.contains(&1));
    }

    #[test]
    fn clear_empties() {
        let mut sh = new_spatial_hash_physics(1.0);
        sh_phys_insert(&mut sh, 1, [1.0, 1.0, 1.0]);
        sh_phys_insert(&mut sh, 2, [2.0, 2.0, 2.0]);
        sh_phys_clear(&mut sh);
        assert_eq!(sh_phys_count(&sh), 0);
    }

    #[test]
    fn multiple_in_same_cell() {
        let mut sh = new_spatial_hash_physics(1.0);
        sh_phys_insert(&mut sh, 1, [0.1, 0.1, 0.1]);
        sh_phys_insert(&mut sh, 2, [0.9, 0.9, 0.9]);
        assert_eq!(sh_phys_count(&sh), 2);
    }

    #[test]
    fn cell_size_stored() {
        let sh = new_spatial_hash_physics(2.5);
        assert!((sh.cell_size - 2.5).abs() < 1e-5);
    }

    #[test]
    fn multiple_cells() {
        let mut sh = new_spatial_hash_physics(1.0);
        for i in 0u32..5 {
            sh_phys_insert(&mut sh, i, [i as f32 * 10.0, 0.0, 0.0]);
        }
        assert_eq!(sh_phys_count(&sh), 5);
    }
}
