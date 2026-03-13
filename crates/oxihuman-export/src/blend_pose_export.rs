// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export blended pose data (weighted combination of multiple poses).

/// A single pose represented by joint rotations (quaternions).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Pose {
    pub name: String,
    pub joint_rotations: Vec<[f32; 4]>,
}

/// A weighted pose contribution.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedPose {
    pub pose: Pose,
    pub weight: f32,
}

/// A blended pose result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendedPose {
    pub joint_rotations: Vec<[f32; 4]>,
    pub joint_count: usize,
}

/// Create a new pose with identity quaternions.
#[allow(dead_code)]
pub fn identity_pose(name: &str, joint_count: usize) -> Pose {
    Pose {
        name: name.to_string(),
        joint_rotations: vec![[0.0, 0.0, 0.0, 1.0]; joint_count],
    }
}

/// Normalise a quaternion.
#[allow(dead_code)]
pub fn normalise_quat(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-8 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

/// Linearly blend quaternions (NLerp) from a list of weighted poses.
#[allow(dead_code)]
pub fn blend_poses(weighted: &[WeightedPose]) -> Option<BlendedPose> {
    if weighted.is_empty() {
        return None;
    }
    let joint_count = weighted[0].pose.joint_rotations.len();
    let total_w: f32 = weighted.iter().map(|wp| wp.weight).sum();
    if total_w < 1e-8 {
        return None;
    }
    let mut result = vec![[0.0_f32; 4]; joint_count];
    for wp in weighted {
        let w = wp.weight / total_w;
        #[allow(clippy::needless_range_loop)]
        for j in 0..joint_count {
            let q = wp.pose.joint_rotations[j];
            result[j][0] += q[0] * w;
            result[j][1] += q[1] * w;
            result[j][2] += q[2] * w;
            result[j][3] += q[3] * w;
        }
    }
    let rotations: Vec<[f32; 4]> = result.iter().map(|&q| normalise_quat(q)).collect();
    Some(BlendedPose {
        joint_rotations: rotations,
        joint_count,
    })
}

/// Serialise blended pose to a flat f32 buffer (for export).
#[allow(dead_code)]
pub fn serialise_blended_pose(pose: &BlendedPose) -> Vec<f32> {
    pose.joint_rotations
        .iter()
        .flat_map(|q| q.iter().cloned())
        .collect()
}

/// Count poses with non-zero weight.
#[allow(dead_code)]
pub fn active_pose_count(weighted: &[WeightedPose]) -> usize {
    weighted.iter().filter(|wp| wp.weight > 0.0).count()
}

/// Check all weights sum to approximately 1.
#[allow(dead_code)]
pub fn weights_normalised(weighted: &[WeightedPose]) -> bool {
    let sum: f32 = weighted.iter().map(|wp| wp.weight).sum();
    (sum - 1.0).abs() < 1e-4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id_pose(name: &str) -> Pose {
        identity_pose(name, 3)
    }

    #[test]
    fn test_identity_pose() {
        let p = identity_pose("idle", 4);
        assert_eq!(p.joint_rotations.len(), 4);
        assert!((p.joint_rotations[0][3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalise_quat() {
        let q = normalise_quat([0.0, 0.0, 0.0, 2.0]);
        assert!((q[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_single_pose() {
        let p = id_pose("a");
        let w = vec![WeightedPose {
            pose: p,
            weight: 1.0,
        }];
        let b = blend_poses(&w).expect("should succeed");
        assert!((b.joint_rotations[0][3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_two_identity_poses() {
        let w = vec![
            WeightedPose {
                pose: id_pose("a"),
                weight: 0.5,
            },
            WeightedPose {
                pose: id_pose("b"),
                weight: 0.5,
            },
        ];
        let b = blend_poses(&w).expect("should succeed");
        assert!((b.joint_rotations[0][3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_empty_returns_none() {
        assert!(blend_poses(&[]).is_none());
    }

    #[test]
    fn test_serialise_blended_pose() {
        let pose = BlendedPose {
            joint_rotations: vec![[0.0, 0.0, 0.0, 1.0]; 2],
            joint_count: 2,
        };
        let buf = serialise_blended_pose(&pose);
        assert_eq!(buf.len(), 8);
    }

    #[test]
    fn test_active_pose_count() {
        let w = vec![
            WeightedPose {
                pose: id_pose("a"),
                weight: 0.0,
            },
            WeightedPose {
                pose: id_pose("b"),
                weight: 1.0,
            },
        ];
        assert_eq!(active_pose_count(&w), 1);
    }

    #[test]
    fn test_weights_normalised_true() {
        let w = vec![
            WeightedPose {
                pose: id_pose("a"),
                weight: 0.4,
            },
            WeightedPose {
                pose: id_pose("b"),
                weight: 0.6,
            },
        ];
        assert!(weights_normalised(&w));
    }

    #[test]
    fn test_weights_normalised_false() {
        let w = vec![WeightedPose {
            pose: id_pose("a"),
            weight: 2.0,
        }];
        assert!(!weights_normalised(&w));
    }

    #[test]
    fn test_joint_count_stored() {
        let w = vec![WeightedPose {
            pose: id_pose("a"),
            weight: 1.0,
        }];
        let b = blend_poses(&w).expect("should succeed");
        assert_eq!(b.joint_count, 3);
    }
}
