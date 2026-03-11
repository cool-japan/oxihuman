// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Priority for a render layer (lower = drawn first).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerPriority(pub u32);

/// A named render layer with visibility and priority.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderLayer {
    pub name: String,
    pub priority: LayerPriority,
    pub visible: bool,
    pub draw_count: usize,
}

/// Create a new render layer.
#[allow(dead_code)]
pub fn new_render_layer_rl(name: &str, priority: u32) -> RenderLayer {
    RenderLayer {
        name: name.to_string(),
        priority: LayerPriority(priority),
        visible: true,
        draw_count: 0,
    }
}

/// Return the layer priority.
#[allow(dead_code)]
pub fn layer_priority(layer: &RenderLayer) -> u32 {
    layer.priority.0
}

/// Check if the layer is visible.
#[allow(dead_code)]
pub fn layer_is_visible(layer: &RenderLayer) -> bool {
    layer.visible
}

/// Set layer visibility.
#[allow(dead_code)]
pub fn set_layer_visible(layer: &mut RenderLayer, visible: bool) {
    layer.visible = visible;
}

/// Return the draw count for this layer.
#[allow(dead_code)]
pub fn layer_draw_count(layer: &RenderLayer) -> usize {
    layer.draw_count
}

/// Return the layer name.
#[allow(dead_code)]
pub fn layer_name_rl(layer: &RenderLayer) -> &str {
    &layer.name
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn layer_to_json(layer: &RenderLayer) -> String {
    format!(
        "{{\"name\":\"{}\",\"priority\":{},\"visible\":{}}}",
        layer.name, layer.priority.0, layer.visible
    )
}

/// Clear the layer draw count.
#[allow(dead_code)]
pub fn layer_clear(layer: &mut RenderLayer) {
    layer.draw_count = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layer() {
        let l = new_render_layer_rl("opaque", 0);
        assert!(layer_is_visible(&l));
    }

    #[test]
    fn priority_accessor() {
        let l = new_render_layer_rl("transparent", 10);
        assert_eq!(layer_priority(&l), 10);
    }

    #[test]
    fn toggle_visibility() {
        let mut l = new_render_layer_rl("x", 0);
        set_layer_visible(&mut l, false);
        assert!(!layer_is_visible(&l));
    }

    #[test]
    fn draw_count_starts_zero() {
        let l = new_render_layer_rl("x", 0);
        assert_eq!(layer_draw_count(&l), 0);
    }

    #[test]
    fn name_accessor() {
        let l = new_render_layer_rl("overlay", 5);
        assert_eq!(layer_name_rl(&l), "overlay");
    }

    #[test]
    fn to_json() {
        let l = new_render_layer_rl("test", 1);
        let j = layer_to_json(&l);
        assert!(j.contains("\"test\""));
    }

    #[test]
    fn clear_draw_count() {
        let mut l = new_render_layer_rl("x", 0);
        l.draw_count = 42;
        layer_clear(&mut l);
        assert_eq!(layer_draw_count(&l), 0);
    }

    #[test]
    fn priority_ordering() {
        let a = LayerPriority(1);
        let b = LayerPriority(10);
        assert!(a < b);
    }

    #[test]
    fn visible_by_default() {
        let l = new_render_layer_rl("default", 0);
        assert!(l.visible);
    }

    #[test]
    fn set_visible_true() {
        let mut l = new_render_layer_rl("x", 0);
        set_layer_visible(&mut l, false);
        set_layer_visible(&mut l, true);
        assert!(layer_is_visible(&l));
    }
}
