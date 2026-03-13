// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::interpolate::{Keyframe, MorphTrack};
use crate::mocap_bvh::{parse_bvh, BvhChannel};
use crate::params::ParamState;
use std::collections::HashMap;

/// A mapping entry: scale one parameter by a factor and optionally offset it.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParamRetarget {
    /// Factor to multiply the parameter value (default 1.0).
    pub scale: f32,
    /// Offset to add after scaling (default 0.0).
    pub offset: f32,
    /// Clamp result to [0, 1] range (default true).
    pub clamp: bool,
}

impl ParamRetarget {
    /// Identity mapping: scale=1.0, offset=0.0, clamp=true.
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            scale: 1.0,
            offset: 0.0,
            clamp: true,
        }
    }

    /// Scaled mapping: scale=factor, offset=0.0, clamp=true.
    #[allow(dead_code)]
    pub fn scaled(factor: f32) -> Self {
        Self {
            scale: factor,
            offset: 0.0,
            clamp: true,
        }
    }

    /// Apply the retarget to a single value.
    #[allow(dead_code)]
    pub fn apply(&self, value: f32) -> f32 {
        let result = value * self.scale + self.offset;
        if self.clamp {
            result.clamp(0.0, 1.0)
        } else {
            result
        }
    }
}

/// Configuration for retargeting a full `ParamState`.
#[allow(dead_code)]
pub struct AnimRetargetConfig {
    pub height: ParamRetarget,
    pub weight: ParamRetarget,
    pub muscle: ParamRetarget,
    pub age: ParamRetarget,
}

impl AnimRetargetConfig {
    /// All identity (no-op).
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            height: ParamRetarget::identity(),
            weight: ParamRetarget::identity(),
            muscle: ParamRetarget::identity(),
            age: ParamRetarget::identity(),
        }
    }

    /// Apply the retarget config to a single `ParamState`.
    #[allow(dead_code)]
    pub fn apply(&self, state: &ParamState) -> ParamState {
        ParamState {
            height: self.height.apply(state.height),
            weight: self.weight.apply(state.weight),
            muscle: self.muscle.apply(state.muscle),
            age: self.age.apply(state.age),
            extra: state.extra.clone(),
        }
    }
}

/// Retarget a single keyframe.
#[allow(dead_code)]
pub fn retarget_keyframe(kf: &Keyframe, config: &AnimRetargetConfig) -> Keyframe {
    Keyframe {
        time: kf.time,
        params: config.apply(&kf.params),
        label: kf.label.clone(),
    }
}

/// Retarget an entire `MorphTrack`.
#[allow(dead_code)]
pub fn retarget_track(track: &MorphTrack, config: &AnimRetargetConfig) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    for kf in track.keyframes_iter() {
        new_track.add_keyframe(retarget_keyframe(kf, config));
    }
    new_track
}

/// Scale animation duration: compress/expand timeline by a factor.
/// All keyframe times are multiplied by `time_scale`.
#[allow(dead_code)]
pub fn scale_track_time(track: &MorphTrack, time_scale: f32) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    for kf in track.keyframes_iter() {
        new_track.add_keyframe(Keyframe {
            time: kf.time * time_scale,
            params: kf.params.clone(),
            label: kf.label.clone(),
        });
    }
    new_track
}

/// Trim a track to a time window [start_time, end_time].
/// Keyframes outside the window are removed. Times are not shifted.
#[allow(dead_code)]
pub fn trim_track(track: &MorphTrack, start_time: f32, end_time: f32) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    for kf in track.keyframes_iter() {
        if kf.time >= start_time && kf.time <= end_time {
            new_track.add_keyframe(kf.clone());
        }
    }
    new_track
}

/// Reverse a track (flip time: new_time = duration - old_time).
#[allow(dead_code)]
pub fn reverse_track(track: &MorphTrack) -> MorphTrack {
    let mut new_track = MorphTrack::new(track.name.clone());
    if track.is_empty() {
        return new_track;
    }
    let kfs: Vec<&Keyframe> = track.keyframes_iter().collect();
    let last_time = kfs[kfs.len() - 1].time;
    for kf in kfs {
        new_track.add_keyframe(Keyframe {
            time: last_time - kf.time,
            params: kf.params.clone(),
            label: kf.label.clone(),
        });
    }
    new_track
}

