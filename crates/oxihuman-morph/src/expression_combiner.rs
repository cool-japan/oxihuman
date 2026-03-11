#![allow(dead_code)]
//! Expression combiner: combines multiple named expressions with weights.

use std::collections::HashMap;

/// An entry in the combiner.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CombinerEntry {
    name: String,
    weight: f32,
}

/// Combines multiple expressions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionCombiner {
    entries: Vec<CombinerEntry>,
}

/// Create a new empty combiner.
#[allow(dead_code)]
pub fn new_expression_combiner() -> ExpressionCombiner {
    ExpressionCombiner {
        entries: Vec::new(),
    }
}

/// Add an expression with a weight.
#[allow(dead_code)]
pub fn add_expression_ec(combiner: &mut ExpressionCombiner, name: &str, weight: f32) {
    combiner.entries.push(CombinerEntry {
        name: name.to_string(),
        weight,
    });
}

/// Combine all expressions into a weight map.
#[allow(dead_code)]
pub fn combine_expressions(combiner: &ExpressionCombiner) -> HashMap<String, f32> {
    let mut result = HashMap::new();
    for entry in &combiner.entries {
        let e = result.entry(entry.name.clone()).or_insert(0.0);
        *e += entry.weight;
    }
    result
}

/// Return the number of entries.
#[allow(dead_code)]
pub fn expression_count_ec(combiner: &ExpressionCombiner) -> usize {
    combiner.entries.len()
}

/// Return the weight of the entry at `index`.
#[allow(dead_code)]
pub fn expression_weight_ec(combiner: &ExpressionCombiner, index: usize) -> f32 {
    combiner.entries.get(index).map_or(0.0, |e| e.weight)
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn combiner_to_json(combiner: &ExpressionCombiner) -> String {
    let entries: Vec<String> = combiner
        .entries
        .iter()
        .map(|e| format!("{{\"name\":\"{}\",\"weight\":{}}}", e.name, e.weight))
        .collect();
    format!("{{\"expressions\":[{}]}}", entries.join(","))
}

/// Remove all entries.
#[allow(dead_code)]
pub fn combiner_clear(combiner: &mut ExpressionCombiner) {
    combiner.entries.clear();
}

/// Normalize combined weights so they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_combined(combiner: &ExpressionCombiner) -> HashMap<String, f32> {
    let mut result = combine_expressions(combiner);
    let total: f32 = result.values().map(|v| v.abs()).sum();
    if total > 1e-9 {
        for v in result.values_mut() {
            *v /= total;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_combiner() {
        let c = new_expression_combiner();
        assert_eq!(expression_count_ec(&c), 0);
    }

    #[test]
    fn test_add_expression() {
        let mut c = new_expression_combiner();
        add_expression_ec(&mut c, "smile", 0.5);
        assert_eq!(expression_count_ec(&c), 1);
    }

    #[test]
    fn test_combine() {
        let mut c = new_expression_combiner();
        add_expression_ec(&mut c, "smile", 0.5);
        let result = combine_expressions(&c);
        assert!((result["smile"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_combine_duplicate() {
        let mut c = new_expression_combiner();
        add_expression_ec(&mut c, "smile", 0.3);
        add_expression_ec(&mut c, "smile", 0.7);
        let result = combine_expressions(&c);
        assert!((result["smile"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_expression_weight() {
        let mut c = new_expression_combiner();
        add_expression_ec(&mut c, "frown", 0.8);
        assert!((expression_weight_ec(&c, 0) - 0.8).abs() < 1e-6);
        assert!((expression_weight_ec(&c, 99) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let c = new_expression_combiner();
        let json = combiner_to_json(&c);
        assert!(json.contains("\"expressions\":[]"));
    }

    #[test]
    fn test_clear() {
        let mut c = new_expression_combiner();
        add_expression_ec(&mut c, "a", 0.5);
        combiner_clear(&mut c);
        assert_eq!(expression_count_ec(&c), 0);
    }

    #[test]
    fn test_normalize() {
        let mut c = new_expression_combiner();
        add_expression_ec(&mut c, "a", 2.0);
        add_expression_ec(&mut c, "b", 2.0);
        let result = normalize_combined(&c);
        assert!((result["a"] - 0.5).abs() < 1e-6);
        assert!((result["b"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_empty() {
        let c = new_expression_combiner();
        let result = normalize_combined(&c);
        assert!(result.is_empty());
    }

    #[test]
    fn test_combine_empty() {
        let c = new_expression_combiner();
        let result = combine_expressions(&c);
        assert!(result.is_empty());
    }
}
