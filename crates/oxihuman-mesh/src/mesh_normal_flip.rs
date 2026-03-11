// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Flip face normals / vertex normals.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalFlipConfig {
    pub flip_faces: bool,
    pub flip_normals: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalFlipResult {
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub face_count: usize,
}

#[allow(dead_code)]
pub fn default_normal_flip_config() -> NormalFlipConfig {
    NormalFlipConfig { flip_faces: true, flip_normals: true }
}

#[allow(dead_code)]
pub fn flip_mesh_normals(
    indices: &[u32],
    normals: &[[f32; 3]],
    config: &NormalFlipConfig,
) -> NormalFlipResult {
    let out_indices = if config.flip_faces { flip_face_winding(indices) } else { indices.to_vec() };
    let out_normals = if config.flip_normals { flip_normals_only(normals) } else { normals.to_vec() };
    let face_count = out_indices.len() / 3;
    NormalFlipResult { indices: out_indices, normals: out_normals, face_count }
}

#[allow(dead_code)]
pub fn flip_face_winding(indices: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(indices.len());
    for tri in indices.chunks_exact(3) {
        out.push(tri[0]);
        out.push(tri[2]);
        out.push(tri[1]);
    }
    out
}

#[allow(dead_code)]
pub fn flip_normals_only(normals: &[[f32; 3]]) -> Vec<[f32; 3]> {
    normals.iter().map(|n| [-n[0], -n[1], -n[2]]).collect()
}

#[allow(dead_code)]
pub fn normal_flip_validate(indices: &[u32]) -> bool {
    indices.len().is_multiple_of(3)
}

#[allow(dead_code)]
pub fn normal_flip_to_json(result: &NormalFlipResult) -> String {
    format!(
        r#"{{"face_count":{},"normal_count":{}}}"#,
        result.face_count,
        result.normals.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_face_winding_reverses_order() {
        let indices = vec![0u32, 1, 2];
        let flipped = flip_face_winding(&indices);
        assert_eq!(flipped, vec![0u32, 2, 1]);
    }

    #[test]
    fn flip_normals_negates() {
        let normals = vec![[0.0f32, 1.0, 0.0]];
        let flipped = flip_normals_only(&normals);
        assert!((flipped[0][1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn flip_mesh_normals_both() {
        let indices = vec![0u32, 1, 2];
        let normals = vec![[0.0f32, 0.0, 1.0]];
        let cfg = NormalFlipConfig { flip_faces: true, flip_normals: true };
        let res = flip_mesh_normals(&indices, &normals, &cfg);
        assert_eq!(res.indices, vec![0u32, 2, 1]);
        assert!((res.normals[0][2] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn flip_mesh_normals_faces_only() {
        let indices = vec![0u32, 1, 2];
        let normals = vec![[0.0f32, 0.0, 1.0]];
        let cfg = NormalFlipConfig { flip_faces: true, flip_normals: false };
        let res = flip_mesh_normals(&indices, &normals, &cfg);
        assert!((res.normals[0][2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn validate_valid_indices() {
        assert!(normal_flip_validate(&[0u32, 1, 2, 3, 4, 5]));
    }

    #[test]
    fn validate_invalid_indices() {
        assert!(!normal_flip_validate(&[0u32, 1]));
    }

    #[test]
    fn face_count_correct() {
        let indices = vec![0u32, 1, 2, 3, 4, 5];
        let normals = vec![[0.0f32; 3]; 2];
        let cfg = default_normal_flip_config();
        let res = flip_mesh_normals(&indices, &normals, &cfg);
        assert_eq!(res.face_count, 2);
    }

    #[test]
    fn to_json_format() {
        let indices = vec![0u32, 1, 2];
        let normals = vec![[0.0f32; 3]];
        let cfg = default_normal_flip_config();
        let res = flip_mesh_normals(&indices, &normals, &cfg);
        let json = normal_flip_to_json(&res);
        assert!(json.contains("face_count"));
        assert!(json.contains("normal_count"));
    }
}
