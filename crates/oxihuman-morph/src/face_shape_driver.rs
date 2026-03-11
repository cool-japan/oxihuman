#![allow(dead_code)]
//! Face shape driver: maps input parameters to output morph targets via rules.

use std::collections::HashMap;

/// A single driver rule: input param scaled by factor produces output.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DriverRule {
    input: String,
    output: String,
    factor: f32,
}

/// Collection of driver rules.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceShapeDriver {
    rules: Vec<DriverRule>,
}

/// Create a new empty driver set.
#[allow(dead_code)]
pub fn new_face_shape_driver() -> FaceShapeDriver {
    FaceShapeDriver { rules: Vec::new() }
}

/// Add a rule: `output = input_value * factor`.
#[allow(dead_code)]
pub fn add_driver_rule(driver: &mut FaceShapeDriver, input: &str, output: &str, factor: f32) {
    driver.rules.push(DriverRule {
        input: input.to_string(),
        output: output.to_string(),
        factor,
    });
}

/// Evaluate all rules given input values, producing output values.
#[allow(dead_code)]
pub fn evaluate_drivers(
    driver: &FaceShapeDriver,
    inputs: &HashMap<String, f32>,
) -> HashMap<String, f32> {
    let mut result = HashMap::new();
    for rule in &driver.rules {
        let input_val = inputs.get(&rule.input).copied().unwrap_or(0.0);
        let entry = result.entry(rule.output.clone()).or_insert(0.0);
        *entry += input_val * rule.factor;
    }
    result
}

/// Return the number of rules.
#[allow(dead_code)]
pub fn driver_count(driver: &FaceShapeDriver) -> usize {
    driver.rules.len()
}

/// Return the output name of the rule at `index`.
#[allow(dead_code)]
pub fn driver_output(driver: &FaceShapeDriver, index: usize) -> &str {
    driver.rules.get(index).map_or("", |r| &r.output)
}

/// Return the input name of the rule at `index`.
#[allow(dead_code)]
pub fn driver_input(driver: &FaceShapeDriver, index: usize) -> &str {
    driver.rules.get(index).map_or("", |r| &r.input)
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn drivers_to_json(driver: &FaceShapeDriver) -> String {
    let entries: Vec<String> = driver
        .rules
        .iter()
        .map(|r| {
            format!(
                "{{\"input\":\"{}\",\"output\":\"{}\",\"factor\":{}}}",
                r.input, r.output, r.factor
            )
        })
        .collect();
    format!("{{\"rules\":[{}]}}", entries.join(","))
}

/// Remove all rules.
#[allow(dead_code)]
pub fn clear_drivers(driver: &mut FaceShapeDriver) {
    driver.rules.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_driver() {
        let d = new_face_shape_driver();
        assert_eq!(driver_count(&d), 0);
    }

    #[test]
    fn test_add_rule() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "jaw_open", "chin_down", 0.5);
        assert_eq!(driver_count(&d), 1);
    }

    #[test]
    fn test_evaluate_single() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "a", "b", 2.0);
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 0.5);
        let out = evaluate_drivers(&d, &inputs);
        assert!((out["b"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_missing_input() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "x", "y", 1.0);
        let inputs = HashMap::new();
        let out = evaluate_drivers(&d, &inputs);
        assert!((out["y"] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_driver_output_name() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "in", "out", 1.0);
        assert_eq!(driver_output(&d, 0), "out");
        assert_eq!(driver_output(&d, 99), "");
    }

    #[test]
    fn test_driver_input_name() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "in", "out", 1.0);
        assert_eq!(driver_input(&d, 0), "in");
    }

    #[test]
    fn test_drivers_to_json() {
        let d = new_face_shape_driver();
        let json = drivers_to_json(&d);
        assert!(json.contains("\"rules\":[]"));
    }

    #[test]
    fn test_clear_drivers() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "a", "b", 1.0);
        clear_drivers(&mut d);
        assert_eq!(driver_count(&d), 0);
    }

    #[test]
    fn test_multiple_rules_same_output() {
        let mut d = new_face_shape_driver();
        add_driver_rule(&mut d, "a", "out", 1.0);
        add_driver_rule(&mut d, "b", "out", 1.0);
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 0.3);
        inputs.insert("b".to_string(), 0.7);
        let out = evaluate_drivers(&d, &inputs);
        assert!((out["out"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let d = new_face_shape_driver();
        let inputs = HashMap::new();
        let out = evaluate_drivers(&d, &inputs);
        assert!(out.is_empty());
    }
}