/// Concatenate two tracks. The second track's times are offset by the first track's last keyframe time.
#[allow(dead_code)]
pub fn concat_tracks(first: &MorphTrack, second: &MorphTrack) -> MorphTrack {
    let mut new_track = MorphTrack::new(first.name.clone());

    for kf in first.keyframes_iter() {
        new_track.add_keyframe(kf.clone());
    }

    let offset = first
        .keyframes_iter()
        .last()
        .map(|kf| kf.time)
        .unwrap_or(0.0);

    for kf in second.keyframes_iter() {
        new_track.add_keyframe(Keyframe {
            time: kf.time + offset,
            params: kf.params.clone(),
            label: kf.label.clone(),
        });
    }

    new_track
}

// ── BVH Animation Retargeting Bridge ─────────────────────────────────────────

/// Per-joint decomposed frame (distinct from `mocap_bvh::BvhFrame` which is flat-channel).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhJointFrame {
    /// Name of the joint.
    pub joint_name: String,
    /// Unit quaternion [x, y, z, w] computed from ZXY Euler angles.
    pub local_rotation: [f32; 4],
    /// Translation for the root joint; zero for non-root joints.
    pub local_position: [f32; 3],
}

/// Parsed BVH data as per-joint, per-frame quaternions plus fps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhData {
    /// Frames per second derived from the BVH `Frame Time` field.
    pub fps: f32,
    /// Outer Vec: frames; inner Vec: one entry per joint that has rotation channels.
    pub frames: Vec<Vec<BvhJointFrame>>,
}

/// Bidirectional name mapping from BVH joint names to target skeleton joint names.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkeletonMapping {
    /// Key: BVH joint name.  Value: target skeleton joint name.
    pub map: HashMap<String, String>,
}

impl SkeletonMapping {
    /// Default CMU BVH → OxiHuman mapping for the most common joints.
    #[allow(dead_code)]
    pub fn default_cmu() -> Self {
        let entries: &[(&str, &str)] = &[
            ("Hips", "pelvis"),
            ("Spine", "torso"),
            ("Spine1", "spine_02"),
            ("Spine2", "spine_03"),
            ("Neck", "neck_01"),
            ("Head", "head"),
            ("LeftArm", "left_shoulder"),
            ("LeftForeArm", "left_elbow"),
            ("LeftHand", "left_wrist"),
            ("RightArm", "right_shoulder"),
            ("RightForeArm", "right_elbow"),
            ("RightHand", "right_wrist"),
            ("LeftUpLeg", "left_hip"),
            ("LeftLeg", "left_knee"),
            ("LeftFoot", "left_ankle"),
            ("RightUpLeg", "right_hip"),
            ("RightLeg", "right_knee"),
            ("RightFoot", "right_ankle"),
        ];
        let map = entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self { map }
    }

    /// Build a mapping from a caller-supplied `HashMap`.
    #[allow(dead_code)]
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self { map }
    }
}

