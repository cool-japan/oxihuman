// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Exported representation of a text object.
#[allow(dead_code)]
pub struct TextObjectExport {
    pub name: String,
    pub text: String,
    pub font: String,
    pub size: f32,
    pub extrude: f32,
    pub align: u8,
}

/// Create a default text object export.
#[allow(dead_code)]
pub fn default_text_object_export(name: &str, text: &str) -> TextObjectExport {
    TextObjectExport {
        name: name.to_string(),
        text: text.to_string(),
        font: "Bfont".to_string(),
        size: 1.0,
        extrude: 0.0,
        align: 0,
    }
}

/// Export a text object to a JSON string.
#[allow(dead_code)]
pub fn export_text_object_to_json(t: &TextObjectExport) -> String {
    format!(
        r#"{{"name":"{}","text":"{}","font":"{}","size":{},"extrude":{},"align":{}}}"#,
        t.name, t.text, t.font, t.size, t.extrude, t.align
    )
}

/// Get the character count of the text.
#[allow(dead_code)]
pub fn text_char_count(t: &TextObjectExport) -> usize {
    t.text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_text_object_export_name() {
        let t = default_text_object_export("MyText", "Hello");
        assert_eq!(t.name, "MyText");
    }

    #[test]
    fn test_default_text_object_export_text() {
        let t = default_text_object_export("T", "World");
        assert_eq!(t.text, "World");
    }

    #[test]
    fn test_default_text_object_export_size() {
        let t = default_text_object_export("T", "X");
        assert!((t.size - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_default_text_object_export_font() {
        let t = default_text_object_export("T", "X");
        assert_eq!(t.font, "Bfont");
    }

    #[test]
    fn test_export_text_object_to_json_contains_name() {
        let t = default_text_object_export("Label", "Hi");
        let json = export_text_object_to_json(&t);
        assert!(json.contains("Label"));
    }

    #[test]
    fn test_export_text_object_to_json_contains_text() {
        let t = default_text_object_export("L", "OpenGL");
        let json = export_text_object_to_json(&t);
        assert!(json.contains("OpenGL"));
    }

    #[test]
    fn test_text_char_count_empty() {
        let t = default_text_object_export("T", "");
        assert_eq!(text_char_count(&t), 0);
    }

    #[test]
    fn test_text_char_count_ascii() {
        let t = default_text_object_export("T", "Hello");
        assert_eq!(text_char_count(&t), 5);
    }

    #[test]
    fn test_export_json_contains_font() {
        let t = default_text_object_export("T", "X");
        let json = export_text_object_to_json(&t);
        assert!(json.contains("Bfont"));
    }

    #[test]
    fn test_export_json_structure() {
        let t = default_text_object_export("A", "B");
        let json = export_text_object_to_json(&t);
        assert!(json.starts_with('{') && json.ends_with('}'));
    }
}
