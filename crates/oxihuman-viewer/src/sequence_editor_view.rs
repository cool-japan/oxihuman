// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Video sequence editor view.

/// A sequence strip in the editor.
#[derive(Debug, Clone)]
pub struct SeqStrip {
    pub id: u32,
    pub name: String,
    pub channel: u32,
    pub start_frame: i32,
    pub end_frame: i32,
}

/// State for the sequence editor view.
#[derive(Debug, Clone)]
pub struct SequenceEditorView {
    pub strips: Vec<SeqStrip>,
    pub current_frame: i32,
    pub fps: f32,
    pub enabled: bool,
}

/// Create a new sequence editor view.
pub fn new_sequence_editor_view() -> SequenceEditorView {
    SequenceEditorView {
        strips: Vec::new(),
        current_frame: 0,
        fps: 24.0,
        enabled: true,
    }
}

/// Add a strip.
pub fn sev_add_strip(
    v: &mut SequenceEditorView,
    id: u32,
    name: &str,
    ch: u32,
    start: i32,
    end: i32,
) {
    v.strips.push(SeqStrip {
        id,
        name: name.to_string(),
        channel: ch,
        start_frame: start,
        end_frame: end.max(start),
    });
}

/// Set current frame.
pub fn sev_set_frame(v: &mut SequenceEditorView, frame: i32) {
    v.current_frame = frame;
}

/// Set FPS (clamped 1–240).
pub fn sev_set_fps(v: &mut SequenceEditorView, fps: f32) {
    v.fps = fps.clamp(1.0, 240.0);
}

/// Return the total duration in frames across all strips.
pub fn sev_total_frames(v: &SequenceEditorView) -> i32 {
    v.strips.iter().map(|s| s.end_frame).max().unwrap_or(0)
}

/// Serialise to JSON.
pub fn sev_to_json(v: &SequenceEditorView) -> String {
    format!(
        r#"{{"strip_count":{},"current_frame":{},"fps":{:.1},"enabled":{}}}"#,
        v.strips.len(),
        v.current_frame,
        v.fps,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_sequence_editor_view();
        assert!(v.strips.is_empty() /* no strips */);
    }

    #[test]
    fn add_strip() {
        let mut v = new_sequence_editor_view();
        sev_add_strip(&mut v, 1, "Video", 1, 0, 100);
        assert_eq!(v.strips.len(), 1 /* one strip */);
    }

    #[test]
    fn strip_end_clamp() {
        let mut v = new_sequence_editor_view();
        sev_add_strip(&mut v, 1, "X", 1, 50, 10);
        assert_eq!(v.strips[0].end_frame, 50 /* end clamped to start */);
    }

    #[test]
    fn set_frame() {
        let mut v = new_sequence_editor_view();
        sev_set_frame(&mut v, 42);
        assert_eq!(v.current_frame, 42 /* frame set */);
    }

    #[test]
    fn fps_clamp() {
        let mut v = new_sequence_editor_view();
        sev_set_fps(&mut v, 0.0);
        assert!((v.fps - 1.0).abs() < 1e-6 /* clamped to 1 */);
    }

    #[test]
    fn total_frames() {
        let mut v = new_sequence_editor_view();
        sev_add_strip(&mut v, 1, "A", 1, 0, 100);
        sev_add_strip(&mut v, 2, "B", 1, 50, 200);
        assert_eq!(sev_total_frames(&v), 200 /* max end frame */);
    }

    #[test]
    fn json_has_fps() {
        let v = new_sequence_editor_view();
        assert!(sev_to_json(&v).contains("fps") /* json has fps */);
    }

    #[test]
    fn enabled_default() {
        let v = new_sequence_editor_view();
        assert!(v.enabled /* enabled */);
    }
}
