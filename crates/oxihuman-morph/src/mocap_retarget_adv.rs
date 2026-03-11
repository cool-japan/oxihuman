// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Advanced skeleton-space BVH retargeting with twist decomposition.

// ── Joint / Pose ──────────────────────────────────────────────────────────────

/// A single joint defined in rest pose.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    pub parent: Option<usize>,
    /// Rest rotation as quaternion [x, y, z, w].
    pub rest_rot: [f32; 4],
    /// Rest position (local offset from parent).
    pub rest_pos: [f32; 3],
}

/// A full skeleton pose (rest skeleton + per-joint local rotations + root translation).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkeletonPose {
    pub joints: Vec<Joint>,
    /// Local rotation per joint [x, y, z, w].
    pub local_rots: Vec<[f32; 4]>,
    pub root_pos: [f32; 3],
}

/// Maps source joint names to target joint names for retargeting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetargetMap {
    pub source_joints: Vec<String>,
    pub target_joints: Vec<String>,
    pub scale: f32,
}

// ── Quaternion math ───────────────────────────────────────────────────────────

/// Multiply two quaternions: result = a * b.
#[allow(dead_code)]
pub fn quat_multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// Return the inverse (conjugate for unit quaternions) of q.
#[allow(dead_code)]
pub fn quat_inverse(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Normalise q to unit length.
#[allow(dead_code)]
pub fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

/// Spherical linear interpolation between two quaternions.
#[allow(dead_code)]
pub fn quat_slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    // Ensure shortest path
    let (b, dot) = if dot < 0.0 {
        ([-b[0], -b[1], -b[2], -b[3]], -dot)
    } else {
        (b, dot)
    };
    let dot = dot.min(1.0);
    if dot > 0.9995 {
        // Linear interpolation fallback
        return quat_normalize([
            a[0] + t * (b[0] - a[0]),
            a[1] + t * (b[1] - a[1]),
            a[2] + t * (b[2] - a[2]),
            a[3] + t * (b[3] - a[3]),
        ]);
    }
    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();
    let s0 = ((1.0 - t) * theta_0).sin() / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;
    quat_normalize([
        s0 * a[0] + s1 * b[0],
        s0 * a[1] + s1 * b[1],
        s0 * a[2] + s1 * b[2],
        s0 * a[3] + s1 * b[3],
    ])
}

/// Decompose q into swing and twist about `twist_axis`.
/// Returns (swing, twist).
#[allow(dead_code)]
pub fn quat_to_swing_twist(q: [f32; 4], twist_axis: [f32; 3]) -> ([f32; 4], [f32; 4]) {
    let [x, y, z, w] = q;
    let [ax, ay, az] = twist_axis;
    // Project rotation axis onto twist axis
    let dot = x * ax + y * ay + z * az;
    let twist = quat_normalize([dot * ax, dot * ay, dot * az, w]);
    let swing = quat_multiply(q, quat_inverse(twist));
    (quat_normalize(swing), twist)
}

// ── Retargeting logic ─────────────────────────────────────────────────────────

/// Retarget a single joint rotation from source skeleton space to target skeleton space.
/// `src_rot` is the local rotation in source, `src_rest` is the source rest rotation,
/// `tgt_rest` is the target rest rotation.
#[allow(dead_code)]
pub fn retarget_joint_rotation(
    src_rot: [f32; 4],
    src_rest: [f32; 4],
    tgt_rest: [f32; 4],
) -> [f32; 4] {
    // Convert source local rotation to rest-relative delta
    let delta = quat_multiply(src_rot, quat_inverse(src_rest));
    // Apply delta in target rest space
    quat_normalize(quat_multiply(delta, tgt_rest))
}

/// Retarget a full pose from source to target skeleton using the provided joint map.
#[allow(dead_code)]
pub fn retarget_pose_adv(
    src: &SkeletonPose,
    tgt_rest: &SkeletonPose,
    map: &RetargetMap,
) -> SkeletonPose {
    let mut out = tgt_rest.clone();
    out.root_pos = scale_root_translation(
        src.root_pos,
        compute_skeleton_height(src),
        compute_skeleton_height(tgt_rest),
    );

    for (si, src_name) in map.source_joints.iter().enumerate() {
        if let Some(tgt_name) = map.target_joints.get(si) {
            // Find source joint index
            let src_idx = src
                .joints
                .iter()
                .position(|j| &j.name == src_name)
                .unwrap_or(usize::MAX);
            // Find target joint index
            let tgt_idx = tgt_rest
                .joints
                .iter()
                .position(|j| &j.name == tgt_name)
                .unwrap_or(usize::MAX);

            if src_idx < src.joints.len()
                && tgt_idx < tgt_rest.joints.len()
                && src_idx < src.local_rots.len()
            {
                let src_rot = src.local_rots[src_idx];
                let src_rest_rot = src.joints[src_idx].rest_rot;
                let tgt_rest_rot = tgt_rest.joints[tgt_idx].rest_rot;
                out.local_rots[tgt_idx] =
                    retarget_joint_rotation(src_rot, src_rest_rot, tgt_rest_rot);
            }
        }
    }
    out
}

