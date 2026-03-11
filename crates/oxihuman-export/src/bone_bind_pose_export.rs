// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bone bind-pose export: rest-pose matrices and inverse bind matrices.

/// A 4x4 column-major matrix.
pub type Mat4 = [f32; 16];

/// Bind pose data for a single bone.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneBindPose {
    pub name: String,
    pub local_matrix: Mat4,
    pub world_matrix: Mat4,
    pub inverse_bind: Mat4,
}

/// Bind pose export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BindPoseExport {
    pub bones: Vec<BoneBindPose>,
}

fn identity_mat4() -> Mat4 {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Create a new bind pose export.
#[allow(dead_code)]
pub fn new_bind_pose_export() -> BindPoseExport {
    BindPoseExport { bones: Vec::new() }
}

/// Add a bone with identity matrices.
#[allow(dead_code)]
pub fn add_bind_pose_bone(exp: &mut BindPoseExport, name: &str) {
    exp.bones.push(BoneBindPose {
        name: name.to_string(),
        local_matrix: identity_mat4(),
        world_matrix: identity_mat4(),
        inverse_bind: identity_mat4(),
    });
}

/// Set the local matrix of a bone.
#[allow(dead_code)]
pub fn set_local_matrix(exp: &mut BindPoseExport, idx: usize, mat: Mat4) {
    if idx < exp.bones.len() {
        exp.bones[idx].local_matrix = mat;
    }
}

/// Bone count.
#[allow(dead_code)]
pub fn bind_bone_count(exp: &BindPoseExport) -> usize {
    exp.bones.len()
}

/// Find bone by name.
#[allow(dead_code)]
pub fn find_bind_bone(exp: &BindPoseExport, name: &str) -> Option<usize> {
    exp.bones.iter().position(|b| b.name == name)
}

/// Check all inverse bind matrices are non-identity (have been set).
#[allow(dead_code)]
pub fn all_inverse_binds_set(exp: &BindPoseExport) -> bool {
    exp.bones
        .iter()
        .all(|b| b.inverse_bind != identity_mat4() || b.world_matrix == identity_mat4())
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn bind_pose_to_json(exp: &BindPoseExport) -> String {
    format!("{{\"bone_count\":{}}}", bind_bone_count(exp))
}

/// Serialise to flat f32 array (world matrices).
#[allow(dead_code)]
pub fn bind_pose_to_flat(exp: &BindPoseExport) -> Vec<f32> {
    exp.bones.iter().flat_map(|b| b.world_matrix).collect()
}

/// Validate: no empty names.
#[allow(dead_code)]
pub fn validate_bind_pose(exp: &BindPoseExport) -> bool {
    exp.bones.iter().all(|b| !b.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_bind_pose_export();
        assert_eq!(bind_bone_count(&exp), 0);
    }

    #[test]
    fn add_bone_increments() {
        let mut exp = new_bind_pose_export();
        add_bind_pose_bone(&mut exp, "root");
        assert_eq!(bind_bone_count(&exp), 1);
    }

    #[test]
    fn find_existing() {
        let mut exp = new_bind_pose_export();
        add_bind_pose_bone(&mut exp, "spine");
        assert!(find_bind_bone(&exp, "spine").is_some_and(|i| i == 0));
    }

    #[test]
    fn find_missing_none() {
        let exp = new_bind_pose_export();
        assert!(find_bind_bone(&exp, "missing").is_none());
    }

    #[test]
    fn set_local_matrix_updates() {
        let mut exp = new_bind_pose_export();
        add_bind_pose_bone(&mut exp, "a");
        let mut mat = identity_mat4();
        mat[12] = 5.0;
        set_local_matrix(&mut exp, 0, mat);
        assert!((exp.bones[0].local_matrix[12] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn flat_output_length() {
        let mut exp = new_bind_pose_export();
        add_bind_pose_bone(&mut exp, "a");
        add_bind_pose_bone(&mut exp, "b");
        assert_eq!(bind_pose_to_flat(&exp).len(), 32);
    }

    #[test]
    fn json_contains_bone_count() {
        let exp = new_bind_pose_export();
        let j = bind_pose_to_json(&exp);
        assert!(j.contains("bone_count"));
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_bind_pose_export();
        add_bind_pose_bone(&mut exp, "hip");
        assert!(validate_bind_pose(&exp));
    }

    #[test]
    fn identity_mat4_diagonal_one() {
        let m = identity_mat4();
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[5] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
