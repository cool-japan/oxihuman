// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Skeleton/joint hierarchy for animation-ready mesh export.

/// A joint in a skeleton hierarchy.
#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    /// Index of the parent joint (None for root).
    pub parent: Option<usize>,
    /// Local translation from parent in bind pose [x, y, z].
    pub translation: [f32; 3],
    /// Local rotation as quaternion [x, y, z, w] (identity = `[0,0,0,1]`).
    pub rotation: [f32; 4],
    /// Local scale [x, y, z] (uniform = `[1,1,1]`).
    pub scale: [f32; 3],
}

/// A joint hierarchy defining a skeleton.
#[derive(Debug, Clone, Default)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self { joints: Vec::new() }
    }

    /// Add a joint and return its index.
    pub fn add_joint(&mut self, joint: Joint) -> usize {
        let idx = self.joints.len();
        self.joints.push(joint);
        idx
    }

    /// Return child indices of a given joint.
    pub fn children_of(&self, parent_idx: usize) -> Vec<usize> {
        self.joints
            .iter()
            .enumerate()
            .filter_map(|(i, j)| {
                if j.parent == Some(parent_idx) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Return the root joint indices (joints with no parent).
    pub fn roots(&self) -> Vec<usize> {
        self.joints
            .iter()
            .enumerate()
            .filter_map(|(i, j)| if j.parent.is_none() { Some(i) } else { None })
            .collect()
    }

    /// Return a pre-built human body skeleton with standard joint names
    /// and approximate bind-pose translations scaled for a 170cm figure.
    ///
    /// Joint hierarchy (approximate):
    ///   root → hips → spine → chest → neck → head
    ///   chest → left_shoulder → left_upper_arm → left_forearm → left_hand
    ///   chest → right_shoulder → right_upper_arm → right_forearm → right_hand
    ///   hips → left_upper_leg → left_lower_leg → left_foot
    ///   hips → right_upper_leg → right_lower_leg → right_foot
    pub fn human_body() -> Self {
        let identity_rot = [0.0f32, 0.0, 0.0, 1.0];
        let unit_scale = [1.0f32, 1.0, 1.0];

        let mut sk = Skeleton::new();

        // 0: root  (ground level, world origin)
        let root = sk.add_joint(Joint {
            name: "root".to_string(),
            parent: None,
            translation: [0.0, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 1: hips  (+0.95 m from root, centre of pelvis)
        let hips = sk.add_joint(Joint {
            name: "hips".to_string(),
            parent: Some(root),
            translation: [0.0, 0.95, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 2: spine  (+0.12 m above hips)
        let spine = sk.add_joint(Joint {
            name: "spine".to_string(),
            parent: Some(hips),
            translation: [0.0, 0.12, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 3: chest  (+0.20 m above spine)
        let chest = sk.add_joint(Joint {
            name: "chest".to_string(),
            parent: Some(spine),
            translation: [0.0, 0.20, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 4: neck  (+0.28 m above chest)
        let neck = sk.add_joint(Joint {
            name: "neck".to_string(),
            parent: Some(chest),
            translation: [0.0, 0.28, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 5: head  (+0.12 m above neck)
        let _head = sk.add_joint(Joint {
            name: "head".to_string(),
            parent: Some(neck),
            translation: [0.0, 0.12, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // ── Left arm ─────────────────────────────────────────────────────────
        // 6: left_shoulder  (chest, +x side)
        let left_shoulder = sk.add_joint(Joint {
            name: "left_shoulder".to_string(),
            parent: Some(chest),
            translation: [0.17, 0.24, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 7: left_upper_arm
        let left_upper_arm = sk.add_joint(Joint {
            name: "left_upper_arm".to_string(),
            parent: Some(left_shoulder),
            translation: [0.08, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 8: left_forearm
        let left_forearm = sk.add_joint(Joint {
            name: "left_forearm".to_string(),
            parent: Some(left_upper_arm),
            translation: [0.27, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 9: left_hand
        let _left_hand = sk.add_joint(Joint {
            name: "left_hand".to_string(),
            parent: Some(left_forearm),
            translation: [0.26, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // ── Right arm ────────────────────────────────────────────────────────
        // 10: right_shoulder  (chest, -x side)
        let right_shoulder = sk.add_joint(Joint {
            name: "right_shoulder".to_string(),
            parent: Some(chest),
            translation: [-0.17, 0.24, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 11: right_upper_arm
        let right_upper_arm = sk.add_joint(Joint {
            name: "right_upper_arm".to_string(),
            parent: Some(right_shoulder),
            translation: [-0.08, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 12: right_forearm
        let right_forearm = sk.add_joint(Joint {
            name: "right_forearm".to_string(),
            parent: Some(right_upper_arm),
            translation: [-0.27, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 13: right_hand
        let _right_hand = sk.add_joint(Joint {
            name: "right_hand".to_string(),
            parent: Some(right_forearm),
            translation: [-0.26, 0.0, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // ── Left leg ─────────────────────────────────────────────────────────
        // 14: left_upper_leg
        let left_upper_leg = sk.add_joint(Joint {
            name: "left_upper_leg".to_string(),
            parent: Some(hips),
            translation: [0.10, -0.04, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 15: left_lower_leg
        let left_lower_leg = sk.add_joint(Joint {
            name: "left_lower_leg".to_string(),
            parent: Some(left_upper_leg),
            translation: [0.0, -0.42, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 16: left_foot
        let _left_foot = sk.add_joint(Joint {
            name: "left_foot".to_string(),
            parent: Some(left_lower_leg),
            translation: [0.0, -0.40, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // ── Right leg ────────────────────────────────────────────────────────
        // 17: right_upper_leg
        let right_upper_leg = sk.add_joint(Joint {
            name: "right_upper_leg".to_string(),
            parent: Some(hips),
            translation: [-0.10, -0.04, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 18: right_lower_leg
        let right_lower_leg = sk.add_joint(Joint {
            name: "right_lower_leg".to_string(),
            parent: Some(right_upper_leg),
            translation: [0.0, -0.42, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        // 19: right_foot
        let _right_foot = sk.add_joint(Joint {
            name: "right_foot".to_string(),
            parent: Some(right_lower_leg),
            translation: [0.0, -0.40, 0.0],
            rotation: identity_rot,
            scale: unit_scale,
        });

        sk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_body_has_expected_joint_count() {
        let sk = Skeleton::human_body();
        assert!(
            sk.joints.len() >= 15,
            "expected at least 15 joints, got {}",
            sk.joints.len()
        );
    }

    #[test]
    fn roots_are_single_root() {
        let sk = Skeleton::human_body();
        let roots = sk.roots();
        assert_eq!(
            roots.len(),
            1,
            "expected exactly 1 root, got {}",
            roots.len()
        );
    }

    #[test]
    fn children_of_hips() {
        let sk = Skeleton::human_body();
        // Find the hips joint index
        let hips_idx = sk
            .joints
            .iter()
            .position(|j| j.name == "hips")
            .expect("hips joint not found");
        let children = sk.children_of(hips_idx);
        // hips should have at least 3 children: spine, left_upper_leg, right_upper_leg
        assert!(
            children.len() >= 2,
            "hips should have at least 2 children, got {}",
            children.len()
        );
    }

    #[test]
    fn joint_rotation_identity_is_unit() {
        let sk = Skeleton::human_body();
        for joint in &sk.joints {
            let [x, y, z, w] = joint.rotation;
            let len = (x * x + y * y + z * z + w * w).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "joint '{}' rotation not unit length: {}",
                joint.name,
                len
            );
        }
    }

    #[test]
    fn add_joint_increments_index() {
        let mut sk = Skeleton::new();
        let i0 = sk.add_joint(Joint {
            name: "a".to_string(),
            parent: None,
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        });
        let i1 = sk.add_joint(Joint {
            name: "b".to_string(),
            parent: Some(0),
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        });
        let i2 = sk.add_joint(Joint {
            name: "c".to_string(),
            parent: Some(1),
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        });
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(i2, 2);
    }
}
