#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Skeleton skinning: apply bone transforms to vertices.

#[allow(dead_code)]
pub struct BoneTransformSkin {
    pub matrix: [[f32; 4]; 4],
}

#[allow(dead_code)]
pub struct SkeletonSkinResult {
    pub verts: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn identity_bone() -> BoneTransformSkin {
    BoneTransformSkin {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

#[allow(dead_code)]
pub fn apply_bone_transform(v: [f32; 3], m: &[[f32; 4]; 4]) -> [f32; 3] {
    let x = v[0];
    let y = v[1];
    let z = v[2];
    [
        m[0][0] * x + m[0][1] * y + m[0][2] * z + m[0][3],
        m[1][0] * x + m[1][1] * y + m[1][2] * z + m[1][3],
        m[2][0] * x + m[2][1] * y + m[2][2] * z + m[2][3],
    ]
}

/// Skin a single vertex with linear blend skinning using two influence sets.
///
/// `weights` and `indices` each hold up to 4 influences (two arrays of 4).
#[allow(dead_code)]
pub fn skin_vertex(
    v: [f32; 3],
    bones: &[BoneTransformSkin],
    weights: &[[f32; 4]; 2],
    indices: &[[u32; 4]; 2],
) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for set in 0..2 {
        for k in 0..4 {
            let w = weights[set][k];
            if w < 1e-9 {
                continue;
            }
            let bi = indices[set][k] as usize;
            if bi >= bones.len() {
                continue;
            }
            let t = apply_bone_transform(v, &bones[bi].matrix);
            out[0] += w * t[0];
            out[1] += w * t[1];
            out[2] += w * t[2];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity() -> BoneTransformSkin {
        identity_bone()
    }

    fn translation_bone(tx: f32, ty: f32, tz: f32) -> BoneTransformSkin {
        let mut b = identity_bone();
        b.matrix[0][3] = tx;
        b.matrix[1][3] = ty;
        b.matrix[2][3] = tz;
        b
    }

    #[test]
    fn identity_bone_is_identity() {
        let b = make_identity();
        let v = [1.0f32, 2.0, 3.0];
        let t = apply_bone_transform(v, &b.matrix);
        assert!((t[0] - 1.0).abs() < 1e-6);
        assert!((t[1] - 2.0).abs() < 1e-6);
        assert!((t[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn translation_bone_moves_vertex() {
        let b = translation_bone(1.0, 2.0, 3.0);
        let v = [0.0f32, 0.0, 0.0];
        let t = apply_bone_transform(v, &b.matrix);
        assert!((t[0] - 1.0).abs() < 1e-6);
        assert!((t[1] - 2.0).abs() < 1e-6);
        assert!((t[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn skin_vertex_identity_bone_unchanged() {
        let bones = vec![make_identity()];
        let weights = [[1.0f32, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]];
        let indices = [[0u32, 0, 0, 0], [0, 0, 0, 0]];
        let v = [3.0f32, 4.0, 5.0];
        let out = skin_vertex(v, &bones, &weights, &indices);
        assert!((out[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn skin_vertex_blend_two_bones() {
        let bones = vec![translation_bone(0.0, 0.0, 0.0), translation_bone(2.0, 0.0, 0.0)];
        let weights = [[0.5f32, 0.5, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]];
        let indices = [[0u32, 1, 0, 0], [0, 0, 0, 0]];
        let v = [0.0f32, 0.0, 0.0];
        let out = skin_vertex(v, &bones, &weights, &indices);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn skin_vertex_zero_weight_returns_zero() {
        let bones = vec![translation_bone(5.0, 5.0, 5.0)];
        let weights = [[0.0f32, 0.0, 0.0, 0.0]; 2];
        let indices = [[0u32; 4]; 2];
        let v = [1.0f32, 1.0, 1.0];
        let out = skin_vertex(v, &bones, &weights, &indices);
        assert!(out[0].abs() < 1e-6);
    }

    #[test]
    fn skin_vertex_out_of_range_bone_skipped() {
        let bones = vec![make_identity()];
        let weights = [[1.0f32, 0.0, 0.0, 0.0], [0.0; 4]];
        let indices = [[99u32, 0, 0, 0], [0u32; 4]];
        let v = [1.0f32, 2.0, 3.0];
        let out = skin_vertex(v, &bones, &weights, &indices);
        // bone index 99 is out of range so weight is skipped => [0,0,0]
        assert!(out[0].abs() < 1e-6);
    }

    #[test]
    fn apply_bone_transform_scale() {
        let m = [
            [2.0f32, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let v = [1.0f32, 1.0, 1.0];
        let t = apply_bone_transform(v, &m);
        assert!((t[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn identity_bone_matrix_is_4x4() {
        let b = identity_bone();
        assert_eq!(b.matrix.len(), 4);
        assert_eq!(b.matrix[0].len(), 4);
    }

    #[test]
    fn identity_diagonal_ones() {
        let b = identity_bone();
        for i in 0..4 {
            assert!((b.matrix[i][i] - 1.0).abs() < 1e-9);
        }
    }
}
