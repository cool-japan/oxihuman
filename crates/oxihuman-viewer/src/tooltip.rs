// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Tooltip display state.

#![allow(dead_code)]

/// Tooltip display state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Tooltip {
    /// Text to display in the tooltip.
    pub text: String,
    /// Screen position for the tooltip.
    pub position: [f32; 2],
    /// Whether the tooltip is currently visible.
    pub visible: bool,
    /// Number of ticks before the tooltip appears.
    pub delay_ticks: u32,
    /// Number of ticks the tooltip has been shown.
    pub shown_ticks: u32,
}

/// Create a new tooltip with the given text (initially hidden).
#[allow(dead_code)]
pub fn new_tooltip(text: &str) -> Tooltip {
    Tooltip {
        text: text.to_string(),
        position: [0.0, 0.0],
        visible: false,
        delay_ticks: 30,
        shown_ticks: 0,
    }
}

/// Show the tooltip at the given screen position.
#[allow(dead_code)]
pub fn show_tooltip(tooltip: &mut Tooltip, pos: [f32; 2]) {
    tooltip.position = pos;
    tooltip.visible = true;
    tooltip.shown_ticks = 0;
}

/// Hide the tooltip.
#[allow(dead_code)]
pub fn hide_tooltip(tooltip: &mut Tooltip) {
    tooltip.visible = false;
    tooltip.shown_ticks = 0;
}

/// Advance the tooltip timer by one tick.
#[allow(dead_code)]
pub fn tick_tooltip(tooltip: &mut Tooltip) {
    if tooltip.visible {
        tooltip.shown_ticks = tooltip.shown_ticks.saturating_add(1);
    }
}

/// Return the tooltip text.
#[allow(dead_code)]
pub fn tooltip_text(tooltip: &Tooltip) -> &str {
    &tooltip.text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tooltip() {
        let t = new_tooltip("Hello");
        assert_eq!(t.text, "Hello");
        assert!(!t.visible);
        assert_eq!(t.shown_ticks, 0);
    }

    #[test]
    fn test_show_tooltip() {
        let mut t = new_tooltip("Info");
        show_tooltip(&mut t, [50.0, 75.0]);
        assert!(t.visible);
        assert_eq!(t.position, [50.0, 75.0]);
    }

    #[test]
    fn test_hide_tooltip() {
        let mut t = new_tooltip("Info");
        show_tooltip(&mut t, [10.0, 20.0]);
        hide_tooltip(&mut t);
        assert!(!t.visible);
        assert_eq!(t.shown_ticks, 0);
    }

    #[test]
    fn test_tick_when_visible() {
        let mut t = new_tooltip("Tip");
        show_tooltip(&mut t, [0.0, 0.0]);
        tick_tooltip(&mut t);
        tick_tooltip(&mut t);
        assert_eq!(t.shown_ticks, 2);
    }

    #[test]
    fn test_no_tick_when_hidden() {
        let mut t = new_tooltip("Tip");
        tick_tooltip(&mut t);
        assert_eq!(t.shown_ticks, 0);
    }

    #[test]
    fn test_tooltip_text() {
        let t = new_tooltip("Press F to pay respects");
        assert_eq!(tooltip_text(&t), "Press F to pay respects");
    }

    #[test]
    fn test_show_resets_ticks() {
        let mut t = new_tooltip("X");
        show_tooltip(&mut t, [0.0, 0.0]);
        tick_tooltip(&mut t);
        tick_tooltip(&mut t);
        show_tooltip(&mut t, [1.0, 1.0]);
        assert_eq!(t.shown_ticks, 0);
    }

    #[test]
    fn test_delay_ticks_default() {
        let t = new_tooltip("X");
        assert_eq!(t.delay_ticks, 30);
    }

    #[test]
    fn test_empty_text() {
        let t = new_tooltip("");
        assert_eq!(tooltip_text(&t), "");
    }
}
