// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Result of edge extrusion.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExtrudeEdgeResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub new_vertex_count: usize,
}

/// Extrude edges along a direction, creating new faces.
#[allow(dead_code)]
pub fn extrude_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    edge_vertices: &[u32],
    direction: [f32; 3],
    distance: f32,
) -> ExtrudeEdgeResult {
    let mut new_pos = positions.to_vec();
    let mut new_idx = indices.to_vec();
    let mut vertex_map = std::collections::HashMap::new();
    for &v in edge_vertices {
        let old_pos = positions[v as usize];
        let new_p = [
            old_pos[0] + direction[0] * distance,
            old_pos[1] + direction[1] * distance,
            old_pos[2] + direction[2] * distance,
        ];
        let new_v = new_pos.len() as u32;
        new_pos.push(new_p);
        vertex_map.insert(v, new_v);
    }
    for pair in edge_vertices.windows(2) {
        let v0 = pair[0];
        let v1 = pair[1];
        if let (Some(&nv0), Some(&nv1)) = (vertex_map.get(&v0), vertex_map.get(&v1)) {
            new_idx.extend_from_slice(&[v0, v1, nv1]);
            new_idx.extend_from_slice(&[v0, nv1, nv0]);
        }
    }
    let new_vertex_count = vertex_map.len();
    ExtrudeEdgeResult { positions: new_pos, indices: new_idx, new_vertex_count }
}

/// Compute edge length between two vertices.
#[allow(dead_code)]
pub fn edge_length(positions: &[[f32; 3]], v0: u32, v1: u32) -> f32 {
    let a = &positions[v0 as usize];
    let b = &positions[v1 as usize];
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Normalize a direction vector.
#[allow(dead_code)]
pub fn normalize_dir(dir: [f32; 3]) -> [f32; 3] {
    let len = (dir[0]*dir[0] + dir[1]*dir[1] + dir[2]*dir[2]).sqrt();
    if len < 1e-10 { return [0.0, 1.0, 0.0]; }
    [dir[0]/len, dir[1]/len, dir[2]/len]
}

/// Serialize extrude result to JSON.
#[allow(dead_code)]
pub fn extrude_to_json(result: &ExtrudeEdgeResult) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"new_verts\":{}}}",
        result.positions.len(),
        result.indices.len() / 3,
        result.new_vertex_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![0,1,2])
    }

    #[test]
    fn test_extrude_basic() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[0, 1], [0.0, 1.0, 0.0], 1.0);
        assert_eq!(result.new_vertex_count, 2);
        assert_eq!(result.positions.len(), 5);
    }

    #[test]
    fn test_extrude_adds_faces() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[0, 1], [0.0, 1.0, 0.0], 1.0);
        assert!(result.indices.len() > idx.len());
    }

    #[test]
    fn test_extrude_position() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[0], [0.0, 1.0, 0.0], 2.0);
        let last = result.positions.last().expect("should succeed");
        assert!((last[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_edge_length() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
        assert!((edge_length(&pos, 0, 1) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_dir() {
        let n = normalize_dir([3.0, 4.0, 0.0]);
        let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_zero() {
        let n = normalize_dir([0.0, 0.0, 0.0]);
        assert!((n[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_extrude_empty_edges() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[], [0.0, 1.0, 0.0], 1.0);
        assert_eq!(result.new_vertex_count, 0);
    }

    #[test]
    fn test_extrude_to_json() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[0, 1], [0.0, 1.0, 0.0], 1.0);
        let json = extrude_to_json(&result);
        assert!(json.contains("new_verts"));
    }

    #[test]
    fn test_preserves_original_faces() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[0, 1], [0.0, 1.0, 0.0], 1.0);
        assert_eq!(result.indices[0], idx[0]);
    }

    #[test]
    fn test_zero_distance() {
        let (pos, idx) = base_mesh();
        let result = extrude_edges(&pos, &idx, &[0, 1], [0.0, 1.0, 0.0], 0.0);
        let last = result.positions.last().expect("should succeed");
        assert!((last[0] - 1.0).abs() < 1e-5);
    }
}
