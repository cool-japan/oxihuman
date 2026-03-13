// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Timecode burn-in overlay for video production.

/// Timecode format type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimecodeFormat {
    Smpte24,
    Smpte25,
    Smpte2997,
    Smpte30,
    Frames,
}

/// Timecode overlay view configuration.
#[derive(Debug, Clone)]
pub struct TimecodeOverlayView {
    pub format: TimecodeFormat,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub frames: u32,
    pub position_x: f32,
    pub position_y: f32,
    pub text_scale: f32,
    pub enabled: bool,
}

impl TimecodeOverlayView {
    pub fn new() -> Self {
        Self {
            format: TimecodeFormat::Smpte24,
            hours: 0,
            minutes: 0,
            seconds: 0,
            frames: 0,
            position_x: 0.05,
            position_y: 0.05,
            text_scale: 1.0,
            enabled: true,
        }
    }
}

impl Default for TimecodeOverlayView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new timecode overlay view.
pub fn new_timecode_overlay_view() -> TimecodeOverlayView {
    TimecodeOverlayView::new()
}

/// Set timecode from component values.
pub fn tcv_set_timecode(view: &mut TimecodeOverlayView, hh: u32, mm: u32, ss: u32, ff: u32) {
    view.hours = hh.min(23);
    view.minutes = mm.min(59);
    view.seconds = ss.min(59);
    view.frames = ff;
}

/// Set timecode format.
pub fn tcv_set_format(view: &mut TimecodeOverlayView, format: TimecodeFormat) {
    view.format = format;
}

/// Set burn-in position in normalized viewport coords.
pub fn tcv_set_position(view: &mut TimecodeOverlayView, x: f32, y: f32) {
    view.position_x = x.clamp(0.0, 1.0);
    view.position_y = y.clamp(0.0, 1.0);
}

/// Toggle timecode overlay visibility.
pub fn tcv_set_enabled(view: &mut TimecodeOverlayView, enabled: bool) {
    view.enabled = enabled;
}

/// Format timecode as standard HH:MM:SS:FF string.
pub fn tcv_format_string(view: &TimecodeOverlayView) -> String {
    format!(
        "{:02}:{:02}:{:02}:{:02}",
        view.hours, view.minutes, view.seconds, view.frames
    )
}

/// Compute total frame count from timecode components.
pub fn tcv_total_frames(view: &TimecodeOverlayView) -> u64 {
    let fps = match view.format {
        TimecodeFormat::Smpte24 => 24u64,
        TimecodeFormat::Smpte25 => 25,
        TimecodeFormat::Smpte2997 => 30,
        TimecodeFormat::Smpte30 => 30,
        TimecodeFormat::Frames => 1,
    };
    let total_secs =
        (view.hours as u64) * 3600 + (view.minutes as u64) * 60 + (view.seconds as u64);
    total_secs * fps + (view.frames as u64)
}

/// Serialize to JSON-like string.
pub fn timecode_overlay_view_to_json(view: &TimecodeOverlayView) -> String {
    format!(
        r#"{{"timecode":"{}","enabled":{}}}"#,
        tcv_format_string(view),
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_timecode_overlay_view();
        assert_eq!(v.hours, 0);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_timecode() {
        let mut v = new_timecode_overlay_view();
        tcv_set_timecode(&mut v, 1, 30, 45, 12);
        assert_eq!(v.hours, 1);
        assert_eq!(v.frames, 12);
    }

    #[test]
    fn test_minutes_clamp() {
        let mut v = new_timecode_overlay_view();
        tcv_set_timecode(&mut v, 0, 100, 0, 0);
        assert_eq!(v.minutes, 59);
    }

    #[test]
    fn test_format_string() {
        let mut v = new_timecode_overlay_view();
        tcv_set_timecode(&mut v, 1, 2, 3, 4);
        assert_eq!(tcv_format_string(&v), "01:02:03:04");
    }

    #[test]
    fn test_total_frames_zero() {
        let v = new_timecode_overlay_view();
        assert_eq!(tcv_total_frames(&v), 0);
    }

    #[test]
    fn test_total_frames_one_second() {
        let mut v = new_timecode_overlay_view();
        tcv_set_timecode(&mut v, 0, 0, 1, 0);
        assert_eq!(tcv_total_frames(&v), 24);
    }

    #[test]
    fn test_position_clamp() {
        let mut v = new_timecode_overlay_view();
        tcv_set_position(&mut v, -1.0, 2.0);
        assert_eq!(v.position_x, 0.0);
        assert_eq!(v.position_y, 1.0);
    }

    #[test]
    fn test_json() {
        let v = new_timecode_overlay_view();
        let s = timecode_overlay_view_to_json(&v);
        assert!(s.contains("timecode"));
    }

    #[test]
    fn test_clone() {
        let v = new_timecode_overlay_view();
        let v2 = v.clone();
        assert_eq!(v2.format, v.format);
    }
}
