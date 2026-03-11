// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Template placeholder substitution map.
#[allow(dead_code)]
pub struct PlaceholderMap {
    vars: HashMap<String, String>,
    open: String,
    close: String,
    substituted: u64,
}

#[allow(dead_code)]
impl PlaceholderMap {
    pub fn new(open: &str, close: &str) -> Self {
        Self {
            vars: HashMap::new(),
            open: open.to_string(),
            close: close.to_string(),
            substituted: 0,
        }
    }
    pub fn set(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }
    pub fn remove(&mut self, key: &str) -> bool {
        self.vars.remove(key).is_some()
    }
    pub fn contains(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }
    pub fn render(&mut self, template: &str) -> String {
        let mut out = template.to_string();
        for (k, v) in &self.vars {
            let placeholder = format!("{}{}{}", self.open, k, self.close);
            if out.contains(&placeholder) {
                out = out.replace(&placeholder, v);
                self.substituted += 1;
            }
        }
        out
    }
    pub fn len(&self) -> usize {
        self.vars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }
    pub fn substituted_count(&self) -> u64 {
        self.substituted
    }
    pub fn clear(&mut self) {
        self.vars.clear();
    }
    pub fn keys(&self) -> Vec<&str> {
        self.vars.keys().map(|s| s.as_str()).collect()
    }
    pub fn has_unresolved(&self, template: &str) -> bool {
        self.vars.keys().any(|k| {
            let placeholder = format!("{}{}{}", self.open, k, self.close);
            template.contains(&placeholder)
        }) || (template.contains(&self.open) && template.contains(&self.close))
    }
}

#[allow(dead_code)]
pub fn new_placeholder_map(open: &str, close: &str) -> PlaceholderMap {
    PlaceholderMap::new(open, close)
}
#[allow(dead_code)]
pub fn plm_set(m: &mut PlaceholderMap, k: &str, v: &str) {
    m.set(k, v);
}
#[allow(dead_code)]
pub fn plm_get<'a>(m: &'a PlaceholderMap, k: &str) -> Option<&'a str> {
    m.get(k)
}
#[allow(dead_code)]
pub fn plm_render(m: &mut PlaceholderMap, template: &str) -> String {
    m.render(template)
}
#[allow(dead_code)]
pub fn plm_len(m: &PlaceholderMap) -> usize {
    m.len()
}
#[allow(dead_code)]
pub fn plm_is_empty(m: &PlaceholderMap) -> bool {
    m.is_empty()
}
#[allow(dead_code)]
pub fn plm_clear(m: &mut PlaceholderMap) {
    m.clear();
}
#[allow(dead_code)]
pub fn plm_substituted(m: &PlaceholderMap) -> u64 {
    m.substituted_count()
}
#[allow(dead_code)]
pub fn plm_contains(m: &PlaceholderMap, k: &str) -> bool {
    m.contains(k)
}
#[allow(dead_code)]
pub fn plm_remove(m: &mut PlaceholderMap, k: &str) -> bool {
    m.remove(k)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_render() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "name", "Alice");
        let r = plm_render(&mut m, "Hello, {{name}}!");
        assert_eq!(r, "Hello, Alice!");
    }
    #[test]
    fn test_multiple_vars() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "a", "X");
        plm_set(&mut m, "b", "Y");
        let r = plm_render(&mut m, "{{a}} and {{b}}");
        assert!(r.contains("X") && r.contains("Y"));
    }
    #[test]
    fn test_no_match() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "x", "X");
        let r = plm_render(&mut m, "no placeholders here");
        assert_eq!(r, "no placeholders here");
    }
    #[test]
    fn test_substituted_count() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "v", "42");
        plm_render(&mut m, "{{v}}+{{v}}");
        assert!(plm_substituted(&m) > 0);
    }
    #[test]
    fn test_contains_remove() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "k", "v");
        assert!(plm_contains(&m, "k"));
        plm_remove(&mut m, "k");
        assert!(!plm_contains(&m, "k"));
    }
    #[test]
    fn test_len() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "a", "1");
        plm_set(&mut m, "b", "2");
        assert_eq!(plm_len(&m), 2);
    }
    #[test]
    fn test_clear() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "a", "1");
        plm_clear(&mut m);
        assert!(plm_is_empty(&m));
    }
    #[test]
    fn test_custom_delimiters() {
        let mut m = new_placeholder_map("${", "}");
        plm_set(&mut m, "v", "world");
        let r = plm_render(&mut m, "Hello ${v}!");
        assert_eq!(r, "Hello world!");
    }
    #[test]
    fn test_get() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "key", "val");
        assert_eq!(plm_get(&m, "key"), Some("val"));
        assert_eq!(plm_get(&m, "missing"), None);
    }
    #[test]
    fn test_keys() {
        let mut m = new_placeholder_map("{{", "}}");
        plm_set(&mut m, "x", "1");
        plm_set(&mut m, "y", "2");
        assert_eq!(m.keys().len(), 2);
    }
}
