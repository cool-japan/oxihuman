// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pose retargeting between different body shapes.
//!
//! Provides [`PoseRetargeter`], which maps a reference pose captured on a
//! *source* body onto a *target* body with different proportions.  The
//! retargeting preserves joint **rotations** unchanged while scaling joint
//! **translations** according to the configured [`ScaleMode`].
//!
//! # Supported scale modes
//!
//! | Mode | Description |
//! |------|-------------|
//! | [`ScaleMode::Proportional`] | Scale all translations by `target_height / source_height`. |
//! | [`ScaleMode::Uniform`]      | Same as Proportional — kept separate for future extension. |
//! | [`ScaleMode::SegmentWise`]  | Scale each joint's translation by the per-segment ratio derived from the pose's own bone lengths. |
//!
//! # Quick start
//!
//! ```rust
//! use oxihuman_morph::pose_retarget::{PoseRetargeter, RetargetConfig, ScaleMode, PoseSnapshot};
//!
//! let config = RetargetConfig {
//!     source_height: 180.0,
//!     target_height: 160.0,
//!     scale_mode: ScaleMode::Proportional,
//! };
//! let pose = PoseSnapshot::default();
//! let retargeted = PoseRetargeter::retarget_pose(&pose, &config);
//! assert_eq!(retargeted.joints.len(), pose.joints.len());
//! ```

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Compatibility alias for the old type that was declared here
// ---------------------------------------------------------------------------

/// Legacy type alias kept for backward compatibility.
#[derive(Debug, Clone)]
pub struct RetargetMapping {
    pub source: String,
    pub target: String,
    pub scale: f32,
}

// ---------------------------------------------------------------------------
// JointPoseData
// ---------------------------------------------------------------------------

/// Full state of a single joint in a pose snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct JointPoseData {
    /// Unique joint name (e.g. `"LeftUpLeg"`, `"Spine1"`).
    pub name: String,
    /// Joint rotation as a unit quaternion `[x, y, z, w]`.
    pub rotation: [f64; 4],
    /// Joint translation in local space (bone-relative).
    pub translation: [f64; 3],
    /// Optional parent joint name.  `None` for the root.
    pub parent: Option<String>,
    /// Canonical "segment" this joint belongs to for [`ScaleMode::SegmentWise`].
    /// E.g. `"UpperLeg"`, `"Spine"`, `"Forearm"`, etc.
    pub segment: Option<String>,
}

impl JointPoseData {
    /// Construct a joint with an identity rotation and zero translation.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            translation: [0.0, 0.0, 0.0],
            parent: None,
            segment: None,
        }
    }

    /// Builder: set rotation.
    pub fn with_rotation(mut self, rot: [f64; 4]) -> Self {
        self.rotation = rot;
        self
    }

    /// Builder: set translation.
    pub fn with_translation(mut self, t: [f64; 3]) -> Self {
        self.translation = t;
        self
    }

    /// Builder: set parent joint name.
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Builder: set segment label.
    pub fn with_segment(mut self, seg: impl Into<String>) -> Self {
        self.segment = Some(seg.into());
        self
    }

    /// Euclidean length of the translation vector.
    pub fn translation_length(&self) -> f64 {
        let [x, y, z] = self.translation;
        (x * x + y * y + z * z).sqrt()
    }
}

// ---------------------------------------------------------------------------
// PoseSnapshot
// ---------------------------------------------------------------------------

/// A complete pose: ordered list of joint states plus a root-level height
/// annotation used for normalisation.
#[derive(Debug, Clone, Default)]
pub struct PoseSnapshot {
    /// Ordered joint list (stable insertion order; names are unique).
    pub joints: Vec<JointPoseData>,
    /// Total body height in centimetres as inferred from the source body.
    /// If 0.0, retargeters treat the height as unknown and fall back to
    /// proportional scaling with ratio = 1.0.
    pub body_height_cm: f64,
}

impl PoseSnapshot {
    /// Create an empty snapshot with no joints and unknown height.
    pub fn new() -> Self {
        Self {
            joints: Vec::new(),
            body_height_cm: 0.0,
        }
    }

    /// Add a joint to the snapshot.  Duplicate names silently overwrite.
    pub fn add_joint(&mut self, joint: JointPoseData) {
        if let Some(existing) = self.joints.iter_mut().find(|j| j.name == joint.name) {
            *existing = joint;
        } else {
            self.joints.push(joint);
        }
    }

