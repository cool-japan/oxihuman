// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DaVinci Resolve timeline stub export.

/// Resolve timeline type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveTimelineType {
    Edit,
    Color,
    Fusion,
    Deliver,
}

/// A Resolve clip entry.
#[derive(Debug, Clone)]
pub struct ResolveClip {
    pub name: String,
    pub start_frame: i32,
    pub end_frame: i32,
    pub reel: String,
}

/// A DaVinci Resolve timeline export.
#[derive(Debug, Clone)]
pub struct ResolveTimeline {
    pub name: String,
    pub timeline_type: ResolveTimelineType,
    pub fps: f32,
    pub clips: Vec<ResolveClip>,
}

/// Create a new Resolve timeline export.
pub fn new_resolve_timeline(name: &str, fps: f32, ttype: ResolveTimelineType) -> ResolveTimeline {
    ResolveTimeline {
        name: name.to_string(),
        timeline_type: ttype,
        fps,
        clips: Vec::new(),
    }
}

/// Add a clip to the timeline.
pub fn resolve_add_clip(tl: &mut ResolveTimeline, name: &str, start: i32, end: i32, reel: &str) {
    tl.clips.push(ResolveClip {
        name: name.to_string(),
        start_frame: start,
        end_frame: end,
        reel: reel.to_string(),
    });
}

/// Return the clip count.
pub fn resolve_clip_count(tl: &ResolveTimeline) -> usize {
    tl.clips.len()
}

/// Total timeline duration in frames.
pub fn resolve_duration_frames(tl: &ResolveTimeline) -> i32 {
    tl.clips.iter().map(|c| c.end_frame).max().unwrap_or(0)
}

/// Validate the timeline.
pub fn validate_resolve_timeline(tl: &ResolveTimeline) -> bool {
    tl.fps > 0.0 && tl.clips.iter().all(|c| c.end_frame > c.start_frame)
}

/// Generate a stub Resolve Python API script.
pub fn resolve_to_python(tl: &ResolveTimeline) -> String {
    let mut out =
        String::from("import DaVinciResolveScript as dvr\nresolve = dvr.scriptapp('Resolve')\n");
    out.push_str("proj = resolve.GetProjectManager().GetCurrentProject()\n");
    out.push_str(&format!("timeline = proj.CreateTimeline('{}')\n", tl.name));
    for clip in &tl.clips {
        out.push_str(&format!(
            "timeline.AddClip('{}', {}, {})\n",
            clip.name, clip.start_frame, clip.end_frame
        ));
    }
    out
}

/// Estimate the script size.
pub fn resolve_script_size(tl: &ResolveTimeline) -> usize {
    resolve_to_python(tl).len()
}

/// Find a clip by reel name.
pub fn resolve_clips_for_reel<'a>(tl: &'a ResolveTimeline, reel: &str) -> Vec<&'a ResolveClip> {
    tl.clips.iter().filter(|c| c.reel == reel).collect()
}

/// Timeline type name as string.
pub fn timeline_type_name(tl: &ResolveTimeline) -> &'static str {
    match tl.timeline_type {
        ResolveTimelineType::Edit => "Edit",
        ResolveTimelineType::Color => "Color",
        ResolveTimelineType::Fusion => "Fusion",
        ResolveTimelineType::Deliver => "Deliver",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tl() -> ResolveTimeline {
        let mut tl = new_resolve_timeline("MyEdit", 24.0, ResolveTimelineType::Edit);
        resolve_add_clip(&mut tl, "shot_010", 1, 100, "A001");
        resolve_add_clip(&mut tl, "shot_020", 101, 200, "A001");
        tl
    }

    #[test]
    fn test_clip_count() {
        assert_eq!(resolve_clip_count(&sample_tl()), 2);
    }

    #[test]
    fn test_duration() {
        assert_eq!(resolve_duration_frames(&sample_tl()), 200);
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate_resolve_timeline(&sample_tl()));
    }

    #[test]
    fn test_validate_zero_fps() {
        let tl = new_resolve_timeline("bad", 0.0, ResolveTimelineType::Edit);
        assert!(!validate_resolve_timeline(&tl));
    }

    #[test]
    fn test_to_python() {
        let tl = sample_tl();
        assert!(resolve_to_python(&tl).contains("shot_010"));
    }

    #[test]
    fn test_clips_for_reel() {
        let tl = sample_tl();
        let reel_clips = resolve_clips_for_reel(&tl, "A001");
        assert_eq!(reel_clips.len(), 2);
    }

    #[test]
    fn test_timeline_type_name() {
        let tl = sample_tl();
        assert_eq!(timeline_type_name(&tl), "Edit");
    }

    #[test]
    fn test_script_size() {
        assert!(resolve_script_size(&sample_tl()) > 0);
    }

    #[test]
    fn test_empty_duration() {
        let tl = new_resolve_timeline("empty", 25.0, ResolveTimelineType::Color);
        assert_eq!(resolve_duration_frames(&tl), 0);
    }
}
