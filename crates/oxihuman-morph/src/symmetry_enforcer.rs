#![allow(dead_code)]
//! Enforces bilateral symmetry on morph parameters.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SymmetryEnforcer {
    axis: String,
    pairs: Vec<(String, String)>,
}

#[allow(dead_code)]
pub fn new_symmetry_enforcer(axis: &str) -> SymmetryEnforcer {
    SymmetryEnforcer {
        axis: axis.to_string(),
        pairs: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn enforce_symmetry(
    enforcer: &SymmetryEnforcer,
    params: &mut HashMap<String, f32>,
) {
    for (l, r) in &enforcer.pairs {
        let avg = {
            let lv = params.get(l).copied().unwrap_or(0.0);
            let rv = params.get(r).copied().unwrap_or(0.0);
            (lv + rv) * 0.5
        };
        params.insert(l.clone(), avg);
        params.insert(r.clone(), avg);
    }
}

#[allow(dead_code)]
pub fn symmetry_error(enforcer: &SymmetryEnforcer, params: &HashMap<String, f32>) -> f32 {
    let mut total = 0.0_f32;
    for (l, r) in &enforcer.pairs {
        let lv = params.get(l).copied().unwrap_or(0.0);
        let rv = params.get(r).copied().unwrap_or(0.0);
        total += (lv - rv).abs();
    }
    total
}

#[allow(dead_code)]
pub fn mirror_param(enforcer: &mut SymmetryEnforcer, left: &str, right: &str) {
    enforcer
        .pairs
        .push((left.to_string(), right.to_string()));
}

#[allow(dead_code)]
pub fn symmetry_axis(enforcer: &SymmetryEnforcer) -> &str {
    &enforcer.axis
}

#[allow(dead_code)]
pub fn left_side_params(enforcer: &SymmetryEnforcer) -> Vec<&str> {
    enforcer.pairs.iter().map(|(l, _)| l.as_str()).collect()
}

#[allow(dead_code)]
pub fn right_side_params(enforcer: &SymmetryEnforcer) -> Vec<&str> {
    enforcer.pairs.iter().map(|(_, r)| r.as_str()).collect()
}

#[allow(dead_code)]
pub fn symmetry_to_json(enforcer: &SymmetryEnforcer) -> String {
    let pairs: Vec<String> = enforcer
        .pairs
        .iter()
        .map(|(l, r)| format!("[\"{l}\",\"{r}\"]"))
        .collect();
    format!(
        "{{\"axis\":\"{}\",\"pairs\":[{}]}}",
        enforcer.axis,
        pairs.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_symmetry_enforcer() {
        let e = new_symmetry_enforcer("x");
        assert_eq!(symmetry_axis(&e), "x");
    }

    #[test]
    fn test_mirror_param() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "arm_l", "arm_r");
        assert_eq!(left_side_params(&e).len(), 1);
    }

    #[test]
    fn test_enforce_symmetry() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "a_l", "a_r");
        let mut params = HashMap::new();
        params.insert("a_l".to_string(), 0.2);
        params.insert("a_r".to_string(), 0.8);
        enforce_symmetry(&e, &mut params);
        assert!((params["a_l"] - 0.5).abs() < 1e-6);
        assert!((params["a_r"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_error() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "a_l", "a_r");
        let mut params = HashMap::new();
        params.insert("a_l".to_string(), 0.0);
        params.insert("a_r".to_string(), 1.0);
        assert!((symmetry_error(&e, &params) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_error_zero() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "a_l", "a_r");
        let mut params = HashMap::new();
        params.insert("a_l".to_string(), 0.5);
        params.insert("a_r".to_string(), 0.5);
        assert!(symmetry_error(&e, &params).abs() < 1e-6);
    }

    #[test]
    fn test_left_side_params() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "eye_l", "eye_r");
        mirror_param(&mut e, "arm_l", "arm_r");
        assert_eq!(left_side_params(&e), vec!["eye_l", "arm_l"]);
    }

    #[test]
    fn test_right_side_params() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "eye_l", "eye_r");
        assert_eq!(right_side_params(&e), vec!["eye_r"]);
    }

    #[test]
    fn test_symmetry_to_json() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "l", "r");
        let json = symmetry_to_json(&e);
        assert!(json.contains("\"axis\":\"x\""));
    }

    #[test]
    fn test_empty_enforcer() {
        let e = new_symmetry_enforcer("y");
        let params = HashMap::new();
        assert!(symmetry_error(&e, &params).abs() < 1e-6);
    }

    #[test]
    fn test_enforce_missing_params() {
        let mut e = new_symmetry_enforcer("x");
        mirror_param(&mut e, "a_l", "a_r");
        let mut params = HashMap::new();
        enforce_symmetry(&e, &mut params);
        assert!((params["a_l"]).abs() < 1e-6);
    }
}
