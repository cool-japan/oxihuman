/// Vertex clustering and merging by proximity.
#[allow(dead_code)]
pub struct ClusterConfig {
    pub merge_distance: f32,
    pub preserve_boundaries: bool,
}

#[allow(dead_code)]
pub struct ClusterResult {
    pub new_positions: Vec<[f32; 3]>,
    pub remap: Vec<usize>,
    pub new_indices: Vec<u32>,
    pub cluster_count: usize,
}

#[allow(dead_code)]
pub fn default_cluster_config() -> ClusterConfig {
    ClusterConfig {
        merge_distance: 0.01,
        preserve_boundaries: true,
    }
}

/// Map a float coordinate to a grid cell index given a cell size.
fn grid_cell(v: f32, cell_size: f32) -> i32 {
    (v / cell_size).floor() as i32
}

/// Build a list of (vertex_idx, grid_cell) pairs for spatial hashing.
#[allow(dead_code)]
pub fn build_cluster_grid(positions: &[[f32; 3]], cell_size: f32) -> Vec<(usize, [i32; 3])> {
    let cs = if cell_size <= 0.0 { 1.0 } else { cell_size };
    positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let cell = [
                grid_cell(p[0], cs),
                grid_cell(p[1], cs),
                grid_cell(p[2], cs),
            ];
            (i, cell)
        })
        .collect()
}

/// Compute the centroid of a set of vertex indices.
#[allow(dead_code)]
pub fn cluster_centroid(positions: &[[f32; 3]], members: &[usize]) -> [f32; 3] {
    if members.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for &idx in members {
        for k in 0..3 {
            sum[k] += positions[idx][k];
        }
    }
    let n = members.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Merge vertices closer than threshold. Returns (new_positions, remap).
#[allow(dead_code)]
pub fn merge_close_vertices(positions: &[[f32; 3]], threshold: f32) -> (Vec<[f32; 3]>, Vec<usize>) {
    let n = positions.len();
    let mut remap = vec![usize::MAX; n];
    let mut new_positions: Vec<[f32; 3]> = Vec::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let new_idx = new_positions.len();
        remap[i] = new_idx;
        // Find all subsequent vertices within threshold and remap them too
        for j in (i + 1)..n {
            if remap[j] == usize::MAX {
                let dx = positions[i][0] - positions[j][0];
                let dy = positions[i][1] - positions[j][1];
                let dz = positions[i][2] - positions[j][2];
                let dist2 = dx * dx + dy * dy + dz * dz;
                if dist2 <= threshold * threshold {
                    remap[j] = new_idx;
                }
            }
        }
        new_positions.push(positions[i]);
    }

    (new_positions, remap)
}

/// Remap index buffer using the provided vertex remap table.
#[allow(dead_code)]
pub fn apply_vertex_remap(indices: &[u32], remap: &[usize]) -> Vec<u32> {
    indices
        .iter()
        .map(|&idx| {
            let new_idx = remap.get(idx as usize).copied().unwrap_or(idx as usize);
            new_idx as u32
        })
        .collect()
}

/// Remove degenerate triangles where any two indices are equal.
#[allow(dead_code)]
pub fn remove_degenerate_triangles(indices: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(indices.len());
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        if a != b && b != c && a != c {
            result.push(a);
            result.push(b);
            result.push(c);
        }
    }
    result
}

/// Grid-based vertex clustering.
#[allow(dead_code)]
pub fn cluster_vertices(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &ClusterConfig,
) -> ClusterResult {
    let (new_positions, remap) = merge_close_vertices(positions, cfg.merge_distance);
    let remapped = apply_vertex_remap(indices, &remap);
    let new_indices = remove_degenerate_triangles(&remapped);
    let cluster_count = new_positions.len();
    ClusterResult {
        new_positions,
        remap,
        new_indices,
        cluster_count,
    }
}

#[allow(dead_code)]
pub fn cluster_reduction_ratio(original: usize, result: &ClusterResult) -> f32 {
    if original == 0 {
        return 1.0;
    }
    1.0 - (result.cluster_count as f32 / original as f32)
}

#[allow(dead_code)]
pub fn cluster_vertex_count(result: &ClusterResult) -> usize {
    result.new_positions.len()
}

#[allow(dead_code)]
pub fn cluster_face_count(result: &ClusterResult) -> usize {
    result.new_indices.len() / 3
}

/// Verify that all remap values are < new_count.
#[allow(dead_code)]
pub fn verify_cluster_remap(remap: &[usize], new_count: usize) -> bool {
    remap.iter().all(|&r| r < new_count)
}

