#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Merge vertices that are within a threshold distance.

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MergeResult {
    pub new_verts: Vec<[f32; 3]>,
    pub index_map: Vec<usize>,
}

#[allow(dead_code)]
pub fn merge_vertices(verts: &[[f32; 3]], threshold: f32) -> MergeResult {
    let n = verts.len();
    let mut index_map = vec![usize::MAX; n];
    let mut new_verts: Vec<[f32; 3]> = Vec::new();

    for i in 0..n {
        if index_map[i] != usize::MAX {
            continue;
        }
        let new_idx = new_verts.len();
        new_verts.push(verts[i]);
        index_map[i] = new_idx;
        for j in (i + 1)..n {
            if index_map[j] != usize::MAX {
                continue;
            }
            let dx = verts[j][0] - verts[i][0];
            let dy = verts[j][1] - verts[i][1];
            let dz = verts[j][2] - verts[i][2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 <= threshold * threshold {
                index_map[j] = new_idx;
            }
        }
    }
    MergeResult { new_verts, index_map }
}

#[allow(dead_code)]
pub fn merge_count(result: &MergeResult) -> usize {
    result.index_map.len().saturating_sub(result.new_verts.len())
}

#[allow(dead_code)]
pub fn apply_index_map(tris: &[[u32; 3]], map: &[usize]) -> Vec<[u32; 3]> {
    tris.iter()
        .map(|t| {
            [
                *map.get(t[0] as usize).unwrap_or(&(t[0] as usize)) as u32,
                *map.get(t[1] as usize).unwrap_or(&(t[1] as usize)) as u32,
                *map.get(t[2] as usize).unwrap_or(&(t[2] as usize)) as u32,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_merge_far_apart() {
        let verts = vec![[0.0f32, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 0.001);
        assert_eq!(r.new_verts.len(), 2);
    }

    #[test]
    fn merge_identical_verts() {
        let verts = vec![[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 0.001);
        assert_eq!(r.new_verts.len(), 1);
    }

    #[test]
    fn index_map_length() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 0.001);
        assert_eq!(r.index_map.len(), 2);
    }

    #[test]
    fn merge_count_zero_no_merge() {
        let verts = vec![[0.0f32, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 0.001);
        assert_eq!(merge_count(&r), 0);
    }

    #[test]
    fn merge_count_one_merge() {
        let verts = vec![[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 0.001);
        assert_eq!(merge_count(&r), 1);
    }

    #[test]
    fn apply_index_map_remaps_correctly() {
        let verts = vec![[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 0.001);
        let tris = vec![[0u32, 1, 2]];
        let mapped = apply_index_map(&tris, &r.index_map);
        // both 0 and 1 map to same new index
        assert_eq!(mapped[0][0], mapped[0][1]);
    }

    #[test]
    fn empty_verts() {
        let r = merge_vertices(&[], 0.01);
        assert!(r.new_verts.is_empty());
        assert!(r.index_map.is_empty());
    }

    #[test]
    fn threshold_exactly_on_boundary() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let r = merge_vertices(&verts, 1.0);
        assert_eq!(r.new_verts.len(), 1);
    }
}