/// Convert ZXY Euler angles (degrees) to a unit quaternion [x, y, z, w].
///
/// The BVH ZXY convention applies rotations in the order Z then X then Y, which
/// corresponds to the matrix product Ry * Rx * Rz.  We compute the quaternion
/// equivalent without any external dependencies.
fn euler_zxy_to_quat(rx_deg: f32, ry_deg: f32, rz_deg: f32) -> [f32; 4] {
    let half = std::f32::consts::PI / 360.0; // deg→rad / 2
    let (sx, cx) = (rx_deg * half).sin_cos();
    let (sy, cy) = (ry_deg * half).sin_cos();
    let (sz, cz) = (rz_deg * half).sin_cos();

    // Quaternion multiply order: Rz * Rx * Ry  (ZXY convention)
    // q_result = q_z ⊗ q_x ⊗ q_y
    // q_z = [0, 0, sz, cz]
    // q_x = [sx, 0,  0,  cx]
    // q_y = [0, sy,  0,  cy]

    // First: q_zx = q_z ⊗ q_x
    let zx_x = cz * sx + sz * cx; // cx*sz*1 - ...
    let zx_y = -sz * sx; // actually: cz*0 + sz*cx*0 - sz*(-sx) ... let's do it explicitly
                         // q1 = (a1,b1,c1,d1) ⊗ q2 = (a2,b2,c2,d2)
                         // result.x = d1*a2 + a1*d2 + b1*c2 - c1*b2
                         // result.y = d1*b2 - a1*c2 + b1*d2 + c1*a2
                         // result.z = d1*c2 + a1*b2 - b1*a2 + c1*d2
                         // result.w = d1*d2 - a1*a2 - b1*b2 - c1*c2
                         // q_z: x=0, y=0, z=sz, w=cz
                         // q_x: x=sx, y=0, z=0, w=cx
    let _ = (zx_x, zx_y); // discard the ad-hoc intermediate
    let zx_xi = cz * sx + sz * 0.0 + 0.0 * 0.0 - 0.0 * cx; // d1*a2 + a1*d2 + b1*c2 - c1*b2 = cz*sx
    let zx_yi = cz * 0.0 - 0.0 * 0.0 + 0.0 * cx + sz * sx; // d1*b2 - a1*c2 + b1*d2 + c1*a2 = sz*sx
    let zx_zi = cz * 0.0 + 0.0 * sx - 0.0 * 0.0 + sz * cx; // d1*c2 + a1*b2 - b1*a2 + c1*d2 = sz*cx
    let zx_w = cz * cx - 0.0 * sx - 0.0 * 0.0 - sz * 0.0; // d1*d2 - a1*a2 - b1*b2 - c1*c2 = cz*cx

    // Now: q_result = q_zx ⊗ q_y
    // q_y: x=0, y=sy, z=0, w=cy
    let rx = zx_w * 0.0 + zx_xi * cy + zx_yi * 0.0 - zx_zi * sy;
    // d1*a2 + a1*d2 + b1*c2 - c1*b2 = zx_xi*cy - zx_zi*sy
    let rx2 = zx_xi * cy - zx_zi * sy;
    let _ = rx;
    let ry2 = zx_w * sy + (-zx_xi) * 0.0 + zx_yi * cy + zx_zi * 0.0;
    // d1*b2 - a1*c2 + b1*d2 + c1*a2 = zx_w*sy + zx_yi*cy
    let ry3 = zx_w * sy + zx_yi * cy;
    let _ = ry2;
    let rz2 = zx_w * 0.0 + zx_xi * sy - zx_yi * 0.0 + zx_zi * cy;
    // d1*c2 + a1*b2 - b1*a2 + c1*d2 = zx_xi*sy + zx_zi*cy
    let rz3 = zx_xi * sy + zx_zi * cy;
    let _ = rz2;
    let rw = zx_w * cy - zx_xi * 0.0 - zx_yi * sy - zx_zi * 0.0;
    // d1*d2 - a1*a2 - b1*b2 - c1*c2 = zx_w*cy - zx_yi*sy
    let rw2 = zx_w * cy - zx_yi * sy;
    let _ = rw;

    // Normalise (should already be unit, but floating-point drift)
    let len = (rx2 * rx2 + ry3 * ry3 + rz3 * rz3 + rw2 * rw2).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [rx2 / len, ry3 / len, rz3 / len, rw2 / len]
    }
}

/// Parse BVH text into a `BvhData` with per-joint quaternion frames.
///
/// Calls [`crate::mocap_bvh::parse_bvh`] internally and decomposes the flat
/// channel arrays into per-joint `BvhJointFrame` entries.
#[allow(dead_code)]
pub fn parse_bvh_text(bvh: &str) -> anyhow::Result<BvhData> {
    let bvh_file = parse_bvh(bvh).map_err(|e| anyhow::anyhow!("BVH parse error: {}", e))?;

    let fps = if bvh_file.frame_time > 0.0 {
        1.0 / bvh_file.frame_time
    } else {
        30.0
    };

    // Build a list of joints that have at least one channel (skip end-sites)
    let skeleton = &bvh_file.skeleton;
    let active_joints: Vec<usize> = skeleton
        .joints
        .iter()
        .enumerate()
        .filter(|(_, j)| !j.channels.is_empty())
        .map(|(i, _)| i)
        .collect();

    let root_idx = skeleton.root_index;

    let mut all_frames: Vec<Vec<BvhJointFrame>> = Vec::with_capacity(bvh_file.frames.len());

    for bvh_frame in &bvh_file.frames {
        let mut joint_frames: Vec<BvhJointFrame> = Vec::with_capacity(active_joints.len());

        for &joint_idx in &active_joints {
            let joint = &skeleton.joints[joint_idx];
            let ch_offset = skeleton.channel_offset_for(joint_idx);
            let is_root = joint_idx == root_idx;

            // Gather rotation and translation values from flat channel array
            let mut rx = 0.0_f32;
            let mut ry = 0.0_f32;
            let mut rz = 0.0_f32;
            let mut tx = 0.0_f32;
            let mut ty = 0.0_f32;
            let mut tz = 0.0_f32;

            for (i, ch) in joint.channels.iter().enumerate() {
                let val = bvh_frame.values.get(ch_offset + i).copied().unwrap_or(0.0);
                match ch {
                    BvhChannel::Xrotation => rx = val,
                    BvhChannel::Yrotation => ry = val,
                    BvhChannel::Zrotation => rz = val,
                    BvhChannel::Xposition => tx = val,
                    BvhChannel::Yposition => ty = val,
                    BvhChannel::Zposition => tz = val,
                }
            }

            let local_rotation = euler_zxy_to_quat(rx, ry, rz);
            let local_position = if is_root {
                [tx, ty, tz]
            } else {
                [0.0, 0.0, 0.0]
            };

            joint_frames.push(BvhJointFrame {
                joint_name: joint.name.clone(),
                local_rotation,
                local_position,
            });
        }

        all_frames.push(joint_frames);
    }

    Ok(BvhData {
        fps,
        frames: all_frames,
    })
}

