// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Int(i64),
    Bool(bool),
    Text(String),
}

/// A named parameter set for storing typed key-value parameters.
#[allow(dead_code)]
pub struct ParamSet {
    params: HashMap<String, ParamValue>,
    dirty: bool,
}

#[allow(dead_code)]
impl ParamSet {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            dirty: false,
        }
    }
    pub fn set_float(&mut self, key: &str, v: f32) {
        self.params.insert(key.to_string(), ParamValue::Float(v));
        self.dirty = true;
    }
    pub fn set_int(&mut self, key: &str, v: i64) {
        self.params.insert(key.to_string(), ParamValue::Int(v));
        self.dirty = true;
    }
    pub fn set_bool(&mut self, key: &str, v: bool) {
        self.params.insert(key.to_string(), ParamValue::Bool(v));
        self.dirty = true;
    }
    pub fn set_text(&mut self, key: &str, v: &str) {
        self.params
            .insert(key.to_string(), ParamValue::Text(v.to_string()));
        self.dirty = true;
    }
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.params.get(key)
    }
    pub fn get_float(&self, key: &str) -> Option<f32> {
        if let Some(ParamValue::Float(v)) = self.params.get(key) {
            Some(*v)
        } else {
            None
        }
    }
    pub fn get_int(&self, key: &str) -> Option<i64> {
        if let Some(ParamValue::Int(v)) = self.params.get(key) {
            Some(*v)
        } else {
            None
        }
    }
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        if let Some(ParamValue::Bool(v)) = self.params.get(key) {
            Some(*v)
        } else {
            None
        }
    }
    pub fn get_text(&self, key: &str) -> Option<&str> {
        if let Some(ParamValue::Text(v)) = self.params.get(key) {
            Some(v.as_str())
        } else {
            None
        }
    }
    pub fn remove(&mut self, key: &str) -> bool {
        self.params.remove(key).is_some()
    }
    pub fn contains(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }
    pub fn len(&self) -> usize {
        self.params.len()
    }
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
    pub fn clear(&mut self) {
        self.params.clear();
        self.dirty = false;
    }
    pub fn keys(&self) -> Vec<&str> {
        self.params.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ParamSet {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_param_set() -> ParamSet {
    ParamSet::new()
}
#[allow(dead_code)]
pub fn ps_set_float(p: &mut ParamSet, k: &str, v: f32) {
    p.set_float(k, v);
}
#[allow(dead_code)]
pub fn ps_set_int(p: &mut ParamSet, k: &str, v: i64) {
    p.set_int(k, v);
}
#[allow(dead_code)]
pub fn ps_set_bool(p: &mut ParamSet, k: &str, v: bool) {
    p.set_bool(k, v);
}
#[allow(dead_code)]
pub fn ps_set_text(p: &mut ParamSet, k: &str, v: &str) {
    p.set_text(k, v);
}
#[allow(dead_code)]
pub fn ps_get_float(p: &ParamSet, k: &str) -> Option<f32> {
    p.get_float(k)
}
#[allow(dead_code)]
pub fn ps_get_int(p: &ParamSet, k: &str) -> Option<i64> {
    p.get_int(k)
}
#[allow(dead_code)]
pub fn ps_get_bool(p: &ParamSet, k: &str) -> Option<bool> {
    p.get_bool(k)
}
#[allow(dead_code)]
pub fn ps_get_text<'a>(p: &'a ParamSet, k: &str) -> Option<&'a str> {
    p.get_text(k)
}
#[allow(dead_code)]
pub fn ps_remove(p: &mut ParamSet, k: &str) -> bool {
    p.remove(k)
}
#[allow(dead_code)]
pub fn ps_contains(p: &ParamSet, k: &str) -> bool {
    p.contains(k)
}
#[allow(dead_code)]
pub fn ps_len(p: &ParamSet) -> usize {
    p.len()
}
#[allow(dead_code)]
pub fn ps_is_empty(p: &ParamSet) -> bool {
    p.is_empty()
}
#[allow(dead_code)]
pub fn ps_clear(p: &mut ParamSet) {
    p.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_get_float() {
        let mut p = new_param_set();
        ps_set_float(&mut p, "speed", std::f32::consts::PI);
        assert!((ps_get_float(&p, "speed").unwrap() - std::f32::consts::PI).abs() < 1e-6);
    }
    #[test]
    fn test_set_get_int() {
        let mut p = new_param_set();
        ps_set_int(&mut p, "count", 42);
        assert_eq!(ps_get_int(&p, "count"), Some(42));
    }
    #[test]
    fn test_set_get_bool() {
        let mut p = new_param_set();
        ps_set_bool(&mut p, "flag", true);
        assert_eq!(ps_get_bool(&p, "flag"), Some(true));
    }
    #[test]
    fn test_set_get_text() {
        let mut p = new_param_set();
        ps_set_text(&mut p, "name", "oxihuman");
        assert_eq!(ps_get_text(&p, "name"), Some("oxihuman"));
    }
    #[test]
    fn test_remove() {
        let mut p = new_param_set();
        ps_set_int(&mut p, "x", 1);
        assert!(ps_remove(&mut p, "x"));
        assert!(!ps_contains(&p, "x"));
    }
    #[test]
    fn test_dirty_flag() {
        let mut p = new_param_set();
        ps_set_float(&mut p, "v", 1.0);
        assert!(p.is_dirty());
        p.mark_clean();
        assert!(!p.is_dirty());
    }
    #[test]
    fn test_len() {
        let mut p = new_param_set();
        assert_eq!(ps_len(&p), 0);
        ps_set_int(&mut p, "a", 1);
        assert_eq!(ps_len(&p), 1);
    }
    #[test]
    fn test_clear() {
        let mut p = new_param_set();
        ps_set_bool(&mut p, "b", false);
        ps_clear(&mut p);
        assert!(ps_is_empty(&p));
    }
    #[test]
    fn test_missing_key_returns_none() {
        let p = new_param_set();
        assert!(ps_get_float(&p, "nope").is_none());
    }
    #[test]
    fn test_overwrite() {
        let mut p = new_param_set();
        ps_set_int(&mut p, "x", 1);
        ps_set_int(&mut p, "x", 2);
        assert_eq!(ps_get_int(&p, "x"), Some(2));
        assert_eq!(ps_len(&p), 1);
    }
}
