// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Skeletal pose library: named pose presets stored as per-joint rotation quaternions.

use crate::skeleton::Skeleton;

/// A quaternion [x, y, z, w] representing a joint rotation.
pub type Quat = [f32; 4];

/// Identity quaternion (no rotation).
pub const IDENTITY_QUAT: Quat = [0.0, 0.0, 0.0, 1.0];

/// A full pose: per-joint rotation overrides (by joint name).
/// Joints not listed use `IDENTITY_QUAT`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pose {
    pub name: String,
    /// Map from joint name to quaternion rotation.
    pub rotations: std::collections::HashMap<String, Quat>,
}

impl Pose {
    /// Create a new, empty pose with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rotations: std::collections::HashMap::new(),
        }
    }

    /// Get the rotation for a joint by name, falling back to identity.
    pub fn rotation_for(&self, joint_name: &str) -> Quat {
        self.rotations
            .get(joint_name)
            .copied()
            .unwrap_or(IDENTITY_QUAT)
    }

    /// Set a joint rotation.
    pub fn set_rotation(&mut self, joint_name: impl Into<String>, quat: Quat) {
        self.rotations.insert(joint_name.into(), quat);
    }

    /// Apply this pose to a skeleton: returns a new Skeleton with rotations overridden.
    pub fn apply_to_skeleton(&self, skeleton: &Skeleton) -> Skeleton {
        let mut new_skeleton = skeleton.clone();
        for joint in &mut new_skeleton.joints {
            joint.rotation = self.rotation_for(&joint.name);
        }
        new_skeleton
    }

    /// Interpolate between two poses at `t` in `[0,1]` using per-joint quaternion SLERP.
    pub fn blend(a: &Pose, b: &Pose, t: f32) -> Pose {
        // Collect all unique joint names from both poses.
        let mut all_names: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for key in a.rotations.keys() {
            all_names.insert(key.as_str());
        }
        for key in b.rotations.keys() {
            all_names.insert(key.as_str());
        }

        let mut result = Pose::new(format!("blend({},{},{})", a.name, b.name, t));
        for name in all_names {
            let qa = a.rotation_for(name);
            let qb = b.rotation_for(name);
            let blended = quat_slerp(qa, qb, t);
            result.set_rotation(name, blended);
        }
        result
    }
}

/// A library of named poses.
pub struct PoseLibrary {
    poses: Vec<Pose>,
}

impl Default for PoseLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl PoseLibrary {
    /// Create an empty pose library.
    pub fn new() -> Self {
        Self { poses: Vec::new() }
    }

    /// Add a pose to the library.
    pub fn add(&mut self, pose: Pose) {
        self.poses.push(pose);
    }

    /// Look up a pose by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&Pose> {
        let lower = name.to_lowercase();
        self.poses.iter().find(|p| p.name.to_lowercase() == lower)
    }

    /// Returns all pose names.
    pub fn names(&self) -> Vec<&str> {
        self.poses.iter().map(|p| p.name.as_str()).collect()
    }

    /// Built-in pose library with standard poses.
    pub fn standard() -> Self {
        let mut lib = PoseLibrary::new();

        // 1. T-pose: arms horizontal (90 degrees from sides).
        //    left_shoulder: 90 degrees around Z (arm goes up/out in local space)
        //    right_shoulder: -90 degrees around Z
        let mut t_pose = Pose::new("t-pose");
        t_pose.set_rotation("left_shoulder", [0.0, 0.0, 0.707, 0.707]);
        t_pose.set_rotation("right_shoulder", [0.0, 0.0, -0.707, 0.707]);
        lib.add(t_pose);

        // 2. A-pose: arms at ~45 degrees.
        let mut a_pose = Pose::new("a-pose");
        a_pose.set_rotation("left_shoulder", [0.0, 0.0, 0.383, 0.924]);
        a_pose.set_rotation("right_shoulder", [0.0, 0.0, -0.383, 0.924]);
        lib.add(a_pose);

        // 3. Standing: natural standing / bind pose (all joints identity).
        let standing = Pose::new("standing");
        lib.add(standing);

        // 4. Sitting: simplified sitting pose.
        //    upper legs rotate forward ~90 degrees around X.
        //    lower legs rotate back ~90 degrees around X.
        let mut sitting = Pose::new("sitting");
        sitting.set_rotation("left_upper_leg", [0.707, 0.0, 0.0, 0.707]);
        sitting.set_rotation("right_upper_leg", [0.707, 0.0, 0.0, 0.707]);
        sitting.set_rotation("left_lower_leg", [-0.707, 0.0, 0.0, 0.707]);
        sitting.set_rotation("right_lower_leg", [-0.707, 0.0, 0.0, 0.707]);
        lib.add(sitting);

        lib
    }
}

