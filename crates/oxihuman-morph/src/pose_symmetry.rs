// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pose/skeleton symmetry enforcement and mirror analysis.
//! Note: body_symmetry.rs handles mesh vertex symmetry; this module covers joint pose symmetry.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointPose {
    pub name: String,
    pub rotation: [f32; 4], // quaternion xyzw
    pub translation: [f32; 3],
    pub scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SymmetryPair {
    pub left_name: String,
    pub right_name: String,
    pub mirror_axis: u8, // 0=X, 1=Y, 2=Z
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSkeleton {
    pub joints: Vec<JointPose>,
}

/// Mirror a quaternion across the given axis (0=X, 1=Y, 2=Z).
/// Mirroring across axis negates the corresponding imaginary components.
#[allow(dead_code)]
pub fn mirror_joint_rotation(q: [f32; 4], axis: u8) -> [f32; 4] {
    // xyzw layout. Mirroring across axis A negates components B and C (the other two).
    // Then also negate w to keep the handedness consistent.
    let [x, y, z, w] = q;
    match axis {
        0 => [-x, y, z, -w], // mirror X: flip yz plane → negate x and w
        1 => [x, -y, z, -w], // mirror Y
        2 => [x, y, -z, -w], // mirror Z
        _ => [x, y, z, w],
    }
}

/// Produce a mirrored copy of the skeleton, swapping left/right joints.
#[allow(dead_code)]
pub fn mirror_pose(skeleton: &PoseSkeleton, pairs: &[SymmetryPair]) -> PoseSkeleton {
    let mut joints = skeleton.joints.clone();

    for pair in pairs {
        let left_idx = joints.iter().position(|j| j.name == pair.left_name);
        let right_idx = joints.iter().position(|j| j.name == pair.right_name);

        if let (Some(li), Some(ri)) = (left_idx, right_idx) {
            let left_rot = joints[li].rotation;
            let right_rot = joints[ri].rotation;
            let left_trans = joints[li].translation;
            let right_trans = joints[ri].translation;

            joints[li].rotation = mirror_joint_rotation(right_rot, pair.mirror_axis);
            joints[ri].rotation = mirror_joint_rotation(left_rot, pair.mirror_axis);

            // Mirror translation across the axis
            let mut new_left_trans = right_trans;
            let mut new_right_trans = left_trans;
            let ax = pair.mirror_axis as usize;
            new_left_trans[ax] = -right_trans[ax];
            new_right_trans[ax] = -left_trans[ax];

            joints[li].translation = new_left_trans;
            joints[ri].translation = new_right_trans;
        }
    }

    PoseSkeleton { joints }
}

/// Blend the skeleton toward its symmetric version by `blend` (0 = original, 1 = fully symmetric).
#[allow(dead_code)]
pub fn enforce_symmetry_pose(skeleton: &mut PoseSkeleton, pairs: &[SymmetryPair], blend: f32) {
    let blend = blend.clamp(0.0, 1.0);
    let mirrored = mirror_pose(skeleton, pairs);

    for (joint, mirrored_joint) in skeleton.joints.iter_mut().zip(mirrored.joints.iter()) {
        joint.rotation = quat_slerp_pose(joint.rotation, mirrored_joint.rotation, blend * 0.5);

        for i in 0..3 {
            joint.translation[i] +=
                (mirrored_joint.translation[i] - joint.translation[i]) * blend * 0.5;
        }
    }
}

/// Compute RMS asymmetry error across all symmetry pairs.
#[allow(dead_code)]
pub fn pose_symmetry_error(skeleton: &PoseSkeleton, pairs: &[SymmetryPair]) -> f32 {
    let mut sum_sq = 0.0_f32;
    let mut count = 0;

    for pair in pairs {
        let left = find_joint_by_name(skeleton, &pair.left_name);
        let right = find_joint_by_name(skeleton, &pair.right_name);

        if let (Some(l), Some(r)) = (left, right) {
            let mirrored_r = mirror_joint_rotation(r.rotation, pair.mirror_axis);
            // Quaternion angle distance
            let dist = quat_angle_distance(l.rotation, mirrored_r);
            sum_sq += dist * dist;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        (sum_sq / count as f32).sqrt()
    }
}

/// Return canonical left/right joint pairs for a standard biped skeleton.
#[allow(dead_code)]
pub fn standard_biped_symmetry_pairs() -> Vec<SymmetryPair> {
    let pairs_data = [
        ("LeftArm", "RightArm"),
        ("LeftForeArm", "RightForeArm"),
        ("LeftHand", "RightHand"),
        ("LeftUpLeg", "RightUpLeg"),
        ("LeftLeg", "RightLeg"),
        ("LeftFoot", "RightFoot"),
        ("LeftToeBase", "RightToeBase"),
        ("LeftShoulder", "RightShoulder"),
        ("LeftHandThumb1", "RightHandThumb1"),
        ("LeftHandIndex1", "RightHandIndex1"),
        ("LeftHandMiddle1", "RightHandMiddle1"),
        ("LeftHandRing1", "RightHandRing1"),
        ("LeftHandPinky1", "RightHandPinky1"),
    ];

    pairs_data
        .iter()
        .map(|(l, r)| SymmetryPair {
            left_name: l.to_string(),
            right_name: r.to_string(),
            mirror_axis: 0, // X axis for biped
        })
        .collect()
}

/// Find a joint by name in a skeleton.
#[allow(dead_code)]
pub fn find_joint_by_name<'a>(skeleton: &'a PoseSkeleton, name: &str) -> Option<&'a JointPose> {
    skeleton.joints.iter().find(|j| j.name == name)
}

