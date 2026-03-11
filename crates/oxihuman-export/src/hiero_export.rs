// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hiero timeline export stub.

/// A Hiero clip entry on the timeline.
#[derive(Debug, Clone)]
pub struct HieroClip {
    pub name: String,
    pub track: usize,
    pub in_frame: i32,
    pub out_frame: i32,
    pub source_path: String,
}

/// A Hiero timeline export.
#[derive(Debug, Clone)]
pub struct HieroTimeline {
    pub name: String,
    pub fps: f32,
    pub clips: Vec<HieroClip>,
}

/// Create a new Hiero timeline.
pub fn new_hiero_timeline(name: &str, fps: f32) -> HieroTimeline {
    HieroTimeline {
        name: name.to_string(),
        fps,
        clips: Vec::new(),
    }
}

/// Add a clip to the timeline.
pub fn hiero_add_clip(
    timeline: &mut HieroTimeline,
    name: &str,
    track: usize,
    in_frame: i32,
    out_frame: i32,
    source_path: &str,
) {
    timeline.clips.push(HieroClip {
        name: name.to_string(),
        track,
        in_frame,
        out_frame,
        source_path: source_path.to_string(),
    });
}

/// Count clips on a specific track.
pub fn clips_on_track(timeline: &HieroTimeline, track: usize) -> usize {
    timeline.clips.iter().filter(|c| c.track == track).count()
}

/// Total clip count.
pub fn hiero_clip_count(timeline: &HieroTimeline) -> usize {
    timeline.clips.len()
}

/// Find a clip by name.
pub fn hiero_find_clip<'a>(timeline: &'a HieroTimeline, name: &str) -> Option<&'a HieroClip> {
    timeline.clips.iter().find(|c| c.name == name)
}

/// Return the total timeline duration in frames.
pub fn timeline_duration_frames(timeline: &HieroTimeline) -> i32 {
    timeline
        .clips
        .iter()
        .map(|c| c.out_frame)
        .max()
        .unwrap_or(0)
}

/// Validate: FPS > 0 and all clips have out > in.
pub fn validate_hiero_timeline(timeline: &HieroTimeline) -> bool {
    timeline.fps > 0.0 && timeline.clips.iter().all(|c| c.out_frame > c.in_frame)
}

/// Serialize the timeline to a stub Python script.
pub fn hiero_to_python(timeline: &HieroTimeline) -> String {
    let mut out = format!(
        "import hiero\nproj = hiero.core.Project('{}')\n",
        timeline.name
    );
    for clip in &timeline.clips {
        out.push_str(&format!(
            "seq.addClip('{}', {}, {})\n",
            clip.name, clip.in_frame, clip.out_frame
        ));
    }
    out
}

/// Estimate the export script size.
pub fn hiero_script_size(timeline: &HieroTimeline) -> usize {
    hiero_to_python(timeline).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_timeline() -> HieroTimeline {
        let mut tl = new_hiero_timeline("MyProject", 24.0);
        hiero_add_clip(&mut tl, "shot_010", 0, 1, 100, "/path/shot010.mov");
        hiero_add_clip(&mut tl, "shot_020", 0, 101, 200, "/path/shot020.mov");
        hiero_add_clip(&mut tl, "vfx_010", 1, 1, 50, "/path/vfx010.mov");
        tl
    }

    #[test]
    fn test_clip_count() {
        let tl = sample_timeline();
        assert_eq!(hiero_clip_count(&tl), 3);
    }

    #[test]
    fn test_clips_on_track() {
        let tl = sample_timeline();
        assert_eq!(clips_on_track(&tl, 0), 2);
        assert_eq!(clips_on_track(&tl, 1), 1);
    }

    #[test]
    fn test_find_clip() {
        let tl = sample_timeline();
        assert!(hiero_find_clip(&tl, "shot_010").is_some());
        assert!(hiero_find_clip(&tl, "none").is_none());
    }

    #[test]
    fn test_duration_frames() {
        let tl = sample_timeline();
        assert_eq!(timeline_duration_frames(&tl), 200);
    }

    #[test]
    fn test_validate_valid() {
        let tl = sample_timeline();
        assert!(validate_hiero_timeline(&tl));
    }

    #[test]
    fn test_validate_bad_fps() {
        let tl = new_hiero_timeline("bad", 0.0);
        assert!(!validate_hiero_timeline(&tl));
    }

    #[test]
    fn test_to_python() {
        let tl = sample_timeline();
        let s = hiero_to_python(&tl);
        assert!(s.contains("shot_010"));
    }

    #[test]
    fn test_script_size() {
        let tl = sample_timeline();
        assert!(hiero_script_size(&tl) > 0);
    }

    #[test]
    fn test_empty_timeline_duration() {
        let tl = new_hiero_timeline("empty", 25.0);
        assert_eq!(timeline_duration_frames(&tl), 0);
    }
}
