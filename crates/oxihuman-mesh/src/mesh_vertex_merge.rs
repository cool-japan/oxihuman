#![allow(dead_code)]

use std::collections::HashMap;

/// Result of a vertex merge operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub merge_map: HashMap<usize, usize>,
    pub merged_count: usize,
    pub original_count: usize,
}

/// Merge vertices that are within a distance threshold.
#[allow(dead_code)]
pub fn merge_vertices_by_distance(vertices: &[[f32; 3]], threshold: f32) -> MergeResult {
    let mut merge_map: HashMap<usize, usize> = HashMap::new();
    let threshold_sq = threshold * threshold;

    for i in 0..vertices.len() {
        if merge_map.contains_key(&i) {
            continue;
        }
        for j in (i + 1)..vertices.len() {
            if merge_map.contains_key(&j) {
                continue;
            }
            let dx = vertices[j][0] - vertices[i][0];
            let dy = vertices[j][1] - vertices[i][1];
            let dz = vertices[j][2] - vertices[i][2];
            if dx * dx + dy * dy + dz * dz <= threshold_sq {
                merge_map.insert(j, i);
            }
        }
    }

    let merged_count = merge_map.len();
    MergeResult {
        merge_map,
        merged_count,
        original_count: vertices.len(),
    }
}

/// Get the number of vertices merged.
#[allow(dead_code)]
pub fn merge_count(result: &MergeResult) -> usize {
    result.merged_count
}

/// Get the merge threshold used (for display purposes).
#[allow(dead_code)]
pub fn merge_threshold(avg_edge_length: f32, factor: f32) -> f32 {
    avg_edge_length * factor
}

/// Merge vertices at specific index pairs.
#[allow(dead_code)]
pub fn merge_at_indices(pairs: &[(usize, usize)]) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    for &(from, to) in pairs {
        map.insert(from, to);
    }
    map
}

/// Build a merge map from a list of vertex groups.
#[allow(dead_code)]
pub fn build_merge_map(groups: &[Vec<usize>]) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    for group in groups {
        if group.len() < 2 {
            continue;
        }
        let target = group[0];
        for &v in &group[1..] {
            map.insert(v, target);
        }
    }
    map
}

/// Apply a merge map to a face index buffer.
#[allow(dead_code)]
pub fn apply_merge_map(faces: &mut [[usize; 3]], map: &HashMap<usize, usize>) {
    for face in faces.iter_mut() {
        for v in face.iter_mut() {
            if let Some(&target) = map.get(v) {
                *v = target;
            }
        }
    }
}

/// Check if merge preserves topology (no degenerate faces created).
#[allow(dead_code)]
pub fn merge_preserves_topology(faces: &[[usize; 3]], map: &HashMap<usize, usize>) -> bool {
    for face in faces {
        let mut mapped = *face;
        for v in mapped.iter_mut() {
            if let Some(&target) = map.get(v) {
                *v = target;
            }
        }
        if mapped[0] == mapped[1] || mapped[1] == mapped[2] || mapped[0] == mapped[2] {
            return false;
        }
    }
    true
}

/// Count the number of unique vertices after merging.
#[allow(dead_code)]
pub fn merged_vertex_count(result: &MergeResult) -> usize {
    result.original_count - result.merged_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_close_vertices() {
        let verts = vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let result = merge_vertices_by_distance(&verts, 0.01);
        assert_eq!(merge_count(&result), 1);
    }

    #[test]
    fn test_no_merge() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let result = merge_vertices_by_distance(&verts, 0.01);
        assert_eq!(merge_count(&result), 0);
    }

    #[test]
    fn test_merged_vertex_count() {
        let verts = vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let result = merge_vertices_by_distance(&verts, 0.01);
        assert_eq!(merged_vertex_count(&result), 2);
    }

    #[test]
    fn test_merge_threshold() {
        let t = merge_threshold(1.0, 0.01);
        assert!((t - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_merge_at_indices() {
        let map = merge_at_indices(&[(1, 0), (3, 2)]);
        assert_eq!(map[&1], 0);
        assert_eq!(map[&3], 2);
    }

    #[test]
    fn test_build_merge_map() {
        let groups = vec![vec![0, 1, 2], vec![3, 4]];
        let map = build_merge_map(&groups);
        assert_eq!(map[&1], 0);
        assert_eq!(map[&2], 0);
        assert_eq!(map[&4], 3);
    }

    #[test]
    fn test_apply_merge_map() {
        let mut faces = vec![[0, 1, 2]];
        let mut map = HashMap::new();
        map.insert(1usize, 0usize);
        apply_merge_map(&mut faces, &map);
        assert_eq!(faces[0], [0, 0, 2]);
    }

    #[test]
    fn test_merge_preserves_topology() {
        let faces = vec![[0, 1, 2]];
        let mut map = HashMap::new();
        map.insert(2usize, 3usize);
        assert!(merge_preserves_topology(&faces, &map));
    }

    #[test]
    fn test_merge_breaks_topology() {
        let faces = vec![[0, 1, 2]];
        let mut map = HashMap::new();
        map.insert(1usize, 0usize);
        assert!(!merge_preserves_topology(&faces, &map));
    }

    #[test]
    fn test_empty_merge() {
        let verts: Vec<[f32; 3]> = vec![];
        let result = merge_vertices_by_distance(&verts, 0.01);
        assert_eq!(merge_count(&result), 0);
    }
}