/// Quaternion slerp used internally.
#[allow(dead_code)]
pub fn quat_slerp_pose(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    let [ax, ay, az, aw] = a;
    let [mut bx, mut by, mut bz, mut bw] = b;

    let mut dot = ax * bx + ay * by + az * bz + aw * bw;
    if dot < 0.0 {
        bx = -bx;
        by = -by;
        bz = -bz;
        bw = -bw;
        dot = -dot;
    }

    if dot > 0.9995 {
        // Linear interpolation for nearly identical quaternions
        let rx = ax + t * (bx - ax);
        let ry = ay + t * (by - ay);
        let rz = az + t * (bz - az);
        let rw = aw + t * (bw - aw);
        let mag = (rx * rx + ry * ry + rz * rz + rw * rw).sqrt().max(1e-8);
        return [rx / mag, ry / mag, rz / mag, rw / mag];
    }

    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();

    let s0 = (theta_0 - theta).sin() / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;

    [
        s0 * ax + s1 * bx,
        s0 * ay + s1 * by,
        s0 * az + s1 * bz,
        s0 * aw + s1 * bw,
    ]
}

/// Interpolate two skeletons joint-by-joint using slerp for rotations.
#[allow(dead_code)]
pub fn interpolate_poses(a: &PoseSkeleton, b: &PoseSkeleton, t: f32) -> PoseSkeleton {
    let t = t.clamp(0.0, 1.0);
    let joints = a
        .joints
        .iter()
        .zip(b.joints.iter())
        .map(|(ja, jb)| {
            let lerp = |x: f32, y: f32| x + (y - x) * t;
            JointPose {
                name: ja.name.clone(),
                rotation: quat_slerp_pose(ja.rotation, jb.rotation, t),
                translation: [
                    lerp(ja.translation[0], jb.translation[0]),
                    lerp(ja.translation[1], jb.translation[1]),
                    lerp(ja.translation[2], jb.translation[2]),
                ],
                scale: lerp(ja.scale, jb.scale),
            }
        })
        .collect();
    PoseSkeleton { joints }
}

/// Auto-detect symmetry pairs from joint names containing "Left" and "Right".
#[allow(dead_code)]
pub fn detect_symmetry_pairs(joint_names: &[String]) -> Vec<SymmetryPair> {
    let mut pairs = Vec::new();
    for name in joint_names {
        if let Some(suffix) = name.strip_prefix("Left") {
            let right_name = format!("Right{suffix}");
            if joint_names.iter().any(|n| n == &right_name) {
                pairs.push(SymmetryPair {
                    left_name: name.clone(),
                    right_name,
                    mirror_axis: 0,
                });
            }
        }
    }
    pairs
}

