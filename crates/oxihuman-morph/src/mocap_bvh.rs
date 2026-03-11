// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! BVH (Biovision Hierarchy) motion capture file parser and skeleton mapper.
//!
//! Supports parsing the HIERARCHY and MOTION sections of BVH files,
//! skeleton traversal, frame interpolation, and mapping to OxiHuman joint names.

use std::collections::HashMap;

// ── Channel type ─────────────────────────────────────────────────────────────

/// A single channel type in a BVH joint definition.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BvhChannel {
    Xposition,
    Yposition,
    Zposition,
    Xrotation,
    Yrotation,
    Zrotation,
}

impl BvhChannel {
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Xposition" => Ok(BvhChannel::Xposition),
            "Yposition" => Ok(BvhChannel::Yposition),
            "Zposition" => Ok(BvhChannel::Zposition),
            "Xrotation" => Ok(BvhChannel::Xrotation),
            "Yrotation" => Ok(BvhChannel::Yrotation),
            "Zrotation" => Ok(BvhChannel::Zrotation),
            _ => Err(format!("Unknown BVH channel: '{s}'")),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            BvhChannel::Xposition => "Xposition",
            BvhChannel::Yposition => "Yposition",
            BvhChannel::Zposition => "Zposition",
            BvhChannel::Xrotation => "Xrotation",
            BvhChannel::Yrotation => "Yrotation",
            BvhChannel::Zrotation => "Zrotation",
        }
    }

    /// Returns true if this channel is a translation channel.
    #[allow(dead_code)]
    pub fn is_translation(&self) -> bool {
        matches!(
            self,
            BvhChannel::Xposition | BvhChannel::Yposition | BvhChannel::Zposition
        )
    }

    /// Returns true if this channel is a rotation channel.
    #[allow(dead_code)]
    pub fn is_rotation(&self) -> bool {
        matches!(
            self,
            BvhChannel::Xrotation | BvhChannel::Yrotation | BvhChannel::Zrotation
        )
    }
}

// ── Joint ─────────────────────────────────────────────────────────────────────

/// A single joint in the BVH hierarchy.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhJoint {
    /// Joint name.
    pub name: String,
    /// Rest offset from parent joint (in BVH units).
    pub offset: [f32; 3],
    /// Ordered list of channels for this joint.
    pub channels: Vec<BvhChannel>,
    /// Indices of child joints into `BvhSkeleton.joints`.
    pub children: Vec<usize>,
    /// Index of parent joint, or `None` for the root.
    pub parent: Option<usize>,
}

impl BvhJoint {
    fn new(name: String, offset: [f32; 3], parent: Option<usize>) -> Self {
        Self {
            name,
            offset,
            channels: Vec::new(),
            children: Vec::new(),
            parent,
        }
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

/// The full skeleton hierarchy read from a BVH file.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhSkeleton {
    /// Flat array of all joints (including End Sites which have no channels).
    pub joints: Vec<BvhJoint>,
    /// Index of the root joint in `joints`.
    pub root_index: usize,
}

impl BvhSkeleton {
    /// Returns the total number of joints (including end-sites).
    #[allow(dead_code)]
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// Finds a joint by name and returns its index, or `None`.
    #[allow(dead_code)]
    pub fn find_joint(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    /// Returns the total number of channels across all joints.
    #[allow(dead_code)]
    pub fn channel_count(&self) -> usize {
        self.joints.iter().map(|j| j.channels.len()).sum()
    }

    /// Returns an ordered slice of joint names.
    #[allow(dead_code)]
    pub fn joint_names(&self) -> Vec<&str> {
        self.joints.iter().map(|j| j.name.as_str()).collect()
    }

    /// Returns the parent index of `joint_idx`, or `None` for the root.
    #[allow(dead_code)]
    pub fn parent_of(&self, joint_idx: usize) -> Option<usize> {
        self.joints.get(joint_idx)?.parent
    }

    /// Returns the children indices of `joint_idx`.
    #[allow(dead_code)]
    pub fn children_of(&self, joint_idx: usize) -> &[usize] {
        self.joints
            .get(joint_idx)
            .map(|j| j.children.as_slice())
            .unwrap_or(&[])
    }

    /// Compute the channel offset (into a frame's flat value array) for joint at `joint_idx`.
    #[allow(dead_code)]
    pub fn channel_offset_for(&self, joint_idx: usize) -> usize {
        self.joints[..joint_idx]
            .iter()
            .map(|j| j.channels.len())
            .sum()
    }
}

// ── Frame ─────────────────────────────────────────────────────────────────────

/// One frame of motion data: flat channel values in joint-declaration order.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhFrame {
    /// All channel values in the same order as joint/channel declarations.
    pub values: Vec<f32>,
}

impl BvhFrame {
    /// Returns the channel slice for the given joint starting at `joint_channel_offset`.
    #[allow(dead_code)]
    pub fn get_channels(&self, joint: &BvhJoint, joint_channel_offset: usize) -> &[f32] {
        let len = joint.channels.len();
        let end = (joint_channel_offset + len).min(self.values.len());
        &self.values[joint_channel_offset..end]
    }
}

// ── BvhFile ───────────────────────────────────────────────────────────────────

/// A fully parsed BVH file.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhFile {
    /// Parsed skeleton hierarchy.
    pub skeleton: BvhSkeleton,
    /// All motion frames.
    pub frames: Vec<BvhFrame>,
    /// Duration of a single frame in seconds.
    pub frame_time: f32,
}

