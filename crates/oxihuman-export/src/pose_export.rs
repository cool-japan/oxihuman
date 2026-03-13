// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pose and animation-frame export with keyframe support.
//!
//! Provides structs and functions for building, sampling, and serializing
//! animation clips composed of pose frames.

// ── Types ──────────────────────────────────────────────────────────────────

/// A single pose frame in a clip.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExportPoseFrame {
    /// Time of this keyframe in seconds.
    pub time: f32,
    /// Per-bone transforms: `(position [x,y,z], rotation [x,y,z,w])`.
    pub bone_transforms: Vec<([f32; 3], [f32; 4])>,
}

/// An ordered sequence of pose frames forming an animation clip.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExportPoseClip {
    /// Clip identifier.
    pub name: String,
    /// Ordered keyframes.
    pub frames: Vec<ExportPoseFrame>,
    /// Playback rate in frames per second.
    pub fps: f32,
    /// Whether the clip loops.
    pub looping: bool,
}

/// Configuration for pose/animation export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseExportConfig {
    /// Target FPS for export baking.
    pub fps: f32,
    /// Include rotation channels.
    pub include_rotation: bool,
    /// Include position channels.
    pub include_position: bool,
    /// Floating-point decimal precision.
    pub precision: u32,
}

// ── Type aliases ───────────────────────────────────────────────────────────

/// Pair of frame references used for interpolation.
pub type FramePair<'a> = (&'a ExportPoseFrame, &'a ExportPoseFrame);

// ── Config ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`PoseExportConfig`].
#[allow(dead_code)]
pub fn default_pose_export_config() -> PoseExportConfig {
    PoseExportConfig {
        fps: 30.0,
        include_rotation: true,
        include_position: true,
        precision: 6,
    }
}

// ── Clip construction ──────────────────────────────────────────────────────

/// Create a new, empty [`ExportPoseClip`].
#[allow(dead_code)]
pub fn new_pose_clip(name: &str, fps: f32) -> ExportPoseClip {
    ExportPoseClip {
        name: name.to_string(),
        frames: Vec::new(),
        fps: fps.max(f32::EPSILON),
        looping: false,
    }
}

/// Append a frame to the clip (kept in insertion order; call `sort` if needed).
#[allow(dead_code)]
pub fn add_frame(clip: &mut ExportPoseClip, frame: ExportPoseFrame) {
    clip.frames.push(frame);
}

/// Return the number of frames in the clip.
#[allow(dead_code)]
pub fn frame_count(clip: &ExportPoseClip) -> usize {
    clip.frames.len()
}

/// Return the clip duration in seconds (time of last frame minus first, or 0).
#[allow(dead_code)]
pub fn pose_clip_duration(clip: &ExportPoseClip) -> f32 {
    if clip.frames.len() < 2 {
        return 0.0;
    }
    clip.frames.last().map_or(0.0, |f| f.time) - clip.frames.first().map_or(0.0, |f| f.time)
}

/// Return the configured FPS of the clip.
#[allow(dead_code)]
pub fn pose_clip_fps(clip: &ExportPoseClip) -> f32 {
    clip.fps
}

/// Update the FPS of the clip.
#[allow(dead_code)]
pub fn set_clip_fps(clip: &mut ExportPoseClip, fps: f32) {
    clip.fps = fps.max(f32::EPSILON);
}

// ── Clip operations ────────────────────────────────────────────────────────

/// Remove frames with timestamps outside `[start, end]` (inclusive).
#[allow(dead_code)]
pub fn trim_clip(clip: &mut ExportPoseClip, start: f32, end: f32) {
    clip.frames.retain(|f| f.time >= start && f.time <= end);
}

