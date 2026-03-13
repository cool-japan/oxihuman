// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A character template with named parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CharacterTemplate {
    pub name: String,
    pub params: HashMap<String, f32>,
}

/// Create a new empty character template.
#[allow(dead_code)]
pub fn new_character_template(name: &str) -> CharacterTemplate {
    CharacterTemplate {
        name: name.to_string(),
        params: HashMap::new(),
    }
}

/// Set a parameter on the template.
#[allow(dead_code)]
pub fn template_set_param(t: &mut CharacterTemplate, key: &str, value: f32) {
    t.params.insert(key.to_string(), value);
}

/// Get a parameter from the template.
#[allow(dead_code)]
pub fn template_get_param(t: &CharacterTemplate, key: &str) -> Option<f32> {
    t.params.get(key).copied()
}

/// Return the number of parameters.
#[allow(dead_code)]
pub fn template_param_count(t: &CharacterTemplate) -> usize {
    t.params.len()
}

/// Return the template name.
#[allow(dead_code)]
pub fn template_name(t: &CharacterTemplate) -> &str {
    &t.name
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn template_to_json(t: &CharacterTemplate) -> String {
    let mut entries: Vec<String> = t
        .params
        .iter()
        .map(|(k, v)| format!("\"{}\":{:.4}", k, v))
        .collect();
    entries.sort();
    format!("{{\"name\":\"{}\",\"params\":{{{}}}}}", t.name, entries.join(","))
}

/// Apply the template by returning its params as a flat HashMap.
#[allow(dead_code)]
pub fn template_apply(t: &CharacterTemplate) -> HashMap<String, f32> {
    t.params.clone()
}

/// Clear all parameters.
#[allow(dead_code)]
pub fn template_clear(t: &mut CharacterTemplate) {
    t.params.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_template() {
        let t = new_character_template("hero");
        assert_eq!(template_param_count(&t), 0);
    }

    #[test]
    fn set_and_get() {
        let mut t = new_character_template("hero");
        template_set_param(&mut t, "height", 1.8);
        assert!((template_get_param(&t, "height").expect("should succeed") - 1.8).abs() < 1e-6);
    }

    #[test]
    fn get_missing() {
        let t = new_character_template("hero");
        assert!(template_get_param(&t, "nope").is_none());
    }

    #[test]
    fn param_count() {
        let mut t = new_character_template("hero");
        template_set_param(&mut t, "a", 1.0);
        template_set_param(&mut t, "b", 2.0);
        assert_eq!(template_param_count(&t), 2);
    }

    #[test]
    fn name_accessor() {
        let t = new_character_template("villain");
        assert_eq!(template_name(&t), "villain");
    }

    #[test]
    fn to_json() {
        let mut t = new_character_template("test");
        template_set_param(&mut t, "x", 0.5);
        let j = template_to_json(&t);
        assert!(j.contains("\"test\""));
    }

    #[test]
    fn apply_returns_params() {
        let mut t = new_character_template("t");
        template_set_param(&mut t, "x", 0.5);
        let p = template_apply(&t);
        assert!((p["x"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clear_params() {
        let mut t = new_character_template("t");
        template_set_param(&mut t, "x", 0.5);
        template_clear(&mut t);
        assert_eq!(template_param_count(&t), 0);
    }

    #[test]
    fn overwrite_param() {
        let mut t = new_character_template("t");
        template_set_param(&mut t, "x", 0.5);
        template_set_param(&mut t, "x", 0.9);
        assert!((template_get_param(&t, "x").expect("should succeed") - 0.9).abs() < 1e-6);
    }

    #[test]
    fn apply_empty() {
        let t = new_character_template("t");
        let p = template_apply(&t);
        assert!(p.is_empty());
    }
}