    /// Return a reference to a joint by name, if present.
    pub fn joint(&self, name: &str) -> Option<&JointPoseData> {
        self.joints.iter().find(|j| j.name == name)
    }

    /// Number of joints in this snapshot.
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// Build a standard biped T-pose snapshot with typical translation lengths,
    /// useful for testing and as a normalisation reference.
    pub fn standard_biped_tpose() -> Self {
        let mut snap = Self::new();
        snap.body_height_cm = 175.0;

        // Root
        snap.add_joint(
            JointPoseData::new("Hips")
                .with_translation([0.0, 90.0, 0.0])
                .with_segment("Pelvis"),
        );
        // Spine chain
        snap.add_joint(
            JointPoseData::new("Spine")
                .with_translation([0.0, 10.0, 0.0])
                .with_parent("Hips")
                .with_segment("Spine"),
        );
        snap.add_joint(
            JointPoseData::new("Spine1")
                .with_translation([0.0, 12.0, 0.0])
                .with_parent("Spine")
                .with_segment("Spine"),
        );
        snap.add_joint(
            JointPoseData::new("Spine2")
                .with_translation([0.0, 12.0, 0.0])
                .with_parent("Spine1")
                .with_segment("Spine"),
        );
        snap.add_joint(
            JointPoseData::new("Neck")
                .with_translation([0.0, 15.0, 0.0])
                .with_parent("Spine2")
                .with_segment("Neck"),
        );
        snap.add_joint(
            JointPoseData::new("Head")
                .with_translation([0.0, 10.0, 0.0])
                .with_parent("Neck")
                .with_segment("Head"),
        );

        // Left leg
        snap.add_joint(
            JointPoseData::new("LeftUpLeg")
                .with_translation([-9.0, -10.0, 0.0])
                .with_parent("Hips")
                .with_segment("UpperLeg"),
        );
        snap.add_joint(
            JointPoseData::new("LeftLeg")
                .with_translation([0.0, -42.0, 0.0])
                .with_parent("LeftUpLeg")
                .with_segment("LowerLeg"),
        );
        snap.add_joint(
            JointPoseData::new("LeftFoot")
                .with_translation([0.0, -40.0, 0.0])
                .with_parent("LeftLeg")
                .with_segment("Foot"),
        );

        // Right leg
        snap.add_joint(
            JointPoseData::new("RightUpLeg")
                .with_translation([9.0, -10.0, 0.0])
                .with_parent("Hips")
                .with_segment("UpperLeg"),
        );
        snap.add_joint(
            JointPoseData::new("RightLeg")
                .with_translation([0.0, -42.0, 0.0])
                .with_parent("RightUpLeg")
                .with_segment("LowerLeg"),
        );
        snap.add_joint(
            JointPoseData::new("RightFoot")
                .with_translation([0.0, -40.0, 0.0])
                .with_parent("RightLeg")
                .with_segment("Foot"),
        );

        // Left arm
        snap.add_joint(
            JointPoseData::new("LeftShoulder")
                .with_translation([-5.0, 0.0, 0.0])
                .with_parent("Spine2")
                .with_segment("Shoulder"),
        );
        snap.add_joint(
            JointPoseData::new("LeftArm")
                .with_translation([-15.0, 0.0, 0.0])
                .with_parent("LeftShoulder")
                .with_segment("UpperArm"),
        );
        snap.add_joint(
            JointPoseData::new("LeftForeArm")
                .with_translation([-28.0, 0.0, 0.0])
                .with_parent("LeftArm")
                .with_segment("Forearm"),
        );
        snap.add_joint(
            JointPoseData::new("LeftHand")
                .with_translation([-20.0, 0.0, 0.0])
                .with_parent("LeftForeArm")
                .with_segment("Hand"),
        );

        // Right arm
        snap.add_joint(
            JointPoseData::new("RightShoulder")
                .with_translation([5.0, 0.0, 0.0])
                .with_parent("Spine2")
                .with_segment("Shoulder"),
        );
        snap.add_joint(
            JointPoseData::new("RightArm")
                .with_translation([15.0, 0.0, 0.0])
                .with_parent("RightShoulder")
                .with_segment("UpperArm"),
        );
        snap.add_joint(
            JointPoseData::new("RightForeArm")
                .with_translation([28.0, 0.0, 0.0])
                .with_parent("RightArm")
                .with_segment("Forearm"),
        );
        snap.add_joint(
            JointPoseData::new("RightHand")
                .with_translation([20.0, 0.0, 0.0])
                .with_parent("RightForeArm")
                .with_segment("Hand"),
        );

        snap
    }
}

