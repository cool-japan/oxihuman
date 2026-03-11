#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A single animation layer.
#[derive(Debug, Clone)]
pub struct AnimLayer {
    pub name: String,
    pub weight: f32,
    pub additive: bool,
    pub enabled: bool,
    pub mask_bones: Vec<String>,
}

/// Stack of animation layers.
#[derive(Debug, Clone)]
pub struct AnimLayerStack {
    pub layers: Vec<AnimLayer>,
}

#[allow(dead_code)]
pub fn new_anim_layer_stack() -> AnimLayerStack {
    AnimLayerStack { layers: Vec::new() }
}

#[allow(dead_code)]
pub fn add_layer(stack: &mut AnimLayerStack, name: &str, weight: f32, additive: bool) {
    stack.layers.push(AnimLayer {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
        additive,
        enabled: true,
        mask_bones: Vec::new(),
    });
}

#[allow(dead_code)]
pub fn set_layer_weight(stack: &mut AnimLayerStack, name: &str, weight: f32) {
    if let Some(l) = stack.layers.iter_mut().find(|l| l.name == name) {
        l.weight = weight.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn enable_layer(stack: &mut AnimLayerStack, name: &str, enabled: bool) {
    if let Some(l) = stack.layers.iter_mut().find(|l| l.name == name) {
        l.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn layer_count(stack: &AnimLayerStack) -> usize {
    stack.layers.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_empty() {
        let s = new_anim_layer_stack();
        assert_eq!(layer_count(&s), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "base", 1.0, false);
        assert_eq!(layer_count(&s), 1);
    }

    #[test]
    fn test_layer_name_stored() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "upper_body", 0.5, true);
        assert_eq!(s.layers[0].name, "upper_body");
    }

    #[test]
    fn test_layer_weight_clamped() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "test", 2.0, false);
        assert!((s.layers[0].weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_layer_weight() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "base", 1.0, false);
        set_layer_weight(&mut s, "base", 0.4);
        assert!((s.layers[0].weight - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_enable_layer_false() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "overlay", 0.5, false);
        enable_layer(&mut s, "overlay", false);
        assert!(!s.layers[0].enabled);
    }

    #[test]
    fn test_enable_layer_true() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "overlay", 0.5, false);
        enable_layer(&mut s, "overlay", false);
        enable_layer(&mut s, "overlay", true);
        assert!(s.layers[0].enabled);
    }

    #[test]
    fn test_additive_stored() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "additive_layer", 0.3, true);
        assert!(s.layers[0].additive);
    }

    #[test]
    fn test_multiple_layers() {
        let mut s = new_anim_layer_stack();
        add_layer(&mut s, "base", 1.0, false);
        add_layer(&mut s, "overlay", 0.5, true);
        assert_eq!(layer_count(&s), 2);
    }
}