impl BvhFile {
    /// Number of frames in the file.
    #[allow(dead_code)]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Total duration in seconds.
    #[allow(dead_code)]
    pub fn duration_seconds(&self) -> f32 {
        self.frames.len() as f32 * self.frame_time
    }

    /// Frames per second.
    #[allow(dead_code)]
    pub fn fps(&self) -> f32 {
        if self.frame_time > 0.0 {
            1.0 / self.frame_time
        } else {
            0.0
        }
    }

    /// Returns the root translation [x, y, z] at frame `frame`.
    ///
    /// The root joint must have Xposition, Yposition, Zposition as its first three channels
    /// (the BVH standard for a 6-DOF root).  If the root has no translation channels the
    /// function returns `[0.0, 0.0, 0.0]`.
    #[allow(dead_code)]
    pub fn root_translation(&self, frame: usize) -> [f32; 3] {
        let root = &self.skeleton.joints[self.skeleton.root_index];
        let frame_data = &self.frames[frame];
        let mut tx = 0.0_f32;
        let mut ty = 0.0_f32;
        let mut tz = 0.0_f32;
        for (offset, ch) in root.channels.iter().enumerate() {
            let v = frame_data.values.get(offset).copied().unwrap_or(0.0);
            match ch {
                BvhChannel::Xposition => tx = v,
                BvhChannel::Yposition => ty = v,
                BvhChannel::Zposition => tz = v,
                _ => {}
            }
        }
        [tx, ty, tz]
    }

    /// Returns the Euler rotation (degrees) [x, y, z] for joint `joint_idx` at `frame`.
    ///
    /// Only the rotation channels of the joint are inspected; translation channels are skipped.
    #[allow(dead_code)]
    pub fn joint_rotation(&self, frame: usize, joint_idx: usize) -> [f32; 3] {
        let joint = &self.skeleton.joints[joint_idx];
        let ch_offset = self.skeleton.channel_offset_for(joint_idx);
        let frame_data = &self.frames[frame];
        let mut rx = 0.0_f32;
        let mut ry = 0.0_f32;
        let mut rz = 0.0_f32;
        for (i, ch) in joint.channels.iter().enumerate() {
            let v = frame_data.values.get(ch_offset + i).copied().unwrap_or(0.0);
            match ch {
                BvhChannel::Xrotation => rx = v,
                BvhChannel::Yrotation => ry = v,
                BvhChannel::Zrotation => rz = v,
                _ => {}
            }
        }
        [rx, ry, rz]
    }

    /// Linearly interpolate between frames `frame_a` and `frame_b` by factor `t` ∈ [0, 1].
    #[allow(dead_code)]
    pub fn interpolate_frame(&self, frame_a: usize, frame_b: usize, t: f32) -> BvhFrame {
        let a = &self.frames[frame_a].values;
        let b = &self.frames[frame_b].values;
        let len = a.len().min(b.len());
        let values = (0..len).map(|i| a[i] + (b[i] - a[i]) * t).collect();
        BvhFrame { values }
    }

