// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! BVH (Biovision Hierarchy) motion-capture format export.
//!
//! Produces `.bvh` text files containing a joint hierarchy and frame-by-frame
//! channel data, following the original Biovision spec.

#![allow(dead_code)]

// ── Channel enum ─────────────────────────────────────────────────────────────

/// A single degree-of-freedom channel within a BVH joint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BvhChannel {
    /// X-axis translation (cm).
    Xposition,
    /// Y-axis translation (cm).
    Yposition,
    /// Z-axis translation (cm).
    Zposition,
    /// X-axis (pitch) rotation in degrees.
    Xrotation,
    /// Y-axis (yaw) rotation in degrees.
    Yrotation,
    /// Z-axis (roll) rotation in degrees.
    Zrotation,
}

impl BvhChannel {
    /// Return the canonical BVH channel name string.
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            BvhChannel::Xposition => "Xposition",
            BvhChannel::Yposition => "Yposition",
            BvhChannel::Zposition => "Zposition",
            BvhChannel::Xrotation => "Xrotation",
            BvhChannel::Yrotation => "Yrotation",
            BvhChannel::Zrotation => "Zrotation",
        }
    }
}

// ── BvhJoint ─────────────────────────────────────────────────────────────────

/// A single joint in the BVH skeleton hierarchy.
#[derive(Debug, Clone)]
pub struct BvhJoint {
    /// Joint name (must be unique within the hierarchy).
    pub name: String,
    /// Offset from parent joint in local space (x, y, z in cm).
    pub offset: [f32; 3],
    /// Channels declared for this joint.
    pub channels: Vec<BvhChannel>,
    /// Indices into the flat joint list for direct children.
    pub children: Vec<usize>,
}

// ── BvhMotion ────────────────────────────────────────────────────────────────

/// A single frame of BVH motion data.
///
/// Channel values are stored in the same order as the flattened channel list
/// produced by a depth-first traversal of the joint hierarchy.
#[derive(Debug, Clone)]
pub struct BvhMotion {
    /// Flat channel values for every joint, in hierarchy DFS order.
    pub frame_data: Vec<f32>,
}

// ── BvhExportConfig ──────────────────────────────────────────────────────────

/// Configuration for BVH export.
#[derive(Debug, Clone)]
pub struct BvhExportConfig {
    /// Frames per second.  Default: 30.0.
    pub fps: f32,
    /// Number of decimal places for floating-point channel values.
    pub float_precision: usize,
    /// Indentation string for the hierarchy section.  Default: `"\t"`.
    pub indent: String,
}

impl Default for BvhExportConfig {
    fn default() -> Self {
        BvhExportConfig {
            fps: 30.0,
            float_precision: 6,
            indent: "\t".to_string(),
        }
    }
}

// ── Type aliases ─────────────────────────────────────────────────────────────

/// A flat list of joints representing a skeleton hierarchy.
pub type JointList = Vec<BvhJoint>;

/// A sequence of motion frames.
pub type FrameList = Vec<BvhMotion>;

/// Result of `validate_bvh`: `Ok(())` or `Err(description)`.
pub type BvhValidationResult = Result<(), String>;

// ── Constructor functions ─────────────────────────────────────────────────────

/// Create a `BvhExportConfig` with default settings (30 fps, tab indent).
#[allow(dead_code)]
pub fn default_bvh_config() -> BvhExportConfig {
    BvhExportConfig::default()
}

/// Create a new `BvhJoint` with the given name and local offset.
///
/// No channels or children are attached; add them after construction.
#[allow(dead_code)]
pub fn new_bvh_joint(name: &str, offset: [f32; 3]) -> BvhJoint {
    BvhJoint {
        name: name.to_string(),
        offset,
        channels: Vec::new(),
        children: Vec::new(),
    }
}

// ── Hierarchy helpers ─────────────────────────────────────────────────────────

