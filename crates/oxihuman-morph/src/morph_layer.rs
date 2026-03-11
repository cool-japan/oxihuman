// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

/// How a layer blends with layers below it
#[derive(Clone, Debug, PartialEq)]
pub enum LayerBlend {
    /// Override: this layer completely replaces weights below
    Override,
    /// Additive: add this layer's weights on top
    Additive,
    /// Multiply: multiply this layer's weights with those below
    Multiply,
    /// Screen: result = 1 - (1 - a) * (1 - b)
    Screen,
    /// Lerp: blend between base and this layer by layer opacity
    Normal,
}

/// A single morph layer
pub struct MorphLayer {
    pub name: String,
    pub blend_mode: LayerBlend,
    pub opacity: f32,
    pub enabled: bool,
    /// Per-morph weights in this layer
    pub weights: HashMap<String, f32>,
    /// Optional per-morph mask (vertex group influence)
    pub mask: Option<Vec<f32>>,
}

impl MorphLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            blend_mode: LayerBlend::Normal,
            opacity: 1.0,
            enabled: true,
            weights: HashMap::new(),
            mask: None,
        }
    }

    pub fn with_blend(mut self, blend: LayerBlend) -> Self {
        self.blend_mode = blend;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn set_weight(&mut self, morph: impl Into<String>, weight: f32) {
        self.weights.insert(morph.into(), weight);
    }

    pub fn get_weight(&self, morph: &str) -> f32 {
        self.weights.get(morph).copied().unwrap_or(0.0)
    }

    pub fn morph_names(&self) -> Vec<&str> {
        self.weights.keys().map(|s| s.as_str()).collect()
    }

    /// Returns true if this layer is enabled and has non-zero opacity
    pub fn is_active(&self) -> bool {
        self.enabled && self.opacity > 0.0
    }
}

/// Ordered stack of morph layers
pub struct MorphLayerStack {
    layers: Vec<MorphLayer>,
}

