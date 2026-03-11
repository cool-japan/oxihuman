// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CursorRenderer — 3-D viewport cursor.

#![allow(dead_code)]

/// Visual style of the viewport cursor.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Default,
    Crosshair,
    Dot,
    Ring,
}

/// A viewport cursor state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViewportCursor {
    pub x: f32,
    pub y: f32,
    pub style: CursorStyle,
    pub visible: bool,
    pub color: [f32; 4],
    pub size: f32,
}

impl Default for ViewportCursor {
    fn default() -> Self {
        ViewportCursor {
            x: 0.0,
            y: 0.0,
            style: CursorStyle::Default,
            visible: true,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 16.0,
        }
    }
}

/// Create a new `ViewportCursor` at (0, 0).
#[allow(dead_code)]
pub fn new_viewport_cursor() -> ViewportCursor {
    ViewportCursor::default()
}

/// Set the cursor screen position.
#[allow(dead_code)]
pub fn set_cursor_pos(cursor: &mut ViewportCursor, x: f32, y: f32) {
    cursor.x = x;
    cursor.y = y;
}

/// Set the cursor style.
#[allow(dead_code)]
pub fn set_cursor_style(cursor: &mut ViewportCursor, style: CursorStyle) {
    cursor.style = style;
}

/// Return whether the cursor is visible.
#[allow(dead_code)]
pub fn cursor_is_visible(cursor: &ViewportCursor) -> bool {
    cursor.visible
}

/// Return the cursor color.
#[allow(dead_code)]
pub fn cursor_color(cursor: &ViewportCursor) -> [f32; 4] {
    cursor.color
}

/// Return the cursor size in pixels.
#[allow(dead_code)]
pub fn cursor_size(cursor: &ViewportCursor) -> f32 {
    cursor.size
}

/// Convert screen position to normalised device coordinates.
#[allow(dead_code)]
pub fn cursor_to_screen_pos(cursor: &ViewportCursor, viewport_w: f32, viewport_h: f32) -> [f32; 2] {
    [
        (cursor.x / viewport_w.max(f32::EPSILON)) * 2.0 - 1.0,
        1.0 - (cursor.y / viewport_h.max(f32::EPSILON)) * 2.0,
    ]
}

/// Return a simple ray direction from the cursor position (stub: always toward -Z).
#[allow(dead_code)]
pub fn cursor_ray_dir(_cursor: &ViewportCursor) -> [f32; 3] {
    [0.0, 0.0, -1.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cursor_default() {
        let c = new_viewport_cursor();
        assert!(cursor_is_visible(&c));
        assert!((cursor_size(&c) - 16.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_cursor_pos() {
        let mut c = new_viewport_cursor();
        set_cursor_pos(&mut c, 100.0, 200.0);
        assert!((c.x - 100.0).abs() < 1e-6);
        assert!((c.y - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_cursor_style() {
        let mut c = new_viewport_cursor();
        set_cursor_style(&mut c, CursorStyle::Crosshair);
        assert_eq!(c.style, CursorStyle::Crosshair);
    }

    #[test]
    fn test_cursor_color_default() {
        let c = new_viewport_cursor();
        let col = cursor_color(&c);
        assert!((col[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cursor_to_screen_pos_center() {
        let mut c = new_viewport_cursor();
        set_cursor_pos(&mut c, 400.0, 300.0);
        let ndc = cursor_to_screen_pos(&c, 800.0, 600.0);
        assert!((ndc[0]).abs() < 1e-5);
        assert!((ndc[1]).abs() < 1e-5);
    }

    #[test]
    fn test_cursor_ray_dir() {
        let c = new_viewport_cursor();
        let d = cursor_ray_dir(&c);
        assert!((d[2] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cursor_style_enum() {
        assert_ne!(CursorStyle::Default, CursorStyle::Dot);
        assert_eq!(CursorStyle::Ring, CursorStyle::Ring);
    }

    #[test]
    fn test_cursor_not_visible_when_set() {
        let mut c = new_viewport_cursor();
        c.visible = false;
        assert!(!cursor_is_visible(&c));
    }
}
