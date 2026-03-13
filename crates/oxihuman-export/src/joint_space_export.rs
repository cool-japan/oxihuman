// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export joint-space transform data (local space per joint).

/// A joint-space transform.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointSpaceTransform {
    pub joint_name: String,
    pub local_translation: [f32; 3],
    pub local_rotation: [f32; 4], // quaternion xyzw
    pub local_scale: [f32; 3],
}

/// Joint space export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointSpaceExport {
    pub transforms: Vec<JointSpaceTransform>,
}

/// Create a new joint space export.
#[allow(dead_code)]
pub fn new_joint_space_export() -> JointSpaceExport {
    JointSpaceExport {
        transforms: Vec::new(),
    }
}

/// Add a joint transform.
#[allow(dead_code)]
pub fn add_joint_transform(export: &mut JointSpaceExport, t: JointSpaceTransform) {
    export.transforms.push(t);
}

/// Count transforms.
#[allow(dead_code)]
pub fn joint_transform_count(export: &JointSpaceExport) -> usize {
    export.transforms.len()
}

/// Find by name.
#[allow(dead_code)]
pub fn find_joint_transform<'a>(
    export: &'a JointSpaceExport,
    name: &str,
) -> Option<&'a JointSpaceTransform> {
    export.transforms.iter().find(|t| t.joint_name == name)
}

/// Check that all quaternions are (approximately) unit length.
#[allow(dead_code)]
pub fn quaternions_unit(export: &JointSpaceExport) -> bool {
    export.transforms.iter().all(|t| {
        let q = t.local_rotation;
        let len2 = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
        (len2 - 1.0).abs() < 1e-3
    })
}

/// Check all scales are positive.
#[allow(dead_code)]
pub fn scales_positive_js(export: &JointSpaceExport) -> bool {
    export
        .transforms
        .iter()
        .all(|t| t.local_scale.iter().all(|&s| s > 0.0))
}

/// Default identity transform.
#[allow(dead_code)]
pub fn identity_joint_transform(name: &str) -> JointSpaceTransform {
    JointSpaceTransform {
        joint_name: name.to_string(),
        local_translation: [0.0; 3],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0, 1.0, 1.0],
    }
}

/// Average translation magnitude.
#[allow(dead_code)]
pub fn avg_translation_magnitude(export: &JointSpaceExport) -> f32 {
    let n = export.transforms.len();
    if n == 0 {
        return 0.0;
    }
    export
        .transforms
        .iter()
        .map(|t| {
            let tr = t.local_translation;
            (tr[0] * tr[0] + tr[1] * tr[1] + tr[2] * tr[2]).sqrt()
        })
        .sum::<f32>()
        / n as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn joint_space_to_json(export: &JointSpaceExport) -> String {
    format!("{{\"joint_count\":{}}}", export.transforms.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_count() {
        let mut e = new_joint_space_export();
        add_joint_transform(&mut e, identity_joint_transform("root"));
        assert_eq!(joint_transform_count(&e), 1);
    }

    #[test]
    fn test_find_existing() {
        let mut e = new_joint_space_export();
        add_joint_transform(&mut e, identity_joint_transform("hip"));
        assert!(find_joint_transform(&e, "hip").is_some());
    }

    #[test]
    fn test_find_missing() {
        let e = new_joint_space_export();
        assert!(find_joint_transform(&e, "nope").is_none());
    }

    #[test]
    fn test_identity_quaternion_unit() {
        let mut e = new_joint_space_export();
        add_joint_transform(&mut e, identity_joint_transform("root"));
        assert!(quaternions_unit(&e));
    }

    #[test]
    fn test_scales_positive_identity() {
        let mut e = new_joint_space_export();
        add_joint_transform(&mut e, identity_joint_transform("root"));
        assert!(scales_positive_js(&e));
    }

    #[test]
    fn test_scales_invalid_zero() {
        let mut e = new_joint_space_export();
        let mut t = identity_joint_transform("root");
        t.local_scale = [0.0, 1.0, 1.0];
        add_joint_transform(&mut e, t);
        assert!(!scales_positive_js(&e));
    }

    #[test]
    fn test_avg_translation_identity() {
        let mut e = new_joint_space_export();
        add_joint_transform(&mut e, identity_joint_transform("root"));
        assert!(avg_translation_magnitude(&e).abs() < 1e-6);
    }

    #[test]
    fn test_joint_space_to_json() {
        let e = new_joint_space_export();
        let j = joint_space_to_json(&e);
        assert!(j.contains("joint_count"));
    }

    #[test]
    fn test_avg_empty_zero() {
        let e = new_joint_space_export();
        assert!(avg_translation_magnitude(&e).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_joints() {
        let mut e = new_joint_space_export();
        for name in ["hip", "spine", "head"] {
            add_joint_transform(&mut e, identity_joint_transform(name));
        }
        assert_eq!(joint_transform_count(&e), 3);
    }
}
