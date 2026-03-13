// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Merge vertices that are within a threshold distance.

/// Result of merge-by-distance.
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub merged_count: usize,
}

/// Merge vertices within `threshold` of each other.
pub fn merge_by_distance(positions: &[[f32; 3]], indices: &[u32], threshold: f32) -> MergeResult {
    let n = positions.len();
    let mut remap: Vec<usize> = (0..n).collect();
    /* naive O(n²) merge */
    for i in 0..n {
        if remap[i] != i {
            continue;
        }
        for j in (i + 1)..n {
            if remap[j] != j {
                continue;
            }
            let dx = positions[i][0] - positions[j][0];
            let dy = positions[i][1] - positions[j][1];
            let dz = positions[i][2] - positions[j][2];
            if dx * dx + dy * dy + dz * dz <= threshold * threshold {
                remap[j] = i;
            }
        }
    }
    /* compact unique positions */
    let mut unique_pos: Vec<[f32; 3]> = Vec::new();
    let mut old_to_new: Vec<usize> = vec![0; n];
    for i in 0..n {
        let canonical = remap[i];
        if canonical == i {
            old_to_new[i] = unique_pos.len();
            unique_pos.push(positions[i]);
        }
    }
    /* fill non-canonical entries */
    for i in 0..n {
        if remap[i] != i {
            old_to_new[i] = old_to_new[remap[i]];
        }
    }
    let merged_count = n - unique_pos.len();
    let new_indices: Vec<u32> = indices
        .iter()
        .map(|&vi| old_to_new[vi as usize] as u32)
        .collect();
    MergeResult {
        positions: unique_pos,
        indices: new_indices,
        merged_count,
    }
}

/// Count unique vertices after merging.
pub fn merge_count_unique(result: &MergeResult) -> usize {
    result.positions.len()
}

/// Apply a vertex remap to a face index buffer.
pub fn merge_apply_to_faces(indices: &[u32], remap: &[usize]) -> Vec<u32> {
    indices.iter().map(|&v| remap[v as usize] as u32).collect()
}

/// Remove degenerate triangles (two or more identical vertex indices).
pub fn merge_remove_degenerate(indices: &[u32]) -> Vec<u32> {
    indices
        .chunks(3)
        .filter(|tri| tri.len() == 3 && tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
        .flat_map(|tri| tri.iter().copied())
        .collect()
}

/// Suitable weld threshold (minimum distance below which vertices should merge).
pub fn merge_weld_threshold() -> f32 {
    1e-4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_by_distance_merges_close() {
        let pos = vec![[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0001], [1.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = merge_by_distance(&pos, &idx, 0.001);
        assert_eq!(r.merged_count, 1);
        assert_eq!(r.positions.len(), 2);
    }

    #[test]
    fn test_merge_count_unique() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = merge_by_distance(&pos, &idx, 0.001);
        assert_eq!(merge_count_unique(&r), 3);
    }

    #[test]
    fn test_merge_remove_degenerate() {
        let idx = vec![0u32, 1, 2, 0, 0, 1];
        let clean = merge_remove_degenerate(&idx);
        assert_eq!(clean.len(), 3);
    }

    #[test]
    fn test_merge_weld_threshold_positive() {
        assert!(merge_weld_threshold() > 0.0);
    }

    #[test]
    fn test_merge_apply_to_faces() {
        let idx = vec![0u32, 1, 2];
        let remap = vec![0, 0, 1];
        let new_idx = merge_apply_to_faces(&idx, &remap);
        assert_eq!(new_idx[1], 0);
    }

    #[test]
    fn test_merge_no_merge_far() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = merge_by_distance(&pos, &idx, 0.0001);
        assert_eq!(r.merged_count, 0);
    }
}
