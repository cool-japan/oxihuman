// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Debug text overlay for the 3D viewer.

/// Text alignment.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// A debug text entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugTextEntry {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: [f32; 4],
    pub align: TextAlign,
}

/// Debug text buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugTextBuffer {
    pub entries: Vec<DebugTextEntry>,
    pub visible: bool,
}

/// Create a new debug text entry.
#[allow(dead_code)]
pub fn new_debug_text(text: &str, x: f32, y: f32) -> DebugTextEntry {
    DebugTextEntry {
        text: text.to_string(),
        x,
        y,
        size: 14.0,
        color: [1.0, 1.0, 1.0, 1.0],
        align: TextAlign::Left,
    }
}

/// Create empty buffer.
#[allow(dead_code)]
pub fn new_debug_text_buffer() -> DebugTextBuffer {
    DebugTextBuffer {
        entries: Vec::new(),
        visible: true,
    }
}

/// Add text entry.
#[allow(dead_code)]
pub fn add_debug_text(buf: &mut DebugTextBuffer, entry: DebugTextEntry) {
    buf.entries.push(entry);
}

/// Clear all text.
#[allow(dead_code)]
pub fn clear_debug_text(buf: &mut DebugTextBuffer) {
    buf.entries.clear();
}

/// Entry count.
#[allow(dead_code)]
pub fn debug_text_count(buf: &DebugTextBuffer) -> usize {
    buf.entries.len()
}

/// Toggle visibility.
#[allow(dead_code)]
pub fn toggle_debug_text(buf: &mut DebugTextBuffer) {
    buf.visible = !buf.visible;
}

/// Add a key-value line.
#[allow(dead_code)]
pub fn add_debug_key_value(buf: &mut DebugTextBuffer, key: &str, value: &str, y: f32) {
    let text = format!("{key}: {value}");
    add_debug_text(buf, new_debug_text(&text, 10.0, y));
}

/// Set text color for all entries.
#[allow(dead_code)]
pub fn set_debug_text_color(buf: &mut DebugTextBuffer, color: [f32; 4]) {
    for entry in &mut buf.entries {
        entry.color = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry() {
        let e = new_debug_text("hello", 10.0, 20.0);
        assert_eq!(e.text, "hello");
    }

    #[test]
    fn test_new_buffer() {
        let b = new_debug_text_buffer();
        assert!(b.entries.is_empty());
    }

    #[test]
    fn test_add() {
        let mut b = new_debug_text_buffer();
        add_debug_text(&mut b, new_debug_text("test", 0.0, 0.0));
        assert_eq!(debug_text_count(&b), 1);
    }

    #[test]
    fn test_clear() {
        let mut b = new_debug_text_buffer();
        add_debug_text(&mut b, new_debug_text("test", 0.0, 0.0));
        clear_debug_text(&mut b);
        assert_eq!(debug_text_count(&b), 0);
    }

    #[test]
    fn test_toggle() {
        let mut b = new_debug_text_buffer();
        assert!(b.visible);
        toggle_debug_text(&mut b);
        assert!(!b.visible);
    }

    #[test]
    fn test_key_value() {
        let mut b = new_debug_text_buffer();
        add_debug_key_value(&mut b, "FPS", "60", 10.0);
        assert_eq!(debug_text_count(&b), 1);
        assert!(b.entries[0].text.contains("FPS"));
    }

    #[test]
    fn test_set_color() {
        let mut b = new_debug_text_buffer();
        add_debug_text(&mut b, new_debug_text("test", 0.0, 0.0));
        set_debug_text_color(&mut b, [1.0, 0.0, 0.0, 1.0]);
        assert!((b.entries[0].color[0] - 1.0).abs() < 1e-6);
        assert!(b.entries[0].color[1].abs() < 1e-6);
    }

    #[test]
    fn test_default_size() {
        let e = new_debug_text("x", 0.0, 0.0);
        assert!((e.size - 14.0).abs() < 1e-6);
    }

    #[test]
    fn test_default_align() {
        let e = new_debug_text("x", 0.0, 0.0);
        assert_eq!(e.align, TextAlign::Left);
    }

    #[test]
    fn test_multiple_entries() {
        let mut b = new_debug_text_buffer();
        for i in 0..5 {
            add_debug_text(&mut b, new_debug_text(&format!("line {i}"), 0.0, i as f32 * 20.0));
        }
        assert_eq!(debug_text_count(&b), 5);
    }
}
