// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Split/multi-viewport viewer with configurable layouts.

/// Layout mode for the split view.
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum SplitLayout {
    Single,
    Horizontal,
    Vertical,
    Quad,
}

/// A single viewport pane with pixel-space rectangle.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ViewportPane {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
}

/// Configuration for the split view system.
#[allow(dead_code)]
pub struct SplitViewConfig {
    pub layout: SplitLayout,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub split_ratio: f32,
    pub active_pane: usize,
    pub maximized_pane: Option<usize>,
    pub gap: f32,
}

/// Type alias for pane rectangle [x, y, w, h].
#[allow(dead_code)]
pub type PaneRect = [f32; 4];

#[allow(dead_code)]
pub fn default_split_view_config(width: f32, height: f32) -> SplitViewConfig {
    SplitViewConfig {
        layout: SplitLayout::Single,
        viewport_width: width,
        viewport_height: height,
        split_ratio: 0.5,
        active_pane: 0,
        maximized_pane: None,
        gap: 2.0,
    }
}

#[allow(dead_code)]
pub fn new_split_view(width: f32, height: f32, layout: SplitLayout) -> SplitViewConfig {
    SplitViewConfig {
        layout,
        viewport_width: width,
        viewport_height: height,
        split_ratio: 0.5,
        active_pane: 0,
        maximized_pane: None,
        gap: 2.0,
    }
}

#[allow(dead_code)]
pub fn set_layout(config: &mut SplitViewConfig, layout: SplitLayout) {
    config.layout = layout;
    config.active_pane = 0;
    config.maximized_pane = None;
}

#[allow(dead_code)]
pub fn pane_count(config: &SplitViewConfig) -> usize {
    match config.layout {
        SplitLayout::Single => 1,
        SplitLayout::Horizontal | SplitLayout::Vertical => 2,
        SplitLayout::Quad => 4,
    }
}

/// Compute pixel rectangle for a given pane index.
#[allow(dead_code)]
pub fn pane_rect(config: &SplitViewConfig, index: usize) -> PaneRect {
    // If a pane is maximized, only that pane gets the full viewport
    if let Some(max_idx) = config.maximized_pane {
        if index == max_idx {
            return [0.0, 0.0, config.viewport_width, config.viewport_height];
        } else {
            return [0.0, 0.0, 0.0, 0.0];
        }
    }

    let w = config.viewport_width;
    let h = config.viewport_height;
    let r = config.split_ratio;
    let g = config.gap;
    let half_gap = g * 0.5;

    match config.layout {
        SplitLayout::Single => [0.0, 0.0, w, h],
        SplitLayout::Horizontal => {
            let split_x = w * r;
            match index {
                0 => [0.0, 0.0, split_x - half_gap, h],
                1 => [split_x + half_gap, 0.0, w - split_x - half_gap, h],
                _ => [0.0, 0.0, 0.0, 0.0],
            }
        }
        SplitLayout::Vertical => {
            let split_y = h * r;
            match index {
                0 => [0.0, 0.0, w, split_y - half_gap],
                1 => [0.0, split_y + half_gap, w, h - split_y - half_gap],
                _ => [0.0, 0.0, 0.0, 0.0],
            }
        }
        SplitLayout::Quad => {
            let split_x = w * r;
            let split_y = h * r;
            match index {
                0 => [0.0, 0.0, split_x - half_gap, split_y - half_gap],
                1 => [
                    split_x + half_gap,
                    0.0,
                    w - split_x - half_gap,
                    split_y - half_gap,
                ],
                2 => [
                    0.0,
                    split_y + half_gap,
                    split_x - half_gap,
                    h - split_y - half_gap,
                ],
                3 => [
                    split_x + half_gap,
                    split_y + half_gap,
                    w - split_x - half_gap,
                    h - split_y - half_gap,
                ],
                _ => [0.0, 0.0, 0.0, 0.0],
            }
        }
    }
}

