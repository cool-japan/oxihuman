// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Multi-layer morph blending.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphLayer {
    pub value: f32,
    pub weight: f32,
    pub blend_mode: u8, /* 0=additive, 1=multiplicative, 2=override */
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphLayerBlend {
    pub layers: Vec<MorphLayer>,
}

#[allow(dead_code)]
pub fn new_morph_layer_blend() -> MorphLayerBlend {
    MorphLayerBlend { layers: Vec::new() }
}

#[allow(dead_code)]
pub fn mlb_add_layer(blend: &mut MorphLayerBlend, value: f32, weight: f32, mode: u8) {
    blend.layers.push(MorphLayer { value, weight, blend_mode: mode });
}

#[allow(dead_code)]
pub fn mlb_evaluate(blend: &MorphLayerBlend, base: f32) -> f32 {
    let mut result = base;
    for layer in &blend.layers {
        match layer.blend_mode {
            0 => { result += layer.value * layer.weight; }
            1 => { result *= layer.value * layer.weight; }
            2 => { result = layer.value * layer.weight; }
            _ => {}
        }
    }
    result
}

#[allow(dead_code)]
pub fn mlb_layer_count(blend: &MorphLayerBlend) -> usize {
    blend.layers.len()
}

#[allow(dead_code)]
pub fn mlb_clear(blend: &mut MorphLayerBlend) {
    blend.layers.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_additive_layer() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 0.5, 1.0, 0);
        let r = mlb_evaluate(&b, 0.2);
        assert!((r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_multiplicative_layer() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 2.0, 1.0, 1);
        let r = mlb_evaluate(&b, 3.0);
        assert!((r - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_override_layer() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 0.9, 1.0, 2);
        let r = mlb_evaluate(&b, 0.1);
        assert!((r - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_layer_count() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 0.5, 1.0, 0);
        mlb_add_layer(&mut b, 0.3, 0.5, 1);
        assert_eq!(mlb_layer_count(&b), 2);
    }

    #[test]
    fn test_clear() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 0.5, 1.0, 0);
        mlb_clear(&mut b);
        assert_eq!(mlb_layer_count(&b), 0);
    }

    #[test]
    fn test_empty_returns_base() {
        let b = new_morph_layer_blend();
        assert!((mlb_evaluate(&b, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_weighted_additive() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 1.0, 0.5, 0);
        let r = mlb_evaluate(&b, 0.0);
        assert!((r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_additive_layers() {
        let mut b = new_morph_layer_blend();
        mlb_add_layer(&mut b, 0.3, 1.0, 0);
        mlb_add_layer(&mut b, 0.2, 1.0, 0);
        let r = mlb_evaluate(&b, 0.0);
        assert!((r - 0.5).abs() < 1e-6);
    }
}
