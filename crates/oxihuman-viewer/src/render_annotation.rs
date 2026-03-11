// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An annotation in the render view.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderAnnotation {
    pub text: String,
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub visible: bool,
}

/// Create a new render annotation.
#[allow(dead_code)]
pub fn new_render_annotation(text: &str, position: [f32; 3]) -> RenderAnnotation {
    RenderAnnotation {
        text: text.to_string(),
        position,
        color: [1.0, 1.0, 1.0, 1.0],
        visible: true,
    }
}

/// Set the annotation text.
#[allow(dead_code)]
pub fn set_annotation_text(ann: &mut RenderAnnotation, text: &str) {
    ann.text = text.to_string();
}

/// Return the annotation position.
#[allow(dead_code)]
pub fn annotation_position(ann: &RenderAnnotation) -> [f32; 3] {
    ann.position
}

/// Return the annotation color.
#[allow(dead_code)]
pub fn annotation_color(ann: &RenderAnnotation) -> [f32; 4] {
    ann.color
}

/// Check if the annotation is visible.
#[allow(dead_code)]
pub fn annotation_is_visible(ann: &RenderAnnotation) -> bool {
    ann.visible
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn annotation_to_json(ann: &RenderAnnotation) -> String {
    format!(
        "{{\"text\":\"{}\",\"pos\":[{:.4},{:.4},{:.4}],\"visible\":{}}}",
        ann.text, ann.position[0], ann.position[1], ann.position[2], ann.visible
    )
}

/// Clear the annotation text.
#[allow(dead_code)]
pub fn annotations_clear(ann: &mut RenderAnnotation) {
    ann.text.clear();
    ann.visible = false;
}

/// Return the number of characters in the annotation (stub for count).
#[allow(dead_code)]
pub fn annotation_count(ann: &RenderAnnotation) -> usize {
    ann.text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_annotation() {
        let a = new_render_annotation("label", [1.0, 2.0, 3.0]);
        assert_eq!(a.text, "label");
    }

    #[test]
    fn set_text() {
        let mut a = new_render_annotation("old", [0.0; 3]);
        set_annotation_text(&mut a, "new");
        assert_eq!(a.text, "new");
    }

    #[test]
    fn position_accessor() {
        let a = new_render_annotation("x", [1.0, 2.0, 3.0]);
        let p = annotation_position(&a);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_default_white() {
        let a = new_render_annotation("x", [0.0; 3]);
        let c = annotation_color(&a);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn visible_by_default() {
        let a = new_render_annotation("x", [0.0; 3]);
        assert!(annotation_is_visible(&a));
    }

    #[test]
    fn to_json() {
        let a = new_render_annotation("test", [0.0; 3]);
        let j = annotation_to_json(&a);
        assert!(j.contains("test"));
    }

    #[test]
    fn clear_annotation() {
        let mut a = new_render_annotation("text", [0.0; 3]);
        annotations_clear(&mut a);
        assert!(!annotation_is_visible(&a));
        assert!(a.text.is_empty());
    }

    #[test]
    fn annotation_count_test() {
        let a = new_render_annotation("hello", [0.0; 3]);
        assert_eq!(annotation_count(&a), 5);
    }

    #[test]
    fn empty_annotation() {
        let a = new_render_annotation("", [0.0; 3]);
        assert_eq!(annotation_count(&a), 0);
    }

    #[test]
    fn set_color() {
        let mut a = new_render_annotation("x", [0.0; 3]);
        a.color = [1.0, 0.0, 0.0, 1.0];
        let c = annotation_color(&a);
        assert!((c[1]).abs() < 1e-6);
    }
}
