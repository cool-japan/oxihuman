// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! High-level expression composition from action units and morph sliders.

use std::collections::HashMap;

/// A single named layer that contributes morph weights to the final expression.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionComposerLayer {
    pub name: String,
    pub weight: f32,
    /// Morph name → weight pairs for this layer.
    pub morphs: HashMap<String, f32>,
}

/// The result of composing all layers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComposedExpression {
    /// Final blended morph weights after all layers are mixed.
    pub weights: HashMap<String, f32>,
    /// Number of layers that contributed.
    pub layer_count: usize,
}

/// Configuration for the expression composer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionComposerConfig {
    /// Whether to normalise output weights to the 0..1 range.
    pub auto_normalize: bool,
    /// Maximum number of layers allowed.
    pub max_layers: usize,
}

/// Runtime state of the expression composer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionComposer {
    pub config: ExpressionComposerConfig,
    pub layers: Vec<ExpressionComposerLayer>,
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Return a sensible default [`ExpressionComposerConfig`].
#[allow(dead_code)]
pub fn default_composer_config() -> ExpressionComposerConfig {
    ExpressionComposerConfig {
        auto_normalize: false,
        max_layers: 32,
    }
}

/// Create an empty [`ComposedExpression`].
#[allow(dead_code)]
pub fn new_composed_expression() -> ComposedExpression {
    ComposedExpression {
        weights: HashMap::new(),
        layer_count: 0,
    }
}

/// Create a new [`ExpressionComposer`] with the given config.
#[allow(dead_code)]
pub fn new_expression_composer(config: ExpressionComposerConfig) -> ExpressionComposer {
    ExpressionComposer {
        config,
        layers: Vec::new(),
    }
}

/// Add a layer by name with the given morphs and weight 1.0.
/// Returns `false` if the layer limit has been reached.
#[allow(dead_code)]
pub fn add_layer(
    composer: &mut ExpressionComposer,
    name: &str,
    morphs: HashMap<String, f32>,
) -> bool {
    if composer.layers.len() >= composer.config.max_layers {
        return false;
    }
    composer.layers.push(ExpressionComposerLayer {
        name: name.to_string(),
        weight: 1.0,
        morphs,
    });
    true
}

/// Remove the first layer whose name matches. Returns `true` if found.
#[allow(dead_code)]
pub fn remove_layer(composer: &mut ExpressionComposer, name: &str) -> bool {
    let before = composer.layers.len();
    composer.layers.retain(|l| l.name != name);
    composer.layers.len() < before
}

/// Blend all layers by their weights (weighted average per morph).
#[allow(dead_code)]
pub fn blend_layers(composer: &ExpressionComposer) -> ComposedExpression {
    let mut weights: HashMap<String, f32> = HashMap::new();
    let weight_sum: f32 = composer.layers.iter().map(|l| l.weight.abs()).sum();
    for layer in &composer.layers {
        let layer_w = if weight_sum > 0.0 {
            layer.weight / weight_sum
        } else {
            0.0
        };
        for (morph, &mv) in &layer.morphs {
            let entry = weights.entry(morph.clone()).or_insert(0.0);
            *entry += mv * layer_w;
        }
    }
    ComposedExpression {
        layer_count: composer.layers.len(),
        weights,
    }
}

/// Evaluate the final expression, applying auto-normalisation if configured.
#[allow(dead_code)]
pub fn evaluate_expression(composer: &ExpressionComposer) -> ComposedExpression {
    let mut expr = blend_layers(composer);
    if composer.config.auto_normalize {
        normalize_expression(&mut expr);
    }
    expr
}

/// Return the number of layers in the composer.
#[allow(dead_code)]
pub fn layer_count(composer: &ExpressionComposer) -> usize {
    composer.layers.len()
}

/// Set the weight for the named layer (clamped 0..1). Returns `false` if not found.
#[allow(dead_code)]
pub fn set_layer_weight(composer: &mut ExpressionComposer, name: &str, weight: f32) -> bool {
    for layer in &mut composer.layers {
        if layer.name == name {
            layer.weight = weight.clamp(0.0, 1.0);
            return true;
        }
    }
    false
}

/// Get the weight of the named layer, or `None` if not found.
#[allow(dead_code)]
pub fn get_layer_weight(composer: &ExpressionComposer, name: &str) -> Option<f32> {
    composer
        .layers
        .iter()
        .find(|l| l.name == name)
        .map(|l| l.weight)
}

/// Serialise the composed expression weights to a simple JSON string.
#[allow(dead_code)]
pub fn expression_to_json(expr: &ComposedExpression) -> String {
    let mut pairs: Vec<String> = expr
        .weights
        .iter()
        .map(|(k, v)| format!("  \"{k}\": {v:.4}"))
        .collect();
    pairs.sort();
    format!("{{\n{}\n}}", pairs.join(",\n"))
}

/// Reset the composer by removing all layers.
#[allow(dead_code)]
pub fn reset_expression(composer: &mut ExpressionComposer) {
    composer.layers.clear();
}

/// Add a named preset layer from a slice of `(morph_name, weight)` pairs.
#[allow(dead_code)]
pub fn add_preset_layer(
    composer: &mut ExpressionComposer,
    name: &str,
    presets: &[(&str, f32)],
) -> bool {
    let morphs: HashMap<String, f32> = presets.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    add_layer(composer, name, morphs)
}

/// Compute the total "energy" of a composed expression (sum of absolute weights).
#[allow(dead_code)]
pub fn expression_energy(expr: &ComposedExpression) -> f32 {
    expr.weights.values().map(|v| v.abs()).sum()
}

