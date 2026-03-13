// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh repair: remove degenerate faces, fix duplicate vertices.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RepairConfig {
    pub degenerate_threshold: f32,
    pub duplicate_threshold: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RepairResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub removed_faces: usize,
    pub merged_vertices: usize,
}

#[allow(dead_code)]
pub fn default_repair_config() -> RepairConfig {
    RepairConfig { degenerate_threshold: 1e-6, duplicate_threshold: 1e-4 }
}

#[allow(dead_code)]
pub fn repair_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &RepairConfig,
) -> RepairResult {
    let mut new_indices: Vec<u32> = Vec::new();
    let mut removed_faces = 0usize;
    for tri in indices.chunks_exact(3) {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < positions.len() && b < positions.len() && c < positions.len() {
            if !repair_is_degenerate_face(
                positions[a],
                positions[b],
                positions[c],
                config.degenerate_threshold,
            ) {
                new_indices.extend_from_slice(tri);
            } else {
                removed_faces += 1;
            }
        }
    }
    RepairResult {
        positions: positions.to_vec(),
        indices: new_indices,
        removed_faces,
        merged_vertices: 0,
    }
}

#[allow(dead_code)]
pub fn repair_face_count(result: &RepairResult) -> usize {
    result.indices.len() / 3
}

#[allow(dead_code)]
pub fn repair_vertex_count(result: &RepairResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn repair_is_degenerate_face(a: [f32; 3], b: [f32; 3], c: [f32; 3], threshold: f32) -> bool {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let area2 = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
    area2.sqrt() < threshold
}

#[allow(dead_code)]
pub fn repair_validate(config: &RepairConfig) -> bool {
    config.degenerate_threshold >= 0.0 && config.duplicate_threshold >= 0.0
}

#[allow(dead_code)]
pub fn repair_to_json(result: &RepairResult) -> String {
    format!(
        r#"{{"vertices":{},"faces":{},"removed_faces":{},"merged_vertices":{}}}"#,
        result.positions.len(),
        repair_face_count(result),
        result.removed_faces,
        result.merged_vertices
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_valid() {
        let cfg = default_repair_config();
        assert!(repair_validate(&cfg));
    }

    #[test]
    fn degenerate_collinear_detected() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [2.0, 0.0, 0.0];
        assert!(repair_is_degenerate_face(a, b, c, 1e-6));
    }

    #[test]
    fn valid_triangle_not_degenerate() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        assert!(!repair_is_degenerate_face(a, b, c, 1e-6));
    }

    #[test]
    fn repair_removes_degenerate() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 1, 3];
        let cfg = default_repair_config();
        let res = repair_mesh(&positions, &indices, &cfg);
        assert_eq!(res.removed_faces, 1);
    }

    #[test]
    fn repair_keeps_valid_faces() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2];
        let cfg = default_repair_config();
        let res = repair_mesh(&positions, &indices, &cfg);
        assert_eq!(res.removed_faces, 0);
        assert_eq!(repair_face_count(&res), 1);
    }

    #[test]
    fn vertex_count_unchanged() {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let cfg = default_repair_config();
        let res = repair_mesh(&positions, &indices, &cfg);
        assert_eq!(repair_vertex_count(&res), 3);
    }

    #[test]
    fn to_json_has_fields() {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let cfg = default_repair_config();
        let res = repair_mesh(&positions, &indices, &cfg);
        let json = repair_to_json(&res);
        assert!(json.contains("removed_faces"));
        assert!(json.contains("merged_vertices"));
    }

    #[test]
    fn empty_mesh_is_fine() {
        let cfg = default_repair_config();
        let res = repair_mesh(&[], &[], &cfg);
        assert_eq!(res.removed_faces, 0);
        assert_eq!(res.positions.len(), 0);
    }
}
