// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Compositing layer management for multi-layer rendering.

/// Blend mode for layer compositing.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerBlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
}

/// A compositing layer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompositingLayer {
    pub name: String,
    pub opacity: f32,
    pub blend_mode: LayerBlendMode,
    pub visible: bool,
    pub order: u32,
}

#[allow(dead_code)]
pub fn new_compositing_layer(name: &str, order: u32) -> CompositingLayer {
    CompositingLayer {
        name: name.to_string(),
        opacity: 1.0,
        blend_mode: LayerBlendMode::Normal,
        visible: true,
        order,
    }
}

#[allow(dead_code)]
pub fn set_layer_opacity(layer: &mut CompositingLayer, opacity: f32) {
    layer.opacity = opacity.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_layer_blend_mode(layer: &mut CompositingLayer, mode: LayerBlendMode) {
    layer.blend_mode = mode;
}

#[allow(dead_code)]
pub fn toggle_layer_visibility(layer: &mut CompositingLayer) {
    layer.visible = !layer.visible;
}

#[allow(dead_code)]
pub fn sort_layers(layers: &mut [CompositingLayer]) {
    layers.sort_by_key(|l| l.order);
}

#[allow(dead_code)]
pub fn composite_opacity(base: f32, layer: f32) -> f32 {
    (base * layer).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn visible_layer_count(layers: &[CompositingLayer]) -> usize {
    layers.iter().filter(|l| l.visible).count()
}

#[allow(dead_code)]
pub fn find_layer_by_name<'a>(layers: &'a [CompositingLayer], name: &str) -> Option<&'a CompositingLayer> {
    layers.iter().find(|l| l.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_layer() {
        let l = new_compositing_layer("base", 0);
        assert_eq!(l.name, "base");
        assert!(l.visible);
    }

    #[test]
    fn test_set_opacity() {
        let mut l = new_compositing_layer("test", 0);
        set_layer_opacity(&mut l, 0.5);
        assert!((l.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_opacity_clamp() {
        let mut l = new_compositing_layer("test", 0);
        set_layer_opacity(&mut l, 1.5);
        assert!((l.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_blend_mode() {
        let mut l = new_compositing_layer("test", 0);
        set_layer_blend_mode(&mut l, LayerBlendMode::Add);
        assert_eq!(l.blend_mode, LayerBlendMode::Add);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut l = new_compositing_layer("test", 0);
        toggle_layer_visibility(&mut l);
        assert!(!l.visible);
        toggle_layer_visibility(&mut l);
        assert!(l.visible);
    }

    #[test]
    fn test_sort_layers() {
        let mut layers = vec![
            new_compositing_layer("c", 2),
            new_compositing_layer("a", 0),
            new_compositing_layer("b", 1),
        ];
        sort_layers(&mut layers);
        assert_eq!(layers[0].name, "a");
        assert_eq!(layers[2].name, "c");
    }

    #[test]
    fn test_composite_opacity() {
        let result = composite_opacity(0.5, 0.5);
        assert!((result - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_visible_count() {
        let mut layers = vec![
            new_compositing_layer("a", 0),
            new_compositing_layer("b", 1),
        ];
        layers[1].visible = false;
        assert_eq!(visible_layer_count(&layers), 1);
    }

    #[test]
    fn test_find_layer_by_name() {
        let layers = vec![
            new_compositing_layer("base", 0),
            new_compositing_layer("overlay", 1),
        ];
        let found = find_layer_by_name(&layers, "overlay");
        assert!(found.is_some_and(|l| l.order == 1));
    }

    #[test]
    fn test_find_layer_missing() {
        let layers = vec![new_compositing_layer("base", 0)];
        assert!(find_layer_by_name(&layers, "missing").is_none());
    }
}