impl MorphLayerStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn push(&mut self, layer: MorphLayer) {
        self.layers.push(layer);
    }

    pub fn pop(&mut self) -> Option<MorphLayer> {
        self.layers.pop()
    }

    pub fn insert(&mut self, index: usize, layer: MorphLayer) {
        self.layers.insert(index, layer);
    }

    pub fn remove(&mut self, index: usize) -> MorphLayer {
        self.layers.remove(index)
    }

    pub fn move_up(&mut self, index: usize) {
        if index > 0 {
            self.layers.swap(index, index - 1);
        }
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.layers.len() {
            self.layers.swap(index, index + 1);
        }
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn get(&self, index: usize) -> Option<&MorphLayer> {
        self.layers.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut MorphLayer> {
        self.layers.get_mut(index)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&MorphLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut MorphLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Evaluate all layers bottom-to-top, return composite morph weights
    pub fn evaluate(&self) -> HashMap<String, f32> {
        let mut base: HashMap<String, f32> = HashMap::new();
        for layer in &self.layers {
            if !layer.is_active() {
                continue;
            }
            base = blend_layer(&base, &layer.weights, &layer.blend_mode, layer.opacity);
        }
        base
    }

    /// All unique morph names across all layers
    pub fn all_morphs(&self) -> Vec<String> {
        let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for layer in &self.layers {
            for key in layer.weights.keys() {
                names.insert(key.clone());
            }
        }
        let mut result: Vec<String> = names.into_iter().collect();
        result.sort();
        result
    }
}

impl Default for MorphLayerStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Blend two morph weight maps using the given mode and opacity
pub fn blend_layer(
    base: &HashMap<String, f32>,
    layer: &HashMap<String, f32>,
    mode: &LayerBlend,
    opacity: f32,
) -> HashMap<String, f32> {
    // Collect all keys from both maps
    let mut all_keys: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for key in base.keys() {
        all_keys.insert(key.as_str());
    }
    for key in layer.keys() {
        all_keys.insert(key.as_str());
    }

    let mut result = HashMap::new();
    for key in all_keys {
        let a = base.get(key).copied().unwrap_or(0.0);
        let b_raw = layer.get(key).copied().unwrap_or(0.0);
        let b = b_raw * opacity;

        let blended = match mode {
            LayerBlend::Override => b,
            LayerBlend::Additive => a + b,
            LayerBlend::Multiply => a * b,
            LayerBlend::Screen => 1.0 - (1.0 - a) * (1.0 - b),
            LayerBlend::Normal => a * (1.0 - opacity) + b,
        };

        result.insert(key.to_string(), blended.clamp(0.0, 1.0));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn test_layer_new() {
        let layer = MorphLayer::new("base");
        assert_eq!(layer.name, "base");
        assert_eq!(layer.blend_mode, LayerBlend::Normal);
        assert_eq!(layer.opacity, 1.0);
        assert!(layer.enabled);
        assert!(layer.weights.is_empty());
        assert!(layer.mask.is_none());
    }

    #[test]
    fn test_layer_set_get_weight() {
        let mut layer = MorphLayer::new("test");
        layer.set_weight("smile", 0.75);
        layer.set_weight("blink", 0.5);
        assert!(approx_eq(layer.get_weight("smile"), 0.75));
        assert!(approx_eq(layer.get_weight("blink"), 0.5));
        assert!(approx_eq(layer.get_weight("missing"), 0.0));
    }

    #[test]
    fn test_layer_is_active() {
        let layer = MorphLayer::new("active");
        assert!(layer.is_active());

        let disabled = MorphLayer::new("disabled");
        let mut disabled = disabled;
        disabled.enabled = false;
        assert!(!disabled.is_active());

        let zero_opacity = MorphLayer::new("zero").with_opacity(0.0);
        assert!(!zero_opacity.is_active());

        let small_opacity = MorphLayer::new("small").with_opacity(0.001);
        assert!(small_opacity.is_active());
    }

    #[test]
    fn test_blend_override() {
        let mut base = HashMap::new();
        base.insert("smile".to_string(), 0.8);
        let mut layer = HashMap::new();
        layer.insert("smile".to_string(), 0.3);

        let result = blend_layer(&base, &layer, &LayerBlend::Override, 1.0);
        // Override: result = b = layer * opacity = 0.3 * 1.0 = 0.3
        assert!(approx_eq(result["smile"], 0.3));
    }

    #[test]
    fn test_blend_additive() {
        let mut base = HashMap::new();
        base.insert("smile".to_string(), 0.5);
        let mut layer = HashMap::new();
        layer.insert("smile".to_string(), 0.3);

        let result = blend_layer(&base, &layer, &LayerBlend::Additive, 1.0);
        // Additive: a + b = 0.5 + 0.3 = 0.8
        assert!(approx_eq(result["smile"], 0.8));
    }

    #[test]
    fn test_blend_multiply() {
        let mut base = HashMap::new();
        base.insert("smile".to_string(), 0.5);
        let mut layer = HashMap::new();
        layer.insert("smile".to_string(), 0.6);

        let result = blend_layer(&base, &layer, &LayerBlend::Multiply, 1.0);
        // Multiply: a * b = 0.5 * 0.6 = 0.3
        assert!(approx_eq(result["smile"], 0.3));
    }

    #[test]
    fn test_blend_screen() {
        let mut base = HashMap::new();
        base.insert("smile".to_string(), 0.5);
        let mut layer = HashMap::new();
        layer.insert("smile".to_string(), 0.5);

        let result = blend_layer(&base, &layer, &LayerBlend::Screen, 1.0);
        // Screen: 1 - (1 - 0.5) * (1 - 0.5) = 1 - 0.25 = 0.75
        assert!(approx_eq(result["smile"], 0.75));
    }

    #[test]
    fn test_blend_normal() {
        let mut base = HashMap::new();
        base.insert("smile".to_string(), 0.0);
        let mut layer = HashMap::new();
        layer.insert("smile".to_string(), 1.0);

        let result = blend_layer(&base, &layer, &LayerBlend::Normal, 0.5);
        // Normal: a * (1 - opacity) + b * opacity = 0.0 * 0.5 + (1.0 * 0.5) * 0.5 = 0.25
        // b = b_raw * opacity = 1.0 * 0.5 = 0.5; result = 0.0 * 0.5 + 0.5 = 0.5 * 0.5 = 0.25 ...
        // Actually: a*(1-opacity) + b = 0.0*0.5 + 0.5 = 0.5
        assert!(approx_eq(result["smile"], 0.5));
    }

    #[test]
    fn test_blend_opacity_zero() {
        let mut base = HashMap::new();
        base.insert("smile".to_string(), 0.8);
        let mut layer = HashMap::new();
        layer.insert("smile".to_string(), 0.3);

        // With opacity=0, all modes: b = b_raw * 0 = 0
        let result_add = blend_layer(&base, &layer, &LayerBlend::Additive, 0.0);
        // Additive: a + b = 0.8 + 0.0 = 0.8
        assert!(approx_eq(result_add["smile"], 0.8));

        let result_override = blend_layer(&base, &layer, &LayerBlend::Override, 0.0);
        // Override: b = 0.0
        assert!(approx_eq(result_override["smile"], 0.0));
    }

    #[test]
    fn test_stack_push_pop() {
        let mut stack = MorphLayerStack::new();
        assert_eq!(stack.layer_count(), 0);

        stack.push(MorphLayer::new("layer1"));
        stack.push(MorphLayer::new("layer2"));
        assert_eq!(stack.layer_count(), 2);

        let popped = stack.pop().unwrap();
        assert_eq!(popped.name, "layer2");
        assert_eq!(stack.layer_count(), 1);
    }

    #[test]
    fn test_stack_evaluate_empty() {
        let stack = MorphLayerStack::new();
        let result = stack.evaluate();
        assert!(result.is_empty());
    }

    #[test]
    fn test_stack_evaluate_single_layer() {
        let mut stack = MorphLayerStack::new();
        let mut layer = MorphLayer::new("base").with_blend(LayerBlend::Override);
        layer.set_weight("smile", 0.6);
        layer.set_weight("blink", 0.4);
        stack.push(layer);

        let result = stack.evaluate();
        // Override with opacity 1.0: b = weight * 1.0
        assert!(approx_eq(result["smile"], 0.6));
        assert!(approx_eq(result["blink"], 0.4));
    }

    #[test]
    fn test_stack_evaluate_two_layers() {
        let mut stack = MorphLayerStack::new();

        // Bottom layer: Override mode sets base
        let mut base_layer = MorphLayer::new("base").with_blend(LayerBlend::Override);
        base_layer.set_weight("smile", 0.5);
        stack.push(base_layer);

        // Top layer: Additive mode adds on top
        let mut additive_layer = MorphLayer::new("additive").with_blend(LayerBlend::Additive);
        additive_layer.set_weight("smile", 0.3);
        stack.push(additive_layer);

        let result = stack.evaluate();
        // After override: smile=0.5; after additive: 0.5 + 0.3 = 0.8
        assert!(approx_eq(result["smile"], 0.8));
    }

    #[test]
    fn test_stack_move_up_down() {
        let mut stack = MorphLayerStack::new();
        stack.push(MorphLayer::new("a"));
        stack.push(MorphLayer::new("b"));
        stack.push(MorphLayer::new("c"));

        // Move index 2 ("c") up to index 1
        stack.move_up(2);
        assert_eq!(stack.get(1).unwrap().name, "c");
        assert_eq!(stack.get(2).unwrap().name, "b");

        // Move index 1 ("c") down to index 2
        stack.move_down(1);
        assert_eq!(stack.get(1).unwrap().name, "b");
        assert_eq!(stack.get(2).unwrap().name, "c");

        // Move up at index 0 is a no-op
        stack.move_up(0);
        assert_eq!(stack.get(0).unwrap().name, "a");

        // Move down at last index is a no-op
        stack.move_down(2);
        assert_eq!(stack.get(2).unwrap().name, "c");
    }

    #[test]
    fn test_stack_all_morphs() {
        let mut stack = MorphLayerStack::new();

        let mut layer1 = MorphLayer::new("l1");
        layer1.set_weight("smile", 0.5);
        layer1.set_weight("blink", 0.3);

        let mut layer2 = MorphLayer::new("l2");
        layer2.set_weight("blink", 0.7);
        layer2.set_weight("frown", 0.2);

        stack.push(layer1);
        stack.push(layer2);

        let mut morphs = stack.all_morphs();
        morphs.sort();
        assert_eq!(morphs, vec!["blink", "frown", "smile"]);
    }
}
