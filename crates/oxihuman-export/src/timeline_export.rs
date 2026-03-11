// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Animation timeline export.

#![allow(dead_code)]

/// Configuration for timeline export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    /// Frames per second.
    pub fps: f32,
    /// First frame index.
    pub start_frame: i32,
    /// Last frame index (inclusive).
    pub end_frame: i32,
}

/// A single event on the timeline.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Frame number of this event.
    pub frame: i32,
    /// Human-readable label.
    pub label: String,
    /// Category/type of the event.
    pub event_type: String,
}

/// Container for a full timeline export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimelineExport {
    /// Timeline configuration.
    pub config: TimelineConfig,
    /// All events on the timeline.
    pub events: Vec<TimelineEvent>,
}

/// Returns the default [`TimelineConfig`].
#[allow(dead_code)]
pub fn default_timeline_config() -> TimelineConfig {
    TimelineConfig {
        fps: 24.0,
        start_frame: 0,
        end_frame: 240,
    }
}

/// Creates a new empty [`TimelineExport`] with the given config.
#[allow(dead_code)]
pub fn new_timeline_export(config: TimelineConfig) -> TimelineExport {
    TimelineExport { config, events: Vec::new() }
}

/// Appends an event to the timeline.
#[allow(dead_code)]
pub fn timeline_add_event(export: &mut TimelineExport, event: TimelineEvent) {
    export.events.push(event);
}

/// Returns the number of events.
#[allow(dead_code)]
pub fn timeline_event_count(export: &TimelineExport) -> usize {
    export.events.len()
}

/// Returns the total duration in seconds.
#[allow(dead_code)]
pub fn timeline_duration_seconds(export: &TimelineExport) -> f32 {
    let frames = (export.config.end_frame - export.config.start_frame).max(0) as f32;
    if export.config.fps > 0.0 {
        frames / export.config.fps
    } else {
        0.0
    }
}

/// Serialises to a minimal JSON string.
#[allow(dead_code)]
pub fn timeline_to_json(export: &TimelineExport) -> String {
    format!(
        "{{\"fps\":{},\"start\":{},\"end\":{},\"events\":{}}}",
        export.config.fps,
        export.config.start_frame,
        export.config.end_frame,
        export.events.len()
    )
}

/// Returns the frame index at a given time in seconds.
#[allow(dead_code)]
pub fn timeline_frame_at_time(export: &TimelineExport, time_sec: f32) -> i32 {
    export.config.start_frame + (time_sec * export.config.fps).round() as i32
}

/// Validates the timeline configuration.
#[allow(dead_code)]
pub fn timeline_validate(export: &TimelineExport) -> bool {
    export.config.fps > 0.0 && export.config.start_frame <= export.config.end_frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_timeline_config();
        assert_eq!(cfg.fps, 24.0);
        assert_eq!(cfg.start_frame, 0);
        assert_eq!(cfg.end_frame, 240);
    }

    #[test]
    fn test_new_export_empty() {
        let export = new_timeline_export(default_timeline_config());
        assert_eq!(timeline_event_count(&export), 0);
    }

    #[test]
    fn test_add_event() {
        let mut export = new_timeline_export(default_timeline_config());
        timeline_add_event(&mut export, TimelineEvent { frame: 10, label: "start".to_string(), event_type: "marker".to_string() });
        assert_eq!(timeline_event_count(&export), 1);
    }

    #[test]
    fn test_duration_seconds() {
        let export = new_timeline_export(default_timeline_config());
        let dur = timeline_duration_seconds(&export);
        assert!((dur - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_to_json() {
        let export = new_timeline_export(default_timeline_config());
        let json = timeline_to_json(&export);
        assert!(json.contains("fps"));
    }

    #[test]
    fn test_frame_at_time() {
        let export = new_timeline_export(default_timeline_config());
        let frame = timeline_frame_at_time(&export, 1.0);
        assert_eq!(frame, 24);
    }

    #[test]
    fn test_validate_valid() {
        let export = new_timeline_export(default_timeline_config());
        assert!(timeline_validate(&export));
    }

    #[test]
    fn test_validate_invalid_fps() {
        let cfg = TimelineConfig { fps: 0.0, start_frame: 0, end_frame: 100 };
        let export = new_timeline_export(cfg);
        assert!(!timeline_validate(&export));
    }

    #[test]
    fn test_validate_invalid_frames() {
        let cfg = TimelineConfig { fps: 24.0, start_frame: 100, end_frame: 0 };
        let export = new_timeline_export(cfg);
        assert!(!timeline_validate(&export));
    }

    #[test]
    fn test_multiple_events() {
        let mut export = new_timeline_export(default_timeline_config());
        for i in 0..5 {
            timeline_add_event(&mut export, TimelineEvent { frame: i, label: format!("ev{i}"), event_type: "marker".to_string() });
        }
        assert_eq!(timeline_event_count(&export), 5);
    }
}
