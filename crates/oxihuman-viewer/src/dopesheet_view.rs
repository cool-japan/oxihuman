// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Dopesheet view state (keyframe overview).

/// State for the dopesheet view.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DopesheetView {
    /// First visible frame.
    pub frame_start: f32,
    /// Last visible frame.
    pub frame_end: f32,
    /// Horizontal zoom factor.
    pub zoom: f32,
    /// Height of each channel row in pixels.
    pub channel_height: f32,
    /// Number of currently visible channels.
    pub visible_channels: u32,
}

/// Return the default dopesheet view.
#[allow(dead_code)]
pub fn default_dopesheet_view() -> DopesheetView {
    DopesheetView {
        frame_start: 0.0,
        frame_end: 250.0,
        zoom: 1.0,
        channel_height: 20.0,
        visible_channels: 8,
    }
}

/// Convert a frame number to an x screen pixel within the given width.
#[allow(dead_code)]
pub fn frame_to_screen_x(view: &DopesheetView, frame: f32, width: f32) -> f32 {
    let range = (view.frame_end - view.frame_start).max(1.0);
    ((frame - view.frame_start) / range) * width * view.zoom
}

/// Return the top-left y pixel of a channel row (0-indexed).
#[allow(dead_code)]
pub fn channel_to_screen_y(view: &DopesheetView, channel: u32) -> f32 {
    channel as f32 * view.channel_height
}

/// Return the number of frames currently visible.
#[allow(dead_code)]
pub fn frames_in_view(view: &DopesheetView) -> f32 {
    (view.frame_end - view.frame_start).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_channel_height() {
        assert_eq!(default_dopesheet_view().channel_height, 20.0);
    }

    #[test]
    fn default_visible_channels() {
        assert_eq!(default_dopesheet_view().visible_channels, 8);
    }

    #[test]
    fn frames_in_view_default() {
        let v = default_dopesheet_view();
        assert!((frames_in_view(&v) - 250.0).abs() < 1e-5);
    }

    #[test]
    fn frame_to_x_at_start_is_zero() {
        let v = default_dopesheet_view();
        assert!(frame_to_screen_x(&v, v.frame_start, 500.0).abs() < 1e-5);
    }

    #[test]
    fn channel_y_zero_at_zero() {
        let v = default_dopesheet_view();
        assert_eq!(channel_to_screen_y(&v, 0), 0.0);
    }

    #[test]
    fn channel_y_increases_with_index() {
        let v = default_dopesheet_view();
        assert!(channel_to_screen_y(&v, 3) > channel_to_screen_y(&v, 2));
    }

    #[test]
    fn channel_y_equals_index_times_height() {
        let v = default_dopesheet_view();
        let y = channel_to_screen_y(&v, 5);
        assert!((y - 5.0 * 20.0).abs() < 1e-5);
    }

    #[test]
    fn frame_to_x_at_end_is_width_when_zoom_one() {
        let v = default_dopesheet_view();
        let x = frame_to_screen_x(&v, v.frame_end, 250.0);
        assert!((x - 250.0).abs() < 1e-3);
    }

    #[test]
    fn zoom_doubles_x() {
        let mut v = default_dopesheet_view();
        let x1 = frame_to_screen_x(&v, 125.0, 500.0);
        v.zoom = 2.0;
        let x2 = frame_to_screen_x(&v, 125.0, 500.0);
        assert!((x2 - x1 * 2.0).abs() < 1e-3);
    }

    #[test]
    fn frames_in_view_non_negative() {
        let v = DopesheetView { frame_start: 100.0, frame_end: 100.0, zoom: 1.0, channel_height: 20.0, visible_channels: 4 };
        assert!(frames_in_view(&v) >= 0.0);
    }
}
