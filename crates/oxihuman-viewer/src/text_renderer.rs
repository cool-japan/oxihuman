// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! On-screen text rendering stub.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub color: [f32; 4],
    pub bold: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextEntry {
    pub text: String,
    pub position: [f32; 2],
    pub style: TextStyle,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextRenderer {
    entries: Vec<TextEntry>,
}

#[allow(dead_code)]
pub fn default_text_style() -> TextStyle {
    TextStyle { font_size: 14.0, color: [1.0, 1.0, 1.0, 1.0], bold: false }
}

#[allow(dead_code)]
pub fn new_text_renderer() -> TextRenderer {
    TextRenderer { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn tr_add_text(renderer: &mut TextRenderer, entry: TextEntry) {
    renderer.entries.push(entry);
}

#[allow(dead_code)]
pub fn tr_clear(renderer: &mut TextRenderer) {
    renderer.entries.clear();
}

#[allow(dead_code)]
pub fn tr_count(renderer: &TextRenderer) -> usize {
    renderer.entries.len()
}

#[allow(dead_code)]
pub fn tr_get(renderer: &TextRenderer, index: usize) -> Option<&TextEntry> {
    renderer.entries.get(index)
}

#[allow(dead_code)]
pub fn tr_remove(renderer: &mut TextRenderer, index: usize) -> bool {
    if index < renderer.entries.len() {
        renderer.entries.remove(index);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn tr_to_json(renderer: &TextRenderer) -> String {
    let entries: Vec<String> = renderer
        .entries
        .iter()
        .map(|e| {
            format!(
                r#"{{"text":"{}","x":{:.2},"y":{:.2},"font_size":{:.1}}}"#,
                e.text, e.position[0], e.position[1], e.style.font_size
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

// ── New types required by task ─────────────────────────────────────────────

/// A simpler screen-space text item with position and content.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScreenText {
    pub content: String,
    pub x: f32,
    pub y: f32,
    pub style: TextStyle,
}

/// Create a new `ScreenText`.
#[allow(dead_code)]
pub fn new_screen_text(content: &str, x: f32, y: f32) -> ScreenText {
    ScreenText { content: content.to_owned(), x, y, style: default_text_style() }
}

/// Set the text content.
#[allow(dead_code)]
pub fn set_text_content(st: &mut ScreenText, content: &str) {
    st.content = content.to_owned();
}

/// Set the text position.
#[allow(dead_code)]
pub fn set_text_position(st: &mut ScreenText, x: f32, y: f32) {
    st.x = x;
    st.y = y;
}

/// Approximate text width: font_size * char_count * 0.6.
#[allow(dead_code)]
pub fn text_width_approx(st: &ScreenText) -> f32 {
    st.style.font_size * st.content.chars().count() as f32 * 0.6
}

/// Approximate text height: font_size * 1.2.
#[allow(dead_code)]
pub fn text_height_approx(st: &ScreenText) -> f32 {
    st.style.font_size * 1.2
}

/// Stub: render text (returns a description string).
#[allow(dead_code)]
pub fn render_text_stub(st: &ScreenText) -> String {
    format!("render@({:.1},{:.1}):\"{}\"", st.x, st.y, st.content)
}

/// Return the number of characters in the text content.
#[allow(dead_code)]
pub fn text_char_count(st: &ScreenText) -> usize {
    st.content.chars().count()
}

/// Return a `TextStyle` with default values.
#[allow(dead_code)]
pub fn text_style_default() -> TextStyle {
    default_text_style()
}

fn make_entry(text: &str, x: f32, y: f32) -> TextEntry {
    TextEntry {
        text: text.to_string(),
        position: [x, y],
        style: default_text_style(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style() {
        let s = default_text_style();
        assert!((s.font_size - 14.0).abs() < 1e-6);
        assert!(!s.bold);
    }

    #[test]
    fn test_new_renderer_empty() {
        let r = new_text_renderer();
        assert_eq!(tr_count(&r), 0);
    }

    #[test]
    fn test_add_and_count() {
        let mut r = new_text_renderer();
        tr_add_text(&mut r, make_entry("Hello", 10.0, 20.0));
        assert_eq!(tr_count(&r), 1);
    }

    #[test]
    fn test_get_valid() {
        let mut r = new_text_renderer();
        tr_add_text(&mut r, make_entry("Test", 0.0, 0.0));
        assert_eq!(tr_get(&r, 0).unwrap().text, "Test");
    }

    #[test]
    fn test_get_out_of_bounds() {
        let r = new_text_renderer();
        assert!(tr_get(&r, 0).is_none());
    }

    #[test]
    fn test_remove_valid() {
        let mut r = new_text_renderer();
        tr_add_text(&mut r, make_entry("A", 0.0, 0.0));
        tr_add_text(&mut r, make_entry("B", 0.0, 0.0));
        assert!(tr_remove(&mut r, 0));
        assert_eq!(tr_count(&r), 1);
    }

    #[test]
    fn test_remove_invalid() {
        let mut r = new_text_renderer();
        assert!(!tr_remove(&mut r, 5));
    }

    #[test]
    fn test_clear() {
        let mut r = new_text_renderer();
        tr_add_text(&mut r, make_entry("X", 0.0, 0.0));
        tr_clear(&mut r);
        assert_eq!(tr_count(&r), 0);
    }

    #[test]
    fn test_to_json_empty() {
        let r = new_text_renderer();
        assert_eq!(tr_to_json(&r), "[]");
    }

    #[test]
    fn test_to_json_has_text() {
        let mut r = new_text_renderer();
        tr_add_text(&mut r, make_entry("hi", 5.0, 10.0));
        let j = tr_to_json(&r);
        assert!(j.contains("hi"));
    }

    #[test]
    fn test_new_screen_text() {
        let st = new_screen_text("hello", 10.0, 20.0);
        assert_eq!(st.content, "hello");
        assert!((st.x - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_text_content() {
        let mut st = new_screen_text("old", 0.0, 0.0);
        set_text_content(&mut st, "new");
        assert_eq!(st.content, "new");
    }

    #[test]
    fn test_set_text_position() {
        let mut st = new_screen_text("x", 0.0, 0.0);
        set_text_position(&mut st, 5.0, 10.0);
        assert!((st.x - 5.0).abs() < 1e-6);
        assert!((st.y - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_text_char_count() {
        let st = new_screen_text("abc", 0.0, 0.0);
        assert_eq!(text_char_count(&st), 3);
    }

    #[test]
    fn test_text_width_approx_nonzero() {
        let st = new_screen_text("hello", 0.0, 0.0);
        assert!(text_width_approx(&st) > 0.0);
    }

    #[test]
    fn test_text_height_approx_nonzero() {
        let st = new_screen_text("x", 0.0, 0.0);
        assert!(text_height_approx(&st) > 0.0);
    }

    #[test]
    fn test_render_text_stub() {
        let st = new_screen_text("test", 1.0, 2.0);
        let s = render_text_stub(&st);
        assert!(s.contains("test"));
    }

    #[test]
    fn test_text_style_default_font_size() {
        let style = text_style_default();
        assert!((style.font_size - 14.0).abs() < 1e-6);
    }
}