/// Reverse the time order of all frames in-place, re-normalizing timestamps
/// so the first frame starts at 0.
#[allow(dead_code)]
pub fn reverse_clip(clip: &mut ExportPoseClip) {
    clip.frames.reverse();
    if let Some(first_t) = clip.frames.first().map(|f| f.time) {
        let total = clip.frames.last().map_or(0.0, |f| f.time) - first_t;
        for frame in &mut clip.frames {
            frame.time = total - (frame.time - first_t);
        }
        clip.frames.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

/// Scale all frame timestamps by `factor` (e.g. 0.5 = double speed).
#[allow(dead_code)]
pub fn scale_clip_timing(clip: &mut ExportPoseClip, factor: f32) {
    let f = factor.max(f32::EPSILON);
    for frame in &mut clip.frames {
        frame.time *= f;
    }
}

/// Merge `other` into `clip`, shifting `other`'s timestamps so they start
/// immediately after the last frame of `clip`.
#[allow(dead_code)]
pub fn merge_clips(clip: &mut ExportPoseClip, other: &ExportPoseClip) {
    let offset = clip.frames.last().map_or(0.0, |f| f.time);
    let first_other = other.frames.first().map_or(0.0, |f| f.time);
    for frame in &other.frames {
        clip.frames.push(ExportPoseFrame {
            time: offset + (frame.time - first_other),
            bone_transforms: frame.bone_transforms.clone(),
        });
    }
}

/// Linearly interpolate between two `[f32; 3]` arrays.
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Linearly interpolate (NLERP) between two quaternions `[f32; 4]`.
fn nlerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let raw = [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ];
    let len = (raw[0] * raw[0] + raw[1] * raw[1] + raw[2] * raw[2] + raw[3] * raw[3])
        .sqrt()
        .max(f32::EPSILON);
    [raw[0] / len, raw[1] / len, raw[2] / len, raw[3] / len]
}

/// Sample the clip at `time_sec` by linearly interpolating between the two
/// nearest frames.  Returns `None` if the clip has no frames.
#[allow(dead_code)]
pub fn sample_clip_at(clip: &ExportPoseClip, time_sec: f32) -> Option<ExportPoseFrame> {
    if clip.frames.is_empty() {
        return None;
    }
    if clip.frames.len() == 1 {
        return Some(clip.frames[0].clone());
    }
    // Clamp to range
    let t_min = clip.frames.first().map_or(0.0, |f| f.time);
    let t_max = clip.frames.last().map_or(0.0, |f| f.time);
    let t = time_sec.clamp(t_min, t_max);

    // Find bracketing pair
    let idx = clip
        .frames
        .partition_point(|f| f.time <= t)
        .saturating_sub(1)
        .min(clip.frames.len() - 2);

    let fa = &clip.frames[idx];
    let fb = &clip.frames[idx + 1];
    let span = fb.time - fa.time;
    let alpha = if span.abs() < f32::EPSILON {
        0.0
    } else {
        (t - fa.time) / span
    };

    let bone_count = fa.bone_transforms.len().min(fb.bone_transforms.len());
    let bone_transforms = (0..bone_count)
        .map(|i| {
            let (pa, ra) = fa.bone_transforms[i];
            let (pb, rb) = fb.bone_transforms[i];
            (lerp3(pa, pb, alpha), nlerp4(ra, rb, alpha))
        })
        .collect();

    Some(ExportPoseFrame {
        time: t,
        bone_transforms,
    })
}

// ── Serialization ──────────────────────────────────────────────────────────

/// Serialize the clip to a compact JSON string.
#[allow(dead_code)]
pub fn clip_to_json(clip: &ExportPoseClip) -> String {
    let frame_strs: Vec<String> = clip
        .frames
        .iter()
        .map(|f| {
            let bt_strs: Vec<String> = f
                .bone_transforms
                .iter()
                .map(|(p, r)| {
                    format!(
                        r#"{{"pos":[{},{},{}],"rot":[{},{},{},{}]}}"#,
                        p[0], p[1], p[2], r[0], r[1], r[2], r[3]
                    )
                })
                .collect();
            format!(r#"{{"time":{},"bones":[{}]}}"#, f.time, bt_strs.join(","))
        })
        .collect();
    format!(
        r#"{{"name":"{}","fps":{},"looping":{},"frames":[{}]}}"#,
        clip.name,
        clip.fps,
        clip.looping,
        frame_strs.join(",")
    )
}

/// Serialize the clip to CSV format (one bone-transform per row).
#[allow(dead_code)]
pub fn clip_to_csv(clip: &ExportPoseClip) -> String {
    let mut out = String::from("frame_time,bone_idx,pos_x,pos_y,pos_z,rot_x,rot_y,rot_z,rot_w\n");
    for f in &clip.frames {
        for (i, (pos, rot)) in f.bone_transforms.iter().enumerate() {
            out.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                f.time, i, pos[0], pos[1], pos[2], rot[0], rot[1], rot[2], rot[3]
            ));
        }
    }
    out
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(time: f32, n: usize) -> ExportPoseFrame {
        ExportPoseFrame {
            time,
            bone_transforms: (0..n)
                .map(|i| ([i as f32, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]))
                .collect(),
        }
    }

    #[test]
    fn test_default_pose_export_config() {
        let cfg = default_pose_export_config();
        assert!((cfg.fps - 30.0).abs() < 1e-5);
        assert!(cfg.include_rotation);
        assert!(cfg.include_position);
    }

    #[test]
    fn test_new_pose_clip() {
        let clip = new_pose_clip("run", 24.0);
        assert_eq!(clip.name, "run");
        assert!((clip.fps - 24.0).abs() < 1e-5);
        assert!(clip.frames.is_empty());
    }

    #[test]
    fn test_add_frame() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 2));
        assert_eq!(frame_count(&clip), 1);
    }

    #[test]
    fn test_frame_count_empty() {
        let clip = new_pose_clip("c", 30.0);
        assert_eq!(frame_count(&clip), 0);
    }

    #[test]
    fn test_pose_clip_duration_two_frames() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 1));
        add_frame(&mut clip, make_frame(1.0, 1));
        assert!((pose_clip_duration(&clip) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pose_clip_duration_single_frame() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.5, 1));
        assert!((pose_clip_duration(&clip) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_pose_clip_fps() {
        let clip = new_pose_clip("c", 60.0);
        assert!((pose_clip_fps(&clip) - 60.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_clip_fps() {
        let mut clip = new_pose_clip("c", 30.0);
        set_clip_fps(&mut clip, 24.0);
        assert!((clip.fps - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_trim_clip() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 1));
        add_frame(&mut clip, make_frame(0.5, 1));
        add_frame(&mut clip, make_frame(1.0, 1));
        add_frame(&mut clip, make_frame(2.0, 1));
        trim_clip(&mut clip, 0.4, 1.1);
        assert_eq!(frame_count(&clip), 2);
    }

    #[test]
    fn test_reverse_clip() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 1));
        add_frame(&mut clip, make_frame(1.0, 1));
        reverse_clip(&mut clip);
        assert_eq!(frame_count(&clip), 2);
        // After reversing, first frame time should be <= last frame time
        assert!(clip.frames[0].time <= clip.frames[1].time);
    }

    #[test]
    fn test_scale_clip_timing() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 1));
        add_frame(&mut clip, make_frame(2.0, 1));
        scale_clip_timing(&mut clip, 0.5);
        assert!((clip.frames[1].time - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_merge_clips() {
        let mut clip1 = new_pose_clip("c1", 30.0);
        add_frame(&mut clip1, make_frame(0.0, 1));
        add_frame(&mut clip1, make_frame(1.0, 1));
        let mut clip2 = new_pose_clip("c2", 30.0);
        add_frame(&mut clip2, make_frame(0.0, 1));
        add_frame(&mut clip2, make_frame(0.5, 1));
        merge_clips(&mut clip1, &clip2);
        assert_eq!(frame_count(&clip1), 4);
    }

    #[test]
    fn test_sample_clip_at_empty() {
        let clip = new_pose_clip("c", 30.0);
        assert!(sample_clip_at(&clip, 0.5).is_none());
    }

    #[test]
    fn test_sample_clip_at_single_frame() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 2));
        let s = sample_clip_at(&clip, 0.5).expect("should succeed");
        assert_eq!(s.bone_transforms.len(), 2);
    }

    #[test]
    fn test_sample_clip_at_midpoint() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(
            &mut clip,
            ExportPoseFrame {
                time: 0.0,
                bone_transforms: vec![([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0])],
            },
        );
        add_frame(
            &mut clip,
            ExportPoseFrame {
                time: 1.0,
                bone_transforms: vec![([2.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0])],
            },
        );
        let s = sample_clip_at(&clip, 0.5).expect("should succeed");
        assert!((s.bone_transforms[0].0[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_clip_to_json_contains_name() {
        let mut clip = new_pose_clip("idle", 30.0);
        add_frame(&mut clip, make_frame(0.0, 1));
        let json = clip_to_json(&clip);
        assert!(json.contains("idle"));
    }

    #[test]
    fn test_clip_to_csv_has_header() {
        let mut clip = new_pose_clip("c", 30.0);
        add_frame(&mut clip, make_frame(0.0, 2));
        let csv = clip_to_csv(&clip);
        assert!(csv.starts_with("frame_time"));
    }
}
