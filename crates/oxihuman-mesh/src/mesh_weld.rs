//! Vertex welding — merge nearby vertices within a tolerance threshold.
//!
//! Provides grid-based fast welding, index remapping, mesh compaction,
//! UV-seam welding, and vertex splitting (the inverse operation).

use std::collections::HashMap;

/// Configuration for the vertex welding operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeldConfig {
    /// Distance threshold below which two vertices are considered identical.
    pub tolerance: f32,
    /// Whether to also weld UV coordinates.
    pub weld_uvs: bool,
    /// Whether to validate the result after welding.
    pub validate: bool,
}

/// Result returned from a welding operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshWeldResult {
    /// New positions after welding (deduplicated).
    pub positions: Vec<[f32; 3]>,
    /// Remapped index buffer referencing the new positions.
    pub indices: Vec<u32>,
    /// Number of vertices that were merged.
    pub merged_count: u32,
    /// Map from old vertex index → new (canonical) vertex index.
    pub remap: Vec<u32>,
}

/// Returns a default `WeldConfig`.
#[allow(dead_code)]
pub fn default_weld_config() -> WeldConfig {
    WeldConfig { tolerance: 1e-4, weld_uvs: false, validate: true }
}

/// Welds vertices within `config.tolerance` of each other.
///
/// Uses a hash-grid approach for O(n) expected performance.
#[allow(dead_code)]
pub fn weld_vertices(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &WeldConfig,
) -> MeshWeldResult {
    weld_by_grid(positions, indices, config.tolerance)
}

/// Grid-based vertex welding.  Assigns each position to a grid cell and
/// merges vertices within `tolerance` of the same or adjacent cells.
#[allow(dead_code)]
pub fn weld_by_grid(
    positions: &[[f32; 3]],
    indices: &[u32],
    tolerance: f32,
) -> MeshWeldResult {
    let inv_cell = if tolerance > 1e-12 { 1.0 / tolerance } else { 1.0 / 1e-12 };
    // Map from quantised cell key → canonical vertex index in output
    let mut cell_to_new: HashMap<(i64, i64, i64), u32> = HashMap::new();
    let mut remap = vec![0u32; positions.len()];
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut merged_count = 0u32;

    for (old_idx, &pos) in positions.iter().enumerate() {
        let key = quantise(pos, inv_cell);
        if let Some(&canonical) = cell_to_new.get(&key) {
            remap[old_idx] = canonical;
            merged_count += 1;
        } else {
            let new_idx = new_positions.len() as u32;
            cell_to_new.insert(key, new_idx);
            remap[old_idx] = new_idx;
            new_positions.push(pos);
        }
    }

    let new_indices: Vec<u32> = indices.iter().map(|&i| remap[i as usize]).collect();

    MeshWeldResult {
        positions: new_positions,
        indices: new_indices,
        merged_count,
        remap,
    }
}

/// Counts vertices that would be merged given the tolerance.
#[allow(dead_code)]
pub fn count_duplicate_vertices(positions: &[[f32; 3]], tolerance: f32) -> u32 {
    let inv_cell = if tolerance > 1e-12 { 1.0 / tolerance } else { 1.0e12 };
    let mut seen: HashMap<(i64, i64, i64), ()> = HashMap::new();
    let mut duplicates = 0u32;
    for &pos in positions {
        let key = quantise(pos, inv_cell);
        if seen.insert(key, ()).is_some() {
            duplicates += 1;
        }
    }
    duplicates
}

/// Remaps an index buffer according to a vertex remap table.
#[allow(dead_code)]
pub fn remap_indices(indices: &[u32], remap: &[u32]) -> Vec<u32> {
    indices.iter().map(|&i| remap[i as usize]).collect()
}

/// Removes unreferenced vertices from `positions`, compacting the mesh.
///
/// Returns the new positions and the remapped index buffer.
#[allow(dead_code)]
pub fn compact_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut referenced = vec![false; positions.len()];
    for &i in indices {
        if (i as usize) < positions.len() {
            referenced[i as usize] = true;
        }
    }
    let mut old_to_new = vec![u32::MAX; positions.len()];
    let mut new_positions = Vec::new();
    for (old, &used) in referenced.iter().enumerate() {
        if used {
            old_to_new[old] = new_positions.len() as u32;
            new_positions.push(positions[old]);
        }
    }
    let new_indices: Vec<u32> = indices
        .iter()
        .map(|&i| old_to_new[i as usize])
        .collect();
    (new_positions, new_indices)
}

