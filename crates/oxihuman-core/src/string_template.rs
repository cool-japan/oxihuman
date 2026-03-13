#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simple string template engine with `{{variable}}` substitution.

use std::collections::HashMap;

/// A string template with named variables.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringTemplate {
    source: String,
    vars: HashMap<String, String>,
}

#[allow(dead_code)]
pub fn new_template(source: &str) -> StringTemplate {
    StringTemplate {
        source: source.to_string(),
        vars: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn render_template(t: &StringTemplate) -> String {
    let mut result = t.source.clone();
    for (k, v) in &t.vars {
        let pat = format!("{{{{{}}}}}", k);
        result = result.replace(&pat, v);
    }
    result
}

#[allow(dead_code)]
pub fn set_variable(t: &mut StringTemplate, key: &str, value: &str) {
    t.vars.insert(key.to_string(), value.to_string());
}

#[allow(dead_code)]
pub fn variable_count(t: &StringTemplate) -> usize {
    t.vars.len()
}

#[allow(dead_code)]
pub fn has_variable(t: &StringTemplate, key: &str) -> bool {
    t.vars.contains_key(key)
}

#[allow(dead_code)]
pub fn template_source(t: &StringTemplate) -> &str {
    &t.source
}

#[allow(dead_code)]
pub fn clear_variables(t: &mut StringTemplate) {
    t.vars.clear();
}

#[allow(dead_code)]
pub fn validate_template(t: &StringTemplate) -> bool {
    // Check that all {{ have matching }}.
    let src = &t.source;
    let opens = src.matches("{{").count();
    let closes = src.matches("}}").count();
    opens == closes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_template() {
        let t = new_template("hello {{name}}");
        assert_eq!(template_source(&t), "hello {{name}}");
    }

    #[test]
    fn test_render_with_var() {
        let mut t = new_template("hi {{who}}!");
        set_variable(&mut t, "who", "world");
        assert_eq!(render_template(&t), "hi world!");
    }

    #[test]
    fn test_render_no_vars() {
        let t = new_template("plain text");
        assert_eq!(render_template(&t), "plain text");
    }

    #[test]
    fn test_variable_count() {
        let mut t = new_template("");
        set_variable(&mut t, "a", "1");
        set_variable(&mut t, "b", "2");
        assert_eq!(variable_count(&t), 2);
    }

    #[test]
    fn test_has_variable() {
        let mut t = new_template("");
        set_variable(&mut t, "x", "y");
        assert!(has_variable(&t, "x"));
        assert!(!has_variable(&t, "z"));
    }

    #[test]
    fn test_clear_variables() {
        let mut t = new_template("");
        set_variable(&mut t, "a", "1");
        clear_variables(&mut t);
        assert_eq!(variable_count(&t), 0);
    }

    #[test]
    fn test_validate_good() {
        let t = new_template("{{a}} and {{b}}");
        assert!(validate_template(&t));
    }

    #[test]
    fn test_validate_bad() {
        let t = new_template("{{a} missing close");
        assert!(!validate_template(&t));
    }

    #[test]
    fn test_multiple_same_var() {
        let mut t = new_template("{{x}} + {{x}}");
        set_variable(&mut t, "x", "1");
        assert_eq!(render_template(&t), "1 + 1");
    }

    #[test]
    fn test_empty_template() {
        let t = new_template("");
        assert_eq!(render_template(&t), "");
        assert!(validate_template(&t));
    }
}