/// Merge duplicate UV coordinates within threshold.
#[allow(dead_code)]
pub fn merge_duplicate_uvs(uvs: &[[f32; 2]], threshold: f32) -> (Vec<[f32; 2]>, Vec<usize>) {
    let n = uvs.len();
    let mut remap = vec![usize::MAX; n];
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let new_idx = new_uvs.len();
        remap[i] = new_idx;
        for j in (i + 1)..n {
            if remap[j] == usize::MAX {
                let du = uvs[i][0] - uvs[j][0];
                let dv = uvs[i][1] - uvs[j][1];
                if du * du + dv * dv <= threshold * threshold {
                    remap[j] = new_idx;
                }
            }
        }
        new_uvs.push(uvs[i]);
    }

    (new_uvs, remap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0001], // very close to vertex 0
            [1.0, 1.0, 0.0],
        ]
    }

    fn make_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 4, 2, 0, 3, 1]
    }

    #[test]
    fn test_default_cluster_config() {
        let cfg = default_cluster_config();
        assert!(cfg.merge_distance > 0.0);
    }

    #[test]
    fn test_cluster_vertices_basic() {
        let positions = make_positions();
        let indices = make_indices();
        let cfg = ClusterConfig {
            merge_distance: 0.01,
            preserve_boundaries: false,
        };
        let result = cluster_vertices(&positions, &indices, &cfg);
        // vertex 3 should merge with vertex 0
        assert!(result.cluster_count < positions.len());
    }

    #[test]
    fn test_merge_close_vertices_collapses_duplicates() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.001], // very close
            [1.0, 0.0, 0.0],
        ];
        let (new_pos, remap) = merge_close_vertices(&positions, 0.01);
        assert!(new_pos.len() < positions.len());
        assert_eq!(remap[0], remap[1]); // merged
        assert_ne!(remap[0], remap[2]);
    }

    #[test]
    fn test_merge_close_vertices_no_merge() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let (new_pos, remap) = merge_close_vertices(&positions, 0.001);
        assert_eq!(new_pos.len(), 3);
        assert_eq!(remap.len(), 3);
    }

    #[test]
    fn test_apply_vertex_remap() {
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let remap = vec![0usize, 1, 2, 0]; // vertex 3 → vertex 0
        let result = apply_vertex_remap(&indices, &remap);
        assert_eq!(result, vec![0, 1, 2, 0, 2, 0]);
    }

    #[test]
    fn test_remove_degenerate_triangles() {
        let indices = vec![0u32, 1, 2, 0, 0, 1, 1, 2, 3]; // second tri is degenerate
        let result = remove_degenerate_triangles(&indices);
        assert_eq!(result.len(), 6); // only 2 valid triangles
    }

    #[test]
    fn test_remove_degenerate_all_degenerate() {
        let indices = vec![0u32, 0, 1];
        let result = remove_degenerate_triangles(&indices);
        assert!(result.is_empty());
    }

    #[test]
    fn test_cluster_reduction_ratio() {
        let result = ClusterResult {
            new_positions: vec![[0.0; 3]; 4],
            remap: vec![0, 1, 2, 3],
            new_indices: vec![],
            cluster_count: 4,
        };
        let ratio = cluster_reduction_ratio(8, &result);
        assert!((ratio - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_cluster_reduction_ratio_zero_original() {
        let result = ClusterResult {
            new_positions: vec![],
            remap: vec![],
            new_indices: vec![],
            cluster_count: 0,
        };
        let ratio = cluster_reduction_ratio(0, &result);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_verify_cluster_remap_valid() {
        let remap = vec![0usize, 1, 2, 0, 1];
        assert!(verify_cluster_remap(&remap, 3));
    }

    #[test]
    fn test_verify_cluster_remap_invalid() {
        let remap = vec![0usize, 1, 5]; // 5 >= new_count=3
        assert!(!verify_cluster_remap(&remap, 3));
    }

    #[test]
    fn test_merge_duplicate_uvs() {
        let uvs = vec![[0.0f32, 0.0], [0.0, 0.001], [1.0, 1.0]];
        let (new_uvs, remap) = merge_duplicate_uvs(&uvs, 0.01);
        assert_eq!(new_uvs.len(), 2); // first two are merged
        assert_eq!(remap[0], remap[1]);
        assert_ne!(remap[0], remap[2]);
    }

    #[test]
    fn test_build_cluster_grid() {
        let positions = vec![[0.5, 0.5, 0.5], [1.5, 1.5, 1.5]];
        let grid = build_cluster_grid(&positions, 1.0);
        assert_eq!(grid.len(), 2);
        assert_eq!(grid[0].1, [0, 0, 0]);
        assert_eq!(grid[1].1, [1, 1, 1]);
    }

    #[test]
    fn test_cluster_centroid() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let members = vec![0, 1, 2];
        let c = cluster_centroid(&positions, &members);
        assert!((c[0] - 2.0 / 3.0).abs() < 1e-5);
        assert!((c[1] - 2.0 / 3.0).abs() < 1e-5);
        assert!((c[2]).abs() < 1e-5);
    }

    #[test]
    fn test_cluster_vertex_count() {
        let result = ClusterResult {
            new_positions: vec![[0.0; 3]; 5],
            remap: vec![0, 1, 2, 3, 4],
            new_indices: vec![0, 1, 2, 2, 3, 4],
            cluster_count: 5,
        };
        assert_eq!(cluster_vertex_count(&result), 5);
        assert_eq!(cluster_face_count(&result), 2);
    }
}