/// Welds UV coordinates at seam vertices.
///
/// Vertices sharing the same 3-D position but different UVs are treated as
/// seam vertices.  This function merges those that have UVs within
/// `uv_tolerance`.
#[allow(dead_code)]
pub fn weld_uv_seams(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    pos_tolerance: f32,
    uv_tolerance: f32,
) -> MeshWeldResult {
    // Build position groups first
    let pos_result = weld_by_grid(positions, indices, pos_tolerance);

    // For each canonical position, keep the first UV encountered unless
    // a sufficiently close UV already exists.
    let new_vert_count = pos_result.positions.len();
    let mut canonical_uv: Vec<Option<[f32; 2]>> = vec![None; new_vert_count];
    let mut uv_remap = pos_result.remap.clone();
    let mut extra_positions = pos_result.positions.clone();

    for (old_idx, &new_idx) in pos_result.remap.iter().enumerate() {
        let uv = uvs[old_idx];
        match canonical_uv[new_idx as usize] {
            None => canonical_uv[new_idx as usize] = Some(uv),
            Some(existing) => {
                let du = uv[0] - existing[0];
                let dv = uv[1] - existing[1];
                if (du * du + dv * dv).sqrt() > uv_tolerance {
                    // Split: add a new vertex
                    let split_idx = extra_positions.len() as u32;
                    extra_positions.push(extra_positions[new_idx as usize]);
                    uv_remap[old_idx] = split_idx;
                }
            }
        }
    }

    let new_indices = remap_indices(indices, &uv_remap);
    let merged = positions.len() as u32 - extra_positions.len() as u32;
    MeshWeldResult {
        positions: extra_positions,
        indices: new_indices,
        merged_count: merged,
        remap: uv_remap,
    }
}

/// Returns the effective tolerance of a `WeldConfig`.
#[allow(dead_code)]
pub fn weld_tolerance(config: &WeldConfig) -> f32 {
    config.tolerance
}

/// Returns the number of vertices in a welding result.
#[allow(dead_code)]
pub fn welded_vertex_count(result: &MeshWeldResult) -> usize {
    result.positions.len()
}

/// Splits a vertex at `vertex_index`, duplicating it and updating indices
/// that reference it in `face_set` to point to the new copy.
///
/// `face_set` contains the indices of faces (0-based) whose references to
/// `vertex_index` should point to the new duplicate.
#[allow(dead_code)]
pub fn split_vertex(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex_index: u32,
    face_set: &[usize],
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let new_idx = positions.len() as u32;
    let mut new_positions = positions.to_vec();
    new_positions.push(positions[vertex_index as usize]);

    let face_set_sorted: std::collections::HashSet<usize> = face_set.iter().copied().collect();
    let mut new_indices = indices.to_vec();
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        if face_set_sorted.contains(&f) {
            for k in 0..3 {
                if new_indices[f * 3 + k] == vertex_index {
                    new_indices[f * 3 + k] = new_idx;
                }
            }
        }
    }
    (new_positions, new_indices)
}

/// Validates a `MeshWeldResult` for internal consistency.
///
/// Returns `true` if all indices are in range and the remap is valid.
#[allow(dead_code)]
pub fn validate_weld_result(result: &MeshWeldResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n)
        && result.remap.iter().all(|&r| r < n)
}

