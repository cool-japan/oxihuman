// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Apply affine transforms to mesh vertices.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshTransformConfig {
    pub preserve_normals: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshTransformResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn default_mesh_transform_config() -> MeshTransformConfig {
    MeshTransformConfig { preserve_normals: true }
}

#[allow(dead_code)]
pub fn apply_translation(positions: &[[f32; 3]], offset: [f32; 3]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|p| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]])
        .collect()
}

#[allow(dead_code)]
pub fn apply_rotation_y(positions: &[[f32; 3]], angle_rad: f32) -> Vec<[f32; 3]> {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    positions
        .iter()
        .map(|p| [
            p[0] * cos_a + p[2] * sin_a,
            p[1],
            -p[0] * sin_a + p[2] * cos_a,
        ])
        .collect()
}

#[allow(dead_code)]
pub fn apply_scale_uniform(positions: &[[f32; 3]], s: f32) -> Vec<[f32; 3]> {
    positions.iter().map(|p| [p[0] * s, p[1] * s, p[2] * s]).collect()
}

#[allow(dead_code)]
pub fn apply_transform_matrix(positions: &[[f32; 3]], mat: &[[f32; 4]; 4]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|p| {
            let x = mat[0][0] * p[0] + mat[0][1] * p[1] + mat[0][2] * p[2] + mat[0][3];
            let y = mat[1][0] * p[0] + mat[1][1] * p[1] + mat[1][2] * p[2] + mat[1][3];
            let z = mat[2][0] * p[0] + mat[2][1] * p[1] + mat[2][2] * p[2] + mat[2][3];
            [x, y, z]
        })
        .collect()
}

#[allow(dead_code)]
pub fn mesh_transform_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let n = positions.len() as f32;
    let sum = positions.iter().fold([0.0f32; 3], |acc, p| [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]);
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[allow(dead_code)]
pub fn mesh_transform_to_json(result: &MeshTransformResult) -> String {
    format!(
        r#"{{"vertex_count":{},"normal_count":{}}}"#,
        result.positions.len(),
        result.normals.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn translation_shifts_coords() {
        let pos = vec![[1.0f32, 2.0, 3.0]];
        let out = apply_translation(&pos, [1.0, 0.0, -1.0]);
        assert!((out[0][0] - 2.0).abs() < 1e-6);
        assert!((out[0][2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn rotation_y_ninety_deg() {
        let pos = vec![[1.0f32, 0.0, 0.0]];
        let out = apply_rotation_y(&pos, PI / 2.0);
        // (1,0,0) → (0,0,-1) for rotation around Y by 90deg
        assert!(out[0][0].abs() < 1e-5);
        assert!((out[0][2] + 1.0).abs() < 1e-5);
    }

    #[test]
    fn scale_uniform_doubles() {
        let pos = vec![[1.0f32, 2.0, 3.0]];
        let out = apply_scale_uniform(&pos, 2.0);
        assert!((out[0][0] - 2.0).abs() < 1e-6);
        assert!((out[0][1] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn transform_matrix_identity() {
        let pos = vec![[1.0f32, 2.0, 3.0]];
        let identity = [
            [1.0f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let out = apply_transform_matrix(&pos, &identity);
        assert!((out[0][0] - 1.0).abs() < 1e-6);
        assert!((out[0][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn centroid_of_square() {
        let pos = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 2.0, 0.0], [0.0, 2.0, 0.0]];
        let c = mesh_transform_centroid(&pos);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn centroid_empty_is_zero() {
        let c = mesh_transform_centroid(&[]);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn to_json_fields() {
        let res = MeshTransformResult { positions: vec![[0.0; 3]; 3], normals: vec![[0.0; 3]; 3] };
        let json = mesh_transform_to_json(&res);
        assert!(json.contains("vertex_count"));
        assert!(json.contains("normal_count"));
    }

    #[test]
    fn translation_preserves_count() {
        let pos = vec![[0.0f32; 3]; 10];
        let out = apply_translation(&pos, [1.0, 1.0, 1.0]);
        assert_eq!(out.len(), 10);
    }
}
