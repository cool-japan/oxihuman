// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tooltip rendering overlay for UI hints and annotations.

/// Anchor mode for tooltip positioning.
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum TooltipAnchor {
    Mouse,
    Fixed,
    WorldProjected,
}

/// Configuration for tooltip appearance.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TooltipConfig {
    pub font_size: f32,
    pub padding: f32,
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub fade_in_time: f32,
    pub fade_out_time: f32,
    pub max_width: f32,
}

/// A single tooltip instance.
#[allow(dead_code)]
pub struct Tooltip {
    pub text: String,
    pub anchor: TooltipAnchor,
    pub position: [f32; 2],
    pub visible: bool,
    pub timer: f32,
    pub fading_out: bool,
    pub config: TooltipConfig,
}

/// Type alias for tooltip bounding rectangle [x, y, w, h].
#[allow(dead_code)]
pub type TooltipRect = [f32; 4];

#[allow(dead_code)]
pub fn default_tooltip_config() -> TooltipConfig {
    TooltipConfig {
        font_size: 14.0,
        padding: 6.0,
        background_color: [0.1, 0.1, 0.1, 0.9],
        text_color: [1.0, 1.0, 1.0, 1.0],
        fade_in_time: 0.15,
        fade_out_time: 0.1,
        max_width: 300.0,
    }
}

#[allow(dead_code)]
pub fn new_tooltip(text: &str, anchor: TooltipAnchor) -> Tooltip {
    Tooltip {
        text: text.to_string(),
        anchor,
        position: [0.0, 0.0],
        visible: false,
        timer: 0.0,
        fading_out: false,
        config: default_tooltip_config(),
    }
}

#[allow(dead_code)]
pub fn show_tooltip(tooltip: &mut Tooltip) {
    tooltip.visible = true;
    tooltip.fading_out = false;
    tooltip.timer = 0.0;
}

#[allow(dead_code)]
pub fn hide_tooltip(tooltip: &mut Tooltip) {
    tooltip.fading_out = true;
    tooltip.timer = 0.0;
}

#[allow(dead_code)]
pub fn update_tooltip_position(tooltip: &mut Tooltip, x: f32, y: f32) {
    tooltip.position = [x, y];
}

#[allow(dead_code)]
pub fn tooltip_visible(tooltip: &Tooltip) -> bool {
    tooltip.visible
}

#[allow(dead_code)]
pub fn set_tooltip_text(tooltip: &mut Tooltip, text: &str) {
    tooltip.text = text.to_string();
}

/// Compute a bounding rectangle for the tooltip based on text length.
/// Uses a simple character-width estimate.
#[allow(dead_code)]
pub fn tooltip_bounds(tooltip: &Tooltip) -> TooltipRect {
    let char_width = tooltip.config.font_size * 0.6;
    let pad = tooltip.config.padding;
    let text_width = tooltip.text.len() as f32 * char_width;
    let w = text_width.min(tooltip.config.max_width) + pad * 2.0;
    let lines = if text_width > tooltip.config.max_width {
        (text_width / tooltip.config.max_width).ceil()
    } else {
        1.0
    };
    let h = lines * (tooltip.config.font_size + 2.0) + pad * 2.0;
    [tooltip.position[0], tooltip.position[1], w, h]
}

/// Compute current fade alpha (0.0 = invisible, 1.0 = fully visible).
#[allow(dead_code)]
pub fn tooltip_fade_alpha(tooltip: &Tooltip) -> f32 {
    if !tooltip.visible {
        return 0.0;
    }
    if tooltip.fading_out {
        let t = tooltip.config.fade_out_time;
        if t <= 0.0 {
            return 0.0;
        }
        (1.0 - tooltip.timer / t).clamp(0.0, 1.0)
    } else {
        let t = tooltip.config.fade_in_time;
        if t <= 0.0 {
            return 1.0;
        }
        (tooltip.timer / t).clamp(0.0, 1.0)
    }
}

/// Advance tooltip timer by dt seconds. Handles fade completion.
#[allow(dead_code)]
pub fn advance_tooltip(tooltip: &mut Tooltip, dt: f32) {
    if !tooltip.visible {
        return;
    }
    tooltip.timer += dt;
    if tooltip.fading_out && tooltip.timer >= tooltip.config.fade_out_time {
        tooltip.visible = false;
        tooltip.fading_out = false;
        tooltip.timer = 0.0;
    }
}

/// Check if a tooltip would contain the given screen position.
#[allow(dead_code)]
pub fn tooltip_at_screen_pos(tooltip: &Tooltip, x: f32, y: f32) -> bool {
    if !tooltip.visible {
        return false;
    }
    let bounds = tooltip_bounds(tooltip);
    x >= bounds[0] && x < bounds[0] + bounds[2] && y >= bounds[1] && y < bounds[1] + bounds[3]
}

#[allow(dead_code)]
pub fn tooltip_background_color(tooltip: &Tooltip) -> [f32; 4] {
    tooltip.config.background_color
}

#[allow(dead_code)]
pub fn tooltip_text_color(tooltip: &Tooltip) -> [f32; 4] {
    tooltip.config.text_color
}

