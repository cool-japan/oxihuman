#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphParameterSet {
    params: HashMap<String, f32>,
}

#[allow(dead_code)]
pub fn new_morph_parameter_set() -> MorphParameterSet {
    MorphParameterSet { params: HashMap::new() }
}

#[allow(dead_code)]
pub fn add_morph_param(set: &mut MorphParameterSet, name: &str, value: f32) {
    set.params.insert(name.to_string(), value);
}

#[allow(dead_code)]
pub fn get_morph_param(set: &MorphParameterSet, name: &str) -> f32 {
    set.params.get(name).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn morph_param_count_mps(set: &MorphParameterSet) -> usize { set.params.len() }

#[allow(dead_code)]
pub fn morph_param_names(set: &MorphParameterSet) -> Vec<String> {
    set.params.keys().cloned().collect()
}

#[allow(dead_code)]
pub fn morph_params_to_json(set: &MorphParameterSet) -> String {
    format!("{{\"count\":{}}}", set.params.len())
}

#[allow(dead_code)]
pub fn clear_morph_params(set: &mut MorphParameterSet) { set.params.clear(); }

#[allow(dead_code)]
pub fn morph_param_exists(set: &MorphParameterSet, name: &str) -> bool {
    set.params.contains_key(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let s = new_morph_parameter_set(); assert_eq!(morph_param_count_mps(&s), 0); }
    #[test] fn test_add_get() { let mut s = new_morph_parameter_set(); add_morph_param(&mut s, "x", 0.5); assert!((get_morph_param(&s, "x") - 0.5).abs() < 1e-6); }
    #[test] fn test_missing() { let s = new_morph_parameter_set(); assert!((get_morph_param(&s, "x")).abs() < 1e-6); }
    #[test] fn test_count() { let mut s = new_morph_parameter_set(); add_morph_param(&mut s, "a", 1.0); add_morph_param(&mut s, "b", 2.0); assert_eq!(morph_param_count_mps(&s), 2); }
    #[test] fn test_names() { let mut s = new_morph_parameter_set(); add_morph_param(&mut s, "x", 1.0); assert_eq!(morph_param_names(&s).len(), 1); }
    #[test] fn test_json() { let s = new_morph_parameter_set(); assert!(morph_params_to_json(&s).contains("count")); }
    #[test] fn test_clear() { let mut s = new_morph_parameter_set(); add_morph_param(&mut s, "x", 1.0); clear_morph_params(&mut s); assert_eq!(morph_param_count_mps(&s), 0); }
    #[test] fn test_exists() { let mut s = new_morph_parameter_set(); add_morph_param(&mut s, "x", 1.0); assert!(morph_param_exists(&s, "x")); }
    #[test] fn test_not_exists() { let s = new_morph_parameter_set(); assert!(!morph_param_exists(&s, "x")); }
    #[test] fn test_overwrite() { let mut s = new_morph_parameter_set(); add_morph_param(&mut s, "x", 1.0); add_morph_param(&mut s, "x", 2.0); assert!((get_morph_param(&s, "x") - 2.0).abs() < 1e-6); }
}
