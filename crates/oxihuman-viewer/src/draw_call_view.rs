// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draw call batching debug view — visualizes draw call counts and batching efficiency.

/// Draw call view configuration.
#[derive(Debug, Clone)]
pub struct DrawCallView {
    pub enabled: bool,
    pub total_draw_calls: u32,
    pub batched_draw_calls: u32,
    pub show_batch_boundaries: bool,
    pub warn_threshold: u32,
}

impl DrawCallView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_draw_calls: 0,
            batched_draw_calls: 0,
            show_batch_boundaries: false,
            warn_threshold: 1000,
        }
    }
}

impl Default for DrawCallView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new draw call view.
pub fn new_draw_call_view() -> DrawCallView {
    DrawCallView::new()
}

/// Enable or disable draw call debug overlay.
pub fn dcv_set_enabled(v: &mut DrawCallView, enabled: bool) {
    v.enabled = enabled;
}

/// Update draw call counts for current frame.
pub fn dcv_update_counts(v: &mut DrawCallView, total: u32, batched: u32) {
    v.total_draw_calls = total;
    v.batched_draw_calls = batched.min(total);
}

/// Set warning threshold for draw call count.
pub fn dcv_set_warn_threshold(v: &mut DrawCallView, threshold: u32) {
    v.warn_threshold = threshold.max(1);
}

/// Toggle batch boundary visualization.
pub fn dcv_set_show_batch_boundaries(v: &mut DrawCallView, show: bool) {
    v.show_batch_boundaries = show;
}

/// Compute batching efficiency (0 = no batching, 1 = all batched).
pub fn dcv_batch_efficiency(v: &DrawCallView) -> f32 {
    if v.total_draw_calls == 0 {
        return 1.0;
    }
    (v.batched_draw_calls as f32 / v.total_draw_calls as f32).clamp(0.0, 1.0)
}

/// Returns true if draw calls exceed warning threshold.
pub fn dcv_is_over_threshold(v: &DrawCallView) -> bool {
    v.total_draw_calls > v.warn_threshold
}

/// Serialize to JSON-like string.
pub fn draw_call_view_to_json(v: &DrawCallView) -> String {
    format!(
        r#"{{"enabled":{},"total_draw_calls":{},"batched_draw_calls":{},"show_batch_boundaries":{},"warn_threshold":{}}}"#,
        v.enabled,
        v.total_draw_calls,
        v.batched_draw_calls,
        v.show_batch_boundaries,
        v.warn_threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_draw_call_view();
        assert!(!v.enabled);
        assert_eq!(v.warn_threshold, 1000);
    }

    #[test]
    fn test_enable() {
        let mut v = new_draw_call_view();
        dcv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_counts() {
        let mut v = new_draw_call_view();
        dcv_update_counts(&mut v, 500, 300);
        assert_eq!(v.total_draw_calls, 500);
        assert_eq!(v.batched_draw_calls, 300);
    }

    #[test]
    fn test_batched_capped_at_total() {
        let mut v = new_draw_call_view();
        dcv_update_counts(&mut v, 100, 200);
        assert_eq!(v.batched_draw_calls, 100);
    }

    #[test]
    fn test_warn_threshold_min() {
        let mut v = new_draw_call_view();
        dcv_set_warn_threshold(&mut v, 0);
        assert_eq!(v.warn_threshold, 1);
    }

    #[test]
    fn test_batch_efficiency_zero_total() {
        let v = new_draw_call_view();
        assert_eq!(dcv_batch_efficiency(&v), 1.0);
    }

    #[test]
    fn test_batch_efficiency_half() {
        let mut v = new_draw_call_view();
        dcv_update_counts(&mut v, 100, 50);
        assert!((dcv_batch_efficiency(&v) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_over_threshold_false() {
        let v = new_draw_call_view();
        assert!(!dcv_is_over_threshold(&v));
    }

    #[test]
    fn test_json_keys() {
        let v = new_draw_call_view();
        let s = draw_call_view_to_json(&v);
        assert!(s.contains("warn_threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_draw_call_view();
        let v2 = v.clone();
        assert_eq!(v2.warn_threshold, v.warn_threshold);
    }
}
