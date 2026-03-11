#![allow(dead_code)]

use std::collections::HashMap;

/// Map of expression names to weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionWeightMap {
    weights: HashMap<String, f32>,
}

#[allow(dead_code)]
pub fn new_expression_weight_map() -> ExpressionWeightMap {
    ExpressionWeightMap { weights: HashMap::new() }
}

#[allow(dead_code)]
pub fn set_expr_weight(m: &mut ExpressionWeightMap, name: &str, w: f32) {
    m.weights.insert(name.to_string(), w);
}

#[allow(dead_code)]
pub fn get_expr_weight(m: &ExpressionWeightMap, name: &str) -> f32 {
    m.weights.get(name).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn expr_weight_count(m: &ExpressionWeightMap) -> usize { m.weights.len() }

#[allow(dead_code)]
pub fn normalize_expr_weights(m: &mut ExpressionWeightMap) {
    let sum: f32 = m.weights.values().map(|w| w.abs()).sum();
    if sum > 1e-9 { for w in m.weights.values_mut() { *w /= sum; } }
}

#[allow(dead_code)]
pub fn weight_map_to_json_ewm(m: &ExpressionWeightMap) -> String {
    let mut keys: Vec<&String> = m.weights.keys().collect();
    keys.sort();
    let e: Vec<String> = keys.iter().map(|k| format!("\"{}\":{:.4}", k, m.weights[*k])).collect();
    format!("{{{}}}", e.join(","))
}

#[allow(dead_code)]
pub fn clear_expr_weights(m: &mut ExpressionWeightMap) { m.weights.clear(); }

#[allow(dead_code)]
pub fn expr_weights_sum(m: &ExpressionWeightMap) -> f32 {
    m.weights.values().sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(expr_weight_count(&new_expression_weight_map()), 0); }
    #[test] fn test_set_get() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "smile", 0.8);
        assert!((get_expr_weight(&m, "smile") - 0.8).abs() < 1e-6);
    }
    #[test] fn test_get_missing() { assert!((get_expr_weight(&new_expression_weight_map(), "x")).abs() < 1e-6); }
    #[test] fn test_count() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "a", 1.0);
        set_expr_weight(&mut m, "b", 0.5);
        assert_eq!(expr_weight_count(&m), 2);
    }
    #[test] fn test_normalize() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "a", 2.0);
        set_expr_weight(&mut m, "b", 2.0);
        normalize_expr_weights(&mut m);
        assert!((get_expr_weight(&m, "a") - 0.5).abs() < 1e-6);
    }
    #[test] fn test_clear() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "x", 1.0);
        clear_expr_weights(&mut m);
        assert_eq!(expr_weight_count(&m), 0);
    }
    #[test] fn test_sum() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "a", 1.0);
        set_expr_weight(&mut m, "b", 2.0);
        assert!((expr_weights_sum(&m) - 3.0).abs() < 1e-6);
    }
    #[test] fn test_to_json() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "x", 1.0);
        assert!(weight_map_to_json_ewm(&m).contains("x"));
    }
    #[test] fn test_overwrite() {
        let mut m = new_expression_weight_map();
        set_expr_weight(&mut m, "x", 1.0);
        set_expr_weight(&mut m, "x", 2.0);
        assert!((get_expr_weight(&m, "x") - 2.0).abs() < 1e-6);
    }
    #[test] fn test_normalize_empty() {
        let mut m = new_expression_weight_map();
        normalize_expr_weights(&mut m);
        assert_eq!(expr_weight_count(&m), 0);
    }
}