    /// Sample motion at `time_seconds` using linear interpolation between neighbouring frames.
    ///
    /// Times before the first frame return the first frame; times beyond the last frame return
    /// the last frame.
    #[allow(dead_code)]
    pub fn sample_at(&self, time_seconds: f32) -> BvhFrame {
        if self.frames.is_empty() {
            return BvhFrame { values: vec![] };
        }
        if self.frame_time <= 0.0 || self.frames.len() == 1 {
            return self.frames[0].clone();
        }
        let total = self.frames.len() - 1;
        let frame_f = (time_seconds / self.frame_time).max(0.0);
        let frame_a = (frame_f.floor() as usize).min(total);
        let frame_b = (frame_a + 1).min(total);
        let t = frame_f.fract();
        self.interpolate_frame(frame_a, frame_b, t)
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Parse a BVH file from its string content.
///
/// Returns a [`BvhFile`] on success or an error string on failure.
#[allow(dead_code)]
pub fn parse_bvh(content: &str) -> Result<BvhFile, String> {
    let mut lines = content.lines().peekable();

    // ── HIERARCHY section ──────────────────────────────────────────────────
    // Skip blank lines / whitespace until "HIERARCHY"
    loop {
        match lines.peek() {
            None => return Err("Unexpected end of file before HIERARCHY".into()),
            Some(l) => {
                if l.trim() == "HIERARCHY" {
                    lines.next();
                    break;
                }
                lines.next();
            }
        }
    }

    // State for hierarchical joint parsing
    let mut joints: Vec<BvhJoint> = Vec::new();
    // Stack of joint indices: top = current parent
    let mut parent_stack: Vec<usize> = Vec::new();
    let mut root_index: Option<usize> = None;
    // Depth of brace nesting inside End Site blocks we want to skip
    let mut end_site_depth: i32 = 0;
    let mut in_end_site = false;

    loop {
        let raw = match lines.next() {
            None => return Err("Unexpected end of file inside HIERARCHY".into()),
            Some(l) => l,
        };
        let line = raw.trim();

        if line == "MOTION" {
            break;
        }
        if line.is_empty() {
            continue;
        }

        // End Site handling — we only read its OFFSET and skip the rest
        if line.starts_with("End Site") {
            in_end_site = true;
            end_site_depth = 0;
            continue;
        }
        if in_end_site {
            if line == "{" {
                end_site_depth += 1;
            } else if line == "}" {
                end_site_depth -= 1;
                if end_site_depth == 0 {
                    in_end_site = false;
                    // Closing brace of End Site — also pop parent (End Site closes the joint block)
                    // Actually no: in BVH the End Site brace pair is *inside* the joint's braces.
                    // The joint's own closing brace will be handled below.
                }
            }
            // Skip all lines inside End Site (including OFFSET)
            continue;
        }

        if line == "{" {
            // Push the most recently added joint onto the parent stack
            if let Some(&last) = joints.last().map(|_| joints.len() - 1).as_ref() {
                parent_stack.push(last);
            }
        } else if line == "}" {
            parent_stack.pop();
        } else if let Some(rest) = line.strip_prefix("ROOT ") {
            let name = rest.trim().to_string();
            let idx = joints.len();
            joints.push(BvhJoint::new(name, [0.0; 3], None));
            root_index = Some(idx);
        } else if let Some(rest) = line.strip_prefix("JOINT ") {
            let name = rest.trim().to_string();
            let idx = joints.len();
            let parent = parent_stack.last().copied();
            joints.push(BvhJoint::new(name, [0.0; 3], parent));
            if let Some(p) = parent {
                joints[p].children.push(idx);
            }
        } else if let Some(rest) = line.strip_prefix("OFFSET ") {
            let nums: Vec<f32> = rest
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if nums.len() < 3 {
                return Err(format!("OFFSET line has fewer than 3 values: '{line}'"));
            }
            if let Some(&idx) = parent_stack.last() {
                joints[idx].offset = [nums[0], nums[1], nums[2]];
            }
        } else if let Some(rest) = line.strip_prefix("CHANNELS ") {
            let mut parts = rest.split_whitespace();
            let count: usize = parts
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| format!("Invalid CHANNELS count in: '{line}'"))?;
            let mut channels = Vec::with_capacity(count);
            for _ in 0..count {
                let ch_str = parts
                    .next()
                    .ok_or_else(|| format!("Not enough channel names in: '{line}'"))?;
                channels.push(BvhChannel::from_str(ch_str)?);
            }
            if let Some(&idx) = parent_stack.last() {
                joints[idx].channels = channels;
            }
        }
    }

    let root_index = root_index.ok_or("No ROOT joint found in HIERARCHY")?;

    // ── MOTION section ────────────────────────────────────────────────────
    // Expect "Frames: N"
    let frames_line =
        next_non_empty(&mut lines).ok_or("Missing 'Frames:' line in MOTION section")?;
    let frames_line = frames_line.trim();
    let frame_count: usize = frames_line
        .strip_prefix("Frames:")
        .or_else(|| frames_line.strip_prefix("Frames: "))
        .and_then(|s| s.trim().parse().ok())
        .ok_or_else(|| format!("Cannot parse frame count from: '{frames_line}'"))?;

    // Expect "Frame Time: T"
    let ftime_line =
        next_non_empty(&mut lines).ok_or("Missing 'Frame Time:' line in MOTION section")?;
    let ftime_line = ftime_line.trim();
    let frame_time: f32 = ftime_line
        .strip_prefix("Frame Time:")
        .or_else(|| ftime_line.strip_prefix("Frame Time: "))
        .and_then(|s| s.trim().parse().ok())
        .ok_or_else(|| format!("Cannot parse frame time from: '{ftime_line}'"))?;

    // Compute expected number of values per frame
    let channel_count: usize = joints.iter().map(|j| j.channels.len()).sum();

    // Parse frame data
    let mut frames: Vec<BvhFrame> = Vec::with_capacity(frame_count);
    let mut remaining = frame_count;

    // Collect remaining tokens (frames may span multiple lines)
    let mut token_buf: Vec<f32> = Vec::new();

    for raw in lines {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        for tok in line.split_whitespace() {
            match tok.parse::<f32>() {
                Ok(v) => token_buf.push(v),
                Err(_) => {
                    return Err(format!("Non-numeric token '{tok}' in MOTION data"));
                }
            }
            if channel_count > 0 && token_buf.len() == channel_count {
                frames.push(BvhFrame {
                    values: token_buf.clone(),
                });
                token_buf.clear();
                remaining = remaining.saturating_sub(1);
            }
        }
        if remaining == 0 {
            break;
        }
    }

    // If channel_count == 0 treat any leftover tokens as single empty frames
    if channel_count == 0 {
        for _ in 0..frame_count {
            frames.push(BvhFrame { values: vec![] });
        }
    }

    let skeleton = BvhSkeleton { joints, root_index };

    Ok(BvhFile {
        skeleton,
        frames,
        frame_time,
    })
}

/// Helper: advance an iterator skipping empty/blank lines, returning the next non-empty one.
fn next_non_empty<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<&'a str> {
    iter.find(|l| !l.trim().is_empty())
}