/// Append `child_idx` as a child of the joint at `parent_idx` in `joints`.
///
/// Returns `false` if either index is out of range or `child_idx == parent_idx`.
#[allow(dead_code)]
pub fn add_joint(joints: &mut JointList, parent_idx: usize, child_idx: usize) -> bool {
    if parent_idx >= joints.len() || child_idx >= joints.len() || parent_idx == child_idx {
        return false;
    }
    joints[parent_idx].children.push(child_idx);
    true
}

/// Count the total number of joints.
#[allow(dead_code)]
pub fn joint_count(joints: &JointList) -> usize {
    joints.len()
}

// ── Motion helpers ────────────────────────────────────────────────────────────

/// Append a motion frame to the frame list.
#[allow(dead_code)]
pub fn add_motion_frame(frames: &mut FrameList, frame: BvhMotion) {
    frames.push(frame);
}

/// Return the number of recorded frames.
#[allow(dead_code)]
pub fn frame_count(frames: &FrameList) -> usize {
    frames.len()
}

/// Compute the total animation duration in seconds.
///
/// Returns 0.0 if either `frames` is empty or `fps` ≤ 0.
#[allow(dead_code)]
pub fn bvh_duration(frames: &FrameList, fps: f32) -> f32 {
    if frames.is_empty() || fps <= 0.0 {
        return 0.0;
    }
    frames.len() as f32 / fps
}

/// Return the configured FPS from a config.
#[allow(dead_code)]
pub fn bvh_fps(cfg: &BvhExportConfig) -> f32 {
    cfg.fps
}

/// Set the FPS on a config, clamping to a minimum of 1.0.
#[allow(dead_code)]
pub fn set_bvh_fps(cfg: &mut BvhExportConfig, fps: f32) {
    cfg.fps = fps.max(1.0);
}

// ── Channel count helper ──────────────────────────────────────────────────────