#[allow(dead_code)]
pub fn active_pane(config: &SplitViewConfig) -> usize {
    config.active_pane
}

#[allow(dead_code)]
pub fn set_active_pane(config: &mut SplitViewConfig, index: usize) {
    if index < pane_count(config) {
        config.active_pane = index;
    }
}

/// Move the divider by changing the split ratio.
#[allow(dead_code)]
pub fn resize_split(config: &mut SplitViewConfig, ratio: f32) {
    config.split_ratio = ratio.clamp(0.1, 0.9);
}

#[allow(dead_code)]
pub fn split_ratio(config: &SplitViewConfig) -> f32 {
    config.split_ratio
}

/// Find which pane a screen position falls in.
#[allow(dead_code)]
pub fn pane_at_position(config: &SplitViewConfig, x: f32, y: f32) -> Option<usize> {
    let count = pane_count(config);
    for i in 0..count {
        let rect = pane_rect(config, i);
        let rx = rect[0];
        let ry = rect[1];
        let rw = rect[2];
        let rh = rect[3];
        if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
            return Some(i);
        }
    }
    None
}

/// Human-readable name of the layout.
#[allow(dead_code)]
pub fn layout_name(config: &SplitViewConfig) -> &'static str {
    match config.layout {
        SplitLayout::Single => "Single",
        SplitLayout::Horizontal => "Horizontal",
        SplitLayout::Vertical => "Vertical",
        SplitLayout::Quad => "Quad",
    }
}

/// Serialize split view config to a JSON string.
#[allow(dead_code)]
pub fn split_view_to_json(config: &SplitViewConfig) -> String {
    let layout_str = match config.layout {
        SplitLayout::Single => "single",
        SplitLayout::Horizontal => "horizontal",
        SplitLayout::Vertical => "vertical",
        SplitLayout::Quad => "quad",
    };
    let max_str = match config.maximized_pane {
        Some(idx) => format!("{}", idx),
        None => "null".to_string(),
    };
    format!(
        "{{\"layout\":\"{}\",\"viewport_width\":{},\"viewport_height\":{},\"split_ratio\":{},\"active_pane\":{},\"maximized_pane\":{},\"gap\":{}}}",
        layout_str, config.viewport_width, config.viewport_height,
        config.split_ratio, config.active_pane, max_str, config.gap
    )
}