/// Clear all tooltips in a collection by hiding them.
#[allow(dead_code)]
pub fn clear_all_tooltips(tooltips: &mut [Tooltip]) {
    for tt in tooltips.iter_mut() {
        tt.visible = false;
        tt.fading_out = false;
        tt.timer = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_tooltip_config();
        assert!((cfg.font_size - 14.0).abs() < f32::EPSILON);
        assert!((cfg.padding - 6.0).abs() < f32::EPSILON);
        assert!((cfg.fade_in_time - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_new_tooltip() {
        let tt = new_tooltip("Hello", TooltipAnchor::Mouse);
        assert_eq!(tt.text, "Hello");
        assert_eq!(tt.anchor, TooltipAnchor::Mouse);
        assert!(!tt.visible);
    }

    #[test]
    fn test_show_tooltip() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Fixed);
        show_tooltip(&mut tt);
        assert!(tooltip_visible(&tt));
    }

    #[test]
    fn test_hide_tooltip() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Fixed);
        show_tooltip(&mut tt);
        hide_tooltip(&mut tt);
        assert!(tt.fading_out);
    }

    #[test]
    fn test_update_position() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Mouse);
        update_tooltip_position(&mut tt, 100.0, 200.0);
        assert!((tt.position[0] - 100.0).abs() < f32::EPSILON);
        assert!((tt.position[1] - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_text() {
        let mut tt = new_tooltip("Old", TooltipAnchor::Fixed);
        set_tooltip_text(&mut tt, "New");
        assert_eq!(tt.text, "New");
    }

    #[test]
    fn test_tooltip_bounds_positive() {
        let mut tt = new_tooltip("Hello World", TooltipAnchor::Mouse);
        update_tooltip_position(&mut tt, 50.0, 50.0);
        let bounds = tooltip_bounds(&tt);
        assert!(bounds[2] > 0.0);
        assert!(bounds[3] > 0.0);
    }

    #[test]
    fn test_tooltip_fade_alpha_not_visible() {
        let tt = new_tooltip("Test", TooltipAnchor::Fixed);
        assert!((tooltip_fade_alpha(&tt) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tooltip_fade_alpha_fade_in() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Fixed);
        show_tooltip(&mut tt);
        // Timer at 0, fade_in_time = 0.15 => alpha ~ 0
        let a0 = tooltip_fade_alpha(&tt);
        assert!(a0 < 0.01);
        // Advance half the fade time
        advance_tooltip(&mut tt, 0.075);
        let a1 = tooltip_fade_alpha(&tt);
        assert!((a1 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_tooltip_fade_alpha_full() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Fixed);
        show_tooltip(&mut tt);
        advance_tooltip(&mut tt, 1.0); // well past fade_in_time
        let a = tooltip_fade_alpha(&tt);
        assert!((a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_advance_tooltip_fade_out_completes() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Fixed);
        show_tooltip(&mut tt);
        advance_tooltip(&mut tt, 1.0);
        hide_tooltip(&mut tt);
        advance_tooltip(&mut tt, 1.0); // past fade_out_time
        assert!(!tooltip_visible(&tt));
    }

    #[test]
    fn test_tooltip_at_screen_pos() {
        let mut tt = new_tooltip("Hello", TooltipAnchor::Fixed);
        update_tooltip_position(&mut tt, 10.0, 10.0);
        show_tooltip(&mut tt);
        assert!(tooltip_at_screen_pos(&tt, 15.0, 15.0));
        assert!(!tooltip_at_screen_pos(&tt, 500.0, 500.0));
    }

    #[test]
    fn test_tooltip_at_screen_pos_not_visible() {
        let tt = new_tooltip("Hello", TooltipAnchor::Fixed);
        assert!(!tooltip_at_screen_pos(&tt, 0.0, 0.0));
    }

    #[test]
    fn test_background_color() {
        let tt = new_tooltip("Test", TooltipAnchor::Fixed);
        let bg = tooltip_background_color(&tt);
        assert!((bg[3] - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_text_color() {
        let tt = new_tooltip("Test", TooltipAnchor::Fixed);
        let tc = tooltip_text_color(&tt);
        assert!((tc[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear_all_tooltips() {
        let mut tooltips = vec![
            new_tooltip("A", TooltipAnchor::Mouse),
            new_tooltip("B", TooltipAnchor::Fixed),
        ];
        show_tooltip(&mut tooltips[0]);
        show_tooltip(&mut tooltips[1]);
        clear_all_tooltips(&mut tooltips);
        for tt in &tooltips {
            assert!(!tooltip_visible(tt));
        }
    }

    #[test]
    fn test_advance_not_visible_is_noop() {
        let mut tt = new_tooltip("Test", TooltipAnchor::Fixed);
        advance_tooltip(&mut tt, 1.0);
        assert!((tt.timer - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tooltip_bounds_empty_text() {
        let tt = new_tooltip("", TooltipAnchor::Fixed);
        let bounds = tooltip_bounds(&tt);
        // Width should be just padding
        assert!((bounds[2] - tt.config.padding * 2.0).abs() < f32::EPSILON);
    }
}