/// Count the total number of channels across all joints (DFS order).
#[allow(dead_code)]
pub fn total_channel_count(joints: &JointList) -> usize {
    joints.iter().map(|j| j.channels.len()).sum()
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate that a BVH skeleton + motion data are self-consistent.
///
/// Checks:
/// - At least one joint exists.
/// - Every frame has exactly as many values as total channels.
/// - FPS is positive.
#[allow(dead_code)]
pub fn validate_bvh(
    joints: &JointList,
    frames: &FrameList,
    cfg: &BvhExportConfig,
) -> BvhValidationResult {
    if joints.is_empty() {
        return Err("BVH has no joints".to_string());
    }
    if cfg.fps <= 0.0 {
        return Err(format!("BVH fps must be positive, got {}", cfg.fps));
    }
    let expected_channels = total_channel_count(joints);
    for (i, frame) in frames.iter().enumerate() {
        if frame.frame_data.len() != expected_channels {
            return Err(format!(
                "Frame {} has {} values, expected {}",
                i,
                frame.frame_data.len(),
                expected_channels
            ));
        }
    }
    Ok(())
}

// ── File-size estimate ────────────────────────────────────────────────────────

/// Estimate the output file size in bytes for a BVH export.
///
/// Uses a conservative heuristic: ~12 bytes per channel value per frame,
/// plus ~40 bytes per joint in the hierarchy section.
#[allow(dead_code)]
pub fn bvh_file_size_estimate(
    joints: &JointList,
    frames: &FrameList,
    _cfg: &BvhExportConfig,
) -> usize {
    let channel_total = total_channel_count(joints);
    let motion_bytes = channel_total * frames.len() * 12;
    let hierarchy_bytes = joints.len() * 40;
    motion_bytes + hierarchy_bytes + 256 // header overhead
}

// ── Serialiser ────────────────────────────────────────────────────────────────

/// Recursively write a joint block into `out` (depth-first).
fn write_joint_block(
    joints: &JointList,
    idx: usize,
    depth: usize,
    indent: &str,
    is_end_site: bool,
    out: &mut String,
) {
    let pad = indent.repeat(depth);
    let pad1 = indent.repeat(depth + 1);
    let pad2 = indent.repeat(depth + 2);

    let j = &joints[idx];

    if is_end_site {
        out.push_str(&format!("{}End Site\n", pad));
        out.push_str(&format!("{}{{\n", pad));
        out.push_str(&format!(
            "{}OFFSET {:.6} {:.6} {:.6}\n",
            pad1, j.offset[0], j.offset[1], j.offset[2]
        ));
        out.push_str(&format!("{}}}\n", pad));
        return;
    }

    let keyword = if depth == 0 { "ROOT" } else { "JOINT" };
    out.push_str(&format!("{}{} {}\n", pad, keyword, j.name));
    out.push_str(&format!("{}{{\n", pad));
    out.push_str(&format!(
        "{}OFFSET {:.6} {:.6} {:.6}\n",
        pad1, j.offset[0], j.offset[1], j.offset[2]
    ));

    if !j.channels.is_empty() {
        let names: Vec<&str> = j.channels.iter().map(BvhChannel::name).collect();
        out.push_str(&format!(
            "{}CHANNELS {} {}\n",
            pad1,
            j.channels.len(),
            names.join(" ")
        ));
    }

    if j.children.is_empty() {
        // Emit a synthetic end-site at depth+1
        let end = BvhJoint {
            name: String::new(),
            offset: [0.0, 0.0, 0.0],
            channels: Vec::new(),
            children: Vec::new(),
        };
        let tmp_idx = joints.len(); // out-of-range sentinel
        let _ = tmp_idx;
        // Inline end-site
        out.push_str(&format!("{}End Site\n", pad1));
        out.push_str(&format!("{}{{\n", pad1));
        out.push_str(&format!("{}OFFSET 0.000000 0.000000 0.000000\n", pad2));
        out.push_str(&format!("{}}}\n", pad1));
        let _ = end;
    } else {
        for &child_idx in &j.children {
            write_joint_block(joints, child_idx, depth + 1, indent, false, out);
        }
    }

    out.push_str(&format!("{}}}\n", pad));
}

/// Serialise a BVH skeleton and motion data to a BVH-format string.
///
/// Joints are written depth-first starting from index 0 as the root.
/// Returns an empty string if `joints` is empty.
#[allow(dead_code)]
pub fn bvh_to_string(
    joints: &JointList,
    frames: &FrameList,
    cfg: &BvhExportConfig,
) -> String {
    if joints.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(256 + frames.len() * 64);

    // Hierarchy section
    out.push_str("HIERARCHY\n");
    write_joint_block(joints, 0, 0, &cfg.indent, false, &mut out);

    // Motion section
    out.push_str("MOTION\n");
    out.push_str(&format!("Frames: {}\n", frames.len()));
    let frame_time = if cfg.fps > 0.0 { 1.0 / cfg.fps } else { 1.0 };
    out.push_str(&format!("Frame Time: {:.6}\n", frame_time));

    let prec = cfg.float_precision;
    for frame in frames {
        let values: Vec<String> = frame
            .frame_data
            .iter()
            .map(|v| format!("{:.prec$}", v, prec = prec))
            .collect();
        out.push_str(&values.join(" "));
        out.push('\n');
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_root_joint() -> BvhJoint {
        BvhJoint {
            name: "Hips".to_string(),
            offset: [0.0, 0.0, 0.0],
            channels: vec![
                BvhChannel::Xposition,
                BvhChannel::Yposition,
                BvhChannel::Zposition,
                BvhChannel::Zrotation,
                BvhChannel::Xrotation,
                BvhChannel::Yrotation,
            ],
            children: vec![],
        }
    }

    fn single_joint_system() -> (JointList, FrameList, BvhExportConfig) {
        let joints = vec![make_root_joint()];
        let frames = vec![BvhMotion {
            frame_data: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }];
        let cfg = default_bvh_config();
        (joints, frames, cfg)
    }

    // 1 – default_bvh_config
    #[test]
    fn test_default_bvh_config_fps() {
        let cfg = default_bvh_config();
        assert!((cfg.fps - 30.0).abs() < 1e-6);
    }

    // 2 – new_bvh_joint
    #[test]
    fn test_new_bvh_joint_name() {
        let j = new_bvh_joint("Spine", [0.0, 10.0, 0.0]);
        assert_eq!(j.name, "Spine");
        assert_eq!(j.offset, [0.0, 10.0, 0.0]);
        assert!(j.channels.is_empty());
        assert!(j.children.is_empty());
    }

    // 3 – add_joint success
    #[test]
    fn test_add_joint_success() {
        let mut joints = vec![
            new_bvh_joint("Root", [0.0, 0.0, 0.0]),
            new_bvh_joint("Spine", [0.0, 5.0, 0.0]),
        ];
        assert!(add_joint(&mut joints, 0, 1));
        assert_eq!(joints[0].children, vec![1]);
    }

    // 4 – add_joint out of range
    #[test]
    fn test_add_joint_out_of_range() {
        let mut joints = vec![new_bvh_joint("Root", [0.0, 0.0, 0.0])];
        assert!(!add_joint(&mut joints, 0, 5));
    }

    // 5 – add_joint self-link rejected
    #[test]
    fn test_add_joint_self_link() {
        let mut joints = vec![new_bvh_joint("Root", [0.0, 0.0, 0.0])];
        assert!(!add_joint(&mut joints, 0, 0));
    }

    // 6 – joint_count
    #[test]
    fn test_joint_count() {
        let joints: JointList = vec![
            new_bvh_joint("A", [0.0; 3]),
            new_bvh_joint("B", [0.0; 3]),
            new_bvh_joint("C", [0.0; 3]),
        ];
        assert_eq!(joint_count(&joints), 3);
    }

    // 7 – add_motion_frame / frame_count
    #[test]
    fn test_add_motion_frame() {
        let mut frames: FrameList = Vec::new();
        add_motion_frame(&mut frames, BvhMotion { frame_data: vec![1.0, 2.0] });
        add_motion_frame(&mut frames, BvhMotion { frame_data: vec![3.0, 4.0] });
        assert_eq!(frame_count(&frames), 2);
    }

    // 8 – bvh_fps / set_bvh_fps
    #[test]
    fn test_set_bvh_fps() {
        let mut cfg = default_bvh_config();
        set_bvh_fps(&mut cfg, 60.0);
        assert!((bvh_fps(&cfg) - 60.0).abs() < 1e-6);
    }

    // 9 – set_bvh_fps clamps to minimum 1
    #[test]
    fn test_set_bvh_fps_clamp() {
        let mut cfg = default_bvh_config();
        set_bvh_fps(&mut cfg, -5.0);
        assert!((bvh_fps(&cfg) - 1.0).abs() < 1e-6);
    }

    // 10 – bvh_duration
    #[test]
    fn test_bvh_duration() {
        let frames: FrameList = vec![
            BvhMotion { frame_data: vec![] },
            BvhMotion { frame_data: vec![] },
            BvhMotion { frame_data: vec![] },
        ];
        let d = bvh_duration(&frames, 30.0);
        assert!((d - 0.1).abs() < 1e-5, "3 frames @ 30fps = 0.1s, got {d}");
    }

    // 11 – bvh_duration empty frames
    #[test]
    fn test_bvh_duration_empty() {
        let frames: FrameList = Vec::new();
        assert!((bvh_duration(&frames, 30.0)).abs() < 1e-6);
    }

    // 12 – validate_bvh ok
    #[test]
    fn test_validate_bvh_ok() {
        let (joints, frames, cfg) = single_joint_system();
        assert!(validate_bvh(&joints, &frames, &cfg).is_ok());
    }

    // 13 – validate_bvh wrong channel count
    #[test]
    fn test_validate_bvh_wrong_channels() {
        let (joints, _, cfg) = single_joint_system();
        let bad_frames = vec![BvhMotion {
            frame_data: vec![1.0, 2.0], // only 2 values, need 6
        }];
        assert!(validate_bvh(&joints, &bad_frames, &cfg).is_err());
    }

    // 14 – validate_bvh no joints
    #[test]
    fn test_validate_bvh_no_joints() {
        let joints: JointList = Vec::new();
        let frames: FrameList = Vec::new();
        let cfg = default_bvh_config();
        assert!(validate_bvh(&joints, &frames, &cfg).is_err());
    }

    // 15 – bvh_to_string contains HIERARCHY and MOTION
    #[test]
    fn test_bvh_to_string_structure() {
        let (joints, frames, cfg) = single_joint_system();
        let s = bvh_to_string(&joints, &frames, &cfg);
        assert!(s.contains("HIERARCHY"), "missing HIERARCHY keyword");
        assert!(s.contains("ROOT Hips"), "missing ROOT joint name");
        assert!(s.contains("MOTION"), "missing MOTION keyword");
        assert!(s.contains("Frames: 1"), "wrong frame count");
    }

    // 16 – bvh_to_string channels listed
    #[test]
    fn test_bvh_to_string_channels() {
        let (joints, frames, cfg) = single_joint_system();
        let s = bvh_to_string(&joints, &frames, &cfg);
        assert!(s.contains("CHANNELS 6"), "expected 6 channels declaration");
        assert!(s.contains("Xposition"), "missing Xposition channel");
        assert!(s.contains("Yrotation"), "missing Yrotation channel");
    }

    // 17 – bvh_to_string frame time matches fps
    #[test]
    fn test_bvh_to_string_frame_time() {
        let (joints, frames, cfg) = single_joint_system();
        let s = bvh_to_string(&joints, &frames, &cfg);
        // 1/30 = 0.033333
        assert!(
            s.contains("Frame Time: 0.033333"),
            "wrong frame time, string:\n{s}"
        );
    }

    // 18 – bvh_to_string empty joints returns empty
    #[test]
    fn test_bvh_to_string_empty_joints() {
        let joints: JointList = Vec::new();
        let frames: FrameList = Vec::new();
        let cfg = default_bvh_config();
        let s = bvh_to_string(&joints, &frames, &cfg);
        assert!(s.is_empty());
    }

    // 19 – bvh_file_size_estimate is positive
    #[test]
    fn test_bvh_file_size_estimate() {
        let (joints, frames, cfg) = single_joint_system();
        let size = bvh_file_size_estimate(&joints, &frames, &cfg);
        assert!(size > 0, "size estimate must be positive");
    }

    // 20 – BvhChannel::name returns correct strings
    #[test]
    fn test_bvh_channel_names() {
        assert_eq!(BvhChannel::Xposition.name(), "Xposition");
        assert_eq!(BvhChannel::Yrotation.name(), "Yrotation");
        assert_eq!(BvhChannel::Zrotation.name(), "Zrotation");
    }

    // 21 – two-joint hierarchy serialises JOINT keyword
    #[test]
    fn test_bvh_to_string_two_joints() {
        let mut joints: JointList = vec![
            BvhJoint {
                name: "Root".to_string(),
                offset: [0.0; 3],
                channels: vec![
                    BvhChannel::Xposition,
                    BvhChannel::Yposition,
                    BvhChannel::Zposition,
                ],
                children: vec![1],
            },
            BvhJoint {
                name: "Child".to_string(),
                offset: [0.0, 10.0, 0.0],
                channels: vec![BvhChannel::Zrotation, BvhChannel::Xrotation, BvhChannel::Yrotation],
                children: vec![],
            },
        ];
        let _ = add_joint(&mut joints, 0, 1); // already set via children above
        let frames = vec![BvhMotion {
            frame_data: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }];
        let cfg = default_bvh_config();
        let s = bvh_to_string(&joints, &frames, &cfg);
        assert!(s.contains("ROOT Root"), "missing ROOT");
        assert!(s.contains("JOINT Child"), "missing JOINT keyword");
    }

    // 22 – total_channel_count
    #[test]
    fn test_total_channel_count() {
        let (joints, _, _) = single_joint_system();
        assert_eq!(total_channel_count(&joints), 6);
    }

    // 23 – BvhChannel PartialEq
    #[test]
    fn test_bvh_channel_eq() {
        assert_eq!(BvhChannel::Xposition, BvhChannel::Xposition);
        assert_ne!(BvhChannel::Xposition, BvhChannel::Yposition);
    }
}
