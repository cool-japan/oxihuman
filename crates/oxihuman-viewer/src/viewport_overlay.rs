#![allow(dead_code)]
//! Viewport overlay: manages overlay elements drawn on top of the viewport.

/// An element in the overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum OverlayElement {
    Text { x: f32, y: f32, content: String },
    Rect { x: f32, y: f32, w: f32, h: f32 },
}

/// A viewport overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViewportOverlay {
    elements: Vec<OverlayElement>,
    visible: bool,
}

/// Create a new viewport overlay.
#[allow(dead_code)]
pub fn new_viewport_overlay() -> ViewportOverlay {
    ViewportOverlay {
        elements: Vec::new(),
        visible: true,
    }
}

/// Add a text element.
#[allow(dead_code)]
pub fn add_overlay_text(overlay: &mut ViewportOverlay, x: f32, y: f32, content: &str) {
    overlay.elements.push(OverlayElement::Text {
        x,
        y,
        content: content.to_string(),
    });
}

/// Add a rectangle element.
#[allow(dead_code)]
pub fn add_overlay_rect(overlay: &mut ViewportOverlay, x: f32, y: f32, w: f32, h: f32) {
    overlay.elements.push(OverlayElement::Rect { x, y, w, h });
}

/// Return the number of elements.
#[allow(dead_code)]
pub fn overlay_element_count(overlay: &ViewportOverlay) -> usize {
    overlay.elements.len()
}

/// Clear all elements.
#[allow(dead_code)]
pub fn overlay_clear(overlay: &mut ViewportOverlay) {
    overlay.elements.clear();
}

/// Check if the overlay is visible.
#[allow(dead_code)]
pub fn overlay_is_visible(overlay: &ViewportOverlay) -> bool {
    overlay.visible
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn overlay_to_json(overlay: &ViewportOverlay) -> String {
    format!(
        "{{\"element_count\":{},\"visible\":{}}}",
        overlay.elements.len(),
        overlay.visible
    )
}

/// Set overlay visibility.
#[allow(dead_code)]
pub fn set_overlay_visible(overlay: &mut ViewportOverlay, visible: bool) {
    overlay.visible = visible;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_overlay() {
        let o = new_viewport_overlay();
        assert_eq!(overlay_element_count(&o), 0);
        assert!(overlay_is_visible(&o));
    }

    #[test]
    fn test_add_text() {
        let mut o = new_viewport_overlay();
        add_overlay_text(&mut o, 10.0, 20.0, "Hello");
        assert_eq!(overlay_element_count(&o), 1);
    }

    #[test]
    fn test_add_rect() {
        let mut o = new_viewport_overlay();
        add_overlay_rect(&mut o, 0.0, 0.0, 100.0, 50.0);
        assert_eq!(overlay_element_count(&o), 1);
    }

    #[test]
    fn test_clear() {
        let mut o = new_viewport_overlay();
        add_overlay_text(&mut o, 0.0, 0.0, "x");
        overlay_clear(&mut o);
        assert_eq!(overlay_element_count(&o), 0);
    }

    #[test]
    fn test_visibility() {
        let mut o = new_viewport_overlay();
        set_overlay_visible(&mut o, false);
        assert!(!overlay_is_visible(&o));
    }

    #[test]
    fn test_to_json() {
        let o = new_viewport_overlay();
        let json = overlay_to_json(&o);
        assert!(json.contains("\"visible\":true"));
    }

    #[test]
    fn test_multiple_elements() {
        let mut o = new_viewport_overlay();
        add_overlay_text(&mut o, 0.0, 0.0, "a");
        add_overlay_rect(&mut o, 0.0, 0.0, 10.0, 10.0);
        assert_eq!(overlay_element_count(&o), 2);
    }

    #[test]
    fn test_set_visible_true() {
        let mut o = new_viewport_overlay();
        set_overlay_visible(&mut o, false);
        set_overlay_visible(&mut o, true);
        assert!(overlay_is_visible(&o));
    }

    #[test]
    fn test_json_with_elements() {
        let mut o = new_viewport_overlay();
        add_overlay_text(&mut o, 0.0, 0.0, "test");
        let json = overlay_to_json(&o);
        assert!(json.contains("\"element_count\":1"));
    }

    #[test]
    fn test_clear_preserves_visibility() {
        let mut o = new_viewport_overlay();
        set_overlay_visible(&mut o, false);
        add_overlay_text(&mut o, 0.0, 0.0, "x");
        overlay_clear(&mut o);
        assert!(!overlay_is_visible(&o));
    }
}
