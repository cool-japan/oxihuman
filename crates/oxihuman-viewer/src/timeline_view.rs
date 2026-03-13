// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Timeline view state (frames, zoom, playhead).

/// State for the timeline view.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineView {
    /// First visible frame.
    pub frame_start: f32,
    /// Last visible frame.
    pub frame_end: f32,
    /// Horizontal zoom factor.
    pub zoom: f32,
    /// Current playhead frame.
    pub playhead: f32,
    /// Frames per second.
    pub fps: f32,
}

/// Return the default timeline view.
#[allow(dead_code)]
pub fn default_timeline_view() -> TimelineView {
    TimelineView {
        frame_start: 0.0,
        frame_end: 250.0,
        zoom: 1.0,
        playhead: 0.0,
        fps: 24.0,
    }
}

/// Convert a frame number to an x pixel position within the given pixel width.
#[allow(dead_code)]
pub fn timeline_frame_to_x(view: &TimelineView, frame: f32, width: f32) -> f32 {
    let range = (view.frame_end - view.frame_start).max(1.0);
    ((frame - view.frame_start) / range) * width * view.zoom
}

/// Convert an x pixel position to a frame number.
#[allow(dead_code)]
pub fn timeline_x_to_frame(view: &TimelineView, x: f32, width: f32) -> f32 {
    let range = (view.frame_end - view.frame_start).max(1.0);
    x / (width * view.zoom) * range + view.frame_start
}

/// Return the total duration of the visible range in frames.
#[allow(dead_code)]
pub fn timeline_duration(view: &TimelineView) -> f32 {
    (view.frame_end - view.frame_start).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fps_is_24() {
        assert_eq!(default_timeline_view().fps, 24.0);
    }

    #[test]
    fn default_playhead_at_zero() {
        assert_eq!(default_timeline_view().playhead, 0.0);
    }

    #[test]
    fn duration_default() {
        let v = default_timeline_view();
        assert!((timeline_duration(&v) - 250.0).abs() < 1e-5);
    }

    #[test]
    fn frame_to_x_at_start_is_zero() {
        let v = default_timeline_view();
        let x = timeline_frame_to_x(&v, v.frame_start, 1000.0);
        assert!(x.abs() < 1e-5);
    }

    #[test]
    fn frame_to_x_at_end_is_width() {
        let v = default_timeline_view();
        let x = timeline_frame_to_x(&v, v.frame_end, 1000.0);
        assert!((x - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn x_to_frame_round_trip() {
        let v = default_timeline_view();
        let frame = 100.0;
        let x = timeline_frame_to_x(&v, frame, 800.0);
        let back = timeline_x_to_frame(&v, x, 800.0);
        assert!((back - frame).abs() < 1e-4);
    }

    #[test]
    fn duration_is_non_negative() {
        let v = TimelineView { frame_start: 50.0, frame_end: 50.0, zoom: 1.0, playhead: 50.0, fps: 30.0 };
        assert!(timeline_duration(&v) >= 0.0);
    }

    #[test]
    fn zoom_affects_x() {
        let mut v = default_timeline_view();
        let x1 = timeline_frame_to_x(&v, 125.0, 1000.0);
        v.zoom = 2.0;
        let x2 = timeline_frame_to_x(&v, 125.0, 1000.0);
        assert!((x2 - x1 * 2.0).abs() < 1e-3);
    }

    #[test]
    fn playhead_update() {
        let mut v = default_timeline_view();
        v.playhead = 50.0;
        assert_eq!(v.playhead, 50.0);
    }
}