/// Normalise all weights in a [`ComposedExpression`] so the max is 1.0.
/// No-op if all weights are zero.
#[allow(dead_code)]
pub fn normalize_expression(expr: &mut ComposedExpression) {
    let max = expr.weights.values().cloned().fold(0.0_f32, f32::max);
    if max > 0.0 {
        for v in expr.weights.values_mut() {
            *v /= max;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_morphs(v: f32) -> HashMap<String, f32> {
        let mut m = HashMap::new();
        m.insert("smile".to_string(), v);
        m.insert("frown".to_string(), v * 0.5);
        m
    }

    fn make_composer() -> ExpressionComposer {
        new_expression_composer(default_composer_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_composer_config();
        assert!(!cfg.auto_normalize);
        assert!(cfg.max_layers > 0);
    }

    #[test]
    fn test_new_composed_expression_empty() {
        let expr = new_composed_expression();
        assert!(expr.weights.is_empty());
        assert_eq!(expr.layer_count, 0);
    }

    #[test]
    fn test_add_layer_increases_count() {
        let mut c = make_composer();
        add_layer(&mut c, "happy", simple_morphs(1.0));
        assert_eq!(layer_count(&c), 1);
    }

    #[test]
    fn test_remove_layer() {
        let mut c = make_composer();
        add_layer(&mut c, "happy", simple_morphs(1.0));
        let removed = remove_layer(&mut c, "happy");
        assert!(removed);
        assert_eq!(layer_count(&c), 0);
    }

    #[test]
    fn test_remove_layer_not_found() {
        let mut c = make_composer();
        let removed = remove_layer(&mut c, "missing");
        assert!(!removed);
    }

    #[test]
    fn test_blend_layers_single_layer() {
        let mut c = make_composer();
        add_layer(&mut c, "happy", simple_morphs(0.8));
        let expr = blend_layers(&c);
        assert!(!expr.weights.is_empty());
        assert_eq!(expr.layer_count, 1);
    }

    #[test]
    fn test_blend_layers_empty() {
        let c = make_composer();
        let expr = blend_layers(&c);
        assert!(expr.weights.is_empty());
    }

    #[test]
    fn test_evaluate_expression_returns_weights() {
        let mut c = make_composer();
        add_layer(&mut c, "sad", simple_morphs(0.5));
        let expr = evaluate_expression(&c);
        assert!(expr.weights.contains_key("smile"));
    }

    #[test]
    fn test_set_layer_weight() {
        let mut c = make_composer();
        add_layer(&mut c, "angry", simple_morphs(1.0));
        let ok = set_layer_weight(&mut c, "angry", 0.3);
        assert!(ok);
        assert!((get_layer_weight(&c, "angry").unwrap() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_layer_weight_not_found() {
        let mut c = make_composer();
        let ok = set_layer_weight(&mut c, "ghost", 0.5);
        assert!(!ok);
    }

    #[test]
    fn test_get_layer_weight_none() {
        let c = make_composer();
        assert!(get_layer_weight(&c, "nonexistent").is_none());
    }

    #[test]
    fn test_expression_to_json_contains_keys() {
        let mut c = make_composer();
        add_layer(&mut c, "test", simple_morphs(1.0));
        let expr = evaluate_expression(&c);
        let json = expression_to_json(&expr);
        assert!(json.contains("smile"));
    }

    #[test]
    fn test_reset_expression() {
        let mut c = make_composer();
        add_layer(&mut c, "layer1", simple_morphs(1.0));
        reset_expression(&mut c);
        assert_eq!(layer_count(&c), 0);
    }

    #[test]
    fn test_add_preset_layer() {
        let mut c = make_composer();
        let ok = add_preset_layer(&mut c, "joy", &[("smile", 0.9), ("brow_raise", 0.4)]);
        assert!(ok);
        assert_eq!(layer_count(&c), 1);
    }

    #[test]
    fn test_expression_energy() {
        let mut c = make_composer();
        add_layer(&mut c, "full", simple_morphs(1.0));
        let expr = evaluate_expression(&c);
        let energy = expression_energy(&expr);
        assert!(energy > 0.0);
    }

    #[test]
    fn test_normalize_expression() {
        let mut expr = new_composed_expression();
        expr.weights.insert("a".to_string(), 2.0);
        expr.weights.insert("b".to_string(), 1.0);
        normalize_expression(&mut expr);
        assert!((expr.weights["a"] - 1.0).abs() < 1e-6);
        assert!((expr.weights["b"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_expression_zero_noop() {
        let mut expr = new_composed_expression();
        expr.weights.insert("x".to_string(), 0.0);
        normalize_expression(&mut expr);
        assert_eq!(expr.weights["x"], 0.0);
    }

    #[test]
    fn test_layer_count_limit() {
        let mut c = new_expression_composer(ExpressionComposerConfig {
            auto_normalize: false,
            max_layers: 2,
        });
        assert!(add_layer(&mut c, "l1", simple_morphs(1.0)));
        assert!(add_layer(&mut c, "l2", simple_morphs(1.0)));
        assert!(!add_layer(&mut c, "l3", simple_morphs(1.0)));
        assert_eq!(layer_count(&c), 2);
    }

    #[test]
    fn test_two_layer_blend() {
        let mut c = make_composer();
        let mut m1 = HashMap::new();
        m1.insert("x".to_string(), 1.0_f32);
        let mut m2 = HashMap::new();
        m2.insert("x".to_string(), 0.0_f32);
        add_layer(&mut c, "a", m1);
        add_layer(&mut c, "b", m2);
        let expr = blend_layers(&c);
        // equal weights → average of 1.0 and 0.0 = 0.5
        assert!((expr.weights["x"] - 0.5).abs() < 1e-6);
    }
}