/// Scale root translation proportionally between skeleton heights.
#[allow(dead_code)]
pub fn scale_root_translation(pos: [f32; 3], src_height: f32, tgt_height: f32) -> [f32; 3] {
    if src_height < 1e-6 {
        return pos;
    }
    let s = tgt_height / src_height;
    [pos[0] * s, pos[1] * s, pos[2] * s]
}

/// Blend two skeleton poses by SLERPing all joint rotations.
#[allow(dead_code)]
pub fn blend_poses(a: &SkeletonPose, b: &SkeletonPose, t: f32) -> SkeletonPose {
    let joints = a.joints.clone();
    let n = joints.len().min(a.local_rots.len()).min(b.local_rots.len());
    let local_rots = (0..n)
        .map(|i| quat_slerp(a.local_rots[i], b.local_rots[i], t))
        .collect();
    let root_pos = [
        a.root_pos[0] + t * (b.root_pos[0] - a.root_pos[0]),
        a.root_pos[1] + t * (b.root_pos[1] - a.root_pos[1]),
        a.root_pos[2] + t * (b.root_pos[2] - a.root_pos[2]),
    ];
    SkeletonPose {
        joints,
        local_rots,
        root_pos,
    }
}

/// Compute approximate skeleton height as max Y extent of rest positions
/// accumulated from root.
#[allow(dead_code)]
pub fn compute_skeleton_height(pose: &SkeletonPose) -> f32 {
    let mut max_y = 0.0_f32;
    // Accumulate world-space Y positions
    let mut world_y = vec![0.0_f32; pose.joints.len()];
    for (i, joint) in pose.joints.iter().enumerate() {
        let parent_y = joint.parent.map_or(0.0, |p| world_y[p]);
        world_y[i] = parent_y + joint.rest_pos[1];
        max_y = max_y.max(world_y[i]);
    }
    max_y.max(0.001)
}

/// Build a standard 14-joint biped retarget map.
#[allow(dead_code)]
pub fn standard_biped_retarget_map() -> RetargetMap {
    let joints = vec![
        "Hips",
        "Spine",
        "Spine1",
        "Neck",
        "Head",
        "LeftArm",
        "LeftForeArm",
        "LeftHand",
        "RightArm",
        "RightForeArm",
        "RightHand",
        "LeftUpLeg",
        "LeftLeg",
        "RightUpLeg",
    ];
    RetargetMap {
        source_joints: joints.iter().map(|s| s.to_string()).collect(),
        target_joints: joints.iter().map(|s| s.to_string()).collect(),
        scale: 1.0,
    }
}

// ── Helper: build a minimal test skeleton ────────────────────────────────────

#[allow(dead_code)]
fn identity_quat() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}