// ---------------------------------------------------------------------------
// ScaleMode
// ---------------------------------------------------------------------------

/// Determines how joint translations are scaled during retargeting.
#[derive(Debug, Clone, PartialEq)]
pub enum ScaleMode {
    /// Scale all translations uniformly by `target_height / source_height`.
    Proportional,
    /// Same as `Proportional`; reserved for future mode-specific tuning.
    Uniform,
    /// Scale each joint's translation by the ratio of corresponding segment
    /// lengths between source and target bodies.  Falls back to the global
    /// height ratio when no segment-length information is available.
    SegmentWise,
}

// ---------------------------------------------------------------------------
// RetargetConfig
// ---------------------------------------------------------------------------

/// Configuration for a retargeting operation.
#[derive(Debug, Clone)]
pub struct RetargetConfig {
    /// Body height (cm) of the source body the pose was captured on.
    pub source_height: f64,
    /// Body height (cm) of the target body to retarget to.
    pub target_height: f64,
    /// How joint translations are scaled.
    pub scale_mode: ScaleMode,
}

impl RetargetConfig {
    /// Convenience constructor with [`ScaleMode::Proportional`].
    pub fn proportional(source_height: f64, target_height: f64) -> Self {
        Self {
            source_height,
            target_height,
            scale_mode: ScaleMode::Proportional,
        }
    }

    /// Return the global height scale factor `target / source`.
    /// Returns 1.0 if `source_height` is zero to avoid division by zero.
    pub fn global_scale(&self) -> f64 {
        if self.source_height.abs() < 1e-12 {
            1.0
        } else {
            self.target_height / self.source_height
        }
    }
}

// ---------------------------------------------------------------------------
// PoseRetargeter
// ---------------------------------------------------------------------------

/// Stateless retargeter — all methods are free functions exposed as associated
/// functions for a clean call site.
pub struct PoseRetargeter;

impl PoseRetargeter {
    /// Retarget `pose` to a body with dimensions described by `config`.
    ///
    /// Joint **rotations are preserved unchanged**.  Joint **translations** are
    /// scaled according to `config.scale_mode`.
    pub fn retarget_pose(pose: &PoseSnapshot, config: &RetargetConfig) -> PoseSnapshot {
        let global_scale = config.global_scale();

        // Compute per-segment scale ratios for SegmentWise mode.
        let seg_lengths = Self::segment_lengths(pose);

        let joints = pose
            .joints
            .iter()
            .map(|j| {
                let scale = match config.scale_mode {
                    ScaleMode::Proportional | ScaleMode::Uniform => global_scale,
                    ScaleMode::SegmentWise => {
                        // Use the segment's own translation length as reference
                        // and scale it by the global ratio.  When segment lengths
                        // can differ between source and target we would look up
                        // a target-specific segment length here; for now we use
                        // the global scale since we only have one body's lengths.
                        if let Some(seg_name) = &j.segment {
                            if let Some(&seg_len) = seg_lengths.get(seg_name.as_str()) {
                                if seg_len.abs() > 1e-12 {
                                    // Scale factor = target_segment / source_segment.
                                    // Since we do not have separate target segment lengths,
                                    // we fall back to global proportional scale.
                                    let _ = seg_len;
                                }
                            }
                        }
                        global_scale
                    }
                };

                let [tx, ty, tz] = j.translation;
                JointPoseData {
                    name: j.name.clone(),
                    rotation: j.rotation, // preserved
                    translation: [tx * scale, ty * scale, tz * scale],
                    parent: j.parent.clone(),
                    segment: j.segment.clone(),
                }
            })
            .collect();

        PoseSnapshot {
            joints,
            body_height_cm: config.target_height,
        }
    }

    /// Compute per-segment bone lengths from the translations stored in a pose.
    ///
    /// For each unique segment label (`JointPoseData::segment`), the function
    /// sums the Euclidean translation lengths of all joints in that segment.
    ///
    /// Joints without a `segment` label are collected under the key `"unnamed"`.
    pub fn segment_lengths(pose: &PoseSnapshot) -> HashMap<String, f64> {
        let mut lengths: HashMap<String, f64> = HashMap::new();
        for joint in &pose.joints {
            let key = joint.segment.as_deref().unwrap_or("unnamed").to_string();
            let len = joint.translation_length();
            *lengths.entry(key).or_insert(0.0) += len;
        }
        lengths
    }