/// Toggle maximize for a pane. If already maximized, restore.
#[allow(dead_code)]
pub fn toggle_maximize_pane(config: &mut SplitViewConfig, index: usize) {
    if config.maximized_pane == Some(index) {
        config.maximized_pane = None;
    } else if index < pane_count(config) {
        config.maximized_pane = Some(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_split_view_config(800.0, 600.0);
        assert_eq!(cfg.layout, SplitLayout::Single);
        assert!((cfg.viewport_width - 800.0).abs() < f32::EPSILON);
        assert!((cfg.viewport_height - 600.0).abs() < f32::EPSILON);
        assert!((cfg.split_ratio - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_new_split_view() {
        let cfg = new_split_view(1024.0, 768.0, SplitLayout::Horizontal);
        assert_eq!(cfg.layout, SplitLayout::Horizontal);
        assert_eq!(cfg.active_pane, 0);
    }

    #[test]
    fn test_set_layout() {
        let mut cfg = default_split_view_config(800.0, 600.0);
        set_layout(&mut cfg, SplitLayout::Quad);
        assert_eq!(cfg.layout, SplitLayout::Quad);
        assert_eq!(cfg.active_pane, 0);
    }

    #[test]
    fn test_pane_count_single() {
        let cfg = default_split_view_config(800.0, 600.0);
        assert_eq!(pane_count(&cfg), 1);
    }

    #[test]
    fn test_pane_count_horizontal() {
        let cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        assert_eq!(pane_count(&cfg), 2);
    }

    #[test]
    fn test_pane_count_quad() {
        let cfg = new_split_view(800.0, 600.0, SplitLayout::Quad);
        assert_eq!(pane_count(&cfg), 4);
    }

    #[test]
    fn test_pane_rect_single() {
        let cfg = default_split_view_config(800.0, 600.0);
        let rect = pane_rect(&cfg, 0);
        assert!((rect[0] - 0.0).abs() < f32::EPSILON);
        assert!((rect[1] - 0.0).abs() < f32::EPSILON);
        assert!((rect[2] - 800.0).abs() < f32::EPSILON);
        assert!((rect[3] - 600.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pane_rect_horizontal() {
        let cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        let r0 = pane_rect(&cfg, 0);
        let r1 = pane_rect(&cfg, 1);
        assert!(r0[2] > 0.0);
        assert!(r1[2] > 0.0);
        // Together they should roughly span the viewport width
        assert!((r0[2] + r1[2] + cfg.gap - 800.0).abs() < 1.0);
    }

    #[test]
    fn test_active_pane() {
        let cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        assert_eq!(active_pane(&cfg), 0);
    }

    #[test]
    fn test_set_active_pane() {
        let mut cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        set_active_pane(&mut cfg, 1);
        assert_eq!(active_pane(&cfg), 1);
    }

    #[test]
    fn test_set_active_pane_out_of_range() {
        let mut cfg = default_split_view_config(800.0, 600.0);
        set_active_pane(&mut cfg, 5);
        assert_eq!(active_pane(&cfg), 0); // unchanged
    }

    #[test]
    fn test_resize_split() {
        let mut cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        resize_split(&mut cfg, 0.3);
        assert!((split_ratio(&cfg) - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_resize_split_clamped() {
        let mut cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        resize_split(&mut cfg, 0.0);
        assert!((split_ratio(&cfg) - 0.1).abs() < f32::EPSILON);
        resize_split(&mut cfg, 1.0);
        assert!((split_ratio(&cfg) - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pane_at_position_single() {
        let cfg = default_split_view_config(800.0, 600.0);
        assert_eq!(pane_at_position(&cfg, 400.0, 300.0), Some(0));
    }

    #[test]
    fn test_pane_at_position_horizontal() {
        let cfg = new_split_view(800.0, 600.0, SplitLayout::Horizontal);
        // Left half
        assert_eq!(pane_at_position(&cfg, 100.0, 300.0), Some(0));
        // Right half
        assert_eq!(pane_at_position(&cfg, 600.0, 300.0), Some(1));
    }

    #[test]
    fn test_layout_name() {
        let cfg = new_split_view(800.0, 600.0, SplitLayout::Vertical);
        assert_eq!(layout_name(&cfg), "Vertical");
    }

    #[test]
    fn test_split_view_to_json() {
        let cfg = default_split_view_config(800.0, 600.0);
        let json = split_view_to_json(&cfg);
        assert!(json.contains("\"layout\":\"single\""));
        assert!(json.contains("\"maximized_pane\":null"));
    }

    #[test]
    fn test_toggle_maximize_pane() {
        let mut cfg = new_split_view(800.0, 600.0, SplitLayout::Quad);
        toggle_maximize_pane(&mut cfg, 2);
        assert_eq!(cfg.maximized_pane, Some(2));
        // Maximized pane gets full rect
        let rect = pane_rect(&cfg, 2);
        assert!((rect[2] - 800.0).abs() < f32::EPSILON);
        // Other pane gets zero rect
        let rect0 = pane_rect(&cfg, 0);
        assert!((rect0[2] - 0.0).abs() < f32::EPSILON);
        // Toggle again to restore
        toggle_maximize_pane(&mut cfg, 2);
        assert_eq!(cfg.maximized_pane, None);
    }

    #[test]
    fn test_pane_at_position_out_of_bounds() {
        let cfg = default_split_view_config(800.0, 600.0);
        assert_eq!(pane_at_position(&cfg, -10.0, -10.0), None);
    }
}
