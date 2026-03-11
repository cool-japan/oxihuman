// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skinned mesh animation sequence — joint transform keyframes for skeletal animation.

/// A 4x4 column-major transform matrix.
pub type Mat4 = [[f32; 4]; 4];

/// Returns the identity 4x4 matrix.
pub fn identity_mat4() -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// A keyframe with per-joint transform matrices.
#[derive(Debug, Clone)]
pub struct SkinnedKeyframe {
    pub time: f32,
    pub joint_matrices: Vec<Mat4>,
}

/// A skinned animation sequence.
#[derive(Debug, Default, Clone)]
pub struct SkinnedAnimSequence {
    pub joint_count: usize,
    pub keyframes: Vec<SkinnedKeyframe>,
    pub frame_rate: f32,
}

impl SkinnedAnimSequence {
    /// Creates a new sequence with a given joint count and frame rate.
    pub fn new(joint_count: usize, frame_rate: f32) -> Self {
        Self {
            joint_count,
            frame_rate,
            keyframes: Vec::new(),
        }
    }

    /// Adds a keyframe.
    pub fn push_keyframe(&mut self, kf: SkinnedKeyframe) {
        self.keyframes.push(kf);
    }

    /// Returns the total frame count.
    pub fn frame_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Returns the total duration in seconds.
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    /// Returns the keyframe at or just before the given time.
    pub fn keyframe_before(&self, time: f32) -> Option<&SkinnedKeyframe> {
        self.keyframes.iter().rfind(|k| k.time <= time)
    }
}

/// Validates that each keyframe has the correct joint count.
pub fn validate_skinned_sequence(seq: &SkinnedAnimSequence) -> bool {
    seq.keyframes
        .iter()
        .all(|k| k.joint_matrices.len() == seq.joint_count)
}

/// Builds a rest-pose keyframe (all identity matrices) at time `t`.
pub fn rest_pose_keyframe(joint_count: usize, time: f32) -> SkinnedKeyframe {
    SkinnedKeyframe {
        time,
        joint_matrices: vec![identity_mat4(); joint_count],
    }
}

/// Linearly interpolates two Mat4 matrices element-wise.
pub fn lerp_mat4(a: &Mat4, b: &Mat4, t: f32) -> Mat4 {
    let t = t.clamp(0.0, 1.0);
    let mut result = identity_mat4();
    for col in 0..4 {
        for row in 0..4 {
            result[col][row] = a[col][row] + (b[col][row] - a[col][row]) * t;
        }
    }
    result
}

/// Extracts the translation component from a Mat4.
pub fn mat4_translation(m: &Mat4) -> [f32; 3] {
    [m[3][0], m[3][1], m[3][2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sequence_empty() {
        /* New sequence should have zero frames */
        assert_eq!(SkinnedAnimSequence::new(20, 30.0).frame_count(), 0);
    }

    #[test]
    fn test_push_keyframe() {
        /* Pushing a keyframe should increase count */
        let mut seq = SkinnedAnimSequence::new(2, 30.0);
        seq.push_keyframe(rest_pose_keyframe(2, 0.0));
        assert_eq!(seq.frame_count(), 1);
    }

    #[test]
    fn test_duration_empty() {
        /* Empty sequence duration should be zero */
        assert_eq!(SkinnedAnimSequence::new(5, 24.0).duration(), 0.0);
    }

    #[test]
    fn test_duration_last_frame() {
        /* Duration should equal time of last keyframe */
        let mut seq = SkinnedAnimSequence::new(1, 30.0);
        seq.push_keyframe(rest_pose_keyframe(1, 0.0));
        seq.push_keyframe(rest_pose_keyframe(1, 2.0));
        assert!((seq.duration() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validate_sequence_valid() {
        /* Rest pose frames should validate */
        let mut seq = SkinnedAnimSequence::new(3, 24.0);
        seq.push_keyframe(rest_pose_keyframe(3, 0.0));
        assert!(validate_skinned_sequence(&seq));
    }

    #[test]
    fn test_identity_mat4_diagonal() {
        /* Identity matrix should have 1s on diagonal */
        let m = identity_mat4();
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][1], 1.0);
        assert_eq!(m[2][2], 1.0);
        assert_eq!(m[3][3], 1.0);
    }

    #[test]
    fn test_lerp_mat4_at_zero() {
        /* Lerp at t=0 should return matrix a */
        let a = identity_mat4();
        let mut b = identity_mat4();
        b[3][0] = 5.0;
        let r = lerp_mat4(&a, &b, 0.0);
        assert_eq!(r[3][0], 0.0);
    }

    #[test]
    fn test_lerp_mat4_at_one() {
        /* Lerp at t=1 should return matrix b */
        let a = identity_mat4();
        let mut b = identity_mat4();
        b[3][0] = 5.0;
        let r = lerp_mat4(&a, &b, 1.0);
        assert_eq!(r[3][0], 5.0);
    }

    #[test]
    fn test_mat4_translation_identity() {
        /* Identity matrix has zero translation */
        let t = mat4_translation(&identity_mat4());
        assert_eq!(t, [0.0; 3]);
    }

    #[test]
    fn test_keyframe_before_returns_correct() {
        /* keyframe_before(0.5) should return the frame at t=0.0 */
        let mut seq = SkinnedAnimSequence::new(1, 24.0);
        seq.push_keyframe(rest_pose_keyframe(1, 0.0));
        seq.push_keyframe(rest_pose_keyframe(1, 1.0));
        let kf = seq.keyframe_before(0.5).unwrap();
        assert!((kf.time - 0.0).abs() < f32::EPSILON);
    }
}
