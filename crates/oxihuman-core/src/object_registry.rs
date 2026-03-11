// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
pub struct ObjectRegistry {
    entries: HashMap<String, String>,
    type_tags: HashMap<String, String>,
}

#[allow(dead_code)]
impl ObjectRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            type_tags: HashMap::new(),
        }
    }
    pub fn register(&mut self, key: &str, value: &str, type_tag: &str) {
        self.entries.insert(key.to_string(), value.to_string());
        self.type_tags.insert(key.to_string(), type_tag.to_string());
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }
    pub fn get_type(&self, key: &str) -> Option<&str> {
        self.type_tags.get(key).map(|s| s.as_str())
    }
    pub fn remove(&mut self, key: &str) -> bool {
        if self.entries.remove(key).is_some() {
            self.type_tags.remove(key);
            true
        } else {
            false
        }
    }
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn by_type(&self, type_tag: &str) -> Vec<&str> {
        self.type_tags
            .iter()
            .filter(|(_, t)| t.as_str() == type_tag)
            .map(|(k, _)| k.as_str())
            .collect()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.type_tags.clear();
    }
    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }
    pub fn to_json(&self) -> String {
        let mut out = String::from("{");
        let mut first = true;
        for (k, v) in &self.entries {
            if !first {
                out.push(',');
            }
            let tag = self.type_tags.get(k).map(|s| s.as_str()).unwrap_or("");
            out.push_str(&format!(
                "\"{}\":{{\"value\":\"{}\",\"type\":\"{}\"}}",
                k, v, tag
            ));
            first = false;
        }
        out.push('}');
        out
    }
}

impl Default for ObjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_object_registry() -> ObjectRegistry {
    ObjectRegistry::new()
}
#[allow(dead_code)]
pub fn or_register(r: &mut ObjectRegistry, key: &str, value: &str, type_tag: &str) {
    r.register(key, value, type_tag);
}
#[allow(dead_code)]
pub fn or_get<'a>(r: &'a ObjectRegistry, key: &str) -> Option<&'a str> {
    r.get(key)
}
#[allow(dead_code)]
pub fn or_remove(r: &mut ObjectRegistry, key: &str) -> bool {
    r.remove(key)
}
#[allow(dead_code)]
pub fn or_contains(r: &ObjectRegistry, key: &str) -> bool {
    r.contains(key)
}
#[allow(dead_code)]
pub fn or_len(r: &ObjectRegistry) -> usize {
    r.len()
}
#[allow(dead_code)]
pub fn or_is_empty(r: &ObjectRegistry) -> bool {
    r.is_empty()
}
#[allow(dead_code)]
pub fn or_by_type<'a>(r: &'a ObjectRegistry, type_tag: &str) -> Vec<&'a str> {
    r.by_type(type_tag)
}
#[allow(dead_code)]
pub fn or_clear(r: &mut ObjectRegistry) {
    r.clear();
}
#[allow(dead_code)]
pub fn or_to_json(r: &ObjectRegistry) -> String {
    r.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_register_get() {
        let mut r = new_object_registry();
        or_register(&mut r, "foo", "bar", "string");
        assert_eq!(or_get(&r, "foo"), Some("bar"));
    }
    #[test]
    fn test_contains() {
        let mut r = new_object_registry();
        or_register(&mut r, "x", "1", "int");
        assert!(or_contains(&r, "x"));
        assert!(!or_contains(&r, "y"));
    }
    #[test]
    fn test_remove() {
        let mut r = new_object_registry();
        or_register(&mut r, "k", "v", "t");
        assert!(or_remove(&mut r, "k"));
        assert!(!or_contains(&r, "k"));
    }
    #[test]
    fn test_len() {
        let mut r = new_object_registry();
        assert_eq!(or_len(&r), 0);
        or_register(&mut r, "a", "1", "t");
        or_register(&mut r, "b", "2", "t");
        assert_eq!(or_len(&r), 2);
    }
    #[test]
    fn test_is_empty() {
        let r = new_object_registry();
        assert!(or_is_empty(&r));
    }
    #[test]
    fn test_by_type() {
        let mut r = new_object_registry();
        or_register(&mut r, "a", "1", "int");
        or_register(&mut r, "b", "2", "float");
        or_register(&mut r, "c", "3", "int");
        assert_eq!(or_by_type(&r, "int").len(), 2);
    }
    #[test]
    fn test_get_type() {
        let mut r = new_object_registry();
        or_register(&mut r, "x", "42", "number");
        assert_eq!(r.get_type("x"), Some("number"));
    }
    #[test]
    fn test_clear() {
        let mut r = new_object_registry();
        or_register(&mut r, "a", "1", "t");
        or_clear(&mut r);
        assert!(or_is_empty(&r));
    }
    #[test]
    fn test_to_json_nonempty() {
        let mut r = new_object_registry();
        or_register(&mut r, "k", "v", "str");
        let j = or_to_json(&r);
        assert!(j.contains("\"k\""));
    }
    #[test]
    fn test_missing_key() {
        let r = new_object_registry();
        assert_eq!(or_get(&r, "missing"), None);
    }
}
