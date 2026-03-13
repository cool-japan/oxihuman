// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Excalidraw JSON export.

/// An Excalidraw element type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExcalidrawElementType {
    Rectangle,
    Ellipse,
    Arrow,
    Text,
    Line,
}

impl ExcalidrawElementType {
    pub fn type_str(self) -> &'static str {
        match self {
            Self::Rectangle => "rectangle",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Text => "text",
            Self::Line => "line",
        }
    }
}

/// An Excalidraw element.
#[derive(Debug, Clone)]
pub struct ExcalidrawElement {
    pub id: String,
    pub element_type: ExcalidrawElementType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: Option<String>,
    pub stroke_color: String,
    pub fill_color: String,
}

impl ExcalidrawElement {
    pub fn new(id: &str, element_type: ExcalidrawElementType, x: f32, y: f32) -> Self {
        Self {
            id: id.to_string(),
            element_type,
            x,
            y,
            width: 160.0,
            height: 80.0,
            label: None,
            stroke_color: "#1e1e1e".to_string(),
            fill_color: "transparent".to_string(),
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
}

/// Excalidraw scene export.
#[derive(Debug, Clone, Default)]
pub struct ExcalidrawExport {
    pub elements: Vec<ExcalidrawElement>,
    pub version: u32,
}

impl ExcalidrawExport {
    pub fn new() -> Self {
        Self { elements: Vec::new(), version: 2 }
    }

    pub fn add_element(&mut self, elem: ExcalidrawElement) {
        self.elements.push(elem);
    }
}

/// Serialize to Excalidraw JSON string.
pub fn to_excalidraw_json(d: &ExcalidrawExport) -> String {
    let mut elements_json = String::new();
    for (i, e) in d.elements.iter().enumerate() {
        if i > 0 {
            elements_json.push(',');
        }
        let text = e.label.as_deref().unwrap_or("");
        elements_json.push_str(&format!(
            "{{\"id\":\"{}\",\"type\":\"{}\",\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"text\":\"{}\"}}",
            e.id,
            e.element_type.type_str(),
            e.x, e.y, e.width, e.height,
            text
        ));
    }
    format!(
        "{{\"type\":\"excalidraw\",\"version\":{},\"elements\":[{}]}}",
        d.version, elements_json
    )
}

/// Count elements.
pub fn excalidraw_element_count(d: &ExcalidrawExport) -> usize {
    d.elements.len()
}

/// Create a new Excalidraw export.
pub fn new_excalidraw_export() -> ExcalidrawExport {
    ExcalidrawExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_excalidraw_export() {
        let d = new_excalidraw_export();
        assert_eq!(d.version, 2);
    }

    #[test]
    fn test_add_element() {
        let mut d = ExcalidrawExport::new();
        d.add_element(ExcalidrawElement::new("e1", ExcalidrawElementType::Rectangle, 0.0, 0.0));
        assert_eq!(excalidraw_element_count(&d), 1);
    }

    #[test]
    fn test_to_excalidraw_json_type() {
        let d = ExcalidrawExport::new();
        let s = to_excalidraw_json(&d);
        assert!(s.contains("excalidraw"));
    }

    #[test]
    fn test_to_excalidraw_json_has_element() {
        let mut d = ExcalidrawExport::new();
        d.add_element(
            ExcalidrawElement::new("e1", ExcalidrawElementType::Ellipse, 0.0, 0.0)
                .with_label("MyNode"),
        );
        let s = to_excalidraw_json(&d);
        assert!(s.contains("ellipse"));
        assert!(s.contains("MyNode"));
    }

    #[test]
    fn test_element_type_str() {
        assert_eq!(ExcalidrawElementType::Arrow.type_str(), "arrow");
    }

    #[test]
    fn test_element_with_label() {
        let e =
            ExcalidrawElement::new("e1", ExcalidrawElementType::Text, 0.0, 0.0).with_label("Hi");
        assert_eq!(e.label.as_deref(), Some("Hi"));
    }

    #[test]
    fn test_to_excalidraw_json_multiple_elements() {
        let mut d = ExcalidrawExport::new();
        d.add_element(ExcalidrawElement::new("e1", ExcalidrawElementType::Rectangle, 0.0, 0.0));
        d.add_element(ExcalidrawElement::new("e2", ExcalidrawElementType::Ellipse, 200.0, 0.0));
        let s = to_excalidraw_json(&d);
        assert!(s.contains("e1"));
        assert!(s.contains("e2"));
    }

    #[test]
    fn test_excalidraw_element_default_dimensions() {
        let e = ExcalidrawElement::new("x", ExcalidrawElementType::Line, 0.0, 0.0);
        assert!((e.width - 160.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_excalidraw_json_empty_elements() {
        let d = ExcalidrawExport::new();
        let s = to_excalidraw_json(&d);
        assert!(s.contains("\"elements\":[]"));
    }

    #[test]
    fn test_element_count_empty() {
        let d = ExcalidrawExport::new();
        assert_eq!(excalidraw_element_count(&d), 0);
    }
}