// ── Writer ────────────────────────────────────────────────────────────────────

/// Serialize a [`BvhFile`] back to BVH text (for round-trip testing).
#[allow(dead_code)]
pub fn write_bvh(file: &BvhFile) -> String {
    let mut out = String::new();
    out.push_str("HIERARCHY\n");

    // Recursive joint writer
    write_joint_recursive(&file.skeleton, file.skeleton.root_index, 0, &mut out);

    out.push_str("MOTION\n");
    out.push_str(&format!("Frames: {}\n", file.frames.len()));
    out.push_str(&format!("Frame Time: {:.6}\n", file.frame_time));

    for frame in &file.frames {
        let line: Vec<String> = frame.values.iter().map(|v| format!("{v:.6}")).collect();
        out.push_str(&line.join(" "));
        out.push('\n');
    }

    out
}

fn write_joint_recursive(skel: &BvhSkeleton, idx: usize, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    let joint = &skel.joints[idx];

    if depth == 0 {
        out.push_str(&format!("{indent}ROOT {}\n", joint.name));
    } else {
        out.push_str(&format!("{indent}JOINT {}\n", joint.name));
    }
    out.push_str(&format!("{indent}{{\n"));
    out.push_str(&format!(
        "{indent}  OFFSET {:.2} {:.2} {:.2}\n",
        joint.offset[0], joint.offset[1], joint.offset[2]
    ));
    if !joint.channels.is_empty() {
        let ch_names: Vec<&str> = joint.channels.iter().map(|c| c.as_str()).collect();
        out.push_str(&format!(
            "{indent}  CHANNELS {} {}\n",
            joint.channels.len(),
            ch_names.join(" ")
        ));
    }
    if joint.children.is_empty() {
        // End Site
        out.push_str(&format!("{indent}  End Site\n"));
        out.push_str(&format!("{indent}  {{\n"));
        out.push_str(&format!("{indent}    OFFSET 0.00 0.00 0.00\n"));
        out.push_str(&format!("{indent}  }}\n"));
    } else {
        for &child_idx in &joint.children {
            write_joint_recursive(skel, child_idx, depth + 1, out);
        }
    }
    out.push_str(&format!("{indent}}}\n"));
}

