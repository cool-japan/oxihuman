// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Mesh topology repair v2: degenerate face removal, duplicate vertex merging.

#[allow(dead_code)]
pub struct RepairStatsV2 {
    pub degenerate_removed: usize,
    pub duplicates_merged: usize,
}

#[allow(dead_code)]
pub fn trv2_triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() * 0.5
}

#[allow(dead_code)]
#[allow(clippy::ptr_arg)]
pub fn trv2_remove_degenerate(positions: &[[f32; 3]], indices: &mut Vec<[u32; 3]>) -> usize {
    let before = indices.len();
    indices.retain(|tri| {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        trv2_triangle_area(a, b, c) >= 1e-10
    });
    before - indices.len()
}

#[allow(dead_code)]
#[allow(clippy::ptr_arg)]
pub fn trv2_merge_duplicate_vertices(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<[u32; 3]>,
    tol: f32,
) -> usize {
    let n = positions.len();
    let mut remap: Vec<usize> = (0..n).collect();
    for i in 0..n {
        for j in 0..i {
            if remap[j] == j {
                let dx = positions[i][0] - positions[j][0];
                let dy = positions[i][1] - positions[j][1];
                let dz = positions[i][2] - positions[j][2];
                if (dx * dx + dy * dy + dz * dz).sqrt() < tol {
                    remap[i] = remap[j];
                    break;
                }
            }
        }
    }
    let merged = remap.iter().enumerate().filter(|&(i, &r)| r != i).count();
    for tri in indices.iter_mut() {
        tri[0] = remap[tri[0] as usize] as u32;
        tri[1] = remap[tri[1] as usize] as u32;
        tri[2] = remap[tri[2] as usize] as u32;
    }
    merged
}

#[allow(dead_code)]
pub fn trv2_repair(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<[u32; 3]>,
    tol: f32,
) -> RepairStatsV2 {
    let duplicates_merged = trv2_merge_duplicate_vertices(positions, indices, tol);
    let degenerate_removed = trv2_remove_degenerate(positions, indices);
    RepairStatsV2 { degenerate_removed, duplicates_merged }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area_unit() {
        let a = trv2_triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_area_degenerate() {
        let a = trv2_triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(a < 1e-10);
    }

    #[test]
    fn test_remove_degenerate() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let mut indices = vec![[0u32, 1, 2], [0, 1, 3]];
        let removed = trv2_remove_degenerate(&positions, &mut indices);
        assert_eq!(removed, 1);
        assert_eq!(indices.len(), 1);
    }

    #[test]
    fn test_remove_degenerate_none() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let mut indices = vec![[0u32, 1, 2]];
        let removed = trv2_remove_degenerate(&positions, &mut indices);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_merge_duplicates() {
        let mut positions = vec![
            [0.0f32, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let mut indices = vec![[0u32, 1, 2]];
        let merged = trv2_merge_duplicate_vertices(&mut positions, &mut indices, 1e-5);
        assert_eq!(merged, 1);
    }

    #[test]
    fn test_repair_stats_clean() {
        let mut positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let mut indices = vec![[0u32, 1, 2]];
        let stats = trv2_repair(&mut positions, &mut indices, 1e-5);
        assert_eq!(stats.degenerate_removed, 0);
        assert_eq!(stats.duplicates_merged, 0);
    }

    #[test]
    fn test_repair_with_degenerate() {
        let mut positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let mut indices = vec![[0u32, 1, 2], [0, 1, 3]];
        let stats = trv2_repair(&mut positions, &mut indices, 1e-5);
        assert_eq!(stats.degenerate_removed, 1);
    }

    #[test]
    fn test_triangle_area_3d() {
        let a = trv2_triangle_area([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]);
        assert!((a - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_no_merge_when_far() {
        let mut positions = vec![
            [0.0f32, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            [0.0, 5.0, 0.0],
        ];
        let mut indices = vec![[0u32, 1, 2]];
        let merged = trv2_merge_duplicate_vertices(&mut positions, &mut indices, 1e-5);
        assert_eq!(merged, 0);
    }
}