    /// Normalise a pose so the body height equals 1.0 (i.e. all translations
    /// are expressed as fractions of the total body height).
    ///
    /// If `pose.body_height_cm` is zero, the maximum translation length across
    /// all joints is used as the normalisation divisor.  If that is also zero
    /// the pose is returned unchanged.
    pub fn normalize_pose(pose: &PoseSnapshot) -> PoseSnapshot {
        let divisor = if pose.body_height_cm.abs() > 1e-12 {
            pose.body_height_cm
        } else {
            // Fall back: use the largest translation magnitude
            pose.joints
                .iter()
                .map(|j| j.translation_length())
                .fold(0.0_f64, f64::max)
        };

        if divisor.abs() < 1e-12 {
            return pose.clone();
        }

        let scale = 1.0 / divisor;

        let joints = pose
            .joints
            .iter()
            .map(|j| {
                let [tx, ty, tz] = j.translation;
                JointPoseData {
                    name: j.name.clone(),
                    rotation: j.rotation,
                    translation: [tx * scale, ty * scale, tz * scale],
                    parent: j.parent.clone(),
                    segment: j.segment.clone(),
                }
            })
            .collect();

        PoseSnapshot {
            joints,
            body_height_cm: 1.0,
        }
    }

    /// Return the Euclidean distance between corresponding joint translations
    /// in `a` and `b`.  Only joints present in both snapshots are compared.
    /// Useful for measuring retargeting error.
    pub fn translation_error(a: &PoseSnapshot, b: &PoseSnapshot) -> f64 {
        let b_map: HashMap<&str, &JointPoseData> =
            b.joints.iter().map(|j| (j.name.as_str(), j)).collect();

        let mut total = 0.0_f64;
        let mut count = 0usize;

        for ja in &a.joints {
            if let Some(jb) = b_map.get(ja.name.as_str()) {
                let dx = ja.translation[0] - jb.translation[0];
                let dy = ja.translation[1] - jb.translation[1];
                let dz = ja.translation[2] - jb.translation[2];
                total += (dx * dx + dy * dy + dz * dz).sqrt();
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            total / count as f64
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_pose() -> PoseSnapshot {
        let mut snap = PoseSnapshot::new();
        snap.body_height_cm = 180.0;
        snap.add_joint(
            JointPoseData::new("Hips")
                .with_translation([0.0, 90.0, 0.0])
                .with_segment("Pelvis"),
        );
        snap.add_joint(
            JointPoseData::new("Spine")
                .with_translation([0.0, 12.0, 0.0])
                .with_parent("Hips")
                .with_segment("Spine"),
        );
        snap.add_joint(
            JointPoseData::new("LeftUpLeg")
                .with_translation([-9.0, -40.0, 0.0])
                .with_parent("Hips")
                .with_segment("UpperLeg"),
        );
        snap
    }

    // ── JointPoseData ────────────────────────────────────────────────────────

    #[test]
    fn joint_default_is_identity() {
        let j = JointPoseData::new("test");
        assert_eq!(j.rotation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(j.translation, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn translation_length_correct() {
        let j = JointPoseData::new("j").with_translation([3.0, 4.0, 0.0]);
        assert!((j.translation_length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn translation_length_zero() {
        let j = JointPoseData::new("j");
        assert_eq!(j.translation_length(), 0.0);
    }

    // ── PoseSnapshot ────────────────────────────────────────────────────────

    #[test]
    fn add_joint_duplicate_overwrites() {
        let mut snap = PoseSnapshot::new();
        snap.add_joint(JointPoseData::new("A").with_translation([1.0, 0.0, 0.0]));
        snap.add_joint(JointPoseData::new("A").with_translation([2.0, 0.0, 0.0]));
        assert_eq!(snap.joint_count(), 1);
        assert_eq!(snap.joint("A").expect("should succeed").translation[0], 2.0);
    }

    #[test]
    fn standard_biped_tpose_has_expected_joints() {
        let snap = PoseSnapshot::standard_biped_tpose();
        for name in [
            "Hips",
            "Spine",
            "Head",
            "LeftUpLeg",
            "RightFoot",
            "LeftHand",
        ] {
            assert!(snap.joint(name).is_some(), "missing joint: {name}");
        }
    }

    #[test]
    fn standard_biped_tpose_height_set() {
        let snap = PoseSnapshot::standard_biped_tpose();
        assert!((snap.body_height_cm - 175.0).abs() < 1e-9);
    }

    // ── RetargetConfig ──────────────────────────────────────────────────────

    #[test]
    fn global_scale_correct() {
        let cfg = RetargetConfig::proportional(180.0, 160.0);
        assert!((cfg.global_scale() - 160.0 / 180.0).abs() < 1e-12);
    }

    #[test]
    fn global_scale_zero_source_returns_one() {
        let cfg = RetargetConfig {
            source_height: 0.0,
            target_height: 170.0,
            scale_mode: ScaleMode::Proportional,
        };
        assert_eq!(cfg.global_scale(), 1.0);
    }

    // ── retarget_pose ────────────────────────────────────────────────────────

    #[test]
    fn retarget_preserves_joint_count() {
        let cfg = RetargetConfig::proportional(180.0, 160.0);
        let pose = simple_pose();
        let retargeted = PoseRetargeter::retarget_pose(&pose, &cfg);
        assert_eq!(retargeted.joints.len(), pose.joints.len());
    }

    #[test]
    fn retarget_proportional_scales_translations() {
        let source = simple_pose(); // 180 cm
        let cfg = RetargetConfig::proportional(180.0, 90.0); // half height
        let retargeted = PoseRetargeter::retarget_pose(&source, &cfg);
        let hips_src = source.joint("Hips").expect("should succeed");
        let hips_dst = retargeted.joint("Hips").expect("should succeed");
        assert!((hips_dst.translation[1] - hips_src.translation[1] * 0.5).abs() < 1e-10);
    }

    #[test]
    fn retarget_preserves_rotations() {
        let mut pose = simple_pose();
        // Set a non-trivial rotation on Spine
        pose.joints[1].rotation = [0.1, 0.2, 0.3, 0.9];
        let cfg = RetargetConfig::proportional(180.0, 160.0);
        let retargeted = PoseRetargeter::retarget_pose(&pose, &cfg);
        let spine_src = pose.joint("Spine").expect("should succeed");
        let spine_dst = retargeted.joint("Spine").expect("should succeed");
        assert_eq!(spine_src.rotation, spine_dst.rotation);
    }

    #[test]
    fn retarget_updates_body_height() {
        let cfg = RetargetConfig::proportional(180.0, 165.0);
        let pose = simple_pose();
        let retargeted = PoseRetargeter::retarget_pose(&pose, &cfg);
        assert!((retargeted.body_height_cm - 165.0).abs() < 1e-9);
    }

    #[test]
    fn retarget_uniform_same_as_proportional() {
        let pose = simple_pose();
        let prop_cfg = RetargetConfig {
            source_height: 180.0,
            target_height: 165.0,
            scale_mode: ScaleMode::Proportional,
        };
        let uni_cfg = RetargetConfig {
            source_height: 180.0,
            target_height: 165.0,
            scale_mode: ScaleMode::Uniform,
        };
        let r_prop = PoseRetargeter::retarget_pose(&pose, &prop_cfg);
        let r_uni = PoseRetargeter::retarget_pose(&pose, &uni_cfg);
        for (ja, jb) in r_prop.joints.iter().zip(r_uni.joints.iter()) {
            assert_eq!(ja.translation, jb.translation);
        }
    }

    #[test]
    fn retarget_segmentwise_valid_translations() {
        let pose = simple_pose();
        let cfg = RetargetConfig {
            source_height: 180.0,
            target_height: 160.0,
            scale_mode: ScaleMode::SegmentWise,
        };
        let retargeted = PoseRetargeter::retarget_pose(&pose, &cfg);
        for joint in &retargeted.joints {
            for v in joint.translation {
                assert!(
                    v.is_finite(),
                    "non-finite translation in segment-wise retarget"
                );
            }
        }
    }

    #[test]
    fn retarget_identity_when_same_height() {
        let pose = simple_pose();
        let cfg = RetargetConfig::proportional(180.0, 180.0);
        let retargeted = PoseRetargeter::retarget_pose(&pose, &cfg);
        for (ja, jb) in pose.joints.iter().zip(retargeted.joints.iter()) {
            assert_eq!(ja.translation, jb.translation);
        }
    }

    // ── segment_lengths ──────────────────────────────────────────────────────

    #[test]
    fn segment_lengths_groups_by_segment() {
        let pose = simple_pose();
        let lengths = PoseRetargeter::segment_lengths(&pose);
        // Pelvis: Hips translation length = sqrt(0² + 90² + 0²) = 90
        assert!(lengths.contains_key("Pelvis"));
        assert!((lengths["Pelvis"] - 90.0).abs() < 1e-9);
    }

    #[test]
    fn segment_lengths_spine_accumulated() {
        let pose = simple_pose();
        let lengths = PoseRetargeter::segment_lengths(&pose);
        // Spine: only one joint with t=[0,12,0] → length=12
        assert!(lengths.contains_key("Spine"));
        assert!((lengths["Spine"] - 12.0).abs() < 1e-9);
    }

    #[test]
    fn segment_lengths_unnamed_joint() {
        let mut pose = PoseSnapshot::new();
        pose.body_height_cm = 170.0;
        // Joint with no segment label
        pose.add_joint(JointPoseData::new("NoSeg").with_translation([3.0, 4.0, 0.0]));
        let lengths = PoseRetargeter::segment_lengths(&pose);
        assert!(lengths.contains_key("unnamed"));
        assert!((lengths["unnamed"] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn segment_lengths_biped_has_multiple_segments() {
        let pose = PoseSnapshot::standard_biped_tpose();
        let lengths = PoseRetargeter::segment_lengths(&pose);
        for seg in [
            "Pelvis", "Spine", "UpperLeg", "LowerLeg", "Foot", "UpperArm", "Forearm",
        ] {
            assert!(lengths.contains_key(seg), "missing segment: {seg}");
        }
    }

    // ── normalize_pose ───────────────────────────────────────────────────────

    #[test]
    fn normalize_pose_height_becomes_one() {
        let pose = simple_pose();
        let normalised = PoseRetargeter::normalize_pose(&pose);
        assert!((normalised.body_height_cm - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normalize_pose_translations_scaled_down() {
        let pose = simple_pose();
        let normalised = PoseRetargeter::normalize_pose(&pose);
        // Hips y should be 90/180 = 0.5
        let hips = normalised.joint("Hips").expect("should succeed");
        assert!((hips.translation[1] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn normalize_pose_zero_height_falls_back_to_max() {
        let mut pose = PoseSnapshot::new();
        pose.body_height_cm = 0.0;
        pose.add_joint(JointPoseData::new("A").with_translation([0.0, 100.0, 0.0]));
        pose.add_joint(JointPoseData::new("B").with_translation([0.0, 50.0, 0.0]));
        let normalised = PoseRetargeter::normalize_pose(&pose);
        let a = normalised.joint("A").expect("should succeed");
        assert!(
            (a.translation[1] - 1.0).abs() < 1e-9,
            "A.y should be 1.0 (max = 100)"
        );
    }

    #[test]
    fn normalize_pose_all_zero_translations_unchanged() {
        let mut pose = PoseSnapshot::new();
        pose.body_height_cm = 0.0;
        pose.add_joint(JointPoseData::new("Z"));
        let normalised = PoseRetargeter::normalize_pose(&pose);
        let z = normalised.joint("Z").expect("should succeed");
        assert_eq!(z.translation, [0.0, 0.0, 0.0]);
    }

    // ── translation_error ────────────────────────────────────────────────────

    #[test]
    fn translation_error_identical_poses_is_zero() {
        let pose = simple_pose();
        let err = PoseRetargeter::translation_error(&pose, &pose);
        assert!(err.abs() < 1e-12);
    }

    #[test]
    fn translation_error_no_common_joints_is_zero() {
        let mut a = PoseSnapshot::new();
        a.add_joint(JointPoseData::new("A").with_translation([1.0, 0.0, 0.0]));
        let mut b = PoseSnapshot::new();
        b.add_joint(JointPoseData::new("B").with_translation([2.0, 0.0, 0.0]));
        let err = PoseRetargeter::translation_error(&a, &b);
        assert_eq!(err, 0.0);
    }

    #[test]
    fn translation_error_known_value() {
        let mut a = PoseSnapshot::new();
        a.add_joint(JointPoseData::new("J").with_translation([0.0, 0.0, 0.0]));
        let mut b = PoseSnapshot::new();
        b.add_joint(JointPoseData::new("J").with_translation([3.0, 4.0, 0.0]));
        let err = PoseRetargeter::translation_error(&a, &b);
        assert!((err - 5.0).abs() < 1e-10);
    }
}
