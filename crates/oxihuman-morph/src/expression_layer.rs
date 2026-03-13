// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ExpressionLayer — weight-stacked expression layers.

#![allow(dead_code)]

/// A single expression layer with a name, weight, and per-param values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionLayer {
    pub name: String,
    pub weight: f32,
    pub params: Vec<f32>,
}

/// A stack of expression layers evaluated from bottom to top.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LayerStack {
    pub layers: Vec<ExpressionLayer>,
}

/// Create an empty `ExpressionLayer` with the given name and weight.
#[allow(dead_code)]
pub fn new_expression_layer(name: &str, weight: f32, params: Vec<f32>) -> ExpressionLayer {
    ExpressionLayer { name: name.to_owned(), weight, params }
}

/// Push a layer onto the stack.
#[allow(dead_code)]
pub fn push_layer(stack: &mut LayerStack, layer: ExpressionLayer) {
    stack.layers.push(layer);
}

/// Pop the topmost layer from the stack.
#[allow(dead_code)]
pub fn pop_layer(stack: &mut LayerStack) -> Option<ExpressionLayer> {
    stack.layers.pop()
}

/// Evaluate the stack: additive blend of all layers weighted by their `weight`.
/// Returns a vec of length equal to the longest param array.
#[allow(dead_code)]
pub fn evaluate_layer_stack(stack: &LayerStack) -> Vec<f32> {
    let max_len = stack.layers.iter().map(|l| l.params.len()).max().unwrap_or(0);
    let mut out = vec![0.0_f32; max_len];
    for layer in &stack.layers {
        for (i, &p) in layer.params.iter().enumerate() {
            out[i] += p * layer.weight;
        }
    }
    out
}

/// Return the number of layers in the stack.
#[allow(dead_code)]
pub fn layer_count(stack: &LayerStack) -> usize {
    stack.layers.len()
}

/// Return the weight of the layer at `index`.
#[allow(dead_code)]
pub fn layer_weight(stack: &LayerStack, index: usize) -> f32 {
    stack.layers.get(index).map(|l| l.weight).unwrap_or(0.0)
}

/// Return the name of the layer at `index`.
#[allow(dead_code)]
pub fn layer_name(stack: &LayerStack, index: usize) -> Option<&str> {
    stack.layers.get(index).map(|l| l.name.as_str())
}

/// Flatten all layers into a single `ExpressionLayer` with the additive blend result.
#[allow(dead_code)]
pub fn flatten_layers(stack: &LayerStack) -> ExpressionLayer {
    let params = evaluate_layer_stack(stack);
    ExpressionLayer { name: "flattened".to_owned(), weight: 1.0, params }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expression_layer() {
        let l = new_expression_layer("smile", 0.5, vec![1.0, 0.5]);
        assert_eq!(l.name, "smile");
        assert!((l.weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_push_layer_increments_count() {
        let mut stack = LayerStack::default();
        push_layer(&mut stack, new_expression_layer("a", 1.0, vec![0.0]));
        assert_eq!(layer_count(&stack), 1);
    }

    #[test]
    fn test_pop_layer_returns_top() {
        let mut stack = LayerStack::default();
        push_layer(&mut stack, new_expression_layer("a", 1.0, vec![]));
        push_layer(&mut stack, new_expression_layer("b", 0.5, vec![]));
        let top = pop_layer(&mut stack).expect("should succeed");
        assert_eq!(top.name, "b");
        assert_eq!(layer_count(&stack), 1);
    }

    #[test]
    fn test_pop_empty_returns_none() {
        let mut stack = LayerStack::default();
        assert!(pop_layer(&mut stack).is_none());
    }

    #[test]
    fn test_evaluate_layer_stack_additive() {
        let mut stack = LayerStack::default();
        push_layer(&mut stack, new_expression_layer("a", 1.0, vec![1.0, 0.0]));
        push_layer(&mut stack, new_expression_layer("b", 0.5, vec![0.0, 2.0]));
        let out = evaluate_layer_stack(&stack);
        assert!((out[0] - 1.0).abs() < 1e-6);
        assert!((out[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_layer_weight() {
        let mut stack = LayerStack::default();
        push_layer(&mut stack, new_expression_layer("x", 0.7, vec![]));
        assert!((layer_weight(&stack, 0) - 0.7).abs() < 1e-6);
        assert_eq!(layer_weight(&stack, 99), 0.0);
    }

    #[test]
    fn test_layer_name() {
        let mut stack = LayerStack::default();
        push_layer(&mut stack, new_expression_layer("joy", 1.0, vec![]));
        assert_eq!(layer_name(&stack, 0), Some("joy"));
        assert!(layer_name(&stack, 1).is_none());
    }

    #[test]
    fn test_flatten_layers() {
        let mut stack = LayerStack::default();
        push_layer(&mut stack, new_expression_layer("a", 1.0, vec![0.5]));
        push_layer(&mut stack, new_expression_layer("b", 1.0, vec![0.5]));
        let flat = flatten_layers(&stack);
        assert_eq!(flat.name, "flattened");
        assert!((flat.params[0] - 1.0).abs() < 1e-6);
    }
}