#[allow(dead_code)]
fn make_test_pose(n: usize) -> SkeletonPose {
    let joints = (0..n)
        .map(|i| Joint {
            name: format!("Joint{i}"),
            parent: if i == 0 { None } else { Some(i - 1) },
            rest_rot: identity_quat(),
            rest_pos: [0.0, 0.1 * i as f32, 0.0],
        })
        .collect();
    let local_rots = vec![identity_quat(); n];
    SkeletonPose {
        joints,
        local_rots,
        root_pos: [0.0, 0.0, 0.0],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    fn nearly_eq(a: [f32; 4], b: [f32; 4]) -> bool {
        (0..4).all(|i| (a[i] - b[i]).abs() < 1e-4)
    }

    fn nearly_eq3(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-4)
    }

    #[test]
    fn test_quat_multiply_identity_left() {
        let q = [0.1, 0.2, 0.3, 0.927];
        let q = quat_normalize(q);
        let result = quat_multiply(id(), q);
        assert!(nearly_eq(result, q));
    }

    #[test]
    fn test_quat_multiply_identity_right() {
        let q = quat_normalize([0.1, 0.2, 0.3, 0.927]);
        let result = quat_multiply(q, id());
        assert!(nearly_eq(result, q));
    }

    #[test]
    fn test_quat_inverse_composed_is_identity() {
        let q = quat_normalize([0.1, 0.2, 0.3, 0.927]);
        let qi = quat_inverse(q);
        let result = quat_normalize(quat_multiply(q, qi));
        assert!(nearly_eq(result, id()));
    }

    #[test]
    fn test_quat_inverse_conjugate() {
        let q = [0.1, 0.2, 0.3, 0.9];
        let qi = quat_inverse(q);
        assert_eq!(qi, [-0.1, -0.2, -0.3, 0.9]);
    }

    #[test]
    fn test_quat_normalize_length_one() {
        let q = quat_normalize([1.0, 2.0, 3.0, 4.0]);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quat_normalize_zero_returns_identity() {
        let q = quat_normalize([0.0, 0.0, 0.0, 0.0]);
        assert_eq!(q, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_quat_slerp_t0() {
        let a = id();
        let frac = std::f32::consts::FRAC_1_SQRT_2;
        let b = quat_normalize([0.0, frac, 0.0, frac]);
        let result = quat_slerp(a, b, 0.0);
        assert!(nearly_eq(result, a));
    }

    #[test]
    fn test_quat_slerp_t1() {
        let a = id();
        let frac = std::f32::consts::FRAC_1_SQRT_2;
        let b = quat_normalize([0.0, frac, 0.0, frac]);
        let result = quat_slerp(a, b, 1.0);
        assert!(nearly_eq(result, b));
    }

    #[test]
    fn test_quat_slerp_t_half_normalized() {
        let a = id();
        let b = id();
        let result = quat_slerp(a, b, 0.5);
        assert!(nearly_eq(result, id()));
    }

    #[test]
    fn test_swing_twist_roundtrip() {
        let q = quat_normalize([0.1, 0.2, 0.0, 0.974]);
        let axis = [0.0, 1.0, 0.0];
        let (swing, twist) = quat_to_swing_twist(q, axis);
        let composed = quat_normalize(quat_multiply(swing, twist));
        assert!(nearly_eq(composed, quat_normalize(q)));
    }

    #[test]
    fn test_swing_twist_pure_twist() {
        // A rotation purely about Y axis — swing should be ~identity
        let q = quat_normalize([0.0, 0.5, 0.0, 0.866]);
        let (swing, _twist) = quat_to_swing_twist(q, [0.0, 1.0, 0.0]);
        assert!((swing[3] - 1.0).abs() < 0.1); // swing w close to 1
    }

    #[test]
    fn test_retarget_pose_no_nan() {
        let src = make_test_pose(5);
        let tgt = make_test_pose(5);
        let map = RetargetMap {
            source_joints: src.joints.iter().map(|j| j.name.clone()).collect(),
            target_joints: tgt.joints.iter().map(|j| j.name.clone()).collect(),
            scale: 1.0,
        };
        let out = retarget_pose_adv(&src, &tgt, &map);
        for r in &out.local_rots {
            for v in r {
                assert!(!v.is_nan());
            }
        }
    }

    #[test]
    fn test_blend_poses_t0() {
        let a = make_test_pose(4);
        let b = make_test_pose(4);
        let out = blend_poses(&a, &b, 0.0);
        for i in 0..4 {
            assert!(nearly_eq(out.local_rots[i], a.local_rots[i]));
        }
    }

    #[test]
    fn test_blend_poses_t1() {
        let a = make_test_pose(4);
        let b = make_test_pose(4);
        let out = blend_poses(&a, &b, 1.0);
        for i in 0..4 {
            assert!(nearly_eq(out.local_rots[i], b.local_rots[i]));
        }
    }

    #[test]
    fn test_blend_poses_root_lerp() {
        let mut a = make_test_pose(2);
        let mut b = make_test_pose(2);
        a.root_pos = [0.0, 0.0, 0.0];
        b.root_pos = [2.0, 4.0, 6.0];
        let out = blend_poses(&a, &b, 0.5);
        assert!(nearly_eq3(out.root_pos, [1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_compute_skeleton_height_positive() {
        let pose = make_test_pose(5);
        let h = compute_skeleton_height(&pose);
        assert!(h > 0.0);
    }

    #[test]
    fn test_scale_root_translation_proportional() {
        let pos = [1.0, 2.0, 3.0];
        let out = scale_root_translation(pos, 1.0, 2.0);
        assert!(nearly_eq3(out, [2.0, 4.0, 6.0]));
    }

    #[test]
    fn test_standard_biped_retarget_map_14_joints() {
        let map = standard_biped_retarget_map();
        assert_eq!(map.source_joints.len(), 14);
        assert_eq!(map.target_joints.len(), 14);
    }

    #[test]
    fn test_retarget_joint_rotation_identity_pass_through() {
        let rot = quat_normalize([0.1, 0.2, 0.3, 0.9]);
        let rest = id();
        let result = retarget_joint_rotation(rot, rest, rest);
        assert!(nearly_eq(quat_normalize(result), quat_normalize(rot)));
    }
}