/// Retarget BVH motion to parameter tracks.
///
/// For each BVH joint in the mapping, extracts the dominant rotation component
/// across all frames, then normalises the per-frame values to [0, 1].
///
/// Returns a `HashMap<target_joint_name, Vec<f32>>` where the `Vec` has one
/// entry per BVH frame.
#[allow(dead_code)]
pub fn retarget_bvh_to_param_tracks(
    bvh: &BvhData,
    mapping: &SkeletonMapping,
) -> HashMap<String, Vec<f32>> {
    let mut result: HashMap<String, Vec<f32>> = HashMap::new();

    for (bvh_name, target_name) in &mapping.map {
        // Collect the dominant rotation scalar (max absolute component of quaternion xyz)
        // across all frames for this joint.
        let mut raw_values: Vec<f32> = Vec::with_capacity(bvh.frames.len());

        for frame_joints in &bvh.frames {
            // Find the joint by name in this frame
            let jf = frame_joints.iter().find(|jf| &jf.joint_name == bvh_name);
            let dominant = match jf {
                None => 0.0_f32,
                Some(jf) => {
                    // Dominant: component of quat [x,y,z] with highest absolute value
                    let [qx, qy, qz, _qw] = jf.local_rotation;
                    let ax = qx.abs();
                    let ay = qy.abs();
                    let az = qz.abs();
                    if ax >= ay && ax >= az {
                        qx
                    } else if ay >= az {
                        qy
                    } else {
                        qz
                    }
                }
            };
            raw_values.push(dominant);
        }

        if raw_values.is_empty() {
            continue;
        }

        // Normalise to [0, 1]: map [-1, 1] → [0, 1]
        // (quaternion components are in [-1, 1])
        let normalised: Vec<f32> = raw_values.iter().map(|&v| (v + 1.0) * 0.5).collect();

        result.insert(target_name.clone(), normalised);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::{Keyframe, MorphTrack};
    use crate::params::ParamState;

    fn make_params(h: f32, w: f32, m: f32, a: f32) -> ParamState {
        ParamState::new(h, w, m, a)
    }

    fn make_track_two_kf() -> MorphTrack {
        let mut track = MorphTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, make_params(0.2, 0.3, 0.4, 0.5)));
        track.add_keyframe(Keyframe::new(1.0, make_params(0.6, 0.7, 0.8, 0.9)));
        track
    }

    #[test]
    fn param_retarget_identity_unchanged() {
        let r = ParamRetarget::identity();
        assert!((r.apply(0.7) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn param_retarget_scaled_doubles() {
        let r = ParamRetarget::scaled(2.0);
        // 0.3 * 2.0 = 0.6 (within [0,1], no clamp needed)
        assert!((r.apply(0.3) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn param_retarget_clamps_above_one() {
        let r = ParamRetarget::scaled(3.0);
        // 0.8 * 3.0 = 2.4, clamped to 1.0
        assert!((r.apply(0.8) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn param_retarget_clamps_below_zero() {
        let r = ParamRetarget {
            scale: 1.0,
            offset: -0.5,
            clamp: true,
        };
        // 0.2 * 1.0 - 0.5 = -0.3, clamped to 0.0
        assert!((r.apply(0.2) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn anim_retarget_config_identity_preserves_params() {
        let config = AnimRetargetConfig::identity();
        let state = make_params(0.1, 0.2, 0.3, 0.4);
        let result = config.apply(&state);
        assert!((result.height - 0.1).abs() < 1e-6);
        assert!((result.weight - 0.2).abs() < 1e-6);
        assert!((result.muscle - 0.3).abs() < 1e-6);
        assert!((result.age - 0.4).abs() < 1e-6);
    }

    #[test]
    fn anim_retarget_config_scale_weight() {
        let config = AnimRetargetConfig {
            height: ParamRetarget::identity(),
            weight: ParamRetarget::scaled(0.5),
            muscle: ParamRetarget::identity(),
            age: ParamRetarget::identity(),
        };
        let state = make_params(0.4, 0.8, 0.6, 0.5);
        let result = config.apply(&state);
        assert!((result.height - 0.4).abs() < 1e-6);
        // 0.8 * 0.5 = 0.4
        assert!((result.weight - 0.4).abs() < 1e-6);
    }

    #[test]
    fn retarget_keyframe_applies_config() {
        let config = AnimRetargetConfig {
            height: ParamRetarget::scaled(0.5),
            weight: ParamRetarget::identity(),
            muscle: ParamRetarget::identity(),
            age: ParamRetarget::identity(),
        };
        let kf = Keyframe::new(2.5, make_params(1.0, 0.5, 0.5, 0.5));
        let result = retarget_keyframe(&kf, &config);
        assert!((result.time - 2.5).abs() < 1e-6);
        // 1.0 * 0.5 = 0.5
        assert!((result.params.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn retarget_track_preserves_length() {
        let track = make_track_two_kf();
        let config = AnimRetargetConfig::identity();
        let result = retarget_track(&track, &config);
        assert_eq!(result.len(), track.len());
    }

    #[test]
    fn scale_track_time_doubles_durations() {
        let track = make_track_two_kf();
        let original_duration = track.duration();
        let scaled = scale_track_time(&track, 2.0);
        assert!((scaled.duration() - original_duration * 2.0).abs() < 1e-5);
    }

    #[test]
    fn trim_track_removes_outside_keyframes() {
        let mut track = MorphTrack::new("trim_test");
        track.add_keyframe(Keyframe::new(0.0, make_params(0.1, 0.1, 0.1, 0.1)));
        track.add_keyframe(Keyframe::new(1.0, make_params(0.2, 0.2, 0.2, 0.2)));
        track.add_keyframe(Keyframe::new(2.0, make_params(0.3, 0.3, 0.3, 0.3)));
        track.add_keyframe(Keyframe::new(3.0, make_params(0.4, 0.4, 0.4, 0.4)));
        let trimmed = trim_track(&track, 1.0, 2.0);
        assert_eq!(trimmed.len(), 2);
    }

    #[test]
    fn trim_track_keeps_inside_keyframes() {
        let mut track = MorphTrack::new("trim_keep");
        track.add_keyframe(Keyframe::new(0.5, make_params(0.1, 0.1, 0.1, 0.1)));
        track.add_keyframe(Keyframe::new(1.5, make_params(0.5, 0.5, 0.5, 0.5)));
        track.add_keyframe(Keyframe::new(2.5, make_params(0.9, 0.9, 0.9, 0.9)));
        let trimmed = trim_track(&track, 0.0, 3.0);
        assert_eq!(trimmed.len(), 3);
    }

    #[test]
    fn reverse_track_flips_order() {
        let track = make_track_two_kf();
        let reversed = reverse_track(&track);
        assert_eq!(reversed.len(), 2);
        // Original: kf at 0.0 and 1.0; reversed: kf at 1.0-0.0=1.0 and 1.0-1.0=0.0
        // After sorting by time: first is 0.0, second is 1.0
        // The params at time=0.0 in reversed should be the params that were at time=1.0 in original
        let original_last = track
            .keyframes_iter()
            .last()
            .expect("should succeed")
            .params
            .clone();
        let reversed_first = reversed
            .keyframes_iter()
            .next()
            .expect("should succeed")
            .params
            .clone();
        assert!((original_last.height - reversed_first.height).abs() < 1e-6);
    }

    #[test]
    fn concat_tracks_total_length() {
        let first = make_track_two_kf();
        let second = make_track_two_kf();
        let combined = concat_tracks(&first, &second);
        assert_eq!(combined.len(), 4);
    }

    #[test]
    fn concat_tracks_second_offset_correctly() {
        let first = make_track_two_kf(); // times: 0.0, 1.0
        let mut second = MorphTrack::new("second");
        second.add_keyframe(Keyframe::new(0.0, make_params(0.1, 0.1, 0.1, 0.1)));
        second.add_keyframe(Keyframe::new(0.5, make_params(0.9, 0.9, 0.9, 0.9)));
        let combined = concat_tracks(&first, &second);
        // second's kf at 0.0 + offset(1.0) = 1.0
        // second's kf at 0.5 + offset(1.0) = 1.5
        // But first already has a kf at 1.0, so combined should have 4 keyframes
        // The last kf should be at time 1.5
        let last_kf = combined.keyframes_iter().last().expect("should succeed");
        assert!((last_kf.time - 1.5).abs() < 1e-6);
    }

    // ── BVH bridge tests ──────────────────────────────────────────────────

    /// Minimal valid two-joint, two-frame BVH string shared by bridge tests.
    fn minimal_bvh_bridge() -> &'static str {
        "HIERARCHY
ROOT Hips
{
  OFFSET 0.00 0.00 0.00
  CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
  JOINT Spine
  {
    OFFSET 0.00 5.21 0.00
    CHANNELS 3 Zrotation Xrotation Yrotation
    End Site
    {
      OFFSET 0.00 5.00 0.00
    }
  }
}
MOTION
Frames: 2
Frame Time: 0.033333
0.00 94.26 0.00 0.00 0.00 0.00 0.00 0.00 0.00
0.00 94.26 0.00 10.00 5.00 0.00 5.00 0.00 0.00
"
    }

    // Test 1: parse minimal BVH text — no error expected
    #[test]
    fn bvh_bridge_parse_no_error() {
        let result = parse_bvh_text(minimal_bvh_bridge());
        assert!(result.is_ok(), "parse_bvh_text returned Err: {:?}", result);
    }

    // Test 2: joint count per frame matches the number of skeleton joints with channels
    #[test]
    fn bvh_bridge_joint_count_per_frame() {
        let bvh = parse_bvh_text(minimal_bvh_bridge()).expect("parse failed");
        // Hips (6 channels) + Spine (3 channels) = 2 joints with channels
        for frame in &bvh.frames {
            assert_eq!(
                frame.len(),
                2,
                "expected 2 joints per frame, got {}",
                frame.len()
            );
        }
    }

    // Test 3: default_cmu() mapping has "Hips" as a key
    #[test]
    fn bvh_bridge_default_cmu_has_hips() {
        let mapping = SkeletonMapping::default_cmu();
        assert!(
            mapping.map.contains_key("Hips"),
            "SkeletonMapping::default_cmu() must contain 'Hips'"
        );
    }

    // Test 4: retarget_bvh_to_param_tracks produces nonempty tracks
    #[test]
    fn bvh_bridge_retarget_nonempty_tracks() {
        let bvh = parse_bvh_text(minimal_bvh_bridge()).expect("parse failed");
        let mapping = SkeletonMapping::default_cmu();
        let tracks = retarget_bvh_to_param_tracks(&bvh, &mapping);
        // "Hips" → "pelvis" is in the mapping; Hips is in BVH → track must be present
        assert!(
            !tracks.is_empty(),
            "retarget_bvh_to_param_tracks must produce at least one track"
        );
    }

    // Test 5: each track Vec has length equal to the frame count
    #[test]
    fn bvh_bridge_track_length_equals_frame_count() {
        let bvh = parse_bvh_text(minimal_bvh_bridge()).expect("parse failed");
        let frame_count = bvh.frames.len();
        let mapping = SkeletonMapping::default_cmu();
        let tracks = retarget_bvh_to_param_tracks(&bvh, &mapping);
        for (name, track) in &tracks {
            assert_eq!(
                track.len(),
                frame_count,
                "track '{}' has {} entries, expected {}",
                name,
                track.len(),
                frame_count
            );
        }
    }
}