// ── Joint name mapping ────────────────────────────────────────────────────────

/// Map common BVH joint names to OxiHuman skeleton joint names.
///
/// Returns a `HashMap<bvh_name, oxihuman_name>`.  Unmapped joints are not
/// included in the result.
#[allow(dead_code)]
pub fn map_bvh_to_oxihuman(bvh: &BvhFile) -> HashMap<String, String> {
    // Canonical mapping table (BVH name → OxiHuman name)
    let table: &[(&str, &str)] = &[
        // ── Spine / torso ──────────────────────────────────────────────────
        ("Hips", "pelvis"),
        ("Spine", "spine_01"),
        ("Spine1", "spine_02"),
        ("Spine2", "spine_03"),
        ("Neck", "neck_01"),
        ("Neck1", "neck_02"),
        ("Head", "head"),
        // ── Left arm ───────────────────────────────────────────────────────
        ("LeftShoulder", "clavicle_l"),
        ("LeftArm", "upperarm_l"),
        ("LeftForeArm", "lowerarm_l"),
        ("LeftHand", "hand_l"),
        // ── Right arm ──────────────────────────────────────────────────────
        ("RightShoulder", "clavicle_r"),
        ("RightArm", "upperarm_r"),
        ("RightForeArm", "lowerarm_r"),
        ("RightHand", "hand_r"),
        // ── Left leg ───────────────────────────────────────────────────────
        ("LeftUpLeg", "thigh_l"),
        ("LeftLeg", "calf_l"),
        ("LeftFoot", "foot_l"),
        ("LeftToeBase", "ball_l"),
        // ── Right leg ──────────────────────────────────────────────────────
        ("RightUpLeg", "thigh_r"),
        ("RightLeg", "calf_r"),
        ("RightFoot", "foot_r"),
        ("RightToeBase", "ball_r"),
        // ── Left fingers ───────────────────────────────────────────────────
        ("LeftHandThumb1", "thumb_01_l"),
        ("LeftHandThumb2", "thumb_02_l"),
        ("LeftHandThumb3", "thumb_03_l"),
        ("LeftHandIndex1", "index_01_l"),
        ("LeftHandIndex2", "index_02_l"),
        ("LeftHandIndex3", "index_03_l"),
        ("LeftHandMiddle1", "middle_01_l"),
        ("LeftHandMiddle2", "middle_02_l"),
        ("LeftHandMiddle3", "middle_03_l"),
        ("LeftHandRing1", "ring_01_l"),
        ("LeftHandRing2", "ring_02_l"),
        ("LeftHandRing3", "ring_03_l"),
        ("LeftHandPinky1", "pinky_01_l"),
        ("LeftHandPinky2", "pinky_02_l"),
        ("LeftHandPinky3", "pinky_03_l"),
        // ── Right fingers ──────────────────────────────────────────────────
        ("RightHandThumb1", "thumb_01_r"),
        ("RightHandThumb2", "thumb_02_r"),
        ("RightHandThumb3", "thumb_03_r"),
        ("RightHandIndex1", "index_01_r"),
        ("RightHandIndex2", "index_02_r"),
        ("RightHandIndex3", "index_03_r"),
        ("RightHandMiddle1", "middle_01_r"),
        ("RightHandMiddle2", "middle_02_r"),
        ("RightHandMiddle3", "middle_03_r"),
        ("RightHandRing1", "ring_01_r"),
        ("RightHandRing2", "ring_02_r"),
        ("RightHandRing3", "ring_03_r"),
        ("RightHandPinky1", "pinky_01_r"),
        ("RightHandPinky2", "pinky_02_r"),
        ("RightHandPinky3", "pinky_03_r"),
    ];

    let lookup: HashMap<&str, &str> = table.iter().copied().collect();
    let mut result = HashMap::new();

    for joint in &bvh.skeleton.joints {
        if let Some(&oxi_name) = lookup.get(joint.name.as_str()) {
            result.insert(joint.name.clone(), oxi_name.to_string());
        }
    }

    result
}

