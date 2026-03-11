// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Label renderer for 3D viewport annotations.

/// Label anchor position.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// A text label.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
    pub position: [f32; 3],
    pub anchor: LabelAnchor,
    pub font_size: f32,
    pub color: [f32; 4],
    pub visible: bool,
}

#[allow(dead_code)]
pub fn new_label(text: &str, x: f32, y: f32, z: f32) -> Label {
    Label {
        text: text.to_string(),
        position: [x, y, z],
        anchor: LabelAnchor::Center,
        font_size: 14.0,
        color: [1.0, 1.0, 1.0, 1.0],
        visible: true,
    }
}

#[allow(dead_code)]
pub fn set_label_text(label: &mut Label, text: &str) {
    label.text = text.to_string();
}

#[allow(dead_code)]
pub fn set_label_position(label: &mut Label, x: f32, y: f32, z: f32) {
    label.position = [x, y, z];
}

#[allow(dead_code)]
pub fn set_label_anchor(label: &mut Label, anchor: LabelAnchor) {
    label.anchor = anchor;
}

#[allow(dead_code)]
pub fn set_label_font_size(label: &mut Label, size: f32) {
    label.font_size = size.clamp(4.0, 128.0);
}

#[allow(dead_code)]
pub fn set_label_color(label: &mut Label, r: f32, g: f32, b: f32, a: f32) {
    label.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a.clamp(0.0, 1.0)];
}

#[allow(dead_code)]
pub fn toggle_label_visibility(label: &mut Label) {
    label.visible = !label.visible;
}

#[allow(dead_code)]
pub fn label_char_count(label: &Label) -> usize {
    label.text.len()
}

#[allow(dead_code)]
pub fn visible_labels(labels: &[Label]) -> usize {
    labels.iter().filter(|l| l.visible).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_label() {
        let l = new_label("test", 1.0, 2.0, 3.0);
        assert_eq!(l.text, "test");
        assert!(l.visible);
    }

    #[test]
    fn test_set_text() {
        let mut l = new_label("old", 0.0, 0.0, 0.0);
        set_label_text(&mut l, "new");
        assert_eq!(l.text, "new");
    }

    #[test]
    fn test_set_position() {
        let mut l = new_label("test", 0.0, 0.0, 0.0);
        set_label_position(&mut l, 5.0, 6.0, 7.0);
        assert!((l.position[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_anchor() {
        let mut l = new_label("test", 0.0, 0.0, 0.0);
        set_label_anchor(&mut l, LabelAnchor::TopLeft);
        assert_eq!(l.anchor, LabelAnchor::TopLeft);
    }

    #[test]
    fn test_set_font_size_clamp() {
        let mut l = new_label("test", 0.0, 0.0, 0.0);
        set_label_font_size(&mut l, 1.0);
        assert!((l.font_size - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_color() {
        let mut l = new_label("test", 0.0, 0.0, 0.0);
        set_label_color(&mut l, 0.5, 0.5, 0.5, 1.0);
        assert!((l.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut l = new_label("test", 0.0, 0.0, 0.0);
        toggle_label_visibility(&mut l);
        assert!(!l.visible);
    }

    #[test]
    fn test_char_count() {
        let l = new_label("hello", 0.0, 0.0, 0.0);
        assert_eq!(label_char_count(&l), 5);
    }

    #[test]
    fn test_visible_labels() {
        let mut labels = vec![
            new_label("a", 0.0, 0.0, 0.0),
            new_label("b", 0.0, 0.0, 0.0),
        ];
        labels[1].visible = false;
        assert_eq!(visible_labels(&labels), 1);
    }

    #[test]
    fn test_empty_label() {
        let l = new_label("", 0.0, 0.0, 0.0);
        assert_eq!(label_char_count(&l), 0);
    }
}
