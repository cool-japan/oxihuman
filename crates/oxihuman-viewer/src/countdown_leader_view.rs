// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Countdown leader view for pre-roll sequence display.

/// Countdown leader view configuration.
#[derive(Debug, Clone)]
pub struct CountdownLeaderView {
    pub current_count: i32,
    pub start_count: i32,
    pub frame_rate: f32,
    pub show_beep_flash: bool,
    pub enabled: bool,
    pub elapsed_frames: u32,
}

impl CountdownLeaderView {
    pub fn new() -> Self {
        Self {
            current_count: 8,
            start_count: 8,
            frame_rate: 24.0,
            show_beep_flash: true,
            enabled: false,
            elapsed_frames: 0,
        }
    }
}

impl Default for CountdownLeaderView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new countdown leader view.
pub fn new_countdown_leader_view() -> CountdownLeaderView {
    CountdownLeaderView::new()
}

/// Reset countdown to its start value.
pub fn clv_reset(view: &mut CountdownLeaderView) {
    view.current_count = view.start_count;
    view.elapsed_frames = 0;
}

/// Advance countdown by one frame; returns true when expired.
pub fn clv_advance_frame(view: &mut CountdownLeaderView) -> bool {
    view.elapsed_frames = view.elapsed_frames.saturating_add(1);
    let frames_per_count = view.frame_rate.round() as u32;
    if frames_per_count > 0 && view.elapsed_frames.is_multiple_of(frames_per_count) {
        view.current_count -= 1;
    }
    view.current_count <= 0
}

/// Set the starting count value.
pub fn clv_set_start_count(view: &mut CountdownLeaderView, count: i32) {
    view.start_count = count.max(1);
    view.current_count = view.start_count;
}

/// Set frame rate for count timing.
pub fn clv_set_frame_rate(view: &mut CountdownLeaderView, fps: f32) {
    view.frame_rate = fps.clamp(1.0, 240.0);
}

/// Compute remaining time in seconds.
pub fn clv_remaining_seconds(view: &CountdownLeaderView) -> f32 {
    if view.frame_rate < 1e-6 {
        return 0.0;
    }
    (view.current_count as f32).max(0.0)
}

/// Serialize to JSON-like string.
pub fn countdown_leader_view_to_json(view: &CountdownLeaderView) -> String {
    format!(
        r#"{{"current_count":{},"start_count":{},"frame_rate":{:.2},"enabled":{}}}"#,
        view.current_count, view.start_count, view.frame_rate, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_countdown_leader_view();
        assert_eq!(v.current_count, 8);
        assert!(!v.enabled);
    }

    #[test]
    fn test_reset() {
        let mut v = new_countdown_leader_view();
        v.current_count = 3;
        clv_reset(&mut v);
        assert_eq!(v.current_count, 8);
        assert_eq!(v.elapsed_frames, 0);
    }

    #[test]
    fn test_set_start_count() {
        let mut v = new_countdown_leader_view();
        clv_set_start_count(&mut v, 5);
        assert_eq!(v.start_count, 5);
        assert_eq!(v.current_count, 5);
    }

    #[test]
    fn test_set_start_count_min() {
        let mut v = new_countdown_leader_view();
        clv_set_start_count(&mut v, 0);
        assert_eq!(v.start_count, 1);
    }

    #[test]
    fn test_frame_rate_clamp() {
        let mut v = new_countdown_leader_view();
        clv_set_frame_rate(&mut v, 0.0);
        assert!((v.frame_rate - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_remaining_seconds() {
        let v = new_countdown_leader_view();
        assert!((clv_remaining_seconds(&v) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_advance_many_frames_expires() {
        let mut v = new_countdown_leader_view();
        clv_set_start_count(&mut v, 1);
        clv_set_frame_rate(&mut v, 1.0);
        let done = clv_advance_frame(&mut v);
        assert!(done);
    }

    #[test]
    fn test_json() {
        let v = new_countdown_leader_view();
        let s = countdown_leader_view_to_json(&v);
        assert!(s.contains("current_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_countdown_leader_view();
        let v2 = v.clone();
        assert_eq!(v2.start_count, v.start_count);
    }
}