// ── Internal quaternion math ──────────────────────────────────────────────────

fn quat_slerp(a: Quat, b: Quat, t: f32) -> Quat {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    // If dot < 0, negate b to take the shorter arc.
    let (b, dot) = if dot < 0.0 {
        ([-b[0], -b[1], -b[2], -b[3]], -dot)
    } else {
        (b, dot)
    };
    if dot > 0.9995 {
        // Linear interpolation for nearly identical quaternions.
        let r = [
            a[0] + t * (b[0] - a[0]),
            a[1] + t * (b[1] - a[1]),
            a[2] + t * (b[2] - a[2]),
            a[3] + t * (b[3] - a[3]),
        ];
        quat_normalize(r)
    } else {
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        let s0 = (theta_0 - theta).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;
        [
            s0 * a[0] + s1 * b[0],
            s0 * a[1] + s1 * b[1],
            s0 * a[2] + s1 * b[2],
            s0 * a[3] + s1 * b[3],
        ]
    }
}

fn quat_normalize(q: Quat) -> Quat {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-10 {
        return IDENTITY_QUAT;
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::Skeleton;

    #[test]
    fn standard_library_has_four_poses() {
        let lib = PoseLibrary::standard();
        assert_eq!(lib.names().len(), 4, "expected 4 standard poses");
    }

    #[test]
    fn t_pose_has_shoulder_rotations() {
        let lib = PoseLibrary::standard();
        let t = lib.get("t-pose").expect("t-pose must exist");
        assert!(
            t.rotations.contains_key("left_shoulder"),
            "t-pose must have left_shoulder rotation"
        );
    }

    #[test]
    fn rotation_for_unknown_returns_identity() {
        let pose = Pose::new("test");
        assert_eq!(pose.rotation_for("nonexistent"), IDENTITY_QUAT);
    }

    #[test]
    fn blend_at_t0_equals_a() {
        let mut a = Pose::new("a");
        a.set_rotation("joint1", [0.0, 0.0, 0.707, 0.707]);
        let mut b = Pose::new("b");
        b.set_rotation("joint1", [0.0, 0.707, 0.0, 0.707]);

        let blended = Pose::blend(&a, &b, 0.0);
        let result = blended.rotation_for("joint1");
        let expected = a.rotation_for("joint1");
        for i in 0..4 {
            assert!(
                (result[i] - expected[i]).abs() < 1e-5,
                "blend(t=0) component {} mismatch: {} vs {}",
                i,
                result[i],
                expected[i]
            );
        }
    }

    #[test]
    fn blend_at_t1_equals_b() {
        let mut a = Pose::new("a");
        a.set_rotation("joint1", [0.0, 0.0, 0.707, 0.707]);
        let mut b = Pose::new("b");
        b.set_rotation("joint1", [0.0, 0.707, 0.0, 0.707]);

        let blended = Pose::blend(&a, &b, 1.0);
        let result = blended.rotation_for("joint1");
        let expected = b.rotation_for("joint1");
        for i in 0..4 {
            assert!(
                (result[i] - expected[i]).abs() < 1e-5,
                "blend(t=1) component {} mismatch: {} vs {}",
                i,
                result[i],
                expected[i]
            );
        }
    }

    #[test]
    fn apply_to_skeleton_overrides_rotations() {
        let lib = PoseLibrary::standard();
        let t_pose = lib.get("t-pose").expect("t-pose must exist");
        let skeleton = Skeleton::human_body();
        let posed = t_pose.apply_to_skeleton(&skeleton);

        let left_shoulder = posed
            .joints
            .iter()
            .find(|j| j.name == "left_shoulder")
            .expect("left_shoulder must exist");

        // The rotation should not be the identity quaternion.
        assert_ne!(
            left_shoulder.rotation, IDENTITY_QUAT,
            "left_shoulder should have non-identity rotation in t-pose"
        );
    }

    #[test]
    fn slerp_identity_to_identity_is_identity() {
        let a = Pose::new("a"); // all identity
        let b = Pose::new("b"); // all identity

        // Blend two empty poses — result should also be empty (no joints).
        // But if we query any joint, it should be identity.
        let blended = Pose::blend(&a, &b, 0.5);
        assert_eq!(blended.rotation_for("any_joint"), IDENTITY_QUAT);
    }
}