/// Mean quaternion rotation distance across matching joints.
#[allow(dead_code)]
pub fn pose_distance_sym(a: &PoseSkeleton, b: &PoseSkeleton) -> f32 {
    let pairs: Vec<_> = a.joints.iter().zip(b.joints.iter()).collect();
    if pairs.is_empty() {
        return 0.0;
    }
    let sum: f32 = pairs
        .iter()
        .map(|(ja, jb)| quat_angle_distance(ja.rotation, jb.rotation))
        .sum();
    sum / pairs.len() as f32
}

/// Apply a rotation delta to a specific joint via quaternion composition.
#[allow(dead_code)]
pub fn apply_pose_offset(skeleton: &mut PoseSkeleton, joint_name: &str, rotation_delta: [f32; 4]) {
    if let Some(joint) = skeleton.joints.iter_mut().find(|j| j.name == joint_name) {
        joint.rotation = quat_multiply_pose(joint.rotation, rotation_delta);
        // Normalize
        let [x, y, z, w] = joint.rotation;
        let mag = (x * x + y * y + z * z + w * w).sqrt().max(1e-8);
        joint.rotation = [x / mag, y / mag, z / mag, w / mag];
    }
}

// Internal helpers

fn quat_multiply_pose(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn quat_angle_distance(a: [f32; 4], b: [f32; 4]) -> f32 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3])
        .abs()
        .clamp(0.0, 1.0);
    2.0 * dot.acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_quat() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    fn make_joint(name: &str) -> JointPose {
        JointPose {
            name: name.to_string(),
            rotation: identity_quat(),
            translation: [0.0, 0.0, 0.0],
            scale: 1.0,
        }
    }

    fn make_simple_skeleton() -> PoseSkeleton {
        PoseSkeleton {
            joints: vec![
                make_joint("LeftArm"),
                make_joint("RightArm"),
                make_joint("Spine"),
            ],
        }
    }

    #[test]
    fn test_mirror_identity_quat() {
        let q = identity_quat();
        let mirrored = mirror_joint_rotation(q, 0);
        // Identity mirrored should have negated x and w components
        assert_eq!(mirrored, [0.0, 0.0, 0.0, -1.0]);
    }

    #[test]
    fn test_mirror_pose_swaps_joints() {
        let mut skel = make_simple_skeleton();
        skel.joints[0].translation = [1.0, 0.0, 0.0]; // LeftArm
        skel.joints[1].translation = [-1.0, 0.0, 0.0]; // RightArm

        let pairs = vec![SymmetryPair {
            left_name: "LeftArm".to_string(),
            right_name: "RightArm".to_string(),
            mirror_axis: 0,
        }];

        let mirrored = mirror_pose(&skel, &pairs);
        // After mirroring, left should have the mirrored-right translation
        assert!((mirrored.joints[0].translation[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_enforce_symmetry_reduces_error() {
        let mut skel = PoseSkeleton {
            joints: vec![
                JointPose {
                    name: "LeftArm".to_string(),
                    rotation: [0.1, 0.0, 0.0, (1.0_f32 - 0.01_f32).sqrt()],
                    translation: [1.0, 0.0, 0.0],
                    scale: 1.0,
                },
                JointPose {
                    name: "RightArm".to_string(),
                    rotation: identity_quat(),
                    translation: [-1.0, 0.0, 0.0],
                    scale: 1.0,
                },
            ],
        };
        let pairs = vec![SymmetryPair {
            left_name: "LeftArm".to_string(),
            right_name: "RightArm".to_string(),
            mirror_axis: 0,
        }];
        let err_before = pose_symmetry_error(&skel, &pairs);
        enforce_symmetry_pose(&mut skel, &pairs, 1.0);
        let err_after = pose_symmetry_error(&skel, &pairs);
        assert!(
            err_after <= err_before + 1e-4,
            "symmetry error should not increase"
        );
    }

    #[test]
    fn test_pose_symmetry_error_symmetric_skeleton() {
        let skel = make_simple_skeleton();
        let pairs = standard_biped_symmetry_pairs();
        // No matching pairs for this small skeleton
        let err = pose_symmetry_error(&skel, &pairs);
        assert_eq!(err, 0.0);
    }

    #[test]
    fn test_standard_biped_symmetry_pairs_not_empty() {
        let pairs = standard_biped_symmetry_pairs();
        assert!(!pairs.is_empty());
        assert!(pairs.iter().any(|p| p.left_name.contains("Arm")));
    }

    #[test]
    fn test_find_joint_by_name() {
        let skel = make_simple_skeleton();
        let joint = find_joint_by_name(&skel, "Spine");
        assert!(joint.is_some());
        assert_eq!(joint.expect("should succeed").name, "Spine");
    }

    #[test]
    fn test_find_joint_missing() {
        let skel = make_simple_skeleton();
        assert!(find_joint_by_name(&skel, "NonExistent").is_none());
    }

    #[test]
    fn test_quat_slerp_t0() {
        let a = identity_quat();
        let b = [0.0, 0.0, 1.0_f32.sin(), 1.0_f32.cos()];
        let result = quat_slerp_pose(a, b, 0.0);
        assert!((result[3] - a[3]).abs() < 1e-4);
    }

    #[test]
    fn test_quat_slerp_t1() {
        let a = identity_quat();
        let b = [0.0, 0.0, (0.5_f32).sin(), (0.5_f32).cos()];
        let result = quat_slerp_pose(a, b, 1.0);
        assert!((result[2] - b[2]).abs() < 1e-4);
        assert!((result[3] - b[3]).abs() < 1e-4);
    }

    #[test]
    fn test_interpolate_poses_midpoint() {
        let a = PoseSkeleton {
            joints: vec![JointPose {
                name: "Root".to_string(),
                rotation: identity_quat(),
                translation: [0.0, 0.0, 0.0],
                scale: 1.0,
            }],
        };
        let b = PoseSkeleton {
            joints: vec![JointPose {
                name: "Root".to_string(),
                rotation: identity_quat(),
                translation: [2.0, 0.0, 0.0],
                scale: 2.0,
            }],
        };
        let mid = interpolate_poses(&a, &b, 0.5);
        assert!((mid.joints[0].translation[0] - 1.0).abs() < 1e-4);
        assert!((mid.joints[0].scale - 1.5).abs() < 1e-4);
    }

    #[test]
    fn test_detect_symmetry_pairs() {
        let names: Vec<String> = vec![
            "LeftArm".to_string(),
            "RightArm".to_string(),
            "LeftLeg".to_string(),
            "RightLeg".to_string(),
            "Spine".to_string(),
        ];
        let pairs = detect_symmetry_pairs(&names);
        assert_eq!(pairs.len(), 2);
        assert!(pairs.iter().any(|p| p.left_name == "LeftArm"));
    }

    #[test]
    fn test_detect_symmetry_pairs_no_match() {
        let names: Vec<String> = vec!["Spine".to_string(), "Hips".to_string()];
        let pairs = detect_symmetry_pairs(&names);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_pose_distance_sym_identity() {
        let a = make_simple_skeleton();
        let b = a.clone();
        let dist = pose_distance_sym(&a, &b);
        assert!(dist < 1e-4);
    }

    #[test]
    fn test_apply_pose_offset() {
        let mut skel = make_simple_skeleton();
        let delta = [0.0, 0.0, (0.1_f32).sin(), (0.1_f32).cos()];
        apply_pose_offset(&mut skel, "LeftArm", delta);
        // Should not remain identity
        let joint = find_joint_by_name(&skel, "LeftArm").expect("should succeed");
        let still_identity = joint.rotation[3].abs() > 0.9999;
        // With a non-trivial delta, rotation should change
        assert!(!still_identity || delta[3] > 0.9999);
    }

    #[test]
    fn test_apply_pose_offset_missing_joint() {
        let mut skel = make_simple_skeleton();
        // Should not panic on missing joint
        apply_pose_offset(&mut skel, "NonExistent", identity_quat());
    }
}
