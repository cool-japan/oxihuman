// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Remove isolated (unreferenced) vertices from mesh buffers.

/// Result of the isolated-vertex removal pass.
#[derive(Debug, Clone, Default)]
pub struct IsolatedVertexResult {
    pub original_count: usize,
    pub isolated_count: usize,
    pub final_count: usize,
}

/// Returns the set of vertex indices that are actually used by the index buffer.
pub fn used_vertex_set(indices: &[u32], vertex_count: usize) -> Vec<bool> {
    let mut used = vec![false; vertex_count];
    for &idx in indices {
        if (idx as usize) < vertex_count {
            used[idx as usize] = true;
        }
    }
    used
}

/// Counts isolated (unreferenced) vertices.
pub fn count_isolated(indices: &[u32], vertex_count: usize) -> usize {
    used_vertex_set(indices, vertex_count)
        .iter()
        .filter(|&&u| !u)
        .count()
}

/// Builds a remap table from old vertex index → new compact index.
/// Isolated vertices are mapped to `u32::MAX`.
pub fn build_compact_remap(indices: &[u32], vertex_count: usize) -> Vec<u32> {
    let used = used_vertex_set(indices, vertex_count);
    let mut remap = vec![u32::MAX; vertex_count];
    let mut next = 0u32;
    for (i, &u) in used.iter().enumerate() {
        if u {
            remap[i] = next;
            next += 1;
        }
    }
    remap
}

/// Removes isolated vertices from a position buffer and remaps indices.
pub fn remove_isolated_vertices(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> (Vec<[f32; 3]>, Vec<u32>, IsolatedVertexResult) {
    let vertex_count = positions.len();
    let remap = build_compact_remap(indices, vertex_count);
    let new_positions: Vec<[f32; 3]> = positions
        .iter()
        .enumerate()
        .filter(|(i, _)| remap[*i] != u32::MAX)
        .map(|(_, p)| *p)
        .collect();
    let new_indices: Vec<u32> = indices.iter().map(|&i| remap[i as usize]).collect();
    let isolated = vertex_count.saturating_sub(new_positions.len());
    let result = IsolatedVertexResult {
        original_count: vertex_count,
        isolated_count: isolated,
        final_count: new_positions.len(),
    };
    (new_positions, new_indices, result)
}

/// Returns a list of isolated vertex indices.
pub fn isolated_vertex_indices(indices: &[u32], vertex_count: usize) -> Vec<usize> {
    used_vertex_set(indices, vertex_count)
        .iter()
        .enumerate()
        .filter(|(_, &u)| !u)
        .map(|(i, _)| i)
        .collect()
}

/// Returns `true` if all vertices are referenced.
pub fn no_isolated_vertices(indices: &[u32], vertex_count: usize) -> bool {
    count_isolated(indices, vertex_count) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh_with_orphan() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [9.0, 9.0, 9.0], /* isolated */
        ];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn count_isolated_one() {
        let (pos, idx) = mesh_with_orphan();
        assert_eq!(count_isolated(&idx, pos.len()), 1);
    }

    #[test]
    fn remove_isolated_reduces_vertex_count() {
        let (pos, idx) = mesh_with_orphan();
        let (new_pos, _, res) = remove_isolated_vertices(&pos, &idx);
        assert_eq!(new_pos.len(), 3);
        assert_eq!(res.isolated_count, 1);
    }

    #[test]
    fn indices_remapped_correctly() {
        let (pos, idx) = mesh_with_orphan();
        let (_, new_idx, _) = remove_isolated_vertices(&pos, &idx);
        assert_eq!(new_idx, vec![0u32, 1, 2]);
    }

    #[test]
    fn no_isolated_when_all_used() {
        let pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        assert!(no_isolated_vertices(&idx, pos.len()));
    }

    #[test]
    fn isolated_vertex_list_contains_orphan() {
        let (pos, idx) = mesh_with_orphan();
        let iso = isolated_vertex_indices(&idx, pos.len());
        assert!(iso.contains(&3));
    }

    #[test]
    fn build_compact_remap_max_for_isolated() {
        let (pos, idx) = mesh_with_orphan();
        let remap = build_compact_remap(&idx, pos.len());
        assert_eq!(remap[3], u32::MAX);
    }

    #[test]
    fn used_vertex_set_correct() {
        let idx = vec![0u32, 1, 2];
        let used = used_vertex_set(&idx, 4);
        assert!(used[0] && used[1] && used[2] && !used[3]);
    }

    #[test]
    fn empty_mesh_no_isolated() {
        let pos: Vec<[f32; 3]> = vec![];
        let idx: Vec<u32> = vec![];
        assert_eq!(count_isolated(&idx, pos.len()), 0);
    }

    #[test]
    fn result_stats_consistent() {
        let (pos, idx) = mesh_with_orphan();
        let (_, _, res) = remove_isolated_vertices(&pos, &idx);
        assert_eq!(res.original_count, res.isolated_count + res.final_count);
    }
}