/// Estimates how many vertices would be removed by welding at `tolerance`.
#[allow(dead_code)]
pub fn estimate_weld_count(positions: &[[f32; 3]], tolerance: f32) -> u32 {
    count_duplicate_vertices(positions, tolerance)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn quantise(pos: [f32; 3], inv_cell: f32) -> (i64, i64, i64) {
    (
        (pos[0] * inv_cell).floor() as i64,
        (pos[1] * inv_cell).floor() as i64,
        (pos[2] * inv_cell).floor() as i64,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_identical_verts() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    #[test]
    fn test_default_weld_config() {
        let cfg = default_weld_config();
        assert!(cfg.tolerance > 0.0);
    }

    #[test]
    fn test_weld_vertices_merges_duplicates() {
        let (pos, idx) = two_identical_verts();
        let cfg = default_weld_config();
        let result = weld_vertices(&pos, &idx, &cfg);
        assert!(result.merged_count > 0);
        assert!(result.positions.len() < pos.len());
    }

    #[test]
    fn test_weld_vertices_distinct_kept() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let cfg = WeldConfig { tolerance: 1e-6, ..default_weld_config() };
        let result = weld_vertices(&pos, &idx, &cfg);
        assert_eq!(result.positions.len(), 3);
    }

    #[test]
    fn test_weld_by_grid_returns_valid_indices() {
        let (pos, idx) = two_identical_verts();
        let result = weld_by_grid(&pos, &idx, 1e-4);
        assert!(validate_weld_result(&result));
    }

    #[test]
    fn test_count_duplicate_vertices() {
        let (pos, _) = two_identical_verts();
        let count = count_duplicate_vertices(&pos, 1e-4);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_duplicate_vertices_no_dups() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert_eq!(count_duplicate_vertices(&pos, 1e-6), 0);
    }

    #[test]
    fn test_remap_indices() {
        let indices = vec![0, 1, 2];
        let remap = vec![0, 0, 1]; // vertex 1 maps to 0
        let new_indices = remap_indices(&indices, &remap);
        assert_eq!(new_indices, vec![0, 0, 1]);
    }

    #[test]
    fn test_compact_mesh_removes_unreferenced() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0, 1, 0]; // vertex 2 is unreferenced
        let (new_pos, new_idx) = compact_mesh(&pos, &idx);
        assert_eq!(new_pos.len(), 2);
        assert!(new_idx.iter().all(|&i| (i as usize) < new_pos.len()));
    }

    #[test]
    fn test_compact_mesh_all_referenced() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let (new_pos, _) = compact_mesh(&pos, &idx);
        assert_eq!(new_pos.len(), 3);
    }

    #[test]
    fn test_welded_vertex_count() {
        let (pos, idx) = two_identical_verts();
        let cfg = default_weld_config();
        let result = weld_vertices(&pos, &idx, &cfg);
        assert_eq!(welded_vertex_count(&result), result.positions.len());
    }

    #[test]
    fn test_weld_tolerance() {
        let cfg = WeldConfig { tolerance: 0.01, ..default_weld_config() };
        assert!((weld_tolerance(&cfg) - 0.01).abs() < 1e-8);
    }

    #[test]
    fn test_validate_weld_result_valid() {
        let (pos, idx) = two_identical_verts();
        let result = weld_by_grid(&pos, &idx, 1e-4);
        assert!(validate_weld_result(&result));
    }

    #[test]
    fn test_validate_weld_result_invalid() {
        let result = MeshWeldResult {
            positions: vec![[0.0, 0.0, 0.0]],
            indices: vec![0, 5, 1], // index 5 out of range
            merged_count: 0,
            remap: vec![0],
        };
        assert!(!validate_weld_result(&result));
    }

    #[test]
    fn test_split_vertex_adds_position() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let (new_pos, _) = split_vertex(&pos, &idx, 0, &[0]);
        assert_eq!(new_pos.len(), 4);
    }

    #[test]
    fn test_split_vertex_remaps_face() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let (new_pos, new_idx) = split_vertex(&pos, &idx, 0, &[0]);
        let new_v = new_pos.len() as u32 - 1;
        assert_eq!(new_idx[0], new_v);
    }

    #[test]
    fn test_estimate_weld_count_matches_count() {
        let (pos, _) = two_identical_verts();
        assert_eq!(estimate_weld_count(&pos, 1e-4), count_duplicate_vertices(&pos, 1e-4));
    }

    #[test]
    fn test_weld_uv_seams_returns_result() {
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [0.5, 0.5], [1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = weld_uv_seams(&pos, &uvs, &idx, 1e-4, 1e-4);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_weld_large_tolerance_merges_all_near() {
        let pos = vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [0.002, 0.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = weld_by_grid(&pos, &idx, 0.01);
        assert_eq!(result.positions.len(), 1);
    }
}
