// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Render queue priority view — visualizes render queue depth and object sort order.

/// Render queue view configuration.
#[derive(Debug, Clone)]
pub struct RenderQueueView {
    pub enabled: bool,
    pub queue_depth: u32,
    pub show_priority_colors: bool,
    pub max_priority: u32,
    pub sort_key_bits: u8,
}

impl RenderQueueView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            queue_depth: 0,
            show_priority_colors: true,
            max_priority: 255,
            sort_key_bits: 64,
        }
    }
}

impl Default for RenderQueueView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new render queue view.
pub fn new_render_queue_view() -> RenderQueueView {
    RenderQueueView::new()
}

/// Enable or disable render queue debug overlay.
pub fn rqv_set_enabled(v: &mut RenderQueueView, enabled: bool) {
    v.enabled = enabled;
}

/// Update current queue depth.
pub fn rqv_set_queue_depth(v: &mut RenderQueueView, depth: u32) {
    v.queue_depth = depth;
}

/// Toggle priority color-coding of objects.
pub fn rqv_set_show_priority_colors(v: &mut RenderQueueView, show: bool) {
    v.show_priority_colors = show;
}

/// Set max priority value for normalization.
pub fn rqv_set_max_priority(v: &mut RenderQueueView, max: u32) {
    v.max_priority = max.max(1);
}

/// Normalize a priority value to 0–1 range.
pub fn rqv_normalize_priority(v: &RenderQueueView, priority: u32) -> f32 {
    (priority.min(v.max_priority) as f32 / v.max_priority as f32).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn render_queue_view_to_json(v: &RenderQueueView) -> String {
    format!(
        r#"{{"enabled":{},"queue_depth":{},"show_priority_colors":{},"max_priority":{},"sort_key_bits":{}}}"#,
        v.enabled, v.queue_depth, v.show_priority_colors, v.max_priority, v.sort_key_bits
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_render_queue_view();
        assert!(!v.enabled);
        assert_eq!(v.max_priority, 255);
    }

    #[test]
    fn test_enable() {
        let mut v = new_render_queue_view();
        rqv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_queue_depth_set() {
        let mut v = new_render_queue_view();
        rqv_set_queue_depth(&mut v, 500);
        assert_eq!(v.queue_depth, 500);
    }

    #[test]
    fn test_priority_colors_toggle() {
        let mut v = new_render_queue_view();
        rqv_set_show_priority_colors(&mut v, false);
        assert!(!v.show_priority_colors);
    }

    #[test]
    fn test_max_priority_min() {
        let mut v = new_render_queue_view();
        rqv_set_max_priority(&mut v, 0);
        assert_eq!(v.max_priority, 1);
    }

    #[test]
    fn test_normalize_priority_mid() {
        let v = new_render_queue_view();
        let n = rqv_normalize_priority(&v, 128);
        assert!(n > 0.0 && n < 1.0);
    }

    #[test]
    fn test_normalize_priority_max() {
        let v = new_render_queue_view();
        assert!((rqv_normalize_priority(&v, 255) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_priority_zero() {
        let v = new_render_queue_view();
        assert_eq!(rqv_normalize_priority(&v, 0), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_render_queue_view();
        let s = render_queue_view_to_json(&v);
        assert!(s.contains("sort_key_bits"));
    }

    #[test]
    fn test_clone() {
        let v = new_render_queue_view();
        let v2 = v.clone();
        assert_eq!(v2.max_priority, v.max_priority);
    }
}