// ── Retargeting ───────────────────────────────────────────────────────────────

/// Scale the translation channels in a frame by `scale`.
///
/// `translation_channels` is the number of leading channels in the frame
/// that are translation channels (typically 3 for a 6-DOF root).
/// Rotation channels are left unchanged.
#[allow(dead_code)]
pub fn retarget_scale(frame: &BvhFrame, scale: f32, translation_channels: usize) -> BvhFrame {
    let values = frame
        .values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            if i < translation_channels {
                v * scale
            } else {
                v
            }
        })
        .collect();
    BvhFrame { values }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal two-joint, two-frame BVH string used by many tests.
    fn minimal_bvh() -> &'static str {
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
0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
1.0 2.0 3.0 1.0 2.0 3.0 4.0 5.0 6.0
"
    }

    // ── Test 1: successful parse ──────────────────────────────────────────
    #[test]
    fn test_parse_minimal_bvh() {
        let bvh = parse_bvh(minimal_bvh()).expect("parse failed");
        assert_eq!(bvh.skeleton.joint_count(), 2);
        assert_eq!(bvh.frame_count(), 2);
    }

    // ── Test 2: joint names ───────────────────────────────────────────────
    #[test]
    fn test_joint_names() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let names = bvh.skeleton.joint_names();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine"));
    }

    // ── Test 3: channel count ─────────────────────────────────────────────
    #[test]
    fn test_channel_count() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        // Hips: 6  +  Spine: 3  = 9
        assert_eq!(bvh.skeleton.channel_count(), 9);
    }

    // ── Test 4: frame time and fps ────────────────────────────────────────
    #[test]
    fn test_fps() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let fps = bvh.fps();
        assert!((fps - 30.0).abs() < 0.5, "fps ≈ 30, got {fps}");
    }

    // ── Test 5: duration ──────────────────────────────────────────────────
    #[test]
    fn test_duration() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let dur = bvh.duration_seconds();
        assert!(dur > 0.0);
        assert!((dur - 2.0 * 0.033333_f32).abs() < 1e-4);
    }

    // ── Test 6: root translation ──────────────────────────────────────────
    #[test]
    fn test_root_translation_frame0() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let t = bvh.root_translation(0);
        assert_eq!(t, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_root_translation_frame1() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let t = bvh.root_translation(1);
        assert_eq!(t, [1.0, 2.0, 3.0]);
    }

    // ── Test 7: joint rotation ────────────────────────────────────────────
    #[test]
    fn test_joint_rotation_spine_frame1() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let spine_idx = bvh.skeleton.find_joint("Spine").expect("Spine not found");
        let rot = bvh.joint_rotation(1, spine_idx);
        // Frame 1 Spine channels: Zrotation=4, Xrotation=5, Yrotation=6
        assert_eq!(rot[0], 5.0); // Xrotation
        assert_eq!(rot[2], 4.0); // Zrotation  — note: returned as [rx, ry, rz]
    }

    // ── Test 8: interpolate_frame ─────────────────────────────────────────
    #[test]
    fn test_interpolate_frame_midpoint() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let mid = bvh.interpolate_frame(0, 1, 0.5);
        assert!((mid.values[0] - 0.5).abs() < 1e-5);
        assert!((mid.values[1] - 1.0).abs() < 1e-5);
    }

    // ── Test 9: sample_at ────────────────────────────────────────────────
    #[test]
    fn test_sample_at_beginning() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let f = bvh.sample_at(0.0);
        assert_eq!(f.values, bvh.frames[0].values);
    }

    // ── Test 10: write_bvh round-trip ────────────────────────────────────
    #[test]
    fn test_write_bvh_round_trip() {
        let original = parse_bvh(minimal_bvh()).unwrap();
        let text = write_bvh(&original);
        let reparsed = parse_bvh(&text).expect("re-parse failed");
        assert_eq!(
            reparsed.skeleton.joint_count(),
            original.skeleton.joint_count()
        );
        assert_eq!(reparsed.frame_count(), original.frame_count());
    }

    // ── Test 11: retarget_scale ───────────────────────────────────────────
    #[test]
    fn test_retarget_scale() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let frame1 = bvh.frames[1].clone();
        let scaled = retarget_scale(&frame1, 2.0, 3);
        assert!((scaled.values[0] - 2.0).abs() < 1e-5); // 1.0 * 2
        assert!((scaled.values[1] - 4.0).abs() < 1e-5); // 2.0 * 2
        assert!((scaled.values[3] - 1.0).abs() < 1e-5); // rotation unchanged
    }

    // ── Test 12: map_bvh_to_oxihuman ─────────────────────────────────────
    #[test]
    fn test_map_bvh_to_oxihuman() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let map = map_bvh_to_oxihuman(&bvh);
        assert_eq!(map.get("Hips").map(|s| s.as_str()), Some("pelvis"));
        assert_eq!(map.get("Spine").map(|s| s.as_str()), Some("spine_01"));
    }

    // ── Test 13: find_joint ───────────────────────────────────────────────
    #[test]
    fn test_find_joint() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        assert!(bvh.skeleton.find_joint("Hips").is_some());
        assert!(bvh.skeleton.find_joint("DoesNotExist").is_none());
    }

    // ── Test 14: parent/children ──────────────────────────────────────────
    #[test]
    fn test_parent_children() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let hips_idx = bvh.skeleton.find_joint("Hips").unwrap();
        let spine_idx = bvh.skeleton.find_joint("Spine").unwrap();
        assert_eq!(bvh.skeleton.parent_of(hips_idx), None);
        assert_eq!(bvh.skeleton.parent_of(spine_idx), Some(hips_idx));
        assert!(bvh.skeleton.children_of(hips_idx).contains(&spine_idx));
    }

    // ── Test 15: BvhChannel helpers ───────────────────────────────────────
    #[test]
    fn test_channel_helpers() {
        assert!(BvhChannel::Xposition.is_translation());
        assert!(!BvhChannel::Xposition.is_rotation());
        assert!(BvhChannel::Yrotation.is_rotation());
        assert!(!BvhChannel::Yrotation.is_translation());
    }

    // ── Test 16: write output to /tmp/ ────────────────────────────────────
    #[test]
    fn test_write_to_tmp() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let text = write_bvh(&bvh);
        std::fs::write("/tmp/test_mocap_bvh_output.bvh", &text)
            .expect("failed to write /tmp/test_mocap_bvh_output.bvh");
        assert!(text.contains("HIERARCHY"));
        assert!(text.contains("MOTION"));
    }

    // ── Test 17: sample_at beyond last frame ──────────────────────────────
    #[test]
    fn test_sample_at_beyond_end() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        // 9999 seconds is way beyond the last frame — should return last frame values
        let f = bvh.sample_at(9999.0);
        assert_eq!(f.values.len(), bvh.frames[0].values.len());
    }

    // ── Test 18: get_channels helper ─────────────────────────────────────
    #[test]
    fn test_get_channels() {
        let bvh = parse_bvh(minimal_bvh()).unwrap();
        let hips_idx = bvh.skeleton.find_joint("Hips").unwrap();
        let hips = &bvh.skeleton.joints[hips_idx];
        let ch_offset = bvh.skeleton.channel_offset_for(hips_idx);
        let frame0 = &bvh.frames[0];
        let ch = frame0.get_channels(hips, ch_offset);
        assert_eq!(ch.len(), 6);
    }
}
