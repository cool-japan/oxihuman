// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Render target debug view — visualizes attachment formats, resolve, and clear state.

/// Render target debug view configuration.
#[derive(Debug, Clone)]
pub struct RenderTargetDebugView {
    pub enabled: bool,
    pub attachment_count: u32,
    pub width: u32,
    pub height: u32,
    pub show_depth_attachment: bool,
}

impl RenderTargetDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            attachment_count: 1,
            width: 1920,
            height: 1080,
            show_depth_attachment: true,
        }
    }
}

impl Default for RenderTargetDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new render target debug view.
pub fn new_render_target_debug_view() -> RenderTargetDebugView {
    RenderTargetDebugView::new()
}

/// Enable or disable render target debug overlay.
pub fn rtdv_set_enabled(v: &mut RenderTargetDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Set render target dimensions.
pub fn rtdv_set_dimensions(v: &mut RenderTargetDebugView, width: u32, height: u32) {
    v.width = width.max(1);
    v.height = height.max(1);
}

/// Set attachment count.
pub fn rtdv_set_attachment_count(v: &mut RenderTargetDebugView, count: u32) {
    v.attachment_count = count.clamp(1, 8);
}

/// Toggle depth attachment display.
pub fn rtdv_set_show_depth_attachment(v: &mut RenderTargetDebugView, show: bool) {
    v.show_depth_attachment = show;
}

/// Compute total pixel count across all color attachments.
pub fn rtdv_total_pixels(v: &RenderTargetDebugView) -> u64 {
    v.width as u64 * v.height as u64 * v.attachment_count as u64
}

/// Serialize to JSON-like string.
pub fn render_target_debug_view_to_json(v: &RenderTargetDebugView) -> String {
    format!(
        r#"{{"enabled":{},"attachment_count":{},"width":{},"height":{},"show_depth_attachment":{}}}"#,
        v.enabled, v.attachment_count, v.width, v.height, v.show_depth_attachment
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_render_target_debug_view();
        assert!(!v.enabled);
        assert_eq!(v.width, 1920);
        assert_eq!(v.height, 1080);
    }

    #[test]
    fn test_enable() {
        let mut v = new_render_target_debug_view();
        rtdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_dimensions() {
        let mut v = new_render_target_debug_view();
        rtdv_set_dimensions(&mut v, 2560, 1440);
        assert_eq!(v.width, 2560);
        assert_eq!(v.height, 1440);
    }

    #[test]
    fn test_dimensions_min() {
        let mut v = new_render_target_debug_view();
        rtdv_set_dimensions(&mut v, 0, 0);
        assert_eq!(v.width, 1);
        assert_eq!(v.height, 1);
    }

    #[test]
    fn test_attachment_count_clamp() {
        let mut v = new_render_target_debug_view();
        rtdv_set_attachment_count(&mut v, 20);
        assert_eq!(v.attachment_count, 8);
    }

    #[test]
    fn test_attachment_count_min() {
        let mut v = new_render_target_debug_view();
        rtdv_set_attachment_count(&mut v, 0);
        assert_eq!(v.attachment_count, 1);
    }

    #[test]
    fn test_show_depth_toggle() {
        let mut v = new_render_target_debug_view();
        rtdv_set_show_depth_attachment(&mut v, false);
        assert!(!v.show_depth_attachment);
    }

    #[test]
    fn test_total_pixels() {
        let mut v = new_render_target_debug_view();
        rtdv_set_dimensions(&mut v, 100, 100);
        rtdv_set_attachment_count(&mut v, 2);
        assert_eq!(rtdv_total_pixels(&v), 20000);
    }

    #[test]
    fn test_json_keys() {
        let v = new_render_target_debug_view();
        let s = render_target_debug_view_to_json(&v);
        assert!(s.contains("attachment_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_render_target_debug_view();
        let v2 = v.clone();
        assert_eq!(v2.width, v.width);
    }
}
