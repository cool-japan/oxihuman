// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2D overlay rendering system — GUI stubs for heads-up display.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayAnchor {
    pub x: f32,
    pub y: f32,
}

impl OverlayAnchor {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl OverlayColor {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::rgba(1.0, 1.0, 1.0, 1.0)
    }

    pub fn red() -> Self {
        Self::rgba(1.0, 0.0, 0.0, 1.0)
    }

    pub fn green() -> Self {
        Self::rgba(0.0, 1.0, 0.0, 1.0)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum OverlayElement {
    Text {
        text: String,
        anchor: OverlayAnchor,
        color: OverlayColor,
        size: f32,
    },
    Line {
        from: OverlayAnchor,
        to: OverlayAnchor,
        color: OverlayColor,
        width: f32,
    },
    Rect {
        min: OverlayAnchor,
        max: OverlayAnchor,
        fill: OverlayColor,
        border: OverlayColor,
    },
    Circle {
        center: OverlayAnchor,
        radius: f32,
        color: OverlayColor,
    },
}

impl OverlayElement {
    /// Returns a simple JSON fragment describing this element.
    pub fn to_json(&self) -> String {
        match self {
            OverlayElement::Text {
                text,
                anchor,
                color,
                size,
            } => {
                format!(
                    r#"{{"type":"text","text":"{text}","x":{:.4},"y":{:.4},"r":{:.4},"g":{:.4},"b":{:.4},"a":{:.4},"size":{size:.4}}}"#,
                    anchor.x, anchor.y, color.r, color.g, color.b, color.a
                )
            }
            OverlayElement::Line {
                from,
                to,
                color,
                width,
            } => {
                format!(
                    r#"{{"type":"line","fx":{:.4},"fy":{:.4},"tx":{:.4},"ty":{:.4},"r":{:.4},"g":{:.4},"b":{:.4},"a":{:.4},"width":{width:.4}}}"#,
                    from.x, from.y, to.x, to.y, color.r, color.g, color.b, color.a
                )
            }
            OverlayElement::Rect {
                min,
                max,
                fill,
                border,
            } => {
                format!(
                    r#"{{"type":"rect","minx":{:.4},"miny":{:.4},"maxx":{:.4},"maxy":{:.4},"fr":{:.4},"fg":{:.4},"fb":{:.4},"fa":{:.4},"br":{:.4},"bg":{:.4},"bb":{:.4},"ba":{:.4}}}"#,
                    min.x,
                    min.y,
                    max.x,
                    max.y,
                    fill.r,
                    fill.g,
                    fill.b,
                    fill.a,
                    border.r,
                    border.g,
                    border.b,
                    border.a
                )
            }
            OverlayElement::Circle {
                center,
                radius,
                color,
            } => {
                format!(
                    r#"{{"type":"circle","cx":{:.4},"cy":{:.4},"radius":{radius:.4},"r":{:.4},"g":{:.4},"b":{:.4},"a":{:.4}}}"#,
                    center.x, center.y, color.r, color.g, color.b, color.a
                )
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OverlayLayer {
    pub name: String,
    pub elements: Vec<OverlayElement>,
    pub visible: bool,
    pub z_order: i32,
}

impl OverlayLayer {
    pub fn new(name: &str, z_order: i32) -> Self {
        Self {
            name: name.to_string(),
            elements: Vec::new(),
            visible: true,
            z_order,
        }
    }

    pub fn to_json(&self) -> String {
        let elems: Vec<String> = self.elements.iter().map(|e| e.to_json()).collect();
        format!(
            r#"{{"name":"{}","z_order":{},"visible":{},"elements":[{}]}}"#,
            self.name,
            self.z_order,
            self.visible,
            elems.join(",")
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OverlayRenderer {
    pub layers: Vec<OverlayLayer>,
}

impl OverlayRenderer {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a new layer and return its index.
    pub fn add_layer(&mut self, name: &str, z_order: i32) -> usize {
        let idx = self.layers.len();
        self.layers.push(OverlayLayer::new(name, z_order));
        idx
    }

    /// Add an element to a specific layer.
    pub fn add_element(&mut self, layer_idx: usize, element: OverlayElement) {
        if let Some(layer) = self.layers.get_mut(layer_idx) {
            layer.elements.push(element);
        }
    }

    /// Remove all elements from a layer.
    pub fn clear_layer(&mut self, layer_idx: usize) {
        if let Some(layer) = self.layers.get_mut(layer_idx) {
            layer.elements.clear();
        }
    }

    /// Set visibility of a layer.
    pub fn set_layer_visible(&mut self, layer_idx: usize, visible: bool) {
        if let Some(layer) = self.layers.get_mut(layer_idx) {
            layer.visible = visible;
        }
    }

    /// Total elements across all layers.
    pub fn total_elements(&self) -> usize {
        self.layers.iter().map(|l| l.elements.len()).sum()
    }

    /// Layers sorted by z_order ascending.
    pub fn sorted_layers(&self) -> Vec<&OverlayLayer> {
        let mut refs: Vec<&OverlayLayer> = self.layers.iter().collect();
        refs.sort_by_key(|l| l.z_order);
        refs
    }

    /// JSON of all visible layers and their elements.
    pub fn render_to_json(&self) -> String {
        let visible: Vec<String> = self
            .sorted_layers()
            .into_iter()
            .filter(|l| l.visible)
            .map(|l| l.to_json())
            .collect();
        format!(r#"{{"layers":[{}]}}"#, visible.join(","))
    }
}

impl Default for OverlayRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a text element at normalized screen position.
pub fn text_element(text: &str, x: f32, y: f32, size: f32) -> OverlayElement {
    OverlayElement::Text {
        text: text.to_string(),
        anchor: OverlayAnchor::new(x, y),
        color: OverlayColor::white(),
        size,
    }
}

/// Build an FPS counter text element at the top-left corner.
pub fn fps_counter_element(fps: f32) -> OverlayElement {
    text_element(&format!("FPS: {fps:.1}"), 0.02, 0.02, 14.0)
}

/// Default HUD overlay with FPS, stats, and debug layers.
pub fn default_hud_overlay() -> OverlayRenderer {
    let mut r = OverlayRenderer::new();
    let hud = r.add_layer("hud", 0);
    let stats = r.add_layer("stats", 1);
    let _debug = r.add_layer("debug", 2);

    r.add_element(hud, fps_counter_element(60.0));
    r.add_element(stats, text_element("Tris: 0", 0.02, 0.05, 12.0));
    r.add_element(stats, text_element("Draw calls: 0", 0.02, 0.08, 12.0));
    r
}

/// Crosshair overlay — two lines crossing at center (0.5, 0.5).
pub fn crosshair_overlay() -> OverlayRenderer {
    let mut r = OverlayRenderer::new();
    let layer = r.add_layer("crosshair", 10);
    // horizontal line
    r.add_element(
        layer,
        OverlayElement::Line {
            from: OverlayAnchor::new(0.45, 0.5),
            to: OverlayAnchor::new(0.55, 0.5),
            color: OverlayColor::white(),
            width: 1.5,
        },
    );
    // vertical line
    r.add_element(
        layer,
        OverlayElement::Line {
            from: OverlayAnchor::new(0.5, 0.45),
            to: OverlayAnchor::new(0.5, 0.55),
            color: OverlayColor::white(),
            width: 1.5,
        },
    );
    r
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_layer_increases_count() {
        let mut r = OverlayRenderer::new();
        assert_eq!(r.layers.len(), 0);
        r.add_layer("a", 0);
        r.add_layer("b", 1);
        assert_eq!(r.layers.len(), 2);
    }

    #[test]
    fn add_layer_returns_correct_index() {
        let mut r = OverlayRenderer::new();
        let i0 = r.add_layer("x", 0);
        let i1 = r.add_layer("y", 5);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
    }

    #[test]
    fn add_element_increases_total() {
        let mut r = OverlayRenderer::new();
        let layer = r.add_layer("hud", 0);
        assert_eq!(r.total_elements(), 0);
        r.add_element(layer, text_element("hello", 0.1, 0.1, 12.0));
        assert_eq!(r.total_elements(), 1);
        r.add_element(layer, text_element("world", 0.2, 0.2, 12.0));
        assert_eq!(r.total_elements(), 2);
    }

    #[test]
    fn clear_layer_removes_elements() {
        let mut r = OverlayRenderer::new();
        let layer = r.add_layer("hud", 0);
        r.add_element(layer, text_element("a", 0.0, 0.0, 10.0));
        r.add_element(layer, text_element("b", 0.1, 0.1, 10.0));
        assert_eq!(r.total_elements(), 2);
        r.clear_layer(layer);
        assert_eq!(r.total_elements(), 0);
    }

    #[test]
    fn set_layer_visible_false_hides_layer() {
        let mut r = OverlayRenderer::new();
        let layer = r.add_layer("hud", 0);
        r.set_layer_visible(layer, false);
        assert!(!r.layers[layer].visible);
    }

    #[test]
    fn set_layer_visible_true() {
        let mut r = OverlayRenderer::new();
        let layer = r.add_layer("hud", 0);
        r.set_layer_visible(layer, false);
        r.set_layer_visible(layer, true);
        assert!(r.layers[layer].visible);
    }

    #[test]
    fn sorted_layers_order() {
        let mut r = OverlayRenderer::new();
        r.add_layer("c", 10);
        r.add_layer("a", -5);
        r.add_layer("b", 3);
        let sorted = r.sorted_layers();
        let orders: Vec<i32> = sorted.iter().map(|l| l.z_order).collect();
        assert_eq!(orders, vec![-5, 3, 10]);
    }

    #[test]
    fn total_elements_spans_layers() {
        let mut r = OverlayRenderer::new();
        let l0 = r.add_layer("a", 0);
        let l1 = r.add_layer("b", 1);
        r.add_element(l0, text_element("x", 0.0, 0.0, 10.0));
        r.add_element(l1, text_element("y", 0.0, 0.0, 10.0));
        r.add_element(l1, text_element("z", 0.0, 0.0, 10.0));
        assert_eq!(r.total_elements(), 3);
    }

    #[test]
    fn render_to_json_non_empty() {
        let mut r = OverlayRenderer::new();
        let l = r.add_layer("hud", 0);
        r.add_element(l, text_element("hi", 0.0, 0.0, 12.0));
        let j = r.render_to_json();
        assert!(!j.is_empty());
        assert!(j.contains("layers"));
    }

    #[test]
    fn render_to_json_excludes_hidden_layer() {
        let mut r = OverlayRenderer::new();
        let l = r.add_layer("secret", 0);
        r.add_element(l, text_element("hidden", 0.1, 0.1, 12.0));
        r.set_layer_visible(l, false);
        let j = r.render_to_json();
        assert!(!j.contains("hidden"));
    }

    #[test]
    fn default_hud_has_layers() {
        let hud = default_hud_overlay();
        assert!(hud.layers.len() >= 3);
    }

    #[test]
    fn default_hud_has_elements() {
        let hud = default_hud_overlay();
        assert!(hud.total_elements() > 0);
    }

    #[test]
    fn text_element_is_text_variant() {
        let e = text_element("FPS", 0.0, 0.0, 12.0);
        assert!(matches!(e, OverlayElement::Text { .. }));
    }

    #[test]
    fn fps_counter_element_contains_fps() {
        let e = fps_counter_element(30.0);
        if let OverlayElement::Text { text, .. } = e {
            assert!(text.contains("30"));
        } else {
            panic!("expected Text variant");
        }
    }

    #[test]
    fn crosshair_overlay_has_two_lines() {
        let ch = crosshair_overlay();
        let total = ch.total_elements();
        assert_eq!(total, 2);
    }

    #[test]
    fn crosshair_all_elements_are_lines() {
        let ch = crosshair_overlay();
        for layer in &ch.layers {
            for elem in &layer.elements {
                assert!(matches!(elem, OverlayElement::Line { .. }));
            }
        }
    }
}
